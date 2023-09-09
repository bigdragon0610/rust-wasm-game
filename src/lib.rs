use std::sync::Mutex;
use std::{collections::HashMap, rc::Rc};

use gloo_utils::format::JsValueSerdeExt;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::CanvasRenderingContext2d;

#[derive(Deserialize)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    spawn_local(async move {
        let json = fetch_json("rhb.json")
            .await
            .expect("Could not fetch rhb.json");
        let sheet: Sheet = json
            .into_serde()
            .expect("Could not convert rhb.json into Sheet structure");

        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);
        let image = web_sys::HtmlImageElement::new().unwrap();

        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                success_tx.send(Ok(()));
            }
        });
        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx.send(Err(err));
            }
        });

        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src("rhb.png");

        success_rx.await;
        let mut frame = -1;
        let interval_callback = Closure::wrap(Box::new(move || {
            frame = (frame + 1) % 8;
            let frame_name = format!("Run ({}).png", frame + 1);
            let sprite = sheet.frames.get(&frame_name).expect("Cell not found");

            context.clear_rect(0.0, 0.0, 600.0, 600.0);
            context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                sprite.frame.x.into(),
                sprite.frame.y.into(),
                sprite.frame.w.into(),
                sprite.frame.h.into(),
                300.0,
                300.0,
                sprite.frame.w.into(),
                sprite.frame.h.into(),
            );
        }) as Box<dyn FnMut()>);
        window.set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            50,
        );
        interval_callback.forget();
    });

    Ok(())
}

async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_str(json_path)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;

    JsFuture::from(resp.json()?).await
}

fn draw_triangle(
    context: &CanvasRenderingContext2d,
    remaining_depth: usize,
    points: [(f64, f64); 3],
    color: (u8, u8, u8),
) {
    if remaining_depth == 0 {
        return;
    }
    let next_points = [
        (
            (points[0].0 + points[1].0) / 2.0,
            (points[0].1 + points[1].1) / 2.0,
        ),
        (
            (points[1].0 + points[2].0) / 2.0,
            (points[1].1 + points[2].1) / 2.0,
        ),
        (
            (points[2].0 + points[0].0) / 2.0,
            (points[2].1 + points[0].1) / 2.0,
        ),
    ];
    context.move_to(next_points[0].0, next_points[0].1);
    context.begin_path();
    context.line_to(next_points[1].0, next_points[1].1);
    context.line_to(next_points[2].0, next_points[2].1);
    context.line_to(next_points[0].0, next_points[0].1);
    context.close_path();
    context.stroke();
    context.set_fill_style(&JsValue::from_str(&format!(
        "rgb({}, {}, {})",
        color.0, color.1, color.2
    )));
    context.fill();

    let mut rng = thread_rng();
    let next_color = (
        rng.gen_range(0..255),
        rng.gen_range(0..255),
        rng.gen_range(0..255),
    );

    draw_triangle(
        context,
        remaining_depth - 1,
        [points[0], next_points[0], next_points[2]],
        next_color,
    );
    draw_triangle(
        context,
        remaining_depth - 1,
        [next_points[0], points[1], next_points[1]],
        next_color,
    );
    draw_triangle(
        context,
        remaining_depth - 1,
        [next_points[2], next_points[1], points[2]],
        next_color,
    );
}
