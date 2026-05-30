use easy_gpu::assets::Material;
use easy_gpu::assets_manager::Handle;
use hecs::{EntityBuilder, World};
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::input_manager::InputManager;
use crate::engine::render::{Light, Sprite};
use crate::game::entities::bomb::Bomb;
use crate::game::items::definition::ItemDefinition;
use crate::game::items::item_registry::ItemRegistry;
use crate::game::physics::collider::Collider;
use crate::game::physics::transform::Transform;

pub struct Player{
    acceleration:f32,
    speed: f32,
    jump_speed:f32,
}

impl Player{
    pub fn new() -> Self{
        Self{
            acceleration: 20.0,
            speed: 15.0,
            jump_speed: 20.0,
        }
    }

    fn move_player(&self,input: &InputManager,collider:&mut Collider,dt: f32){
        if input.up && collider.on_ground{
            collider.y_vel = self.jump_speed;
        }

        if input.left {
            collider.x_vel -= self.acceleration * dt;
            if collider.x_vel > 0. {collider.x_vel -= self.acceleration * dt} // if changing diretion, change faster to feel more responsive
            if collider.x_vel < -self.speed { collider.x_vel = -self.speed; }

        }

        if input.right {
            collider.x_vel += self.acceleration * dt;
            if collider.x_vel < 0. {collider.x_vel += self.acceleration * dt}  // if changing diretion, change faster to feel more responsive
            if collider.x_vel > self.speed { collider.x_vel = self.speed; }
        }

        if !input.right && !input.left{ // slow down if no input
            if collider.x_vel > 0.{
                collider.x_vel -= self.acceleration*2. * dt;//slow fast
                if collider.x_vel < 0.{ collider.x_vel = 0.; }
            }
            else if collider.x_vel < 0.{
                collider.x_vel += self.acceleration*2. * dt;
                if collider.x_vel > 0.{ collider.x_vel = 0.; }
            }
        }
    }
}

fn throw_projectile(
    pos: [f32; 2],
    target: [f32; 2],
    world: &mut World,
    item: &ItemDefinition,
) {
    let dx = target[0] - pos[0];
    let dy = target[1] - pos[1];

    let len = (dx * dx + dy * dy).sqrt();

    if len <= f32::EPSILON {
        return;
    }

    let dir_x = dx / len;
    let dir_y = dy / len;

    if let Some(throwable) = &item.projectile {
        let mut builder = EntityBuilder::new();

        throwable.add_components(&mut builder,dir_x,dir_y,pos);
        builder.add(item.sprite);

        world.spawn(builder.build());
    }
}


pub fn update_player(world: &mut World,input_manager: &InputManager,item_registry: &ItemRegistry,dt: f32) -> [f32;2]{
    let mut pos = [0.,0.];
    for (_, (player, transform,collider)) in world.query::<(&Player, &Transform,&mut Collider)>().iter() {
        player.move_player(input_manager,collider,dt);
        pos = transform.translation;
    }
    if input_manager.left_mouse{
        throw_projectile(pos, input_manager.mouse_world_pos, world, item_registry.definitions.get("bomb").unwrap());
    }
    if input_manager.right_mouse{
        throw_projectile(pos, input_manager.mouse_world_pos, world, item_registry.definitions.get("glow_stick").unwrap());
    }
    pos
}