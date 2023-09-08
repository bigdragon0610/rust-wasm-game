use std::task::Context;

use rand::{thread_rng, Rng};
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

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

    context.move_to(300.0, 0.0);
    context.begin_path();
    context.line_to(0.0, 600.0);
    context.line_to(600.0, 600.0);
    context.line_to(300.0, 0.0);
    context.close_path();
    context.stroke();
    draw_triangle(
        &context,
        7,
        [(300.0, 0.0), (0.0, 600.0), (600.0, 600.0)],
        (255, 255, 0),
    );
    Ok(())
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
