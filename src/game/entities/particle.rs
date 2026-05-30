use hecs::World;

pub struct Particle{
    lifetime: f32
}

impl Particle{
    pub fn new(lifetime: f32) -> Self{
        Self{
            lifetime
        }
    }
}

pub fn update_particles(world: &mut World,dt: f32){
    let mut destroy = Vec::new();
    for (entity,particle) in world.query::<&mut Particle>().iter(){
        particle.lifetime -= dt;
        if particle.lifetime <= 0.{
            destroy.push(entity)
        }
    }
    for entity in destroy {
        let _ = world.despawn(entity);
    }
}