use super::{
  Color, ColorBufferAttr, Point, RenderCommand, Rendering2DLayer,
  TextureBufferAttr, Transform,
};
use lyon::tessellation::VertexBuffers;

#[repr(C)]
struct Vertex {
  pos: Point,
  prim_id: u32,
}

#[repr(C)]
struct ColorPrimitive {
  color: Color,
  line_width: f32,
  transform: Transform,
}

struct PureColorCommand {
  vertices: Vec<Vertex>,
  indices: Vec<u16>,
  primitives: Vec<ColorPrimitive>,
}

pub struct Canvas {
  buffer: Option<RenderCommand>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  swap_chain: wgpu::SwapChain,
  render_pipeline: wgpu::RenderPipeline,
}

impl Canvas {
  /// Create a canvas by a native window.
  pub async fn new<W: raw_window_handle::HasRawWindowHandle>(
    window: &W,
    width: u32,
    height: u32,
  ) -> Self {
    let surface = wgpu::Surface::create(window);
    let adapter = wgpu::Adapter::request(
      &wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        compatible_surface: Some(&surface),
      },
      wgpu::BackendBit::PRIMARY,
    )
    .await
    .unwrap();

    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
          anisotropic_filtering: false,
        },
        limits: Default::default(),
      })
      .await;

    let sc_desc = wgpu::SwapChainDescriptor {
      usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
      format: wgpu::TextureFormat::Bgra8UnormSrgb,
      width,
      height,
      present_mode: wgpu::PresentMode::Fifo,
    };
    let swap_chain = device.create_swap_chain(&surface, &sc_desc);
    let render_pipeline = Self::create_render_pipeline(&device, &sc_desc);

    Canvas {
      device,
      queue,
      buffer: None,
      swap_chain,
      render_pipeline,
    }
  }

  /// Create a new 2d layer to drawing, and not effect current canvas before
  /// compose back to the canvas.
  #[inline]
  pub fn new_2d_layer(&self) -> Rendering2DLayer { Rendering2DLayer::new() }

  /// Compose a layer into the canvas.
  #[inline]
  pub fn compose_2d_layer(
    &mut self,
    other_layer: Rendering2DLayer,
  ) -> &mut Self {
    self.compose_2d_layer_buffer(&other_layer.finish())
  }

  /// Compose a layer buffer into current drawing. Layer buffer is the result
  /// of a layer drawing finished.
  #[inline]
  pub fn compose_2d_layer_buffer(
    &mut self,
    commands: &Vec<RenderCommand>,
  ) -> &mut Self {
    // if the first render command is same type with last layer's last render
    // command, will merge them into one to commit.
    let mut merged = false;
    if let Some(last) = &mut self.buffer {
      if let Some(first) = commands.first() {
        merged = last.merge(first);
      }
    }

    // Skip the first command if it merged into buffer, and retain the last
    // command as new buffer to merge new layer.
    let rg = merged as usize..commands.len() - 1;
    if rg.start < rg.end {
      if let Some(cmd) = &self.buffer.take() {
        self.commit_command(cmd);
      }
      commands[rg].iter().for_each(|cmd| self.commit_command(cmd));
      self.buffer = commands.last().map(|cmd| cmd.clone());
    }

    self
  }

  /// Commit all composed layer to gpu for painting on screen.
  pub fn render(&mut self) {
    let frame = self
      .swap_chain
      .get_next_texture()
      .expect("Timeout getting texture");

    let mut encoder =
      self
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("Render Encoder"),
        });
    {
      let mut render_pass =
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
          color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &frame.view,
            resolve_target: None,
            load_op: wgpu::LoadOp::Clear,
            store_op: wgpu::StoreOp::Store,
            clear_color: wgpu::Color::WHITE,
          }],
          depth_stencil_attachment: None,
        });

      render_pass.set_pipeline(&mut self.render_pipeline);
      render_pass.draw(0..6, 0..1);
    }

    self.queue.submit(&[encoder.finish()]);
  }

  fn create_render_pipeline(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
  ) -> wgpu::RenderPipeline {
    let render_pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[],
      });

    let (vs_module, fs_module) = Self::shaders(device);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      layout: &render_pipeline_layout,
      vertex_stage: wgpu::ProgrammableStageDescriptor {
        module: &vs_module,
        entry_point: "main",
      },
      fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
        module: &fs_module,
        entry_point: "main",
      }),
      rasterization_state: Some(wgpu::RasterizationStateDescriptor {
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: wgpu::CullMode::Back,
        depth_bias: 0,
        depth_bias_slope_scale: 0.0,
        depth_bias_clamp: 0.0,
      }),
      color_states: &[wgpu::ColorStateDescriptor {
        format: sc_desc.format,
        color_blend: wgpu::BlendDescriptor::REPLACE,
        alpha_blend: wgpu::BlendDescriptor::REPLACE,
        write_mask: wgpu::ColorWrite::ALL,
      }],
      primitive_topology: wgpu::PrimitiveTopology::TriangleList,
      depth_stencil_state: None,
      vertex_state: wgpu::VertexStateDescriptor {
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[],
      },
      sample_count: 1,
      sample_mask: !0,
      alpha_to_coverage_enabled: false,
    })
  }

  fn commit_command(&mut self, command: &RenderCommand) {
    match command {
      RenderCommand::PureColor { geometry, attrs } => {
        self.commit_pure_color_command(geometry, attrs);
      }
      RenderCommand::Texture { geometry, attrs } => {
        self.commit_texture_command(geometry, attrs)
      }
    }
  }

  fn commit_pure_color_command(
    &mut self,
    geometry: &VertexBuffers<Point, u16>,
    attrs: &Vec<ColorBufferAttr>,
  ) {
    unimplemented!();
  }

  fn commit_texture_command(
    &mut self,
    geometry: &VertexBuffers<Point, u16>,
    attrs: &Vec<TextureBufferAttr>,
  ) {
    self.buffer = None;
    unimplemented!();
  }

  fn shaders(
    device: &wgpu::Device,
  ) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let vs_bytes = include_bytes!("./shaders/geometry.vert.spv");
    let fs_bytes = include_bytes!("./shaders/geometry.frag.spv");
    let vs_spv = wgpu::read_spirv(std::io::Cursor::new(&vs_bytes[..])).unwrap();
    let fs_spv = wgpu::read_spirv(std::io::Cursor::new(&fs_bytes[..])).unwrap();
    let vs_module = device.create_shader_module(&vs_spv);
    let fs_module = device.create_shader_module(&fs_spv);

    (vs_module, fs_module)
  }
}
