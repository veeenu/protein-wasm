use js_sys::Math;

pub(crate) fn distance<const D: usize>(a: &[f32; D], b: &[f32; D]) -> f32 {
    a.zip(*b).map(|(a, b)| (a - b).powf(2.)).iter().sum::<f32>()
}

pub(crate) fn length<const D: usize>(a: &[f32; D]) -> f32 {
    a.map(|a| a.powf(2.)).iter().sum::<f32>()
}

pub(crate) fn threshold_alpha<const T: u8>(rgba: [u8; 4]) -> [u8; 4] {
    if rgba[3] < T {
        [0u8; 4]
    } else {
        rgba
    }
}

pub(crate) fn bytes2floats(rgba: [u8; 4]) -> [f32; 4] {
    let [r, g, b, a] = rgba;
    [
        r as f32 / 255.,
        g as f32 / 255.,
        b as f32 / 255.,
        a as f32 / 255.,
    ]
}

pub(crate) fn floats2bytes(rgba: [f32; 4]) -> [u8; 4] {
    let [r, g, b, a] = rgba;
    [
        (r * 255.) as u8,
        (g * 255.) as u8,
        (b * 255.) as u8,
        (a * 255.) as u8,
    ]
}

// https://en.wikipedia.org/wiki/HSL_and_HSV#From_RGB
pub(crate) fn rgba2hsva(rgba: [f32; 4]) -> [f32; 4] {
    let [r, g, b, a] = rgba;

    let value = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = value - min;

    let hue = if chroma == 0. {
        0.
    } else if value == r {
        60. * (0. + (g - b) / chroma)
    } else if value == g {
        60. * (2. + (b - r) / chroma)
    } else if value == b {
        60. * (4. + (r - g) / chroma)
    } else {
        unreachable!()
    }.rem_euclid(360.);

    let saturation = if value == 0. { 0. } else { chroma / value };

    [hue, saturation, value, a]
}

// https://en.wikipedia.org/wiki/HSL_and_HSV#To_RGB
pub(crate) fn hsva2rgba(hsva: [f32; 4]) -> [f32; 4] {
    let [h, s, v, a] = hsva;

    if h.is_nan() {
        return [0., 0., 0., a];
    }

    let c = v * s;
    let hue_range = h / 60.;
    let x = c * (1. - (hue_range.rem_euclid(2.) - 1.).abs());
    let m = v - c;
    let (r1, g1, b1) = match hue_range {
        _ if (0. ..1.).contains(&hue_range) => (c, x, 0.),
        _ if (1. ..2.).contains(&hue_range) => (x, c, 0.),
        _ if (2. ..3.).contains(&hue_range) => (0., c, x),
        _ if (3. ..4.).contains(&hue_range) => (0., x, c),
        _ if (4. ..5.).contains(&hue_range) => (x, 0., c),
        _ if (5. ..6.).contains(&hue_range) => (c, 0., x),
        _ => (0., 0., 0.),
    };

    [r1 + m, g1 + m, b1 + m, a]
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

    for _ in 0..20 {
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

        let labels_weights: [f32; K] = labels_hist.map(|h| if h == 0 { 0. } else { 1. / (h as f32) });

        centroids = [[0f32; D]; K];
        centroids_std = [[0f32; D]; K];

        labels.iter().enumerate().for_each(|(i, &l)| {
            for d in 0..D {
                centroids[l][d] += points[i][d] * labels_weights[l];
            }
        });

        labels.iter().enumerate().for_each(|(i, &l)| {
            for d in 0..D {
                centroids_std[l][d] += points[i][d] - centroids[l][d];
            }
        });

        centroids_std = centroids_std.map(|c| c.map(f32::sqrt));
    }

    (centroids, centroids_std)
}
