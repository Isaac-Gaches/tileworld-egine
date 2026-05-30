use std::sync::{Arc};
use std::time::Instant;
use ahash::{AHashMap, HashMapExt};
use easy_gpu::frame::Frame;
use hecs::World;
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::file_manager::FileManager;
use crate::engine::input_manager::InputManager;
use crate::engine::render::{Light, LightingEngine, LightSource, Renderer, Sprite};
use crate::game::entities::bomb::{update_bombs};
use crate::game::entities::particle::update_particles;
use crate::game::items::item_registry::ItemRegistry;
use crate::game::physics::collider::{Collider, update_colliders};
use crate::game::physics::transform::Transform;
use crate::game::player::player::{Player, update_player};
use crate::game::terrain::chunk_manager::ChunkManager;
use crate::game::terrain::terrain_generator::TerrainGenerator;

pub struct Game{
    pub world: World,
    pub chunk_manager: ChunkManager,
    terrain_generator: Arc<TerrainGenerator>,
    pub player_position: [f32;2],
    unload_timer: Instant,
    pub item_registry: ItemRegistry,
}

impl Game{
    pub fn new()->Self{
        Self{
            world: World::new(),
            chunk_manager: ChunkManager::new(),
            terrain_generator: Arc::new(TerrainGenerator::new()),
            player_position: [0.,0.],
            unload_timer: Instant::now(),
            item_registry: ItemRegistry::new(),
        }
    }

    pub fn generate_terrain(&mut self,egpu: &mut easy_gpu::Renderer, file_manager: &Arc<FileManager>){
        for _ in 0..10 {
            self.chunk_manager.update_data_queue(self.player_position);
            self.chunk_manager.load_chunks_data(file_manager, &self.terrain_generator);
            self.chunk_manager.update_mesh_queue(self.player_position);
            self.chunk_manager.generate_chunk_meshes(egpu);
        }
    }

    pub fn spawn_player(&mut self,renderer: &mut Renderer){
        let texture = renderer.egpu.load_texture_from_file(include_bytes!("../../textures/player.png").to_vec());
        let mut atlas = renderer.create_atlas();
        atlas.add_frame([0.,0.],[1.0,1.0]);
        let material = renderer.create_sprite_material(texture,atlas);

        self.world.spawn((
            Player::new(),
            Collider::new(1.8,1.8,[0.,0.],0.,0.,true,0.),
            Transform::new([5.,0.],2.0),
            Sprite::new(material, 0),
        ));

    }

    pub fn update(&mut self,egpu: &mut easy_gpu::Renderer, file_manager: &Arc<FileManager>,input_manager: &InputManager, asset_registry: &AssetRegistry,dt: f32){
      //  self.chunk_manager.handle_input(&input_manager);

        self.chunk_manager.update_data_queue(self.player_position);
        self.chunk_manager.load_chunks_data(file_manager,&self.terrain_generator);
        self.chunk_manager.update_mesh_queue(self.player_position);
        self.chunk_manager.generate_chunk_meshes(egpu);

        if self.unload_timer.elapsed().as_secs() > 20{
            self.chunk_manager.save_chunks(file_manager);
        self.chunk_manager.unload_chunks(self.player_position);
            self.unload_timer = Instant::now();
        }

        update_colliders(&mut self.world,&mut self.chunk_manager,dt);
        update_bombs(&mut self.world,dt,&mut self.chunk_manager,asset_registry);
        update_particles(&mut self.world,dt);
        self.player_position = update_player(&mut self.world,input_manager,&self.item_registry,dt);
    }

    pub fn draw(&self, frame: &mut Frame){
        self.chunk_manager.draw(frame);
    }

    pub fn extract_tiles(&self) -> Vec<u8>{
        self.chunk_manager.extract_tiles(self.player_position)
    }

    pub fn extract_lights(&self,lighting_engine: &mut LightingEngine){
        let mut lights= AHashMap::new();

        for (_,(light,transform)) in self.world.query::<(&Light,&Transform)>().iter() {
            let pos = [(transform.translation[0]-0.5) as i32, (transform.translation[1]-0.5) as i32];
            lights
                .entry(pos)
                .and_modify(|existing: &mut LightSource| {
                    existing.colour[0] = existing.colour[0].max(light.colour[0]);
                    existing.colour[1] = existing.colour[1].max(light.colour[1]);
                    existing.colour[2] = existing.colour[2].max(light.colour[2]);
                })
                .or_insert(LightSource::new(transform.translation,light.colour));
        }

        lighting_engine.lights = lights.into_values().collect();
    }
}

