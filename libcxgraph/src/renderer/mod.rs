use std::{num::NonZeroU64, io::Cursor};

use wgpu::util::DeviceExt;

#[derive(Debug)]
#[repr(C)]
pub struct Uniforms {
	pub variables: [f32; 16],
	pub resolution: (u32, u32),
	pub bounds_min: (f32, f32),
	pub bounds_max: (f32, f32),
	pub shading_intensity: f32,
	pub contour_intensity: f32,
	pub decorations: u32,
	pub coloring: u32,
	_padding: [u8; 8],
}

const UNIFORM_SIZE: usize = std::mem::size_of::<Uniforms>();

impl Uniforms {
	pub fn encode(&self, buf: &mut impl std::io::Write) -> Result<(), std::io::Error> {
		for var in self.variables {
			buf.write_all(&var.to_le_bytes())?;
		}
		buf.write_all(&self.resolution.0.to_le_bytes())?;
		buf.write_all(&self.resolution.1.to_le_bytes())?;
		buf.write_all(&self.bounds_min.0.to_le_bytes())?;
		buf.write_all(&self.bounds_min.1.to_le_bytes())?;
		buf.write_all(&self.bounds_max.0.to_le_bytes())?;
		buf.write_all(&self.bounds_max.1.to_le_bytes())?;
		buf.write_all(&self.shading_intensity.to_le_bytes())?;
		buf.write_all(&self.contour_intensity.to_le_bytes())?;
		buf.write_all(&self.decorations.to_le_bytes())?;
		buf.write_all(&self.coloring.to_le_bytes())?;
		Ok(())
	}
}

pub struct WgpuState {
	pub uniforms: Uniforms,
	surface: wgpu::Surface,
	device: wgpu::Device,
	config: wgpu::SurfaceConfiguration,
	render_pipeline: Option<wgpu::RenderPipeline>,
	uniform_bind_group: wgpu::BindGroup,
	uniform_layout: wgpu::BindGroupLayout,
	uniform_buffer: wgpu::Buffer,
	queue: wgpu::Queue
}

impl WgpuState {
	pub async fn new<W>(window: &W, size: (u32, u32)) -> Self
	where W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle {

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
				limits: wgpu::Limits::downlevel_webgl2_defaults(),
			},
			None
		).await.map_err(|e| e.to_string()).unwrap();

		let format = surface.get_capabilities(&adapter).formats[0];

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format,
			width: size.0,
			height: size.1,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			view_formats: vec![format],
		};
		surface.configure(&device, &config);

		//  Uniforms  //

		let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			contents: &[0; UNIFORM_SIZE],
		});

		let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: None,
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			]
		});

		let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &uniform_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &uniform_buffer,
						offset: 0,
						size: Some(NonZeroU64::new(UNIFORM_SIZE as u64).unwrap()),
					}),
				},
			],
		});

		//  Done  //

		let uniforms = Uniforms {
			variables: [0.0; 16],
			resolution: size.into(),
			bounds_min: (-0.0, -0.0),
			bounds_max: ( 0.0,  0.0),
			shading_intensity: 0.0,
			contour_intensity: 0.0,
			decorations: 0,
			coloring: 0,
			_padding: [0; 8],
		};

		Self {
			uniforms,
			surface,
			config,
			device,
			render_pipeline: None,
			uniform_bind_group,
			uniform_layout,
			uniform_buffer,
			queue,
		}
	}

	pub fn load_shaders(&mut self, userdefs: &str) {
		//  Shaders  //
		let vertex_src = include_str!("vertex.wgsl").to_owned();
		let fragment_src = include_str!("fragment.wgsl").to_owned() + userdefs;

		let vertex_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(vertex_src.into())
		});

		let fragment_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(fragment_src.into())
		});

		let vertex = wgpu::VertexState {
			module: &vertex_module,
			entry_point: "main",
			buffers: &[]
		};

		let fragment = wgpu::FragmentState {
			module: &fragment_module,
			entry_point: "main",
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
			bind_group_layouts: &[&self.uniform_layout],
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

	pub fn redraw(&self) {
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
		let mut cursor = Cursor::new([0; UNIFORM_SIZE]);
		self.uniforms.encode(&mut cursor).unwrap();
		self.queue.write_buffer(&self.uniform_buffer, 0, &cursor.into_inner());
		self.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	pub fn resize(&mut self, size: (u32, u32)) {
		let size = (size.0.max(1).min(8192), size.1.max(1).min(8192));
		self.config.width = size.0;
		self.config.height = size.1;
		self.surface.configure(&self.device, &self.config);
		self.uniforms.resolution = size.into();
	}
}
