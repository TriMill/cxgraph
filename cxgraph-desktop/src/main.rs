
use libcxgraph::{renderer::WgpuState, language::compile};
use winit::{event_loop::EventLoop, window::Window, event::{Event, WindowEvent}};

fn main() {
	env_logger::builder()
		.filter_level(log::LevelFilter::Warn)
		.init();

	let src = "plot(z) = 27^z - 9^z - 3^z";
	let wgsl = compile(src).unwrap();
	println!("{wgsl}");

	let event_loop = EventLoop::new();
	let window = Window::new(&event_loop).unwrap();
	window.set_title("window");

	pollster::block_on(run(event_loop, window, &wgsl));
}

async fn run(event_loop: EventLoop<()>, window: Window, code: &str) {
    let size = window.inner_size();
	let mut state = WgpuState::new(&window, size.into()).await;

    state.load_shaders(code);

	state.set_bounds((-5.0, -5.0), (5.0, 5.0));
	state.set_shading_intensity(0.05);

	event_loop.run(move |event, _, control_flow| {
		control_flow.set_wait();
		match event {
			Event::WindowEvent { event: WindowEvent::CloseRequested, .. }
				=> control_flow.set_exit(),
			Event::RedrawRequested(_)
				=> state.redraw(),
			Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
				state.resize(size.into());
				window.request_redraw();
			}
			_ => (),
		}
	});
}
