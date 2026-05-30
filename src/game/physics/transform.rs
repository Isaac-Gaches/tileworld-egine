pub struct Transform{
    pub translation: [f32;2],
    pub rotation: f32,
    pub scale: f32,
}

impl Transform{
    pub fn new(translation: [f32;2],scale: f32)-> Self{
        Self{
            translation,
            rotation: 0.0,
            scale,
        }
    }
}