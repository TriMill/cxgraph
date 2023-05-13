use language::compile;
use renderer::run;
use winit::{event_loop::EventLoop, window::Window};

mod language;
mod renderer;
mod complex;

fn main() {
	env_logger::init();

	let src = r#"
		f(z,a,b,c) = (z-a)*(z-b)*(z-c)
		g(z,a,b,c) = z - f(z,a,b,c)/f'(z,a,b,c)
		plot(z) = iter4(g, 50, z/3, z, 1, -1)
	"#;
	let wgsl = compile(src).unwrap();
	println!("{}", wgsl);

	let event_loop = EventLoop::new();
	let window = Window::new(&event_loop).unwrap();
	window.set_title("window");

	pollster::block_on(run(event_loop, window, &wgsl));

}
