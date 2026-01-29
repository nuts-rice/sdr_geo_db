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
    let vert_shader = get_vert_shader(&ctx)?;
    let frag_shader = get_frag_shader(&ctx)?;
    let program = link_program(&ctx, &vert_shader, &frag_shader)?;


}


pub fn get_vert_shader(ctx: WebGl2RenderingContext) -> Result<WebGlShader, String> {

todo!()
}


// Uses inigo quilez's Input - Sound 
//https://www.shadertoy.com/view/Xds3Rr
pub fn get_frag_shader(ctx: WebGl2RenderingContext) -> Result<WebGlShader, String> {
    let frag_shader = compile_shader (
    &ctx,
    WebGl2RenderingContext::FRAGMENT_SHADER,
    r##"#version 300 es
void main(out vec4 fragColor, in vec2 fragCoord) {
vec2 uv = fragCoord/iResolution.xy;
int tx = int(uv.x * 8.0);
float fft = texelFetch




    "##,)?;



}

