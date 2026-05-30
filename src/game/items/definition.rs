use crate::engine::render::Sprite;
use crate::game::items::projectile::Projectile;

pub struct ItemDefinition{
    pub name: String,
    pub sprite: Sprite,

    pub projectile: Option<Projectile>,
}