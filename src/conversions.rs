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
        0.0193339, 0.119192,  0.9503041,
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
       -0.969266,   1.8760108,  0.0415560,
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
