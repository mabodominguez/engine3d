use cgmath::num_traits::Pow;
use cgmath::prelude::*;
use cgmath::*;
use rand;
use std::iter;
use std::ops::Range;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
pub type Pos3 = cgmath::Point3<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

mod geom;
mod model;
mod texture;

use geom::{BBox};
use model::{DrawModel, Vertex};

mod camera;
use camera::Camera;
mod camera_control;
use camera_control::CameraController;
mod player;
use player::Player;
// mod collision;


const CHUNK_SIZE: usize = 8; // Size of lenght, width, and height of a chunk
const VOXEL_HALFWIDTH: f32 = 1.0;  // Size of a voxel (halfwidth)
const DT: f32 = 1.0 / 30.0;
const WORLD_DIMS: (usize, usize, usize) = (8,5,8); // The number of chunks that you want to load in 3D space

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    #[allow(dead_code)]
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::< InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Material { // Enumeration to determine the material of a voxel. Is useful for differntiating them
    Grass,
    Dirt,
    Iron,
}

impl Material {
    fn strength(&self) -> i32 { // Possibly useful function to determine how much time it takes to break a block
        match *self {
            Material::Grass => 1,
            Material::Dirt => 2,
            Material::Iron => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Voxel { // A voxel holds position and material info
    pub center: Pos3,
    pub material: Material,
}

impl Voxel {
    fn to_raw(&self) -> InstanceRaw { // Turns vector position into gpu-friendly data
        InstanceRaw { 
            model: (Mat4::from_translation(self.center.to_vec()) * Mat4::from_scale(VOXEL_HALFWIDTH)).into(),
        }
    }
}

pub struct Chunk{ // Array that holds the vector info. It dimensions are CHUNK_SIZE^3 
    // Holds a position and the data (which is just numbers)
    pub origin: Pos3,
    pub data:  [[[usize; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    voxel_model: model::Model,
    camera: Camera,
    camera_controller: CameraController,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    voxels: Vec<Voxel>, // Array holding all the voxels. TODO: Don't make this hold every single voxel if we're going to expand the entire world
                        // rather, it should hold the voxels to be rendered
    #[allow(dead_code)]
    voxel_buffers: Vec<wgpu::Buffer>, // Wgpu buffer vector containing buffers for each individual type of voxel (i.e grass, ore, etc.)
    depth_texture: texture::Texture,
    chunks: Vec<Chunk>, // chunks in the world (or to be rendered. TBD)
    instance_data : Vec<Vec<InstanceRaw>>,
    player: Player,
}

impl State {
    async fn new(window: &Window) -> Self {
        use rand::Rng;
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera = Camera {
            eye: (0.0, 5.0, -10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 200.0,
        };
        let window_size = window.inner_size();
        let camera_controller = CameraController::new(0.2, (window_size.width / 2) as i32, (window_size.height / 2) as i32);
        let mut player_hitbox = BBox{center: cgmath::point3(0.0, 0.0, 0.0), halfwidth: 5.0};
        let mut player = Player::new(player_hitbox);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        // let mut rng = rand::thread_rng();
        let mut chunks: Vec<Chunk> = Vec::new();
        let mut rng = rand::thread_rng();

        // Create Chunks here
        // Iterate through the world chunks, and in each chunk place a random voxel
        let vox_size  = VOXEL_HALFWIDTH*2.0; // Offset for chunks in 3D space depending on the size of the voxel size
        let mut chunk_data: [[[usize; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] = [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]; 
        for cx in 0..WORLD_DIMS.0{
            for cy in 0..WORLD_DIMS.1{
                for cz in 0..WORLD_DIMS.2{
                    for x in 0..CHUNK_SIZE {
                        for y in 0..CHUNK_SIZE {
                            for z in 0..CHUNK_SIZE {
                                chunk_data[x][y][z] = rng.gen_range::<usize, Range<usize>>(0..3);

                            }
                        }
                    }
                    chunks.push(Chunk{
                        origin: Pos3::new(cx as f32 * CHUNK_SIZE as f32 *  vox_size, cy as f32 * CHUNK_SIZE as f32 * -1.0 *  vox_size, 
                                          cz as f32 * CHUNK_SIZE as f32 *  vox_size),
                        data: chunk_data
                    });
                }
            }
        } 
        
        let mut voxels: Vec<Voxel> = Vec::new();
        // Turn every chunk data into a voxel. TODO: this could be made faster by just ditching the Voxel struct all together
        for i in 0..chunks.len(){
            voxels.append( &mut voxels_from_chunk(&chunks[i]));
        }

        let res_dir = std::path::Path::new(env!("OUT_DIR")).join("content");
        // Create voxel model struct. This is a simple cube that's used as base for every voxel 
        let voxel_model = model::Model::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            res_dir.join("cube.obj"),
        )
        .unwrap();


        // Individual data arrays that hold data about each material 
        let mut instance_data : Vec<Vec<InstanceRaw>> = Vec::new();
        for _ in 0..voxel_model.materials.len() {
            instance_data.push(Vec::new());
        }

        for i in 0..voxels.len(){
            match voxels[i].material {
                Material::Grass => instance_data[0].push(voxels[i].to_raw()),
                Material::Dirt => instance_data[1].push(voxels[i].to_raw()),
                Material::Iron => instance_data[2].push(voxels[i].to_raw()),
            }
        }

        // Push data into unique buffers so we know what material to use
        // TODO: This might not be super necessary, because the raw data is just a bunch of positions
        let mut voxel_buffers: Vec<wgpu::Buffer> = Vec::new();
        for i in 0..instance_data.len() {
            voxel_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some( &i.to_string()),
                    contents: bytemuck::cast_slice(&instance_data[i]),
                    usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                }),
            );
        }   

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // Setting this to true requires Features::DEPTH_CLAMPING
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            voxel_model,
            camera,
            camera_controller,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            voxels,
            voxel_buffers,
            depth_texture,
            chunks,
            instance_data,
            player,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.camera.aspect = self.sc_desc.width as f32 / self.sc_desc.height as f32;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        //TODO shift plane
        self.player.process_events(event);
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.player.update(&mut self.camera);
        self.camera_controller.update_camera(&mut self.camera);
        // we ~could~ move the plane, or we could just tweak gravity.
        // this time we'll move the plane.
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        // Update buffers based on dynamics
        
        // TODO: This is a repeated line of code, but I needed to have it here to make sure everything works
        //       We could just put the data into the struct so we avoid iterating a whole bunch of times
        //       The only thing that worries me is how we're going to handle updating voxels in a single chunk efficiently
        //       In the future, we could just update data on the current chunk that the player is standing every frame, while we don't worry
        //       about chunks that are further away
                // Individual data arrays that hold data about each material 

        
        // Add buffers to the queue
        for i in 0..self.instance_data.len(){
            self.queue
            .write_buffer(&self.voxel_buffers[i], 0, bytemuck::cast_slice(&self.instance_data[i]));
        }



        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);

            // Render each voxel buffer, passing information about the material that we're using (the last argument in the function call)
            // Materials info is stored in  "cube.mtl"
            for i in 0..self.instance_data.len(){
                render_pass.set_vertex_buffer(1, self.voxel_buffers[i].slice(..));
                render_pass.draw_voxels(
                    &self.voxel_model,
                    0..self.instance_data[i].len() as u32,
                    &self.uniform_bind_group,
                    i,
                );
            }
        }

        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

fn main() {

    use std::time::Instant;
    env_logger::init();
    
    let event_loop = EventLoop::new();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();
    // grab cursor for camera
    let _window_grab = window.set_cursor_grab(true);
    window.set_cursor_icon(winit::window::CursorIcon::Crosshair);
    use futures::executor::block_on;
    let mut state = block_on(State::new(&window));

    // How many frames have we simulated?
    let mut frame_count: usize = 0;
    // How many unsimulated frames have we saved up?
    let mut available_time: f32 = 0.0;
    let mut since = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => {}
                        },
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                let _window_set_cursor = window.set_cursor_position(winit::dpi::PhysicalPosition::new(state.camera_controller.center_x, state.camera_controller.center_y));
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
                // The renderer "produces" time...
                available_time += since.elapsed().as_secs_f32();
                since = Instant::now();
            }
            _ => {}
        }
        // And the simulation "consumes" it
        while available_time >= DT {
            // Eat up one frame worth of time
            available_time -= DT;

            state.update();

            // Increment the frame counter
            //frame_count += 1;
        }
    });
}

// Function to create voxels from the info matrix in a chunk
pub fn voxels_from_chunk(chunk: & Chunk) -> Vec<Voxel>{
    let mut voxels: Vec<Voxel> = vec!();
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let x_pos = (x as f32 * VOXEL_HALFWIDTH/0.5 ) + chunk.origin.x;
                let y_pos = (y as f32 * VOXEL_HALFWIDTH/0.5 ) + chunk.origin.y;
                let z_pos = (z as f32 * VOXEL_HALFWIDTH/0.5 ) + chunk.origin.z;
                let material = match chunk.data[x][y][z] {
                    0 => Material::Dirt,
                    1 => Material::Iron,
                    _ => Material::Grass
                };
                voxels.push(
                    Voxel {
                        center: Pos3::new(x_pos, y_pos, z_pos),
                        material,
                    }
                )
            }
        }
    }
    return  voxels
}