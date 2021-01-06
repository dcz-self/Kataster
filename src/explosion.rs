use bevy::prelude::*;
use bevy::sprite::entity::SpriteBundle;
use super::assets;
use super::components::*;
use super::state::*;


#[derive(Default)]
pub struct SpawnExplosionState {
    event_reader: EventReader<ExplosionSpawnEvent>,
}

pub struct Explosion {
    timer: Timer,
}
pub fn spawn(
    commands: &mut Commands,
    mut state: Local<SpawnExplosionState>,
    asset_server: Res<AssetServer>,
    assets: Res<assets::Assets>,
    audio_output: Res<Audio>,
    events: Res<Events<ExplosionSpawnEvent>>,
) {
    for event in state.event_reader.iter(&events) {
        let (texture_handle, sound_name, duration) = match event.kind {
            ExplosionKind::ShipDead => (
                None,
                "Explosion_ship.mp3",
                1.5,
            ),
            ExplosionKind::ShipContact => (
                None,
                "Explosion.mp3",
                0.5,
            ),
            ExplosionKind::LaserOnAsteroid => (
                assets.removal.clone(),
                "Explosion.mp3",
                0.5,
            ),
        };
        let t = Transform {
            translation: Vec3::new(event.x, event.y, -1.0),
            scale: Vec3::splat(1.0 / 16.0),
            ..Default::default()
        };
        commands
            .spawn(SpriteBundle {
                transform: t.clone(),
                // FIXME: make sure the global transform is not set explicitly
                global_transform: t.into(),
                material: texture_handle.unwrap_or(Default::default()),
                ..Default::default()
            })
            .with(Explosion {
                timer: Timer::from_seconds(duration, false),
            })
            .with(ForStates::from_func(GameState::is_arena));
        let sound = asset_server.load(sound_name);
        audio_output.play(sound);
    }
}

pub fn handle(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(Entity, Mut<Explosion>)>,
) {
    let elapsed = time.delta_seconds();
    for (entity, mut explosion) in &mut query.iter_mut() {
        explosion.timer.tick(elapsed);
        if explosion.timer.finished() {
            commands.despawn(entity);
        }
    }
}
