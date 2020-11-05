use bevy::prelude::Timer;
use bevy_rapier2d::na::Point2;
use super::mob;


pub struct Borg {
    /// Ship rotation speed in rad/s
    pub rotation_speed: f32,
    /// Max movement speed
    pub speed: f32,
    /// Ship life points
    pub life: u32,
    /// Cannon auto-fire timer
    pub cannon_timer: Timer,
    pub looks_at: Point2<f32>,
}
pub type Ship = Borg;


pub struct UiScore {}
pub struct UiLife {
    pub min: u32,
}

pub enum ExplosionKind {
    ShipDead,
    ShipContact,
    LaserOnAsteroid,
}
pub struct ExplosionSpawnEvent {
    pub kind: ExplosionKind,
    pub x: f32,
    pub y: f32,
}

pub struct AsteroidSpawnEvent {
    pub size: AsteroidSize,
    pub x: f32,
    pub y: f32,
    pub brain: mob::Brain,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AsteroidSize {
    Big,
    Medium,
    Small,
}
pub struct Mob {
    pub size: AsteroidSize,
    /// Despawn when expired
    pub lifeforce: Timer,
    pub brain: mob::Brain,
    /// Max rotation speed in rad/s
    pub rotation_speed: f32,
    /// Max movement speed
    pub speed: f32,
}
pub type Asteroid = Mob;

impl Mob {
    pub fn genotype(&self) -> &mob::Brain {
        &self.brain
    }
}

pub struct Laser {
    pub despawn_timer: Timer,
}
pub struct Damage {
    pub value: u32,
}
