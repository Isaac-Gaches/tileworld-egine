use std::ops::Range;
use ahash::{AHashMap, HashMap};
use easy_gpu::assets::{Buffer, BufferLayout, BufferUsages, GpuInstance, GpuVertex, Material, MaterialBuilder, Mesh, render_storage, render_texture, render_uniform, RenderPipeline, RenderPipelineBuilder, sampler, Sampler, SamplerBuilder, Texture};
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use easy_gpu::wgpu::{BlendState, FilterMode, TextureFormat, VertexFormat, VertexStepMode};
use hecs::World;
use serde::{Deserialize, Serialize};
use crate::engine::render::{Camera, MeshVertex};
use crate::engine::render::lighting::LightingEngine;
use crate::game::physics::transform::Transform;

pub struct SpriteBatchEngine{
    sprite_batch_pipeline: Handle<RenderPipeline>,
    quad_mesh: Handle<Mesh>,
    sampler: Handle<Sampler>,
    batches: AHashMap<Handle<Material>,Vec<Instance>>,
}

impl SpriteBatchEngine{
    pub fn new(egpu: &mut easy_gpu::Renderer) -> Self{
        let scale = 0.5;
        let vertices = [
            SpriteVertex::new([-scale, -scale,0.0]),
            SpriteVertex::new([scale, -scale,0.0]),
            SpriteVertex::new([scale, scale,0.0]),
            SpriteVertex::new([-scale, scale,0.0])
        ];

        let indices = [0, 1, 2, 0, 2, 3];

        let quad_mesh = egpu.create_mesh(&vertices, &indices);

        let shader = egpu.load_shader(include_str!("shaders/sprite_batch.wgsl"));

        let sprite_batch_pipeline = RenderPipelineBuilder::new(shader)
            .material_layout(&[
                render_uniform(0),
                render_texture(1),
                sampler(2),
                render_texture(3),
                sampler(4),
                render_uniform(5),
                render_storage(6,true)
            ])
            .vertex_layout(SpriteVertex::buffer_layout())
            .vertex_layout(Instance::buffer_layout())
            .depth_format(TextureFormat::Depth24Plus)
            .blend_mode(BlendState::REPLACE)
            .build(egpu);

        let sampler = SamplerBuilder::new()
            .filter_mode(FilterMode::Nearest)
            .build(egpu);

        Self{
            sprite_batch_pipeline,
            quad_mesh,
            sampler,
            batches: AHashMap::new(),
        }
    }

    pub(super) fn create_sprite_material(&self, egpu: &mut easy_gpu::Renderer, camera: &Camera, lighting_engine: &LightingEngine, texture: Handle<Texture>,atlas_buffer: Handle<Buffer>) -> Handle<Material>{
        MaterialBuilder::new(self.sprite_batch_pipeline)
            .buffer(0,camera.buffer)
            .texture(1,texture)
            .sampler(2,self.sampler)
            .texture(3,lighting_engine.smooth_texture_a)
            .sampler(4,lighting_engine.light_sampler)
            .buffer(5,lighting_engine.light_uniform)
            .buffer(6,atlas_buffer)
            .build(egpu)
    }

    pub fn draw_sprites(&mut self, frame: &mut Frame,world: &World){
        for (_,(sprite, transform)) in world.query::<(&Sprite,&Transform)>().iter(){
            let instance = Instance{
                position: transform.translation,
                rotation: transform.rotation,
                scale: transform.scale,
                colour: sprite.colour,
                tex_index: sprite.tex_index,
            };

            if let Some(batch) = self.batches.get_mut(&sprite.material) {
                batch.push(instance);
            }
            else{
                self.batches.insert(sprite.material,vec![instance]);
            }
        }

        for (material,instances) in self.batches.drain(){
            frame.draw_batch(
                instances.as_slice(),
                material,
                self.quad_mesh,
            );
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AtlasFrame{
    min_uv: [f32;2],
    max_uv: [f32;2]
}

pub struct Atlas{
    pub(super) buffer: Handle<Buffer>,
    pub(super) frames: Vec<AtlasFrame>
}

impl Atlas{
    pub fn add_frame(&mut self,min_uv: [f32;2],max_uv: [f32;2]){
        self.frames.push(AtlasFrame{
            min_uv,
            max_uv,
        });
    }
}

#[derive(Copy,Clone)]
pub struct Sprite{
    pub material: Handle<Material>,
    pub tex_index: u32,
    pub colour: [f32;4],
}

impl Sprite{
    pub fn new(
        material: Handle<Material>,
        tex_index: u32,
    ) -> Self{
        Self{
            material,
            tex_index,
            colour: [1.;4],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub position: [f32;2],
    pub rotation: f32,
    pub scale: f32,
    pub tex_index: u32,
    pub colour: [f32; 4],
}


impl GpuInstance for Instance{
    fn buffer_layout() -> BufferLayout {
        BufferLayout::new()
            .stride(size_of::<Self>() as u64)
            .step_mode(VertexStepMode::Instance)
            .attribute(1,0,VertexFormat::Float32x2)
            .attribute(2,8,VertexFormat::Float32)
            .attribute(3,12,VertexFormat::Float32)
            .attribute(4,16,VertexFormat::Uint32)
            .attribute(5,20,VertexFormat::Float32x4)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex{
    pub position: [f32;3],
}

impl SpriteVertex{
    pub fn new(position: [f32;3]) -> Self{
        Self{
            position,
        }
    }
}

impl GpuVertex for SpriteVertex{
    fn buffer_layout() -> BufferLayout {
        BufferLayout::new()
            .stride(size_of::<Self>() as u64)
            .attribute(0,0,VertexFormat::Float32x3)
    }
}