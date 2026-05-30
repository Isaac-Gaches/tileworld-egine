use std::mem;
use easy_gpu::assets::{Buffer, BufferLayout, BufferUsages, GpuInstance, GpuVertex, Material, MaterialBuilder, Mesh, render_uniform, RenderPipelineBuilder};
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use easy_gpu::wgpu::{TextureFormat, VertexFormat, VertexStepMode};
use crate::engine::render::MeshVertex;

pub struct Sky{
    light_colour: [f32;3],
    time: f32,
    quad: Handle<Mesh>,

    sky_material: Handle<Material>,
    sky_uniform: SkyUniform,
    sky_buffer: Handle<Buffer>,

    nebular_material: Handle<Material>,

    star_material: Handle<Material>,
    star_uniform: Handle<Buffer>,
    star_buffer: Handle<Buffer>,
}

impl Sky{
    pub fn new(egpu: &mut easy_gpu::Renderer)-> Self {
        let sky_shader = egpu.load_shader(include_str!("shaders/sky.wgsl"));
        let sky_pipeline = RenderPipelineBuilder::new(sky_shader)
            .depth_writes_enabled(false)
            .depth_format(TextureFormat::Depth24Plus)
            .vertex_layout(SkyVertex::buffer_layout())
            .material_layout(&[render_uniform(0)])
            .build(egpu);

        let sky_buffer = egpu.create_buffer_with_contents(
            BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            bytemuck::cast_slice(&[SkyUniform::new()])
        );
        let sky_material = MaterialBuilder::new(sky_pipeline)
            .buffer(0,sky_buffer)
            .build(egpu);

        let vertices = [
            SkyVertex::new([-1.,-1.]),
            SkyVertex::new([1.,-1.]),
            SkyVertex::new([1.,1.]),
            SkyVertex::new([-1.,1.]),
        ];

        let indices = [0, 1, 2, 0, 2, 3];

        let quad = egpu.create_mesh(&vertices, &indices);

        let shader = egpu.load_shader(include_str!("shaders/stars.wgsl"));
        let pipeline = RenderPipelineBuilder::new(shader)
            .depth_writes_enabled(false)
            .depth_format(TextureFormat::Depth24Plus)
            .vertex_layout(SkyVertex::buffer_layout())
            .vertex_layout(Star::buffer_layout())
            .material_layout(&[render_uniform(0)])
            .additive_alpha_blending()
            .build(egpu);

        let star_uniform = egpu.create_buffer(
            BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            size_of::<StarUniform>() as u64
        );
        let star_material = MaterialBuilder::new(pipeline)
            .buffer(0,star_uniform)
            .build(egpu);

        let stars = (0..1000).map(|i|{
            Star{
                position: [rand::random_range(-1.0..1.0),rand::random_range(-1.0..1.0)],
            }
        }).collect::<Vec<Star>>();

        let nebular_pipeline  = RenderPipelineBuilder::new(sky_shader)
            .depth_writes_enabled(false)
            .depth_format(TextureFormat::Depth24Plus)
            .vertex_layout(SkyVertex::buffer_layout())
            .material_layout(&[render_uniform(0)])
            .fs_entry_point("nebular_fs")
            .additive_alpha_blending()
            .build(egpu);

        let nebular_material = MaterialBuilder::new(nebular_pipeline)
            .buffer(0,sky_buffer)
            .build(egpu);

        let star_buffer = egpu.create_buffer_with_contents(
          BufferUsages::COPY_DST | BufferUsages::VERTEX,
          bytemuck::cast_slice(&stars),
        );

        Self{
            light_colour: [1.0,1.0,1.0],
            time: 0.71,
            sky_material,
            quad,
            sky_uniform: SkyUniform::new(),
            sky_buffer,
            nebular_material,
            star_material,
            star_uniform,
            star_buffer,
        }
    }

    pub fn update(&mut self,egpu: &mut easy_gpu::Renderer,sky_light_buffer: Handle<Buffer>,dt: f32) {
        self.time += dt * 0.002;
        self.time = self.time % 1.0;

        self.blend_sky();
        self.sky_uniform.set_time(self.time);

        egpu.write_buffer(self.sky_buffer, self.sky_uniform);
        egpu.write_buffer(sky_light_buffer, self.light_colour);
        egpu.write_buffer(self.star_uniform,StarUniform{time: self.time});
    }

    fn blend_sky(&mut self){
        const DAY_TOP: [f32;3]      =[0.22, 0.55, 0.95];
        const DAY_HORIZON: [f32;3]  = [0.65, 0.80, 1.00];
        const DAY_LIGHT: [f32;3] = [1.0,1.0,1.0];
        const DAY_CLOUD_MAIN: [f32;3] = [1.0,1.0,1.0];
        const DAY_CLOUD_EDGE: [f32;3] = [0.3,0.7,1.0];

        const LOW_SUN_TOP: [f32;3]   = [0.5, 0.05, 0.20];
        const LOW_SUN_HORIZON: [f32;3] = [0.7, 0.3, 0.10];
        const LOW_SUN_LIGHT: [f32;3] = [0.8,0.4,0.1];
        const LOW_SUN_CLOUD_MAIN: [f32;3] = [1.0,0.4,0.8];
        const LOW_SUN_CLOUD_EDGE: [f32;3] = [0.6,0.2,0.1];

        const NIGHT_TOP: [f32;3]    = [0., 0., 0.];
        const NIGHT_HORIZON: [f32;3] = [0.002, 0.001, 0.004];
        const NIGHT_LIGHT: [f32;3] = [0.01,0.01,0.01];
        const NIGHT_CLOUD_MAIN: [f32;3] =  [0.08,0.08,0.1];
        const NIGHT_CLOUD_EDGE: [f32;3] = [0.05,0.05,0.1];

        const DAWN_START: f32 = 0.15;
        const SUNRISE_END: f32 = 0.25;
        const DAY_START: f32 = 0.3;
        const SUNSET_START: f32 = 0.7;
        const DUSK_START: f32 = 0.75;
        const NIGHT_START: f32 = 0.85;

        let colours = if self.time < DAWN_START {
            // Full night
            (
                NIGHT_TOP,
                NIGHT_HORIZON,
                NIGHT_LIGHT,
                NIGHT_CLOUD_MAIN,
                NIGHT_CLOUD_EDGE
            )
        }
        else if self.time < SUNRISE_END {
            // Night -> Sunrise

            let t = (self.time - DAWN_START)
                / (SUNRISE_END - DAWN_START);

            (
                lerp(NIGHT_TOP, LOW_SUN_TOP, t),
                lerp(NIGHT_HORIZON, LOW_SUN_HORIZON, t),
                lerp(NIGHT_LIGHT, LOW_SUN_LIGHT, t),
                lerp(NIGHT_CLOUD_MAIN, LOW_SUN_CLOUD_MAIN, t),
                lerp(NIGHT_CLOUD_EDGE, LOW_SUN_CLOUD_EDGE, t),
            )
        }
        else if self.time < DAY_START {
            // Sunrise -> Day

            let t = (self.time - SUNRISE_END)
                / (DAY_START - SUNRISE_END);

            (
                lerp(LOW_SUN_TOP, DAY_TOP, t),
                lerp(LOW_SUN_HORIZON, DAY_HORIZON, t),
                lerp(LOW_SUN_LIGHT, DAY_LIGHT, t),
                lerp(LOW_SUN_CLOUD_MAIN, DAY_CLOUD_MAIN, t),
                lerp(LOW_SUN_CLOUD_EDGE, DAY_CLOUD_EDGE, t),
            )
        }
        else if self.time < SUNSET_START {
            // Full day
            (
                DAY_TOP,
                DAY_HORIZON,
                DAY_LIGHT,
                DAY_CLOUD_MAIN,
                DAY_CLOUD_EDGE,
            )
        }
        else if self.time < DUSK_START {
            // Day -> Sunset

            let t = (self.time - SUNSET_START)
                / (DUSK_START - SUNSET_START);

            (
                lerp(DAY_TOP, LOW_SUN_TOP, t),
                lerp(DAY_HORIZON, LOW_SUN_HORIZON, t),
                lerp(DAY_LIGHT, LOW_SUN_LIGHT, t),
                lerp(DAY_CLOUD_MAIN, LOW_SUN_CLOUD_MAIN, t),
                lerp(DAY_CLOUD_EDGE, LOW_SUN_CLOUD_EDGE, t),
            )
        }
        else if self.time < NIGHT_START {
            // Sunset -> Night

            let t = (self.time - DUSK_START)
                / (NIGHT_START - DUSK_START);

            (
                lerp(LOW_SUN_TOP, NIGHT_TOP, t),
                lerp(LOW_SUN_HORIZON, NIGHT_HORIZON, t),
                lerp(LOW_SUN_LIGHT, NIGHT_LIGHT, t),
                lerp(LOW_SUN_CLOUD_MAIN, NIGHT_CLOUD_MAIN, t),
                lerp(LOW_SUN_CLOUD_EDGE, NIGHT_CLOUD_EDGE, t),
            )
        }
        else {
            // Full night
            (
                NIGHT_TOP,
                NIGHT_HORIZON,
                NIGHT_LIGHT,
                NIGHT_CLOUD_MAIN,
                NIGHT_CLOUD_EDGE,
            )
        };

        self.light_colour = colours.2;

        self.sky_uniform.set_sky_colour(
            colours.0,
            colours.1,
        );

        self.sky_uniform.set_cloud_colour(
            colours.3,
            colours.4,
        );
    }

    pub fn draw(&self,frame: &mut Frame){
        frame.draw(
            self.sky_material,
            self.quad
        );

        frame.draw(
            self.nebular_material,
            self.quad
        );

        frame.draw_manual_batch(
            vec![self.star_buffer],
            self.star_material,
            self.quad,
            0..1000
        );
    }
}
fn lerp(a: [f32;3], b: [f32;3], t: f32) -> [f32;3] {
    [
        a[0]+(b[0]-a[0]) * t,
        a[1]+(b[1]-a[1]) * t,
        a[2]+(b[2]-a[2]) * t,
    ]
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyUniform{
    time: f32,
    _pad0: [f32; 3],
    top_colour: [f32;3],
    _pad1:f32,
    bottom_colour: [f32;3],
    _pad2:f32,
    cloud_main: [f32;3],
    _pad3: f32,
    cloud_edge: [f32;3],
    _pad4: f32,
}
impl SkyUniform{
    fn new() -> Self{
        Self{
            top_colour: [0.,0.,0.],
            _pad1: 0.0,
            bottom_colour: [0.,0.,0.],
            _pad2: 0.0,
            cloud_main: [0.0,0.0,0.0],
            _pad3: 0.0,
            cloud_edge: [0.,0.,0.],
            time: 0.0,
            _pad0: [0.,0.,0.],
            _pad4: 0.0,
        }
    }
    fn set_sky_colour(&mut self,top_colour:[f32;3],bottom_colour:[f32;3]){
        self.top_colour = top_colour;
        self.bottom_colour = bottom_colour;
    }

    fn set_cloud_colour(&mut self, main: [f32;3], edge: [f32;3]){
        self.cloud_main = main;
        self.cloud_edge = edge;
    }

    fn set_time(&mut self,time: f32){
        self.time = time;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyVertex{
    position: [f32;2],
}

impl SkyVertex{
    fn new(position: [f32;2])->Self{
        Self{
            position,
        }
    }
}

impl GpuVertex for SkyVertex{
    fn buffer_layout() -> BufferLayout {
        BufferLayout::new()
            .stride(size_of::<SkyVertex>() as u64)
            .attribute(0,0,VertexFormat::Float32x2)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct StarUniform{
    time: f32
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Star {
    position: [f32;2],
}

impl GpuInstance for Star {
    fn buffer_layout() -> BufferLayout {
        BufferLayout::new()
            .stride(size_of::<Self>() as u64)
            .step_mode(VertexStepMode::Instance)
            .attribute(1,0,VertexFormat::Float32x2)
    }
}