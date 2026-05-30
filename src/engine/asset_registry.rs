use ahash::AHashMap;
use easy_gpu::assets::{Material, Texture};
use easy_gpu::assets_manager::Handle;
use crate::engine::render::Renderer;

pub struct AssetRegistry{
    pub throwable_mat: Handle<Material>,
    pub particle_mat: Handle<Material>,
}

impl AssetRegistry{
    pub fn new(renderer: &mut Renderer)-> Self{
        let texture = renderer.egpu.load_texture_from_file(include_bytes!("../../textures/throwables.png").to_vec());
        let mut atlas = renderer.create_atlas();
        atlas.add_frame([0.,0.],[0.25,1.0]);
        atlas.add_frame([0.25,0.],[0.5,1.0]);
        atlas.add_frame([0.5,0.],[0.75,1.0]);
        atlas.add_frame([0.75,0.],[1.0,1.0]);
        let throwable_mat = renderer.create_sprite_material(texture,atlas);

        let texture = renderer.egpu.load_texture_from_file(include_bytes!("../../textures/particles.png").to_vec());
        let mut atlas = renderer.create_atlas();
        atlas.add_frame([0.25,0.],[0.5,1.0]);
        let particle_mat = renderer.create_sprite_material(texture,atlas);

        Self{
            throwable_mat,
            particle_mat,
        }
    }
}