use bevy::prelude::{ Entity, GlobalTransform, Mut, Quat, Query, Res, Timer, Transform, Without, Vec3 };
use bevy_rapier2d::na;
use bevy_rapier2d::na::{ Point2, Rotation2, UnitComplex, Vector2 };
use super::mob;


use bevy_rapier2d::physics::RigidBodyHandleComponent;
use bevy_rapier2d::rapier::dynamics::RigidBodySet;

pub struct AttachedToEntity(pub Entity);


pub struct Borg {
    /// Ship rotation speed in rad/s
    pub rotation_speed: f32,
    /// Max movement speed
    pub speed: f32,
    /// Ship life points
    pub life: u32,
    /// Cannon auto-fire timer
    pub cannon_timer: Timer,
}
pub type Ship = Borg;

/// Always directed towards this point in level space
pub struct LooksAt(pub Point2<f32>);

impl Default for LooksAt {
    fn default() -> LooksAt {
        LooksAt(Point2::new(0.0, 0.0))
    }
}

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

pub fn swivel_at(
    mut query: Query<(&AttachedToEntity, &LooksAt, Mut<GlobalTransform>, Mut<Transform>)>,
    entities: Query<Without<AttachedToEntity, &GlobalTransform>>,
) {
    for (target_entity, looks_at, mut gtransform, mut transform) in query.iter_mut() {
        if let Ok(parent_transform) = entities.get(target_entity.0) {
            transform.translation = parent_transform.translation.clone();
            gtransform.translation = parent_transform.translation.clone();
            let translation = na::Translation2::new(
                gtransform.translation.x(),
                gtransform.translation.y(),
            );
            // Lol, this is so inefficient it's funny
            let point = translation.inverse_transform_point(&looks_at.0);
            let rot = Rotation2::rotation_between(
                &Vector2::new(0.0, 1.0),
                &Vector2::new(point.x, point.y)
            );

            let c = UnitComplex::from_rotation_matrix(&rot);
            transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), c.angle());
            gtransform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), c.angle());
        }
    }
}
