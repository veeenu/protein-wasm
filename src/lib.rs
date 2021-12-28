#![feature(array_chunks)]
#![feature(array_zip)]

mod conversions;
mod processing;

use conversions::*;
use processing::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::*;

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

    let w = src.natural_width();
    let h = src.natural_width();

    canvas.set_width(w);
    canvas.set_height(h);

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    ctx.clear_rect(0., 0., w as f64, h as f64);
    ctx.draw_image_with_html_image_element(&src, 0., 0.)
        .unwrap();

    ctx.get_image_data(0., 0., w as f64, h as f64).unwrap()
}

#[wasm_bindgen]
pub fn find_k_means(src: HtmlImageElement, output_container: HtmlElement) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let src_data = into_image_data(src);

    let src_data_rgba: Vec<[f32; 4]> = src_data
        .data()
        .as_slice()
        .array_chunks::<4>()
        .copied()
        .map(threshold_alpha::<200>)
        .map(bytes2floats)
        .collect();

    let src_data_xyza: Vec<[f32; 4]> = src_data_rgba.iter().copied().map(rgba2xyza).collect();

    let src_means = k_means::<3, 4, 10>(&src_data_xyza);

    for &mean in src_means.means.iter() {
        let [r, g, b, a] = xyza2rgba(mean);

        let canvas = window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        canvas.set_width(64);
        canvas.set_height(64);
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        ctx.set_fill_style(&format!("rgba({}, {}, {}, {})", r * 255., g * 255., b * 255., a).into());
        ctx.set_stroke_style(&"#ccc".into());
        ctx.fill_rect(2., 2., 62., 62.);
        ctx.stroke_rect(1., 1., 63., 63.);
        output_container.append_child(&canvas).unwrap();
    }
}

#[wasm_bindgen]
pub fn transfer_colors(
    src: HtmlImageElement,
    dst: HtmlImageElement,
    output_container: HtmlElement,
) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let src_data = into_image_data(src);
    let dst_data = into_image_data(dst);

    let src_data_xyza: Vec<[f32; 4]> = src_data
        .data()
        .as_slice()
        .array_chunks::<4>()
        .copied()
        .map(threshold_alpha::<200>)
        .map(bytes2floats)
        .map(rgba2xyza)
        .collect();

    let dst_data_xyza: Vec<[f32; 4]> = dst_data
        .data()
        .as_slice()
        .array_chunks::<4>()
        .copied()
        .map(threshold_alpha::<200>)
        .map(bytes2floats)
        .map(rgba2xyza)
        .collect();

    //let src_data_xyza: Vec<[f32; 4]> = src_data_rgba.iter().copied().map(rgba2xyza).collect();
    //let dst_data_xyza: Vec<[f32; 4]> = dst_data_rgba.iter().copied().map(rgba2xyza).collect();

    let src_means = k_means::<3, 4, 10>(&src_data_xyza);
    let dst_means = k_means::<2, 4, 10>(&dst_data_xyza);

    for src_mean in src_means.means {
        let output = dst_data_xyza
            .iter()
            .copied()
            .enumerate()
            .map(|(idx, mut dst_color)| {
                let label = dst_means.labels[idx];
                let dst_mean = dst_means.means[label];
                for i in 0..=2 {
                    dst_color[i] = dst_color[i] - dst_mean[i] + src_mean[i];
                }
                dst_color
            })
            .map(xyza2rgba)
            .flat_map(floats2bytes)
            .collect::<Vec<_>>();

        let img_data =
            ImageData::new_with_u8_clamped_array(Clamped(&output), dst_data.width()).unwrap();
        let canvas = window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        canvas.set_width(dst_data.width());
        canvas.set_height(dst_data.height());
        canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap()
            .put_image_data(&img_data, 0.0, 0.0)
            .unwrap();
        output_container.append_child(&canvas).unwrap();
    }
}
