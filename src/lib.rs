#![feature(array_chunks)]
#![feature(array_zip)]
#![feature(const_fn_floating_point_arithmetic)]

mod conversions;
mod processing;
mod utils;

use conversions::*;
use processing::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::*;

mod externs {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub(crate) fn log(s: &str);
    }
}

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn into_image_data(src: HtmlImageElement) -> ImageData {
    let canvas = window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();
    canvas.set_width(256);
    canvas.set_height(256);

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    ctx.clear_rect(0., 0., 256., 256.);
    ctx.draw_image_with_html_image_element(&src, 0., 0.)
        .unwrap();

    ctx.get_image_data(0., 0., 256., 256.).unwrap()
}

fn log(s: &str) {
    externs::log(s);
}

fn log_color(rgba: [f32; 3]) {
    let [r, g, b] = rgba;
    web_sys::console::log_2(
        &format!("%c {} {} {}", r, g, b).into(),
        &format!("background: rgba({}, {}, {})", r * 255., g * 255., b * 255.,).into(),
    );
}

#[wasm_bindgen]
pub fn process(src: HtmlImageElement, dst: HtmlImageElement) {
    let src_data = into_image_data(src);
    let dst_data = into_image_data(dst);

    let src_data_rgba: Vec<[f32; 4]> = src_data
        .data()
        .as_slice()
        .array_chunks::<4>()
        .copied()
        .map(threshold_alpha::<200>)
        .map(bytes2floats)
        .collect();

    let dst_data_rgba: Vec<[f32; 4]> = dst_data
        .data()
        .as_slice()
        .array_chunks::<4>()
        .copied()
        .map(threshold_alpha::<200>)
        .map(bytes2floats)
        .collect();

    let src_data_xyza: Vec<[f32; 4]> = src_data_rgba.iter().copied().map(rgba2xyza).collect();
    let dst_data_xyza: Vec<[f32; 4]> = dst_data_rgba.iter().copied().map(rgba2xyza).collect();

    let src_data_xyz: Vec<[f32; 3]> = src_data_xyza
        .iter()
        .copied()
        .filter_map(|[h, s, v, a]| if a > 0.9 { Some([h, s, v]) } else { None })
        .collect();

    let src = k_means_std::<3, 3, 10>(&src_data_xyz);
    let dst = k_means_std::<2, 4, 10>(&dst_data_xyza);

    for i in src.means {
        log_color(xyz2rgb(i));
    }

    let document_body = window().unwrap().document().unwrap().body().unwrap();

    for (_, &mean) in src.means.iter().enumerate() {
        let output = dst_data_xyza
            .iter()
            .copied()
            .enumerate()
            .map(|(idx, mut xyza)| {
                let label = dst.labels[idx];
                for i in 0..=2 {
                    xyza[i] = xyza[i] - dst.means[label][i] + mean[i];
                }
                xyza
            })
            .map(xyza2rgba)
            .map(floats2bytes)
            .flatten()
            .collect::<Vec<_>>();

        let img_data = ImageData::new_with_u8_clamped_array(Clamped(&output), 256).unwrap();
        let canvas = window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        canvas.set_width(256);
        canvas.set_height(256);
        canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap()
            .put_image_data(&img_data, 0.0, 0.0)
            .unwrap();
        document_body.append_child(&canvas).unwrap();
    }
}
