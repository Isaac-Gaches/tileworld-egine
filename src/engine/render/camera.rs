use easy_gpu::assets::{Buffer, BufferUsages};
use easy_gpu::assets_manager::Handle;
use crate::engine::input_manager::InputManager;
use crate::game::terrain::chunk::CHUNK_SIZE;

pub struct Camera{
    pub buffer: Handle<Buffer>,
    data: Data,
}

impl Camera{
    pub fn new(egpu: &mut easy_gpu::Renderer) -> Self{
        let buffer = egpu.create_buffer(
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size_of::<Data>() as u64
        );
        Self{
            buffer,
            data: Data {
                position: [0.,0.],
                zoom: 0.015,
                ratio: 1.0,
            },
        }
    }

    pub fn update(&mut self,player_pos: [f32;2],input: &InputManager,egpu: &mut easy_gpu::Renderer,dt:f32){
        const FOLLOW_SPEED: f32 = 5.0;
        let diff = [player_pos[0] - self.data.position[0],player_pos[1] - self.data.position[1]];
        self.data.position = [self.data.position[0] + (diff[0]*FOLLOW_SPEED) * dt,self.data.position[1] + (diff[1]*FOLLOW_SPEED)* dt];
        self.data.ratio = egpu.window_aspect();

       // self.data.position = player_pos

        const ZOOM: f32 = 0.01;

        if input.plus{
            self.data.zoom += ZOOM * dt;
        }
        if input.minus{
            self.data.zoom -= ZOOM * dt;
        }

        self.data.zoom = self.data.zoom.clamp(0.018,0.1);

        egpu.write_buffer(self.buffer,self.data);
    }

    pub fn screen_to_world(&self,pos:[f32;2])->[f32;2]{
        [
            ((pos[0]/(self.data.zoom))+self.data.position[0]+CHUNK_SIZE as f32/2.)+0.5-16.,
          (  (pos[1]/(self.data.ratio*self.data.zoom))+self.data.position[1]+CHUNK_SIZE as f32/2.)+0.5-16.
        ]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Data {
    position: [f32;2],
    zoom: f32,
    ratio: f32,
}