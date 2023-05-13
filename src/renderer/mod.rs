use std::num::NonZeroU64;

use encase::{ShaderType, ShaderSize, UniformBuffer};
use wgpu::util::DeviceExt;
use winit::{event_loop::EventLoop, window::Window, event::{Event, WindowEvent}, dpi::PhysicalSize};

type Vec2u = cgmath::Vector2<u32>;
type Vec2f = cgmath::Vector2<f32>;

#[derive(ShaderType)]
struct Uniforms {
	resolution: Vec2u,
	bounds_min: Vec2f,
	bounds_max: Vec2f,
	shading_intensity: f32,
}

struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	config: wgpu::SurfaceConfiguration,
	render_pipeline: Option<wgpu::RenderPipeline>,
	uniform_bind_group: wgpu::BindGroup,
	uniform_bind_group_layout: wgpu::BindGroupLayout,
	uniform_buffer: wgpu::Buffer,
	queue: wgpu::Queue,
}

impl State {
	async fn new(window: &Window) -> Self {
		let size = window.inner_size();

		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
		let surface = unsafe { instance.create_surface(&window) }.unwrap();

		let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::default(),
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		}).await.unwrap();

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				label: None,
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::default(),
			},
			None
		).await.unwrap();

		let format = surface.get_capabilities(&adapter).formats[0];

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Mailbox,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			view_formats: vec![format],
		};
		surface.configure(&device, &config);

		//  Uniforms  //

		let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			contents: &[0; Uniforms::SHADER_SIZE.get() as usize],
		});

		let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: None,
			entries: &[
				wgpu::BindGroupLayoutEntry {
					visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
					binding: 1,
				},
			]
		});

		let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &uniform_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &uniform_buffer,
						offset: 0,
						size: Some(NonZeroU64::new(Uniforms::SHADER_SIZE.get()).unwrap()),
					}),
				},
			],
		});

		//  Done  //

		Self {
			surface,
			device,
			config,
			render_pipeline: None,
			queue,
			uniform_bind_group,
			uniform_bind_group_layout,
			uniform_buffer,
		}
	}

	fn load_shaders(&mut self, userdefs: &str) {
		//  Shaders  //
		let src = include_str!("shader.wgsl");
		let src = src.replace("//INCLUDE//\n", userdefs);

		let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(src.into())
		});

		let vertex = wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[]
		};

		let fragment = wgpu::FragmentState {
			module: &shader,
			entry_point: "fs_main",
			targets: &[Some(wgpu::ColorTargetState {
				format: self.config.format,
				blend: Some(wgpu::BlendState {
					color: wgpu::BlendComponent::REPLACE,
					alpha: wgpu::BlendComponent::REPLACE,
				}),
				write_mask: wgpu::ColorWrites::ALL,
			})],
		};

		//  Pipeline  //

		let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&self.uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

		let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex,
			fragment: Some(fragment),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
		});

		self.render_pipeline = Some(render_pipeline);
	}

	fn redraw(&self, uniforms: &Uniforms) {
		let frame = self.surface.get_current_texture().unwrap();
		let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			let color_attachment = wgpu::RenderPassColorAttachment {
				view: &view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
					store: true,
				}
			};

			let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(color_attachment)],
				depth_stencil_attachment: None,
			});
			if let Some(pipeline) = &self.render_pipeline {
                rpass.set_pipeline(pipeline);
                rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
                rpass.draw(0..3, 0..1);
                rpass.draw(1..4, 0..1);
			}
		}
		let mut uniform_buffer = UniformBuffer::new([0; Uniforms::SHADER_SIZE.get() as usize]);
		uniform_buffer.write(uniforms).unwrap();
		self.queue.write_buffer(&self.uniform_buffer, 0, &uniform_buffer.into_inner());
		self.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	fn resize(&mut self, size: PhysicalSize<u32>) {
		self.config.width = size.width;
		self.config.height = size.height;
		self.surface.configure(&self.device, &self.config);
	}
}

pub async fn run(event_loop: EventLoop<()>, window: Window, code: &str) {
	let mut state = State::new(&window).await;
	state.load_shaders(code);
	let mut uniforms = Uniforms {
		resolution: [window.inner_size().width, window.inner_size().height].into(),
		bounds_min: [0.73, 0.65].into(),
		bounds_max: [0.98, 0.9].into(),
		shading_intensity: 0.001,
	};

	event_loop.run(move |event, _, control_flow| {
		control_flow.set_wait();
		match event {
			Event::WindowEvent { event: WindowEvent::CloseRequested, .. }
				=> control_flow.set_exit(),
			Event::RedrawRequested(_)
				=> state.redraw(&uniforms),
			Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
				uniforms.resolution = [size.width, size.height].into();
				state.resize(size);
				window.request_redraw()
			}
			Event::MainEventsCleared => {
				//window.request_redraw();
			}
			_ => (),
		}
	});
}
