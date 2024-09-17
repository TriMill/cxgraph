use std::collections::HashMap;

use libcxgraph::{renderer::WgpuState, language::{compile, show_ast}};
use log::info;
use winit::{window::WindowBuilder, event_loop::EventLoop, platform::web::WindowBuilderExtWebSys};
use wasm_bindgen::{prelude::*, JsValue};
use web_sys::HtmlCanvasElement;

// wasm is single-threaded so there's no possibility of Bad happening
// due to mutable global state. this will be Some(..) once start runs
static mut WGPU_STATE: Option<WgpuState> = None;

fn with_state<F>(f: F)
where F: Fn(&mut WgpuState) {
	let mut state = unsafe { WGPU_STATE.take().unwrap() };
	f(&mut state);
	unsafe { WGPU_STATE = Some(state) };
}

#[wasm_bindgen(start)]
pub async fn start() {
	use winit::dpi::PhysicalSize;

	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
	console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

	let canvas_elem = web_sys::window()
		.and_then(|win| win.document())
		.and_then(|doc| Some(doc.get_element_by_id("canvas")?))
		.expect("Could not find canvas element");

	let canvas: HtmlCanvasElement = canvas_elem
		.dyn_into()
		.expect("Canvas was not a canvas");

	let event_loop = EventLoop::new().unwrap();
	let window = WindowBuilder::new()
		.with_canvas(Some(canvas))
        .with_prevent_default(false)
		.with_inner_size(PhysicalSize::new(100, 100))
		.with_title("window")
		.build(&event_loop)
		.expect("Failed to build window");

    let window_ref = Box::leak(Box::new(window));

	let mut state = WgpuState::new(window_ref, (100, 100)).await;
	state.uniforms.bounds_min = (-5.0, -5.0).into();
	state.uniforms.bounds_max = ( 5.0,  5.0).into();
	state.uniforms.shading_intensity = 0.3;
	state.uniforms.contour_intensity = 0.0;
	unsafe { WGPU_STATE = Some(state) };
	info!("Initialized");
}

#[wasm_bindgen]
pub fn load_shader(src: &str, var_names: Box<[JsValue]>) -> Result<(), JsValue> {
	let names: HashMap<String, usize> = var_names.iter()
		.enumerate()
		.map(|(i, e)| (e.as_string().unwrap(), i))
		.collect();
	let wgsl = compile(src, &names).map_err(|e| e.to_string())?;
	info!("Generated WGSL:\n{}", wgsl);
	with_state(|state| state.load_shaders(&wgsl));
	Ok(())
}

#[wasm_bindgen]
pub fn show_shader_ast(src: &str) -> Result<String, JsValue> {
	show_ast(src).map_err(|e| e.to_string().into())
}

#[wasm_bindgen]
pub fn redraw() {
	with_state(|state| state.redraw());
}

#[wasm_bindgen]
pub fn resize(width: u32, height: u32) {
	with_state(|state| state.resize((width, height)));
}

#[wasm_bindgen]
pub fn set_res_scale(scale: f32) {
	with_state(|state| state.uniforms.res_scale = scale);
}

#[wasm_bindgen]
pub fn set_bounds(min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
	with_state(|state| {
		state.uniforms.bounds_min = (min_x, min_y).into();
		state.uniforms.bounds_max = (max_x, max_y).into();
	});
}

#[wasm_bindgen]
pub fn set_shading_intensity(value: f32) {
	with_state(|state| state.uniforms.shading_intensity = value);
}

#[wasm_bindgen]
pub fn set_contour_intensity(value: f32) {
	with_state(|state| state.uniforms.contour_intensity = value);
}


#[wasm_bindgen]
pub fn set_coloring(value: u32) {
	with_state(|state| state.uniforms.coloring = value);
}

#[wasm_bindgen]
pub fn set_decorations(value: u32) {
	with_state(|state| state.uniforms.decorations = value);
}

#[wasm_bindgen]
pub fn set_grid_mode(value: u32) {
	with_state(|state| state.uniforms.grid_mode = value);
}

#[wasm_bindgen]
pub fn set_variable(idx: usize, re: f32, im: f32) {
	with_state(|state| {
		state.uniforms.variables[idx*2 + 0] = re;
		state.uniforms.variables[idx*2 + 1] = im;
	});
}
