use crate::assets::{Asset2d, Assets, Object2d};
use crate::camera::Camera;
use crate::camera_control::CameraController;
use crate::model::*;
use crate::texture::Texture;
use crate::voxel::{Chunk, Material, Voxel};
use crate::Game;
use cgmath::num_traits::Pow;
use cgmath::prelude::*;
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
pub type Pos2 = cgmath::Point2<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

// TODO: make these parameters?
const CHUNK_SIZE: usize = 8; // Size of lenght, width, and height of a chunk
const VOXEL_HALFWIDTH: f32 = 1.0; // Size of a voxel (halfwidth)
const DT: f32 = 1.0 / 30.0;
const WORLD_DIMS: (usize, usize, usize) = (8, 5, 8); // The number of chunks that you want to load in 3D space
const HOTBAR_HEIGHT: f32 = 0.0;
const HOTBAR_WIDTH: f32 = 0.0;
// tl, bl, tr, br
const HOTBAR_VERTS: &[VertexTwoD] = &[
    VertexTwoD {
        position: [-0.9, -0.5], // make 0s -1s (x and y go from -1 to 1)
        tex_coords: [0.0, 0.0],
    },
    VertexTwoD {
        position: [-0.9, -0.9],
        tex_coords: [0.0, 1.0],
    },
    VertexTwoD {
        position: [0.9, -0.5],
        tex_coords: [1.0, 0.0],
    },
    VertexTwoD {
        position: [0.9, -0.9],
        tex_coords: [1.0, 1.0],
    },
];

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Copy, Clone)]
pub struct TwoDID(usize, usize, pub bool);

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
pub(crate) struct InstanceRaw {
    #[allow(dead_code)]
    pub(crate) model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
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
pub struct Render {
    surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_layout: wgpu::BindGroupLayout,
    pub(crate) camera: Camera,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    voxels: Vec<Voxel>,
    voxel_model: Model, // Array holding all the voxels. TODO: Don't make this hold every single voxel if we're going to expand the entire world
    // rather, it should hold the voxels to be rendered
    #[allow(dead_code)]
    voxel_buffers: Vec<wgpu::Buffer>, // Wgpu buffer vector containing buffers for each individual type of voxel (i.e grass, ore, etc.)
    depth_texture: Texture,
    chunks: Vec<Chunk>, // chunks in the world (or to be rendered. TBD)
    instance_data: Vec<Vec<InstanceRaw>>,
    buffers_2d: Vec<wgpu::Buffer>,
    bind_groups_2d: Vec<wgpu::BindGroup>,
    pub objects_2d: Vec<TwoDID>,
    hotbar_buffer: wgpu::Buffer,
    hotbar_bind_group: wgpu::BindGroup,
    render_2d_pipeline: wgpu::RenderPipeline,
}

impl Render {
    pub(crate) async fn new(window: &Window) -> Self {
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
        let camera_controller = CameraController::new(0.2, 0, 0);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        use rand::Rng;
        // let mut rng = rand::thread_rng();
        let mut chunks: Vec<Chunk> = Vec::new();
        let mut rng = rand::thread_rng();

        // Create Chunks here
        // Iterate through the world chunks, and in each chunk place a random voxel
        let vox_size = VOXEL_HALFWIDTH * 2.0; // Offset for chunks in 3D space depending on the size of the voxel size
        let mut chunk_data: [[[usize; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] =
            [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        for cx in 0..WORLD_DIMS.0 {
            for cy in 0..WORLD_DIMS.1 {
                for cz in 0..WORLD_DIMS.2 {
                    for x in 0..CHUNK_SIZE {
                        for y in 0..CHUNK_SIZE {
                            for z in 0..CHUNK_SIZE {
                                chunk_data[x][y][z] = rng.gen_range::<usize, Range<usize>>(0..3);
                            }
                        }
                    }
                    chunks.push(Chunk {
                        origin: Pos3::new(
                            cx as f32 * CHUNK_SIZE as f32 * vox_size,
                            cy as f32 * CHUNK_SIZE as f32 * -1.0 * vox_size,
                            cz as f32 * CHUNK_SIZE as f32 * vox_size,
                        ),
                        data: chunk_data,
                    });
                }
            }
        }
        let mut voxels: Vec<Voxel> = Vec::new();
        // Turn every chunk data into a voxel. TODO: this could be made faster by just ditching the Voxel struct all together
        for i in 0..chunks.len() {
            voxels.append(&mut Chunk::voxels_from_chunk(&chunks[i]));
        }

        let res_dir = std::path::Path::new(env!("OUT_DIR")).join("content");
        // Create voxel model struct. This is a simple cube that's used as base for every voxel
        let voxel_model = Model::load(
            &device,
            &queue,
            &texture_bind_group_layout,
            res_dir.join("cube.obj"),
        )
        .unwrap();

        // Individual data arrays that hold data about each material
        let mut instance_data: Vec<Vec<InstanceRaw>> = Vec::new();
        for _ in 0..voxel_model.materials.len() {
            instance_data.push(Vec::new());
        }
        let buffers_2d = vec![];
        let bind_groups_2d = vec![];
        let objects_2d = vec![];
        for i in 0..voxels.len() {
            match voxels[i].material {
                Material::Grass => instance_data[0].push(voxels[i].to_raw()),
                Material::Dirt => instance_data[1].push(voxels[i].to_raw()),
                Material::Iron => instance_data[2].push(voxels[i].to_raw()),
            }
        }
        let hotbar_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(HOTBAR_VERTS),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));
        let vs_2d_module = device.create_shader_module(&wgpu::include_spirv!("shader2d.vert.spv"));

        let render_2d_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render 2d Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_2d_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render 2d Pipeline"),
            layout: Some(&render_2d_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_2d_module,
                entry_point: "main",
                buffers: &[VertexTwoD::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &fs_module, // can use same fs module
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    // 4.
                    format: sc_desc.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip, // TODO: how does this change for a rectangle?
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: wgpu::CullMode::None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // Setting this to true requires Features::DEPTH_CLAMPING
                clamp_depth: false,
            }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
        });

        // Push data into unique buffers so we know what material to use
        // TODO: This might not be super necessary, because the raw data is just a bunch of positions
        let mut voxel_buffers: Vec<wgpu::Buffer> = Vec::new();
        for i in 0..instance_data.len() {
            voxel_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&i.to_string()),
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
        let diffuse_path = "hotbar.png";
        let diffuse_texture = Texture::load(&device, &queue, res_dir.join(diffuse_path)).unwrap();

        let hotbar_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("hotbar_bind_group"),
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
        // let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));

        let depth_texture = Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render 3d Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
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
                format: Texture::DEPTH_FORMAT,
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
            texture_layout: texture_bind_group_layout,
            camera,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            voxels,
            voxel_model,
            voxel_buffers,
            depth_texture,
            chunks,
            instance_data,
            buffers_2d,
            bind_groups_2d,
            objects_2d,
            hotbar_buffer,
            hotbar_bind_group,
            render_2d_pipeline,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.camera.aspect = self.sc_desc.width as f32 / self.sc_desc.height as f32;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
    }

    /// Use to set up 2d objects to be drawn
    pub fn set_2d_buffers(&mut self, objects_2d: &Vec<Object2d>) -> Vec<TwoDID> {
        // self.buffers_2d.clear();
        // re use buffers ... have add/remove 2d funtion called from update
        let mut ids = vec![];
        for object in objects_2d {
            let buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&object.verts),
                    usage: wgpu::BufferUsage::VERTEX,
                });
            self.buffers_2d.push(buffer);
            self.objects_2d
                .push(TwoDID(self.objects_2d.len(), object.bg, object.visible));
            ids.push(TwoDID(self.objects_2d.len(), object.bg, object.visible));
        }
        ids
    }

    /// Use to update a 2d buffer
    pub fn update_2d_buffer(&mut self, object: &Object2d, object_id: TwoDID) {
        self.queue.write_buffer(
            &self.buffers_2d[object_id.0],
            0,
            bytemuck::cast_slice(&object.verts),
        );
    }

    /// Use to set up all the textures to be drawn
    pub fn set_2d_bind_groups(&mut self, assets_2d: &Vec<Asset2d>) {
        // self.bind_groups_2d.clear();
        for asset in assets_2d {
            let diffuse_texture = Texture::load(&self.device, &self.queue, &asset.0).unwrap();

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.texture_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
                label: Some("bind_group"), // change to be dependent
            });
            self.bind_groups_2d.push(bind_group);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        true
        //TODO shift plane
        // self.camera_controller.process_events(event)
    }

    pub(crate) fn render<R, G: Game<StaticData = R>>(
        &mut self,
        game: &mut G,
        rules: &R,
        assets: &mut Assets,
    ) -> Result<(), wgpu::SwapChainError> {
        // Update buffers based on dynamics
        // TODO: This is a repeated line of code, but I needed to have it here to make sure everything works
        //       We could just put the data into the struct so we avoid iterating a whole bunch of times
        //       The only thing that worries me is how we're going to handle updating voxels in a single chunk efficiently
        //       In the future, we could just update data on the current chunk that the player is standing every frame, while we don't worry
        //       about chunks that are further away
        // Individual data arrays that hold data about each material

        // Add buffers to the queue
        for i in 0..self.instance_data.len() {
            self.queue.write_buffer(
                &self.voxel_buffers[i],
                0,
                bytemuck::cast_slice(&self.instance_data[i]),
            );
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
            for i in 0..self.instance_data.len() {
                render_pass.set_vertex_buffer(1, self.voxel_buffers[i].slice(..));
                render_pass.draw_voxels(
                    &self.voxel_model,
                    0..self.instance_data[i].len() as u32,
                    &self.uniform_bind_group,
                    i,
                );
            }
            // set 2d pipeline, make sure texture is updated, provide a texture bindgroup
            // call draw on what vertices to draw

            render_pass.set_pipeline(&self.render_2d_pipeline);
            for object in &self.objects_2d {
                // if visible, draw it
                if object.2 {
                    render_pass.set_vertex_buffer(0, self.buffers_2d[object.0].slice(..));
                    render_pass.set_bind_group(0, &self.bind_groups_2d[object.1], &[]);
                    render_pass.draw(0..4 as u32, 0..1);
                }
            }
        }

        self.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}
