use ahash::AHashMap;
use easy_gpu::assets::{Buffer, BufferUsages, compute_texture_float, compute_texture_uint, ComputeBindGroup, ComputeBindGroupBuilder, ComputePipeline, ComputePipelineBuilder, RenderPipeline, Sampler, SamplerBuilder, storage_texture, Texture, TextureBuilder, compute_uniform, compute_storage};
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use easy_gpu::wgpu::{Extent3d, FilterMode, TextureFormat, TextureUsages};
use easy_gpu::wgpu::TextureFormat::{Rgba16Float, Rgba8Unorm};
use crate::game::terrain::chunk::CHUNK_SIZE;
use crate::game::terrain::chunk_manager::{HORIZONTAL_CHUNK_LOAD_DISTANCE, VERTICAL_CHUNK_LOAD_DISTANCE};

pub struct LightingEngine{
    pub smooth_texture_a: Handle<Texture>,
    smooth_texture_b: Handle<Texture>,
    diffuse_texture_a: Handle<Texture>,
    diffuse_texture_b: Handle<Texture>,
    pub occlusion_texture: Handle<Texture>,
    tile_storage_texture: Handle<Texture>,

    smooth_pipeline: Handle<ComputePipeline>,
    diffuse_pipeline: Handle<ComputePipeline>,
    set_sky_light_pipeline: Handle<ComputePipeline>,
    set_light_sources_pipeline: Handle<ComputePipeline>,
    occlusion_pipeline: Handle<ComputePipeline>,
    upscale_pipeline: Handle<ComputePipeline>,

    smooth_bg_a_to_b: Handle<ComputeBindGroup>,
    smooth_bg_b_to_a: Handle<ComputeBindGroup>,
    diffuse_bg_a_to_b: Handle<ComputeBindGroup>,
    diffuse_bg_b_to_a: Handle<ComputeBindGroup>,
    set_sky_light_bg: Handle<ComputeBindGroup>,
    set_light_sources_bg: Handle<ComputeBindGroup>,
    occlusion_bg: Handle<ComputeBindGroup>,
    upscale_bg: Handle<ComputeBindGroup>,

    pub light_sampler: Handle<Sampler>,
    pub light_uniform: Handle<Buffer>,
    light_meta: LightMeta,

    pub sky_light: Handle<Buffer>,

    dynamic_lights_meta: Handle<Buffer>,
    lights_buffer: Handle<Buffer>,
    pub lights: Vec<LightSource>,
    num_lights: u32
}

impl LightingEngine{
    pub fn new(egpu: &mut easy_gpu::Renderer) -> Self{
        let diffuse_texture_builder = TextureBuilder::new()
            .size(Extent3d{
                width: HORIZONTAL_CHUNK_LOAD_DISTANCE as u32*CHUNK_SIZE as u32*2 + CHUNK_SIZE as u32,
                height: VERTICAL_CHUNK_LOAD_DISTANCE as u32*CHUNK_SIZE as u32*2 + CHUNK_SIZE as u32,
                depth_or_array_layers: 1,
            })
            .format(Rgba16Float)
            .usage(TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT);

        let diffuse_texture_a = diffuse_texture_builder.build(egpu);
        let diffuse_texture_b = diffuse_texture_builder.build(egpu);

        let occlusion_texture_builder = diffuse_texture_builder
            .format(Rgba8Unorm);

        let occlusion_texture = occlusion_texture_builder.build(egpu);

        let smooth_texture_builder = occlusion_texture_builder
            .size(Extent3d{
                width: HORIZONTAL_CHUNK_LOAD_DISTANCE as u32*CHUNK_SIZE as u32*4 + CHUNK_SIZE as u32 * 2,
                height: VERTICAL_CHUNK_LOAD_DISTANCE as u32*CHUNK_SIZE as u32*4 + CHUNK_SIZE as u32 * 2,
                depth_or_array_layers: 1,
            })
            .format(Rgba16Float);

        let smooth_texture_a = smooth_texture_builder.build(egpu);
        let smooth_texture_b = smooth_texture_builder.build(egpu);

        let tile_storage_texture = smooth_texture_builder
            .size(Extent3d{
                width: HORIZONTAL_CHUNK_LOAD_DISTANCE as u32*CHUNK_SIZE as u32*2 + CHUNK_SIZE as u32,
                height: VERTICAL_CHUNK_LOAD_DISTANCE as u32*CHUNK_SIZE as u32*2 + CHUNK_SIZE as u32,
                depth_or_array_layers: 1,
            })
            .usage(TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST)
            .format(TextureFormat::R8Uint)
            .build(egpu);

        let diffuse_shader = egpu.load_shader(include_str!("shaders/lighting/diffuse_light.wgsl"));
        let upscale_shader = egpu.load_shader(include_str!("shaders/lighting/upscale_lightmap.wgsl"));
        let smooth_shader = egpu.load_shader(include_str!("shaders/lighting/smooth_light.wgsl"));
        let occlusion_shader = egpu.load_shader(include_str!("shaders/lighting/ambient_occlusion.wgsl"));
        let set_sky_shader = egpu.load_shader(include_str!("shaders/lighting/sky_light.wgsl"));
        let set_sources_shader = egpu.load_shader(include_str!("shaders/lighting/light_sources.wgsl"));

        let diffuse_pipeline = ComputePipelineBuilder::new(diffuse_shader)
            .bind_group_layout(&[
                compute_texture_float(0),
                storage_texture(1,Rgba16Float),
                compute_texture_uint(2),
            ])
            .entry_point("diffuse_light")
            .build(egpu);

        let diffuse_bg_a_to_b = ComputeBindGroupBuilder::new(diffuse_pipeline.clone())
            .texture(0,diffuse_texture_a)
            .texture(1,diffuse_texture_b)
            .texture(2,tile_storage_texture)
            .build(egpu);

        let diffuse_bg_b_to_a = ComputeBindGroupBuilder::new(diffuse_pipeline)
            .texture(0,diffuse_texture_b)
            .texture(1,diffuse_texture_a)
            .texture(2,tile_storage_texture)
            .build(egpu);

        let sky_light = egpu.create_buffer(
            BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            32
        );

        let set_sky_light_pipeline = ComputePipelineBuilder::new(set_sky_shader)
            .bind_group_layout(&[
                compute_texture_float(0),
                storage_texture(1,Rgba16Float),
                compute_texture_uint(2),
                compute_uniform(3)
            ])
            .entry_point("set_sky_light")
            .build(egpu);

        let set_sky_light_bg = ComputeBindGroupBuilder::new(set_sky_light_pipeline)
            .texture(0,diffuse_texture_b)
            .texture(1,diffuse_texture_a)
            .texture(2,tile_storage_texture)
            .buffer(3,sky_light)
            .build(egpu);

        let set_light_sources_pipeline = ComputePipelineBuilder::new(set_sources_shader)
            .bind_group_layout(&[
                storage_texture(0,Rgba16Float),
                compute_storage(1,true),
                compute_uniform(2)
            ])
            .entry_point("set_light_sources")
            .build(egpu);


        let lights_buffer = egpu.create_buffer(
            BufferUsages::COPY_DST | BufferUsages::STORAGE,
            2_u64.pow(18)
        );

        let dynamic_lights_meta = egpu.create_buffer(
            BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            size_of::<DynamicLightMeta>() as u64
        );

        let set_light_sources_bg = ComputeBindGroupBuilder::new(set_light_sources_pipeline)
            .texture(0,diffuse_texture_b)
            .buffer(1,lights_buffer)
            .buffer(2,dynamic_lights_meta)
            .build(egpu);

        let smooth_pipeline = ComputePipelineBuilder::new(smooth_shader)
            .bind_group_layout(&[
                compute_texture_float(0),
                storage_texture(1,TextureFormat::Rgba16Float)
            ])
            .entry_point("smooth_light")
            .build(egpu);

        let smooth_bg_a_to_b = ComputeBindGroupBuilder::new(smooth_pipeline)
            .texture(0,smooth_texture_a)
            .texture(1,smooth_texture_b)
            .build(egpu);

        let smooth_bg_b_to_a = ComputeBindGroupBuilder::new(smooth_pipeline)
            .texture(0,smooth_texture_b)
            .texture(1,smooth_texture_a)
            .build(egpu);

        let upscale_pipeline = ComputePipelineBuilder::new(upscale_shader)
            .bind_group_layout(&[
                compute_texture_float(0),
                storage_texture(1,TextureFormat::Rgba16Float),
            ])
            .entry_point("upscale_lightmap")
            .build(egpu);

        let upscale_bg = ComputeBindGroupBuilder::new(upscale_pipeline)
            .texture(0,diffuse_texture_a)
            .texture(1,smooth_texture_a)
            .build(egpu);

        let occlusion_pipeline = ComputePipelineBuilder::new(occlusion_shader)
            .bind_group_layout(&[
                compute_texture_uint(0),
                storage_texture(1,Rgba8Unorm),
            ])
            .entry_point("set_occlusion_map")
            .build(egpu);

        let occlusion_bg = ComputeBindGroupBuilder::new(occlusion_pipeline)
            .texture(0,tile_storage_texture)
            .texture(1,occlusion_texture)
            .build(egpu);

        let light_sampler = SamplerBuilder::new()
            .filter_mode(FilterMode::Linear)
            .build(egpu);

        let light_uniform = egpu.create_buffer(
            BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            size_of::<LightMeta>() as u64
        );

        let light_meta = LightMeta::new();

        Self{
            smooth_texture_a,
            smooth_texture_b,
            diffuse_texture_a,
            diffuse_texture_b,
            occlusion_texture,
            tile_storage_texture,
            smooth_pipeline,
            diffuse_pipeline,
            set_sky_light_pipeline,
            set_light_sources_pipeline,
            occlusion_pipeline,
            upscale_pipeline,
            smooth_bg_a_to_b,
            smooth_bg_b_to_a,
            diffuse_bg_a_to_b,
            diffuse_bg_b_to_a,
            set_sky_light_bg,
            set_light_sources_bg,
            occlusion_bg,
            upscale_bg,
            light_sampler,
            light_uniform,
            light_meta,
            sky_light,
            dynamic_lights_meta,
            lights_buffer,
            lights: Vec::new(),
            num_lights: 0,
        }
    }
    
    pub fn update(&mut self, egpu: &mut easy_gpu::Renderer,tiles: Vec<u8>,player_pos: [f32;2]){
        egpu.write_texture(self.tile_storage_texture,tiles.as_slice(),1,Extent3d{
            width: (HORIZONTAL_CHUNK_LOAD_DISTANCE*CHUNK_SIZE as i32) as u32 * 2+ CHUNK_SIZE as u32,
            height: (VERTICAL_CHUNK_LOAD_DISTANCE*CHUNK_SIZE as i32) as u32 * 2+ CHUNK_SIZE as u32,
            depth_or_array_layers: 1,
        });

        self.light_meta.pos = [
            (player_pos[0]/CHUNK_SIZE as f32).floor()*CHUNK_SIZE as f32,
            (player_pos[1]/CHUNK_SIZE as f32).floor()*CHUNK_SIZE as f32
        ];
        egpu.write_buffer(self.light_uniform,self.light_meta);

        egpu.write_buffer(self.dynamic_lights_meta,DynamicLightMeta{
            pos: self.light_meta.pos,
            light_count: self.lights.len() as u32,
            pad: 0.0,
        });

        egpu.write_array_buffer(self.lights_buffer,self.lights.as_slice());
        self.num_lights = self.lights.len() as u32;

        self.lights.clear();
    }

    pub fn compute(&self, frame: &mut Frame){
        frame.request_texture_clear(self.diffuse_texture_a);
        frame.request_texture_clear(self.diffuse_texture_b);

        let mut pixels = (
            (HORIZONTAL_CHUNK_LOAD_DISTANCE as f32*2.*CHUNK_SIZE as f32 + CHUNK_SIZE as f32) as u32,
            (VERTICAL_CHUNK_LOAD_DISTANCE as f32*2.*CHUNK_SIZE as f32 + CHUNK_SIZE as f32) as u32,
            1
        );

        frame.compute(
            self.set_light_sources_bg,
            self.set_light_sources_pipeline,
            (self.num_lights/64 + 1, 1, 1,),
        );
        frame.compute(
            self.set_sky_light_bg,
            self.set_sky_light_pipeline,
            (pixels.0/16, pixels.1/16, pixels.2),
        );


        frame.compute(
            self.occlusion_bg,
            self.occlusion_pipeline,
            (pixels.0/16, pixels.1/16, pixels.2)
        );
        for _ in 0..12{
            frame.compute(
                self.diffuse_bg_a_to_b,
                self.diffuse_pipeline,
                (pixels.0/16, pixels.1/16, pixels.2)
            );
            frame.compute(
                self.diffuse_bg_b_to_a,
                self.diffuse_pipeline,
                (pixels.0/16, pixels.0/16, pixels.2)
            );
        }

        frame.compute(
            self.set_light_sources_bg,
            self.set_light_sources_pipeline,
            (self.num_lights/64 + 1, 1, 1,),
        );
        frame.compute(
            self.set_sky_light_bg,
            self.set_sky_light_pipeline,
            (pixels.0/16, pixels.1/16, pixels.2)
        );

        pixels = (
            pixels.0 * 2,
            pixels.1 * 2,
            1
        );

        frame.compute(
            self.upscale_bg,
            self.upscale_pipeline,
            (pixels.0/16, pixels.1/16, pixels.2)
        );
        for _ in 0..4{
            frame.compute(
                self.smooth_bg_a_to_b,
                self.smooth_pipeline,
                (pixels.0, pixels.1/16, pixels.2)
            );
            frame.compute(
                self.smooth_bg_b_to_a,
                self.smooth_pipeline,
                (pixels.0/16, pixels.1, pixels.2)
            );
        }
    }
}

pub struct Light{
    pub colour: [f32;3]
}
impl Light{
    pub fn new(colour: [f32;3]) -> Self{
        Self{
            colour,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightSource {
    pos: [f32; 2],
    _pad0: [f32; 2],
    pub colour: [f32; 3],
    _pad1: f32,
}

impl LightSource{
    pub fn new(pos: [f32;2],colour: [f32;3]) -> Self{
        Self{
            pos,
            _pad0: [0.;2],
            colour,
            _pad1: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DynamicLightMeta {
    pub pos: [f32; 2],
    pub light_count: u32,
    pad: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightMeta{
    pub pos:[f32;2],
    vertical_render_distance:f32,
    horizontal_render_distance: f32,
    chunk_size: f32,
    _pad: f32
}

impl LightMeta{
    pub fn new() -> Self{
        Self{
            pos: [0.,0.],
            vertical_render_distance: VERTICAL_CHUNK_LOAD_DISTANCE as f32 * CHUNK_SIZE as f32,
            chunk_size: CHUNK_SIZE as f32,
            horizontal_render_distance: HORIZONTAL_CHUNK_LOAD_DISTANCE as f32 * CHUNK_SIZE as f32,
            _pad: 0.0,
        }
    }
}