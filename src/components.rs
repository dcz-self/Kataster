use bevy::asset::AssetServer;
use bevy::audio::Audio;
use bevy::core::Time;
use bevy::ecs::{ Commands, Res };
use bevy::prelude::{ Entity, GlobalTransform, Mut, Quat, Query, Timer, Transform, Without, Vec3 };
use bevy_rapier2d::na;
use bevy_rapier2d::na::{ Point2, Rotation2, UnitComplex, Vector2 };
use super::assets;
use super::mob;
use super::laser as projectile;


pub struct AttachedToEntity(pub Entity);

pub struct Borg {
    /// Ship rotation speed in rad/s
    pub rotation_speed: f32,
    /// Max movement speed
    pub speed: f32,
    /// Ship life points
    pub life: u32,
    pub time_alive: f32,
    /// Better way of estimating success?
    pub score: u32,
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
    pub life: u32,
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

/// The entity is a weapon, and must have a Transform
/// because it gives direction to projectiles.
pub struct Weapon {
    pub repeat_timer: Timer,
}


pub fn weapon_repeat(time: Res<Time>, mut weapons: Query<Mut<Weapon>>) {
    for mut weapon in &mut weapons.iter_mut() {
        weapon.repeat_timer.tick(time.delta_seconds);
    }
}

pub fn weapon_trigger(
    weapon: &mut Weapon,
    transform: &Transform,
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    assets: &Res<assets::Assets>,
    audio_output: &Res<Audio>,
) {
    if weapon.repeat_timer.finished {
        projectile::spawn(&mut commands, &asset_server, &assets, &audio_output, transform);
        weapon.repeat_timer.reset();
    }
}


pub struct Laser {
    pub despawn_timer: Timer,
}
pub struct Damage {
    pub value: u32,
}


pub fn follow(
    mut query: Query<(&AttachedToEntity, Mut<GlobalTransform>, Mut<Transform>)>,
    entities: Query<Without<AttachedToEntity, &GlobalTransform>>,
) {
    for (target_entity, mut gtransform, mut transform) in query.iter_mut() {
        if let Ok(parent_transform) = entities.get(target_entity.0) {
            transform.translation = parent_transform.translation.clone();
            // Rapier is broken by not updating deterministically,
            // so let's work around transforms and just update them manually.
            gtransform.translation = parent_transform.translation.clone();
        }
    }
}

pub fn swivel_at(
    mut query: Query<(&LooksAt, Mut<GlobalTransform>, Mut<Transform>)>,
) {
    for (looks_at, mut gtransform, mut transform) in query.iter_mut() {
        // Rapier is broken by not updating deterministically,
        // so let's work around transforms and just update them manually.
        let translation = na::Translation2::new(
            transform.translation.x(),
            transform.translation.y(),
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
