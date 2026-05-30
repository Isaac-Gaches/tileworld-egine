use easy_gpu::assets::{Material, Mesh};
use easy_gpu::assets_manager::Handle;
use easy_gpu::frame::Frame;
use serde::{Deserialize, Serialize};
use crate::engine::render::MeshVertex;
use crate::game::terrain::chunk_manager::ChunkBorders;
use crate::game::terrain::terrain_generator::TerrainGenerator;
use crate::game::terrain::region::{RegionPosition, REGION_WIDTH};
use crate::game::terrain::tile::{Tile};

pub const CHUNK_SIZE: usize = 32;

const TEX_ATLAS_DIV: [f32;2] = [1./4.,1./3.];
const MARCH_SQR_DIV: f32 = 1./7.;

const TILE_TEX_COORDS: [[[f32;2];9];2] = [[ //bg
    [0.,0.],//1
    [1.,0.],//2
    [2.,0.],//3
    [3.,2.],//4
    [2.,1.],//5
    [2.,2.],//6
    [1.,2.],//7
    [0.,2.],//8
    [0.,2.],//9
],[ //fg
    [0.,0.],//1
    [1.,0.],//2
    [2.,0.],//3
    [3.,2.],//4
    [2.,1.],//5
    [2.,2.],//6
    [1.,2.],//7
    [0.,2.],//8
    [0.,2.],//9
]];

#[derive(Serialize,Deserialize)]
pub struct ChunkData {
    tiles: [Vec<Tile>;2],
}

impl ChunkData {
    pub fn new(position: &ChunkPosition,generator: &TerrainGenerator) -> Self{
        let tiles = generator.chunk_tiles(position);

        Self{
            tiles
        }
    }
}

pub struct Chunk{
    data: ChunkData,
    meshes: [Option<Handle<Mesh>>;2],
    pub dirty: [bool;2],
    pub save: bool,
}

impl Chunk{
    pub fn new(data:ChunkData)->Self{
        Self{
            data,
            meshes: [None,None],
            dirty: [true;2],
            save: false,
        }
    }

    #[inline(always)]
    pub fn get_tile(&self, x: usize, y: usize, layer: usize) -> &Tile{
        &self.data.tiles[layer][x * CHUNK_SIZE + y]
    }

    pub fn set_tile(&mut self, x: usize, y: usize,id:u8, layer: usize){
        self.data.tiles[layer][x * CHUNK_SIZE + y].id = id;
        self.dirty[layer] = true;
        self.save = true;
    }

    pub fn remove_mark(&mut self){
        self.dirty[0] = false;
        self.dirty[1] = false;
    }

    pub fn has_mesh(&self)-> bool{
        self.meshes[0].is_some() || self.meshes[1].is_some()
    }

    pub fn dirty(&self)-> bool{
        self.dirty[0] || self.dirty[1]
    }

    pub fn data(&self) -> &ChunkData{
        &self.data
    }

    pub fn destroy_mesh(&mut self){
        self.dirty = [true;2];
        self.meshes = [None,None];
    }

    pub fn draw(&self,frame: &mut Frame,materials: &Vec<Handle<Material>>){
        for (mesh,material) in self.meshes.iter().zip(materials.iter()){
            if let Some(mesh_handle) = mesh{
                frame.draw(
                    material.clone(),
                    mesh_handle.clone(),
                );
            }
        }
    }

    pub fn build_mesh(
        &self,
        layer: usize,
        position: &ChunkPosition,
        borders: &ChunkBorders,
    ) -> Option<(Vec<MeshVertex>, Vec<u16>)> {
        const EXPECTED_QUADS: usize = CHUNK_SIZE * CHUNK_SIZE;

        let mut vertices = Vec::with_capacity(EXPECTED_QUADS * 4);
        let mut indices = Vec::with_capacity(EXPECTED_QUADS * 6);

        let mut sides: u16 = 0;

        let z = 0.8 - layer as f32 * 0.5;

        let chunk_world_x = position.x as f32 * CHUNK_SIZE as f32;
        let chunk_world_y = position.y as f32 * CHUNK_SIZE as f32;

        for x in 0..CHUNK_SIZE {
            let world_x = x as f32 + chunk_world_x;

            for y in 0..CHUNK_SIZE {
                let tile = self.get_tile(x, y, layer);

                if tile.id == 0 {
                    continue;
                }

                let world_y = y as f32 + chunk_world_y;

                let mask = self.tile_neighbours(x as i32,y as i32,layer,&borders);
                let ms = Self::map_marching_squares(mask);

                let tex =
                    TILE_TEX_COORDS[layer][tile.id as usize - 1];

                let u0 = (tex[0] + MARCH_SQR_DIV * ms[0]) * TEX_ATLAS_DIV[0];
                let v0 = (tex[1] + MARCH_SQR_DIV * ms[1]) * TEX_ATLAS_DIV[1];

                let u1 = u0 + MARCH_SQR_DIV * TEX_ATLAS_DIV[0];
                let v1 = v0 + MARCH_SQR_DIV * TEX_ATLAS_DIV[1];

                vertices.extend_from_slice(&[
                    MeshVertex::new(
                        [world_x - 0.5, world_y - 0.5, z],
                        [u0, v1],
                    ),
                    MeshVertex::new(
                        [world_x + 0.5, world_y - 0.5, z],
                        [u1, v1],
                    ),
                    MeshVertex::new(
                        [world_x + 0.5, world_y + 0.5, z],
                        [u1, v0],
                    ),
                    MeshVertex::new(
                        [world_x - 0.5, world_y + 0.5, z],
                        [u0, v0],
                    ),
                ]);

                indices.extend_from_slice(&[
                    sides + 1,
                    sides + 3,
                    sides,
                    sides + 1,
                    sides + 2,
                    sides + 3,
                ]);

                sides += 4;
            }
        }

        if indices.is_empty() {
            return None;
        }

        Some((vertices, indices))
    }

    pub fn set_mesh(&mut self,layer: usize,mesh: Handle<Mesh>){
        self.meshes[layer] = Some(mesh);
    }

    #[inline(always)]
    fn tile_neighbours(&self,x: i32, y: i32, layer: usize, borders: &ChunkBorders) -> u8{
        let mut mask = 0u8;
        let mut bit = 0;

        for i in -1..=1{
            for j in -1..=1{
                if i == 0 && j == 0 {
                    continue;
                }
                let filled  = if x+i < 0 || x+i == CHUNK_SIZE as i32 || y+j < 0 || y+j == CHUNK_SIZE as i32 {
                    if if j == 1 && y == CHUNK_SIZE as i32 - 1{
                        borders.top[(x+i+1) as usize]
                    }
                    else if j == -1 && y == 0{
                        borders.bottom[(x+i+1) as usize]
                    }
                    else if i == 1{
                        borders.right[(y+j) as usize]
                    }
                    else{
                        borders.left[(y+j) as usize]
                    }{ 1 } else { 0 }
                }
                else if self.get_tile((x+i) as usize,(y+j) as usize, layer).id > 0
                { 1 } else { 0 };

                mask |= (filled as u8) << bit;
                bit += 1;
            }
        }

        mask
    }

    #[inline(always)]
    fn map_marching_squares(mask: u8) -> [f32; 2] {
        LUT[mask as usize]
    }

    pub fn mesh(&self,layer: usize) -> &Option<Handle<Mesh>>{
        &self.meshes[layer]
    }
}

#[derive(Hash, PartialEq,Clone,Debug,Eq,Copy)]
pub struct ChunkPosition{
    pub x: i32,
    pub y: i32,
}

impl ChunkPosition{
    pub fn new(x:i32,y:i32)->Self{
        Self{
            x,
            y,
        }
    }
    pub fn to_region_space(&self) -> RegionPosition {
        RegionPosition {
            x: self.x.div_euclid(REGION_WIDTH),
            y: self.y.div_euclid(REGION_WIDTH),
        }
    }

    pub fn to_world_space(&self) -> (i32,i32){
        (self.x * CHUNK_SIZE as i32, self.y * CHUNK_SIZE as i32)
    }

    pub fn from_world_space(x:i32,y:i32) -> Self{
        Self{
            x: x.div_euclid(CHUNK_SIZE as i32),
            y: y.div_euclid(CHUNK_SIZE as i32),
        }
    }
}

const LUT: [[f32; 2]; 256] = [
    [0.0, 0.0],
    [0.0, 0.0],
    [3.0, 0.0],
    [3.0, 0.0],
    [0.0, 0.0],
    [0.0, 0.0],
    [3.0, 0.0],
    [3.0, 0.0],
    [3.0, 1.0],
    [3.0, 1.0],
    [4.0, 1.0],
    [2.0, 1.0],
    [3.0, 1.0],
    [3.0, 1.0],
    [4.0, 1.0],
    [2.0, 1.0],
    [3.0, 3.0],
    [3.0, 3.0],
    [4.0, 0.0],
    [4.0, 0.0],
    [3.0, 3.0],
    [3.0, 3.0],
    [2.0, 3.0],
    [2.0, 3.0],
    [3.0, 2.0],
    [3.0, 2.0],
    [6.0, 0.0],
    [6.0, 4.0],
    [3.0, 2.0],
    [3.0, 2.0],
    [5.0, 5.0],
    [2.0, 2.0],
    [0.0, 0.0],
    [0.0, 0.0],
    [3.0, 0.0],
    [3.0, 0.0],
    [0.0, 0.0],
    [0.0, 0.0],
    [3.0, 0.0],
    [3.0, 0.0],
    [3.0, 1.0],
    [3.0, 1.0],
    [4.0, 1.0],
    [2.0, 1.0],
    [3.0, 1.0],
    [3.0, 1.0],
    [4.0, 1.0],
    [2.0, 1.0],
    [3.0, 3.0],
    [3.0, 3.0],
    [4.0, 0.0],
    [4.0, 0.0],
    [3.0, 3.0],
    [3.0, 3.0],
    [2.0, 3.0],
    [2.0, 3.0],
    [3.0, 2.0],
    [3.0, 2.0],
    [6.0, 0.0],
    [6.0, 4.0],
    [3.0, 2.0],
    [3.0, 2.0],
    [5.0, 5.0],
    [2.0, 2.0],
    [1.0, 0.0],
    [1.0, 0.0],
    [2.0, 0.0],
    [2.0, 0.0],
    [1.0, 0.0],
    [1.0, 0.0],
    [2.0, 0.0],
    [2.0, 0.0],
    [1.0, 4.0],
    [1.0, 4.0],
    [6.0, 1.0],
    [6.0, 2.0],
    [1.0, 4.0],
    [1.0, 4.0],
    [6.0, 1.0],
    [6.0, 2.0],
    [0.0, 4.0],
    [0.0, 4.0],
    [5.0, 1.0],
    [5.0, 1.0],
    [0.0, 4.0],
    [0.0, 4.0],
    [6.0, 3.0],
    [6.0, 3.0],
    [5.0, 0.0],
    [5.0, 0.0],
    [4.0, 4.0],
    [1.0, 5.0],
    [5.0, 0.0],
    [5.0, 0.0],
    [1.0, 6.0],
    [0.0, 5.0],
    [1.0, 0.0],
    [1.0, 0.0],
    [2.0, 0.0],
    [2.0, 0.0],
    [1.0, 0.0],
    [1.0, 0.0],
    [2.0, 0.0],
    [2.0, 0.0],
    [0.0, 1.0],
    [0.0, 1.0],
    [5.0, 2.0],
    [1.0, 1.0],
    [0.0, 1.0],
    [0.0, 1.0],
    [5.0, 2.0],
    [1.0, 1.0],
    [0.0, 4.0],
    [0.0, 4.0],
    [5.0, 1.0],
    [5.0, 1.0],
    [0.0, 4.0],
    [0.0, 4.0],
    [6.0, 3.0],
    [6.0, 3.0],
    [5.0, 4.0],
    [5.0, 4.0],
    [3.0, 6.0],
    [2.0, 5.0],
    [5.0, 4.0],
    [5.0, 4.0],
    [4.0, 6.0],
    [2.0, 4.0],
    [0.0, 0.0],
    [0.0, 0.0],
    [3.0, 0.0],
    [3.0, 0.0],
    [0.0, 0.0],
    [0.0, 0.0],
    [3.0, 0.0],
    [3.0, 0.0],
    [3.0, 1.0],
    [3.0, 1.0],
    [4.0, 1.0],
    [2.0, 1.0],
    [3.0, 1.0],
    [3.0, 1.0],
    [4.0, 1.0],
    [2.0, 1.0],
    [3.0, 3.0],
    [3.0, 3.0],
    [4.0, 0.0],
    [4.0, 0.0],
    [3.0, 3.0],
    [3.0, 3.0],
    [2.0, 3.0],
    [2.0, 3.0],
    [3.0, 2.0],
    [3.0, 2.0],
    [6.0, 0.0],
    [6.0, 4.0],
    [3.0, 2.0],
    [3.0, 2.0],
    [5.0, 5.0],
    [2.0, 2.0],
    [0.0, 0.0],
    [0.0, 0.0],
    [3.0, 0.0],
    [3.0, 0.0],
    [0.0, 0.0],
    [0.0, 0.0],
    [3.0, 0.0],
    [3.0, 0.0],
    [3.0, 1.0],
    [3.0, 1.0],
    [4.0, 1.0],
    [2.0, 1.0],
    [3.0, 1.0],
    [3.0, 1.0],
    [4.0, 1.0],
    [2.0, 1.0],
    [3.0, 3.0],
    [3.0, 3.0],
    [4.0, 0.0],
    [4.0, 0.0],
    [3.0, 3.0],
    [3.0, 3.0],
    [2.0, 3.0],
    [2.0, 3.0],
    [3.0, 2.0],
    [3.0, 2.0],
    [6.0, 0.0],
    [6.0, 4.0],
    [3.0, 2.0],
    [3.0, 2.0],
    [5.0, 5.0],
    [2.0, 2.0],
    [1.0, 0.0],
    [1.0, 0.0],
    [2.0, 0.0],
    [2.0, 0.0],
    [1.0, 0.0],
    [1.0, 0.0],
    [2.0, 0.0],
    [2.0, 0.0],
    [1.0, 4.0],
    [1.0, 4.0],
    [6.0, 1.0],
    [6.0, 2.0],
    [1.0, 4.0],
    [1.0, 4.0],
    [6.0, 1.0],
    [6.0, 2.0],
    [0.0, 3.0],
    [0.0, 3.0],
    [5.0, 3.0],
    [5.0, 3.0],
    [0.0, 3.0],
    [0.0, 3.0],
    [1.0, 3.0],
    [1.0, 3.0],
    [4.0, 5.0],
    [4.0, 5.0],
    [3.0, 5.0],
    [5.0, 6.0],
    [4.0, 5.0],
    [4.0, 5.0],
    [0.0, 6.0],
    [3.0, 4.0],
    [1.0, 0.0],
    [1.0, 0.0],
    [2.0, 0.0],
    [2.0, 0.0],
    [1.0, 0.0],
    [1.0, 0.0],
    [2.0, 0.0],
    [2.0, 0.0],
    [0.0, 1.0],
    [0.0, 1.0],
    [5.0, 2.0],
    [1.0, 1.0],
    [0.0, 1.0],
    [0.0, 1.0],
    [5.0, 2.0],
    [1.0, 1.0],
    [0.0, 3.0],
    [0.0, 3.0],
    [5.0, 3.0],
    [5.0, 3.0],
    [0.0, 3.0],
    [0.0, 3.0],
    [1.0, 3.0],
    [1.0, 3.0],
    [0.0, 2.0],
    [0.0, 2.0],
    [2.0, 6.0],
    [4.0, 3.0],
    [0.0, 2.0],
    [0.0, 2.0],
    [4.0, 2.0],
    [1.0, 2.0],
];



