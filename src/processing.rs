pub(crate) fn distance<const D: usize>(a: &[f32; D], b: &[f32; D]) -> f32 {
    a.zip(*b).map(|(a, b)| (a - b).powf(2.)).iter().sum::<f32>()
}

fn k_means_init<const K: usize, const D: usize>(points: &[[f32; D]]) -> [[f32; D]; K] {
    let mut centroids = [[0f32; D]; K];

    centroids[0] = points[points.len() / 2];
    for i in 1..K {
        centroids[i] = points
            .iter()
            .fold(([0f32; D], 0f32), |farthest, cur| {
                let closest_centroid = centroids[0..i]
                    .iter()
                    .fold(([0f32; D], f32::MAX), |closest, centroid| {
                        let dist = distance(cur, centroid);

                        if closest.1 < dist {
                            closest
                        } else {
                            (*centroid, dist)
                        }
                    })
                    .0;

                let dist = distance(&closest_centroid, cur);

                if farthest.1 > dist {
                    farthest
                } else {
                    (*cur, dist)
                }
            })
            .0;
    }

    centroids.reverse();
    centroids
}

pub(crate) struct KMeans<const K: usize, const D: usize> {
    pub(crate) means: [[f32; D]; K],
    pub(crate) labels: Vec<usize>,
}

pub(crate) fn k_means<const K: usize, const D: usize, const ITERS: usize>(
    points: &[[f32; D]],
) -> KMeans<K, D> {
    let mut centroids = k_means_init(points);

    let mut labels = Vec::with_capacity(points.len());

    for _ in 0..ITERS {
        let mut labels_hist = [0usize; K];
        labels.clear();
        labels.extend(points.iter().map(|p| {
            let label = centroids
                .iter()
                .map(|c| distance(c, p))
                .enumerate()
                .fold((0usize, f32::MAX), |(label, distance), (l, d)| {
                    if d < distance {
                        (l, d)
                    } else {
                        (label, distance)
                    }
                })
                .0;
            labels_hist[label] += 1;
            label
        }));

        let labels_weights: [f32; K] =
            labels_hist.map(|h| if h == 0 { 0. } else { 1. / (h as f32) });

        centroids = [[0f32; D]; K];

        labels.iter().enumerate().for_each(|(i, &l)| {
            for d in 0..D {
                centroids[l][d] += points[i][d] * labels_weights[l];
            }
        });
    }

    KMeans {
        means: centroids,
        labels,
    }
}
