use bevy::prelude::{ Entity, Mut, Quat, Query, Timer, Transform, Without, Vec2, Vec3 };
use bevy_rapier2d::na;
use bevy_rapier2d::na::{ Point2, Rotation2, Translation, Translation2, UnitComplex, Vector2 };
use bevy_rapier2d::rapier::math::{ Isometry, Vector };
use super::mob;


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

fn from_isometry(pos: Isometry<f32>, scale: f32, transform: &mut Mut<Transform>) {
    // Do not touch the 'z' part of the translation, used in Bevy for 2d layering
    *transform.translation.x_mut() = pos.translation.vector.x * scale;
    *transform.translation.y_mut() = pos.translation.vector.y * scale;

    let rot = na::UnitQuaternion::new(na::Vector3::z() * pos.rotation.angle());
    transform.rotation = Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w);
}

fn to_isometry(translation: Vec2, rotation_angle: f32) -> Isometry<f32> {
    Isometry::from_parts(
        Translation::from(Vector::new(translation.x(), translation.y())),
        UnitComplex::new(rotation_angle),
    )
}

pub fn swivel_at(
    mut query: Query<(&AttachedToEntity, &LooksAt, Mut<Transform>)>,
    entities: Query<Without<AttachedToEntity, &Transform>>,
) {
    for (target_entity, looks_at, mut transform) in query.iter_mut() {
        if let Ok(parent_transform) = entities.get(target_entity.0) {
            transform.translation = parent_transform.translation.clone();
            let translation = na::Translation2::new(
                transform.translation.x(),
                transform.translation.y(),
            );
            // Lol, this is so inefficient it's funny
            let (axis, angle) = transform.rotation.to_axis_angle();
            // Axis must always be Z or Idunno

            let point = translation.inverse_transform_point(&looks_at.0);
            let rot = Rotation2::rotation_between(
                &Vector2::new(0.0, 1.0),
                &Vector2::new(point.x, point.y)
            );
            //println!(
            let c = UnitComplex::from_rotation_matrix(&rot);
            transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), c.angle());
        }
    }
}
