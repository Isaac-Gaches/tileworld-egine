use ahash::AHashMap;
use crate::engine::asset_registry::AssetRegistry;
use crate::engine::render::Sprite;
use crate::game::items::definition::ItemDefinition;
use crate::game::items::projectile::{BombConfig, ColliderConfig, ProjectileConfig, Projectile, LightConfig};

pub struct ItemRegistry{
    pub definitions: AHashMap<ItemID,ItemDefinition>,
}

pub type ItemID = String;

impl ItemRegistry{
    pub fn new() -> Self{
        Self{
            definitions: AHashMap::new(),
        }
    }
    pub fn load(&mut self,asset_registry: &AssetRegistry){
        self.definitions.insert("bomb".to_string(),ItemDefinition{
            name: "Bomb".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,0),
            projectile: Some(Projectile {
                speed: 40.0,
                projectile_config: ProjectileConfig{
                    bomb: Some(BombConfig{
                        timer: 3.0,
                        radius: 5,
                        num_particles: 10,
                        particle_lifespan: 1.0,
                    }),
                    light: Some(LightConfig{
                        colour: [0.5,0.1,0.0],
                    }),
                },
                collider_config: ColliderConfig{
                    width: 0.9,
                    height: 0.9,
                    offset: [0.,0.],
                    auto_jump: false,
                    bounce: 0.8,
                },
            }),
        });

        self.definitions.insert("glow_stick".to_string(),ItemDefinition{
            name: "Glow Stick".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,1),
            projectile: Some(Projectile {
                speed: 50.0,
                projectile_config: ProjectileConfig{
                    bomb: None,
                    light: Some(LightConfig{
                        colour: [0.4,1.0,0.2],
                    }),
                },
                collider_config: ColliderConfig{
                    width: 0.8,
                    height: 0.8,
                    offset: [0.,0.],
                    auto_jump: false,
                    bounce: 0.5,
                },
            }),
        });

        self.definitions.insert("big_bomb".to_string(),ItemDefinition{
            name: "Big Bomb".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,2),
            projectile: Some(Projectile {
                speed: 30.0,
                projectile_config: ProjectileConfig{
                    bomb: Some(BombConfig{
                        timer: 5.,
                        radius: 10,
                        num_particles: 20,
                        particle_lifespan: 1.5,
                    }),
                    light: Some(LightConfig{
                        colour: [0.7,0.05,0.0],
                    }),
                },
                collider_config: ColliderConfig{
                    width: 0.8,
                    height: 0.8,
                    offset: [0.,0.],
                    auto_jump: false,
                    bounce: 0.6,
                },
            }),
        });

        self.definitions.insert("potion".to_string(),ItemDefinition{
            name: "Potion".to_string(),
            sprite: Sprite::new(asset_registry.throwable_mat,3),
            projectile: Some(Projectile {
                speed: 40.0,
                projectile_config: ProjectileConfig{
                    bomb: None,
                    light: Some(LightConfig{
                        colour: [0.5,0.1,1.0],
                    }),
                },
                collider_config: ColliderConfig{
                    width: 0.8,
                    height: 0.8,
                    offset: [0.,0.],
                    auto_jump: false,
                    bounce: 0.1,
                },
            }),
        });
    }
}