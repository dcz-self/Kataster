use super::components::*;
use super::state::*;
use bevy::prelude::*;

#[derive(Default)]
pub struct SpawnExplosionState {
    event_reader: EventReader<ExplosionSpawnEvent>,
}

pub struct Explosion {
    timer: Timer,
    start_scale: f32,
    end_scale: f32,
}
pub fn spawn_explosion(
    mut commands: Commands,
    mut state: Local<SpawnExplosionState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio_output: Res<Audio>,
    events: Res<Events<ExplosionSpawnEvent>>,
) {
    for event in state.event_reader.iter(&events) {
        let (texture_name, sound_name, start_scale, end_scale, duration) = match event.kind {
            ExplosionKind::ShipDead => (
                "explosion01.png",
                "Explosion_ship.mp3",
                0.1 / 15.0,
                0.5 / 15.0,
                1.5,
            ),
            ExplosionKind::ShipContact => (
                "flash00.png",
                "Explosion.mp3",
                0.05 / 15.0,
                0.1 / 15.0,
                0.5,
            ),
            ExplosionKind::LaserOnAsteroid => (
                "flash00.png",
                "Explosion.mp3",
                0.1 / 15.0,
                0.15 / 15.0,
                0.5,
            ),
        };
        let texture_handle = asset_server.load(texture_name);
        commands
            .spawn(SpriteComponents {
                transform: {
                    Transform::from_translation(Vec3::new(event.x, event.y, -1.0))
                        .mul_transform(Transform::from_scale(Vec3::splat(1.0 / 16.0)))
                },
                material: materials.add(texture_handle.into()),
                ..Default::default()
            })
            .with(Explosion {
                timer: Timer::from_seconds(duration, false),
                start_scale,
                end_scale,
            })
            .with(ForStates {
                states: vec![GameState::Game, GameState::Pause, GameState::GameOver],
            });
        let sound = asset_server.load(sound_name);
        audio_output.play(sound);
    }
}

pub fn handle_explosion(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, Mut<Transform>, Mut<Explosion>)>,
) {
    let elapsed = time.delta_seconds;
    for (entity, mut transform, mut explosion) in &mut query.iter_mut() {
        explosion.timer.tick(elapsed);
        if explosion.timer.finished {
            commands.despawn(entity);
        }
    }
}
