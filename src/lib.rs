#![feature(array_chunks)]
#![feature(array_zip)]

mod conversions;
mod processing;
mod utils;

use conversions::*;
use processing::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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

#[wasm_bindgen]
pub fn process(src: HtmlImageElement, dst: HtmlImageElement, canvas: HtmlCanvasElement) {
    let src_data = into_image_data(src);
    let dst_data = into_image_data(dst);

    log("src_data");
    let src_data_hsva: Vec<[f32; 4]> = src_data
        .data()
        .as_slice()
        .array_chunks::<4>()
        .copied()
        .map(threshold_alpha::<200>)
        .map(bytes2floats)
        .map(rgba2xyza)
        .map(xyza2laba)
        .collect();

    log("dst_data");
    let dst_data_hsva: Vec<[f32; 4]> = dst_data
        .data()
        .as_slice()
        .array_chunks::<4>()
        .copied()
        .map(threshold_alpha::<200>)
        .map(bytes2floats)
        .map(rgba2xyza)
        .map(xyza2laba)
        .collect();

    log("kmeans1");
    let (src_means, src_stds) = k_means_std::<3, 4>(&src_data_hsva);
    log(&format!("means={:?} stds={:?}", src_means, src_stds));
    log("kmeans2");
    let (dst_means, dst_stds) = k_means_std::<1, 4>(&dst_data_hsva);
    log(&format!("means={:?} stds={:?}", dst_means, dst_stds));

    let document_body = window().unwrap().document().unwrap().body().unwrap();

    for (_, &(mean, std)) in src_means.zip(src_stds).iter().enumerate() {
        {
            let [r, g, b, a] = xyza2rgba(laba2xyza(mean));
            let [sr, sg, sb, sa] = xyza2rgba(laba2xyza(std));
            web_sys::console::log_2(
                &format!("mean %c {} {} {} {}", r, g, b, a).into(),
                &format!("background: rgba({}, {}, {}, {})", r*255., g*255., b*255., a*255.).into(),
            );

            web_sys::console::log_2(
                &format!("std  %c {} {} {} {}", sr, sg, sb, sa).into(),
                &format!("background: rgba({}, {}, {}, {})", r*255., g*255., b*255., a*255.).into(),
            );
        }
        let output = dst_data_hsva
            .iter()
            .map(|laba| {
                let apply = |i, sm, ss, tm, ts| -> f32 { (i - tm) * ts / ss + sm };

                let mut r = *laba;
                for i in 0..=2 {
                    r[i] = apply(laba[i], mean[i], std[i].sqrt(), dst_means[0][i], dst_stds[0][i].sqrt());
                }

                r
            })
            .map(laba2xyza)
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
        web_sys::console::log_1(&img_data.into());
    }
}
