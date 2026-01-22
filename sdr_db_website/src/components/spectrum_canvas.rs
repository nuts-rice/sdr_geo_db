use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
const AMBER_BRIGHT: &str = "#FFAA00";
const AMBER_MID: &str = "#AA6600";
const AMBER_DIM: &str = "#553300";

const CATPPUCIN_YELLOW: &str = "#EED49F";
const CATPPUCIN_ORANGE: &str = "#f5a97f";
const CATPPUCIN_BLUE: &str = "#8aadf4";

pub fn draw_spectrum(canvas: &HtmlCanvasElement, data: &[f64], center_freq: f64, span: f64) {
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    ctx.set_fill_style_str("black");
    ctx.fill_rect(0.0, 0.0, width, height);

    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str(AMBER_BRIGHT));
    ctx.set_line_width(1.5);
    for (i, &dbm) in data.iter().enumerate() {
        let x = (i as f64 / data.len() as f64) * width;
        let y = height - ((dbm + 100.0) / 100.0) * height;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke();

    let freq_label = format!("{:.2} MHz", center_freq / 1_000_000.0);
    ctx.set_fill_style_str("white");
    ctx.set_font("16px Arial");
    ctx.fill_text(&freq_label, 10.0, 20.0).unwrap();
}
