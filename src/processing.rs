use js_sys::Math;

pub(crate) fn distance<const D: usize>(a: &[f32; D], b: &[f32; D]) -> f32 {
    a.zip(*b).map(|(a, b)| (a - b).powf(2.)).iter().sum::<f32>()
}

pub(crate) fn length<const D: usize>(a: &[f32; D]) -> f32 {
    a.map(|a| a.powf(2.)).iter().sum::<f32>()
}

pub(crate) fn k_means_std<const K: usize, const D: usize>(
    points: &[[f32; D]],
) -> ([[f32; D]; K], [[f32; D]; K]) {
    let mut centroids = [[0f32; D]; K];
    let mut centroids_std = [[0f32; D]; K];

    let ranges = points
        .iter()
        .fold(([f32::MAX; D], [f32::MIN; D]), |(mins, maxs), &p| {
            (
                mins.zip(p).map(|(m, pv)| m.min(pv)),
                maxs.zip(p).map(|(m, pv)| m.max(pv)),
            )
        });

    super::log(&format!("mins={:?} maxs={:?}", ranges.0, ranges.1));

    for k in 0..K {
        for d in 0..D {
            let (a, b) = (ranges.0[d], ranges.1[d]);
            fn random() -> f32 {
                Math::random() as f32
            }
            centroids[k][d] = random() * (b - a) + a;
        }
    }

    let mut labels = Vec::with_capacity(points.len());

    for _ in 0..10 {
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

        super::log(&format!("{:?}", labels_hist));

        let labels_weights: [f32; K] =
            labels_hist.map(|h| if h == 0 { 0. } else { 1. / (h as f32) });

        centroids = [[0f32; D]; K];
        centroids_std = [[0f32; D]; K];

        labels.iter().enumerate().for_each(|(i, &l)| {
            for d in 0..D {
                centroids[l][d] += points[i][d] * labels_weights[l];
            }
        });

        labels.iter().enumerate().for_each(|(i, &l)| {
            for d in 0..D {
                centroids_std[l][d] += (points[i][d] - centroids[l][d]).powf(2.);
            }
        });

        centroids_std = centroids_std.map(|c| c.map(|c| c / (labels.len() as f32)).map(f32::sqrt));
    }

    (centroids, centroids_std)
}
