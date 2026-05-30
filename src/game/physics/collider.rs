use hecs::World;
use crate::game::physics::transform::Transform;
use crate::game::terrain::chunk_manager::ChunkManager;

//constants
const SUB_STEPS:f32 = 5.;
const TERMINAL_VELOCITY:f32 = 100.0;
const GRAVITY:f32 = 60.0;
//components
pub struct Collider{
    left:f32,
    right:f32,
    top:f32,
    bottom:f32,
    pub x_vel:f32,
    pub y_vel:f32,
    pub on_ground:bool,
    auto_jump:bool,
    bounce: f32,
}

impl Collider{

    pub fn new(width: f32, height: f32, offset:[f32;2],x_vel: f32,y_vel: f32, auto_jump: bool,bounce: f32) -> Self{
        Self{
            left: -(width/2.) - offset[0],
            right: (width/2.) - offset[0],
            top: (height/2.) - offset[1],
            bottom: -(height/2.) - offset[1],
            x_vel,
            y_vel,
            on_ground: false,
            auto_jump,
            bounce,
        }
    }
    pub fn handle_collider(&mut self,transform: &mut Transform,terrain: &ChunkManager,dt: f32){
        //velocity update
        self.y_vel -= GRAVITY * dt;
        if self.y_vel < -TERMINAL_VELOCITY { self.y_vel = -TERMINAL_VELOCITY; }
        //sub steps
        let x_vel = (self.x_vel*dt)/SUB_STEPS;
        let y_vel = (self.y_vel*dt)/SUB_STEPS;

        for _i in 0..SUB_STEPS as i8{
            //x
            let mut origin = transform.translation;
            transform.translation[0] += x_vel;
            //find intersecting tiles
            let mut left:i32 = (self.left+transform.translation[0]).round() as i32;
            let mut right:i32 = (self.right+transform.translation[0]).round() as i32;
            let mut top:i32 = (self.top+transform.translation[1]).round() as i32;
            let mut bottom:i32 = (self.bottom+transform.translation[1]).round() as i32;

            //if intersecting block is solid
            'outer: for x in left.min(right)..=right.max(left){
                for y in (bottom..=top).rev(){
                    if terrain.get_tile(x,y,1).unwrap().id != 0{
                        if self.auto_jump && y == bottom && (left.min(right)..=right.max(left)).find(|i|{ terrain.get_tile(*i,top+1,1).unwrap().id != 0 }).is_none(){
                            transform.translation[1] += 1.;
                        }
                        else{
                            transform.translation = origin;
                            self.x_vel = -self.x_vel * self.bounce;
                        }
                        break 'outer;
                    }
                }
            }
            //y
            origin = transform.translation;
            transform.translation[1] += y_vel;
            self.on_ground = false;
            //find intersecting tiles
            left= ((self.left+transform.translation[0])/1.).round() as i32;
            right = ((self.right+transform.translation[0])/1.).round() as i32;
            top = ((self.top+transform.translation[1])/1.).round() as i32;
            bottom = ((self.bottom+transform.translation[1])/1.).round() as i32;

            'outer: for x in left.min(right)..=right.max(left){
                for y in (bottom..=top).rev(){
                    if terrain.get_tile(x,y,1).unwrap().id != 0{

                        transform.translation = origin;
                        self.y_vel = -self.y_vel * self.bounce;
                        if y == bottom{ self.on_ground = true; }

                        break 'outer;
                    }
                }
            }
        }
    }
}

pub fn update_colliders(world: &mut World,terrain: &ChunkManager,dt:f32){
    for (_, (transform, collider)) in world.query::<(&mut Transform, &mut Collider)>().iter() {
        collider.handle_collider(transform,terrain,dt);
    }
}
