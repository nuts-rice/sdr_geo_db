use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContect, WebGlProgram, WebGlShader ,Position, PositionError};
use yew::hook;
use yew_hooks::prelude::*;

#[hook]
pub fn use_webgl_context() -> Option<WebGl2RenderingContext> {
    let document = web_sys::window()?.document()?;
    let canvas = document.get_element_by_id("webgl-canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().ok()?;
    let ctx = canvas.get_context("webgl2").ok()??;


}


pub fn get_vert_shader() -> Result<WebGlShader, String> {

todo!()
}

pub fn get_frag_shader() -> Result<WebGlShader, String> {
    todo!()
}

