use std::sync::Arc;

use egui::{Color32, ColorImage};
use ndarray::{Array, ArrayBase, Axis, Dim, OwnedRepr};
use ndarray_rand::{rand_distr::Uniform, RandomExt};
use std::ops::AddAssign;

type Matrix = ArrayBase<OwnedRepr<f64>, Dim<[usize; 2]>>;
type Cluster = Vec<Vec<usize>>;
type Centroids = Matrix;
type Features = Matrix;

pub fn segmentation(image: ColorImage, clusters: u8) -> ColorImage {
    let mut ex = Array::zeros((image.width() * image.height(), 1));

    for i in image.pixels.into_iter().enumerate() {
        ex[[i.0, 0]] =
            0.299_f64 * i.1.r() as f64 + 0.587_f64 * i.1.g() as f64 + 0.114_f64 * i.1.b() as f64;
    }

    let mut segmentation_map = ColorImage::new(image.size, Color32::TRANSPARENT);

    let x = Arc::new(ex);

    let (centroids, cluster) = k_means(clusters as usize, 100, &x);

    for (cluster_idx, row_indexes) in cluster.iter().enumerate() {
        let brightness = centroids[[0, cluster_idx]] as u8;

        for row_idx in row_indexes {
            segmentation_map.pixels[*row_idx] = Color32::from_gray(brightness);
        }
    }
    segmentation_map
}

fn assign_to_centroids(centroids: &Centroids, x: &Arc<Features>) -> (Cluster, f64) {
    let mut total_err = 0.0;
    let mut cluster = vec![vec![]; centroids.ncols()];

    for (i, row) in x.axis_iter(Axis(0)).enumerate() {
        let (closet_centroid_idx, err) = centroids
            .axis_iter(Axis(1))
            .map(|centroid| {
                let d = &centroid - &row.t();
                (&d * &d).sum()
            })
            .enumerate()
            .min_by(|(_, v_1), (_, v_2)| v_1.total_cmp(v_2))
            .unwrap();

        cluster[closet_centroid_idx].push(i);
        total_err += err;
    }
    (cluster, total_err)
}

fn compute_centroids_from_cluster(cluster: &Cluster, x: &Arc<Features>) -> Centroids {
    let mut centroids = Array::zeros((x.ncols(), cluster.len()));
    for (cluster_idx, row_indexes) in cluster.iter().enumerate() {
        let n = f64::from(u32::try_from(row_indexes.len()).expect("overflows u32"));

        if row_indexes.is_empty() {
            centroids
                .column_mut(cluster_idx)
                .assign(&Array::random((x.ncols(), 1), Uniform::new(0.0, 1.0)).column(0));
        }

        for row_idx in row_indexes {
            centroids
                .column_mut(cluster_idx)
                .add_assign(&(1.0 / n * &x.row(*row_idx).t()));
        }
    }

    centroids
}

fn k_means(k: usize, max_iter: usize, x: &Arc<Features>) -> (Centroids, Cluster) {
    let mut centroids = Array::random((x.ncols(), k), Uniform::new(0.0, 1.0));

    let mut cluster = vec![vec![]; centroids.ncols()];
    let mut prev_total_err = f64::INFINITY;
    let mut total_err = 0.0;
    let mut iter = 0;

    while iter < max_iter && (iter == 0 || (prev_total_err - total_err) / total_err > 0.01) {
        prev_total_err = if iter > 0 { total_err } else { prev_total_err };
        (cluster, total_err) = assign_to_centroids(&centroids, x);

        centroids = compute_centroids_from_cluster(&cluster, x);
        iter += 1;
    }
    (centroids, cluster)
}
