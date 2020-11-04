use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::prelude::KeyCode;
use bevy_rapier2d::{
    na::Vector2,
    physics::{RapierConfiguration, RigidBodyHandleComponent},
    rapier::{
        dynamics::{RigidBodyBuilder, RigidBodySet},
        geometry::ColliderBuilder,
        //        math::Point,
    },
};
use super::components::*;
use super::laser::*;
use super::state::*;
use super::START_LIFE;


pub fn spawn_player(
    mut commands: Commands,
    mut runstate: ResMut<RunState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("playerShip2_red.png");
    let body = RigidBodyBuilder::new_dynamic();
    let collider = ColliderBuilder::ball(1.0);
    // The triangle Collider does not compute mass
    //let collider = ColliderBuilder::triangle(
    //    Point::new(1.0, -0.5),
    //    Point::new(0.0, 0.8),
    //    Point::new(-1.0, -0.5),
    //);
    commands
        .spawn(SpriteComponents {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -5.0),
                scale: Vec3::splat(1.0 / 37.0),
                ..Default::default()
            },
            material: materials.add(texture_handle.into()),
            ..Default::default()
        })
        .with(Ship {
            rotation_speed: std::f32::consts::TAU,
            thrust: 160.0,
            life: START_LIFE,
            cannon_timer: Timer::from_seconds(0.2, false),
        })
        .with(body)
        .with(collider)
        .with(ForStates {
            states: vec![GameState::Game, GameState::Pause, GameState::GameOver],
        });
    let player_entity = commands.current_entity().unwrap();
    runstate.player = Some(player_entity);

    // Helper points to visualize some points in space for Collider
    //commands
    //    .spawn(SpriteComponents {
    //        translation: Translation::new(1.2, -1.0, 2.0),
    //        material: materials.add(texture_handle.into()),
    //        scale: Scale(0.001),
    //        ..Default::default()
    //    })
    //    .spawn(SpriteComponents {
    //        translation: Translation::new(0.0, 1.0, 2.0),
    //        material: materials.add(texture_handle.into()),
    //        scale: Scale(0.001),
    //        ..Default::default()
    //    })
    //    .spawn(SpriteComponents {
    //        translation: Translation::new(-1.2, -1.0, 2.0),
    //        material: materials.add(texture_handle.into()),
    //        scale: Scale(0.001),
    //        ..Default::default()
    //    });
}

pub fn player_dampening_system(
    time: Res<Time>,
    runstate: Res<RunState>,
    mut bodies: ResMut<RigidBodySet>,
    query: Query<&RigidBodyHandleComponent>,
) {
    if runstate.gamestate.is(GameState::Game) {
        if let Ok(body_handle) = query.get(runstate.player.unwrap()) {
            let elapsed = time.delta_seconds;
            let mut body = bodies.get_mut(body_handle.handle()).unwrap();
            body.angvel = body.angvel * 0.001f32.powf(elapsed);
            body.linvel = body.linvel * 0.01f32.powf(elapsed);
        }
    }
}

pub fn ship_cannon_system(time: Res<Time>, mut ship: Query<Mut<Ship>>) {
    for mut ship in &mut ship.iter_mut() {
        ship.cannon_timer.tick(time.delta_seconds);
    }
}

pub fn user_input_system(
    commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<ColorMaterial>>,
    audio_output: Res<Audio>,
    mut runstate: ResMut<RunState>,
    input: Res<Input<KeyCode>>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
    mut bodies: ResMut<RigidBodySet>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut query: Query<(&RigidBodyHandleComponent, Mut<Ship>)>,
) {
    if !runstate.gamestate.is(GameState::StartMenu) {
        if input.just_pressed(KeyCode::Back) {
            runstate.gamestate.transit_to(GameState::StartMenu);
        }
    }
    if runstate.gamestate.is(GameState::Game) {
        let player = runstate.player.unwrap();
        let thrust = if input.pressed(KeyCode::W) || input.pressed(KeyCode::Up) {
            1
        } else { 0 };
        let rotation = if input.pressed(KeyCode::A) || input.pressed(KeyCode::Left) {
            1
        } else if input.pressed(KeyCode::D) || input.pressed(KeyCode::Right) {
            -1
        } else {
            0
        };
        
        if let Ok((body_handle, ship)) = query.get_mut(player) {
            let mut body = bodies.get_mut(body_handle.handle()).unwrap();
            let rotation = rotation as f32 * ship.rotation_speed;
            if rotation != body.angvel {
                body.wake_up(true);
                body.angvel = rotation;
            }
            if thrust != 0 {
                let force = body.position.rotation.transform_vector(&Vector2::y())
                    * thrust as f32
                    * ship.thrust;
                body.wake_up(true);
                body.apply_force(force);
            }
        }
        if input.pressed(KeyCode::Space) {
            if let Ok((body_handle, mut ship)) = query.get_mut(player) {
                if ship.cannon_timer.finished {
                    let body = bodies.get(body_handle.handle()).unwrap();
                    spawn_laser(commands, body, asset_server, materials, audio_output);
                    ship.cannon_timer.reset();
                }
            }
        }
        if input.just_pressed(KeyCode::Escape) {
            runstate.gamestate.transit_to(GameState::Pause);
            rapier_configuration.physics_pipeline_active = false;
        }
    } else if runstate.gamestate.is(GameState::StartMenu) {
        if input.just_pressed(KeyCode::Return) {
            runstate.gamestate.transit_to(GameState::Game);
        }
        if input.just_pressed(KeyCode::Escape) {
            app_exit_events.send(AppExit);
        }
    } else if runstate.gamestate.is(GameState::GameOver) {
        if input.just_pressed(KeyCode::Return) {
            runstate.gamestate.transit_to(GameState::StartMenu);
        }
        if input.just_pressed(KeyCode::Escape) {
            app_exit_events.send(AppExit);
        }
    } else if runstate.gamestate.is(GameState::Pause) {
        if input.just_pressed(KeyCode::Escape) {
            runstate.gamestate.transit_to(GameState::Game);
            rapier_configuration.physics_pipeline_active = true;
        }
    }
}
