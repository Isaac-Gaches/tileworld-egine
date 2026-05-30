use hecs::{EntityBuilder};
use crate::engine::render::Light;
use crate::game::entities::bomb::Bomb;
use crate::game::physics::collider::Collider;
use crate::game::physics::transform::Transform;

pub struct Projectile {
    pub speed: f32,
    pub projectile_config: ProjectileConfig,
    pub collider_config: ColliderConfig,
}

impl Projectile {
    pub fn add_components(&self,builder: &mut EntityBuilder,x_dir: f32, y_dir:f32,pos: [f32;2]){
        self.projectile_config.add_component(builder);
        self.collider_config.add_component(x_dir*self.speed,y_dir*self.speed,builder);
        builder.add(Transform::new(pos,1.0));
    }
}

pub struct  ProjectileConfig{
    pub bomb: Option<BombConfig>,
    pub light: Option<LightConfig>
}

impl ProjectileConfig {
    pub fn add_component(&self,builder: &mut EntityBuilder) {
        if let Some(bomb) = &self.bomb{
            builder.add(bomb.build());
        }
        if let Some(light) = &self.light{
            builder.add(light.build());
        }
    }
}

pub struct LightConfig{
    pub colour: [f32;3]
}

impl LightConfig {
    pub fn build(&self)-> Light{
        Light::new(self.colour)
    }
}

pub struct BombConfig{
    pub timer: f32,
    pub radius: u32,
    pub num_particles: u32,
    pub particle_lifespan: f32,
}

impl BombConfig{
    pub fn build(&self)-> Bomb{
        Bomb::new(self.timer,self.radius,self.num_particles,self.particle_lifespan)
    }
}


pub struct ColliderConfig{
    pub width: f32,
    pub height: f32,
    pub offset:[f32;2],
    pub auto_jump: bool,
    pub bounce: f32
}
impl ColliderConfig{
    pub fn add_component(&self, x_vel: f32, y_vel: f32,builder: &mut EntityBuilder){
        builder.add(Collider::new(self.width, self.height, self.offset, x_vel, y_vel, self.auto_jump, self.bounce));
    }
}

