use super::components::*;
use super::state::*;
use bevy::prelude::*;
use bevy::math::{ Quat, Vec3 };
use bevy_rapier2d::{
    na::Vector2,
    rapier::{
        dynamics::RigidBodyBuilder,
        geometry::ColliderBuilder,
        //        math::Point,
    },
};
use bevy_rapier2d::rapier::math::{ Isometry, Translation, Vector };
use bevy_rapier2d::na::UnitComplex;


fn into_isometry(translation: Vec3, rotation: Quat) -> Isometry<f32> {
    Isometry::from_parts(
        Translation::from(Vector::new(translation.x(), translation.y())),
        UnitComplex::new(rotation.to_axis_angle().1),
    )
}

pub fn spawn(
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    audio_output: &Res<Audio>,
    transform: &Transform,
) {
    let texture_handle = asset_server.load("laserRed07.png");
    let isometry = into_isometry(
        transform.translation.clone(),
        transform.rotation.clone()
    );
    let v = isometry.rotation * Vector2::y() * 50.0;
    let body = RigidBodyBuilder::new_dynamic()
        .position(isometry)
        .linvel(v.x, v.y);
    let collider = ColliderBuilder::cuboid(0.25, 1.0).sensor(true);
    let transform = Transform {
        translation: Vec3::new(
            transform.translation.x(),
            transform.translation.y(),
            -4.0,
        ),
        scale: Vec3::splat(1.0 / 18.0),
        ..Default::default()
    };
    commands
        .spawn(SpriteComponents {
            transform,
            // Spawn needs to happen before transform in order for the global
            // to be corrrectly rendered.
            // But it also must happen after transform to
            // start from the correct position.
            // Compromise: update renderer position manually.
            global_transform: transform.into(),
            material: materials.add(texture_handle.into()),
            ..Default::default()
        })
        .with(Laser {
            despawn_timer: Timer::from_seconds(5.0, false),
        })
        .with(body)
        .with(collider)
        .with(ForStates {
            states: vec![GameState::Game, GameState::Pause, GameState::GameOver],
        });
    let sound = asset_server.load("sfx_laser1.mp3");
    audio_output.play(sound);
}

pub fn despawn_laser_system(
    mut commands: Commands,
    runstate: Res<RunState>,
    time: Res<Time>,
    mut query: Query<(Entity, Mut<Laser>)>,
) {
    if runstate.gamestate.is(GameState::Game) {
        for (entity, mut laser) in &mut query.iter_mut() {
            laser.despawn_timer.tick(time.delta_seconds);
            if laser.despawn_timer.finished {
                commands.despawn(entity);
            }
        }
    }
}
