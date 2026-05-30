use hecs::World;
use rand::random_range;
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::render::{Light, Sprite};
use crate::game::entities::particle::Particle;
use crate::game::physics::collider::Collider;
use crate::game::physics::transform::Transform;
use crate::game::terrain::chunk_manager::ChunkManager;

pub struct Bomb{
    timer: f32,
    radius: u32,
    num_particles: u32,
    particle_lifespan: f32,
}

impl Bomb{
    pub fn new(timer: f32, radius: u32,num_particles: u32,particle_lifespan: f32) -> Self{
        Self{
            timer,
            radius,
            num_particles,
            particle_lifespan,
        }
    }
    pub fn tick(&mut self, dt: f32) -> bool{
        self.timer-=dt;
        if self.timer <= 0.{
            return true
        }
        false
    }
}

pub fn update_bombs(world: &mut World,dt: f32,terrain: &mut ChunkManager,asset_registry: &AssetRegistry){
    let mut explosions = Vec::new();

    for (entity,(bomb,transform)) in world.query::<(&mut Bomb,&Transform)>().iter() {
        if bomb.tick(dt){
            terrain.explode(bomb.radius as i32,transform.translation[0] as i32,transform.translation[1] as i32);
            explosions.push((entity,transform.translation,bomb.num_particles,bomb.particle_lifespan,bomb.radius as f32));
        }
    }

    for (entity,translation,particles,lifespan,power) in explosions{
        for i in 0..particles{
            world.spawn((
                Transform::new(translation,1.0),
                Collider::new(0.,0.,[0.,0.],random_range(-power*3.0..power*3.0),random_range(-power*3.0..power*3.0),false,0.1),
                Light::new([0.8,0.5,0.1]),
                Particle::new(random_range(lifespan/4.0..lifespan)),
                Sprite::new(asset_registry.particle_mat,0)
            ));
        }

        let _ = world.despawn(entity);
    }
}