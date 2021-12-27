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
pub(crate) fn rgb2hsv(rgb: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = rgb;

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
    }
    .rem_euclid(360.);

    let saturation = if value == 0. { 0. } else { chroma / value };

    [hue, saturation, value]
}

// https://en.wikipedia.org/wiki/HSL_and_HSV#To_RGB
pub(crate) fn hsv2rgb(hsv: [f32; 3]) -> [f32; 3] {
    let [h, s, v] = hsv;

    if h.is_nan() {
        return [0., 0., 0.];
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

    [r1 + m, g1 + m, b1 + m]
}

pub(crate) fn rgba2hsva(rgba: [f32; 4]) -> [f32; 4] {
    let [r, g, b, a] = rgba;
    let [h, s, v] = rgb2hsv([r, g, b]);
    [h, s, v, a]
}

pub(crate) fn hsva2rgba(hsva: [f32; 4]) -> [f32; 4] {
    let [h, s, v, a] = hsva;
    let [r, g, b] = hsv2rgb([h, s, v]);
    [r, g, b, a]
}

pub(crate) fn rgba2xyza(rgba: [f32; 4]) -> [f32; 4] {
    let [r, g, b, a] = rgba;
    let [x, y, z] = rgb2xyz([r, g, b]);
    [x, y, z, a]
}

pub(crate) fn xyza2rgba(xyza: [f32; 4]) -> [f32; 4] {
    let [x, y, z, a] = xyza;
    let [r, g, b] = xyz2rgb([x, y, z]);
    [r, g, b, a]
}

// http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
pub(crate) fn rgb2xyz(rgb: [f32; 3]) -> [f32; 3] {
    #[rustfmt::skip]
    const M: [f32; 9] = [
        0.4124564, 0.3575761, 0.1804375,
        0.2126729, 0.7151522, 0.0721750,
        0.0193339, 0.1191920, 0.9503041,
    ];

    let [m11, m12, m13, m21, m22, m23, m31, m32, m33] = M;
    let [r, g, b] = rgb;

    [
        m11 * r + m12 * g + m13 * b,
        m21 * r + m22 * g + m23 * b,
        m31 * r + m32 * g + m33 * b,
    ]
}

pub(crate) fn xyz2rgb(xyz: [f32; 3]) -> [f32; 3] {
    #[rustfmt::skip]
    const MINV: [f32; 9] = [
        3.2404542, -1.5371385, -0.4985314,
       -0.9692660,  1.8760108,  0.0415560,
        0.0556434, -0.2040259,  1.0572252,
    ];

    let [m11, m12, m13, m21, m22, m23, m31, m32, m33] = MINV;
    let [x, y, z] = xyz;

    [
        m11 * x + m12 * y + m13 * z,
        m21 * x + m22 * y + m23 * z,
        m31 * x + m32 * y + m33 * z,
    ]
}

// https://en.wikipedia.org/wiki/CIELAB_color_space#From_CIEXYZ_to_CIELAB
pub(crate) fn xyz2lab(xyz: [f32; 3]) -> [f32; 3] {
    fn f(t: f32) -> f32 {
        const DELTA: f32 = 6. / 29.;
        const DELTA2: f32 = DELTA * DELTA;
        const DELTA3: f32 = DELTA2 * DELTA;
        if t > DELTA3 {
            t.powf(1. / 3.)
        } else {
            t / (3. * DELTA2) + 4. / 29.
        }
    }

    const XN: f32 = 95.0489;
    const YN: f32 = 100.;
    const ZN: f32 = 108.8840;

    let [x, y, z] = xyz;
    let f_x = f(x / XN);
    let f_y = f(y / YN);
    let f_z = f(z / ZN);
    let l = 116. * f_y - 16.;
    let a = 500. * (f_x - f_y);
    let b = 200. * (f_y - f_z);

    [l, a, b]
}

pub(crate) fn lab2xyz(lab: [f32; 3]) -> [f32; 3] {
    fn f_inv(t: f32) -> f32 {
        const DELTA: f32 = 6. / 29.;
        const DELTA2: f32 = DELTA * DELTA;

        if t > DELTA {
            t.powf(3.)
        } else {
            3. * DELTA2 * (t - 4. / 29.)
        }
    }

    const XN: f32 = 95.0489;
    const YN: f32 = 100.;
    const ZN: f32 = 108.8840;

    let [l, a, b] = lab;
    let lstar = (l + 16.) / 116.;
    let x = XN * f_inv(lstar + a / 500.);
    let y = YN * f_inv(lstar);
    let z = ZN * f_inv(lstar - b / 200.);

    [x, y, z]
}
