#![cfg(target_arch="wasm32")]

use crate::{renderer::WgpuState, language::compile};
use winit::{event_loop::EventLoop, window::Window};
use wasm_bindgen::prelude::*;

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
	use winit::platform::web::WindowExtWebSys;

	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
	console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

	let event_loop = EventLoop::new();
	let window = Window::new(&event_loop).unwrap();

	window.set_inner_size(PhysicalSize::new(1200, 1200));
	web_sys::window()
		.and_then(|win| win.document())
		.and_then(|doc| {
			let dst = doc.get_element_by_id("canvas_container")?;
			let canvas = web_sys::Element::from(window.canvas());
			dst.append_child(&canvas).ok()?;
			Some(())
		}).expect("Couldn't append canvas to document body.");

	window.set_title("window");

    let size = window.inner_size();
	let mut state = WgpuState::new(&window, size.into()).await;
	state.set_bounds((-5.0, -5.0), (5.0, 5.0));
	state.set_shading_intensity(0.01);
	unsafe { WGPU_STATE = Some(state) };
}

#[wasm_bindgen]
pub fn load_shader(src: &str) {
	let wgsl = compile(src).unwrap();
	with_state(|state| {
		state.load_shaders(&wgsl);
		state.redraw();
	});
}

#[wasm_bindgen]
pub fn redraw() {
	with_state(|state| {
		state.redraw();
	});
}

#[wasm_bindgen]
pub fn resize(width: u32, height: u32) {
	with_state(|state| state.resize((width, height)));
}

#[wasm_bindgen]
pub fn set_bounds(min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
	with_state(|state| state.set_bounds((min_x, min_y), (max_x, max_y)));
}

#[wasm_bindgen]
pub fn set_shading_intensity(value: f32) {
	with_state(|state| state.set_shading_intensity(value));
}