use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::prelude::{ EventReader, KeyCode };
use bevy::window::CursorMoved;
use bevy_rapier2d::{
    physics::{RapierConfiguration, RigidBodyHandleComponent},
    rapier::{
        dynamics::{RigidBodyBuilder, RigidBodySet},
        geometry::ColliderBuilder,
        //        math::Point,
    },
};
use bevy_rapier2d::na::{ Point2, Rotation2, Translation2, UnitComplex, Vector2 };
use super::arena;
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
    let arrow = asset_server.load("arrow.png");
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
        .with(Borg {
            rotation_speed: std::f32::consts::TAU,
            speed: 10.0,
            life: START_LIFE,
            cannon_timer: Timer::from_seconds(0.2, false),
            looks_at: Point2::new(0.0, 0.0),
        })
        .with(body)
        .with(collider)
        .with(ForStates {
            states: vec![GameState::Game, GameState::Pause, GameState::GameOver],
        })
        .with_children(|parent| {
            parent.spawn(SpriteComponents {
                transform: Transform {
                    translation: Vec3::new(0.0, 300.0, -5.0),
                    scale: Vec3::splat(1.0 / 10.0),
                    ..Default::default()
                },
                material: materials.add(arrow.into()),
                ..Default::default()
            });
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

pub fn point_at(
    mut bodies: ResMut<RigidBodySet>,
    cursor_moved: Res<Events<CursorMoved>>,
    body: &RigidBodyHandleComponent,
    mut borg: Mut<Borg>,
) {
    let mut body = bodies.get_mut(body.handle()).unwrap();
    for event in EventReader::<CursorMoved>::default().iter(&cursor_moved) {
        let event_position = Point2::new(event.position.x(), event.position.y());
        let target_position = Translation2::new(
            arena::WINDOW_WIDTH as f32 / 2.0,
            arena::WINDOW_HEIGHT as f32 / 2.0,
        ).inverse_transform_point(&event_position);
        let target_position = target_position * arena::CAMERA_SCALE;
        borg.looks_at = target_position;
        // TODO: move this to rendering/movement.
        // Position needs to be updated on every move.
        let point = body.position.translation.inverse_transform_point(&borg.looks_at);
        // Omg, why is dealing with Rapier so hard?
        // Every property has 3 representations
        // and they never convert into each other directly.
        let rot = Rotation2::rotation_between(
            &Vector2::new(0.0, 1.0),
            &Vector2::new(point.x, point.y)
        );
        body.position.rotation = UnitComplex::from_rotation_matrix(&rot);
    }
    body.wake_up(true);
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
        let speed = if input.pressed(KeyCode::W) || input.pressed(KeyCode::Up) {
            1
        } else {
            0
        };
        let rotation = if input.pressed(KeyCode::A) || input.pressed(KeyCode::Left) {
            1
        } else if input.pressed(KeyCode::D) || input.pressed(KeyCode::Right) {
            -1
        } else {
            0
        };
        
        let player = runstate.player.unwrap();
        if let Ok((body_handle, ship)) = query.get_mut(player) {
            let mut body = bodies.get_mut(body_handle.handle()).unwrap();
            let rotation = rotation as f32 * ship.rotation_speed;
            if rotation != body.angvel {
                body.wake_up(true);
                body.angvel = rotation;
            }
            // if neither rotation nor speed changed, can ignore
            body.wake_up(true);
            body.linvel = if speed != 0 {
                let velocity = body.position.rotation.transform_vector(&Vector2::y())
                    * speed as f32
                    * ship.speed;
                velocity
            } else {
                Vector2::zeros()
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
