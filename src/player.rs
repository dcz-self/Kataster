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
use bevy_rapier2d::na::{ Point2, Translation2, Vector2 };
use super::arena;
use super::components::{AttachedToEntity, Borg, LooksAt, Weapon};
use super::laser as projectile;
use super::state::*;
use super::START_LIFE;


/// Marks entities walking with the keyboard.
pub struct KeyboardWalk;


pub fn spawn_player(
    mut commands: Commands,
    mut runstate: ResMut<RunState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("survivor-shoot_rifle_0.png");
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
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Borg {
            rotation_speed: std::f32::consts::TAU,
            speed: 10.0,
            life: START_LIFE,
        })
        .with(body)
        .with(collider)
        .with(ForStates {
            states: vec![GameState::Game, GameState::Pause],
        })
        .with(KeyboardWalk)
        .with_children(|parent| {
            parent.spawn(SpriteComponents {
                transform: Transform {
                    translation: Vec3::new(0.0, 100.0, 0.0),
                    scale: Vec3::splat(1.0/32.0),
                    ..Default::default()
                },
                material: materials.add(arrow.into()),
                ..Default::default()
            }).with(ForStates {
                states: vec![GameState::Game, GameState::Pause],
            });
        });
    
    let borg_entity = commands.current_entity().unwrap();
    commands
        .spawn(SpriteComponents {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::splat(1.0/8.0),
                ..Default::default()
            },
            material: materials.add(texture_handle.into()),
            ..Default::default()
        })
        .with(Weapon {
            repeat_timer: Timer::from_seconds(0.2, false),
        })
        .with(AttachedToEntity(borg_entity))
        .with(LooksAt::default())
        .with(ForStates {
            states: vec![GameState::Game, GameState::Pause],
        });
    runstate.player = Some(borg_entity);

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


pub fn point_at_mouse(
    cursor_moved: Res<Events<CursorMoved>>,
    mut looks_at: Mut<LooksAt>,
) {
    for event in EventReader::<CursorMoved>::default().iter(&cursor_moved) {
        let event_position = Point2::new(event.position.x(), event.position.y());
        let target_position = Translation2::new(
            arena::WINDOW_WIDTH as f32 / 2.0,
            arena::WINDOW_HEIGHT as f32 / 2.0,
        ).inverse_transform_point(&event_position);
        let target_position = target_position * arena::CAMERA_SCALE;
        looks_at.0 = target_position;
        // TODO: move this to rendering/movement.
        // Position needs to be updated on every move.
        
        //transform.rotation
        /*
        let point = body.position.translation.inverse_transform_point(&borg.looks_at);
        // Omg, why is dealing with Rapier so hard?
        // Every property has 3 representations
        // and they never convert into each other directly.
        let rot = Rotation2::rotation_between(
            &Vector2::new(0.0, 1.0),
            &Vector2::new(point.x, point.y)
        );
        body.position.rotation = UnitComplex::from_rotation_matrix(&rot);*/
    }
}


pub fn mouse_shoot(
    mut commands: Commands,
    runstate: Res<RunState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio_output: Res<Audio>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut weapons: Query<(&Transform, Mut<Weapon>)>,
) {
    if !runstate.gamestate.is(GameState::Game) {
        return;
    }
    if mouse_button_input.pressed(MouseButton::Left) {
        for (transform, mut weapon) in weapons.iter_mut() {
            if weapon.repeat_timer.finished {
                projectile::spawn(&mut commands, &asset_server, &mut materials, &audio_output, transform);
                weapon.repeat_timer.reset();
            }
        }
    }
}

pub fn keyboard_walk(
    input: Res<Input<KeyCode>>,
    mut bodies: ResMut<RigidBodySet>,
    query: Query<(&RigidBodyHandleComponent, &Borg, &KeyboardWalk)>,
) {
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
    
    for (body_handle, borg, _walk) in query.iter() {
        let mut body = bodies.get_mut(body_handle.handle()).unwrap();
        let rotation = rotation as f32 * borg.rotation_speed;
        if rotation != body.angvel {
            body.wake_up(true);
            body.angvel = rotation;
        }
        // if neither rotation nor speed changed, can ignore
        body.wake_up(true);
        body.linvel = if speed != 0 {
            let velocity = body.position.rotation.transform_vector(&Vector2::y())
                * speed as f32
                * borg.speed;
            velocity
        } else {
            Vector2::zeros()
        }
    }
}

pub fn user_input_system(
    mut runstate: ResMut<RunState>,
    input: Res<Input<KeyCode>>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
    mut app_exit_events: ResMut<Events<AppExit>>,
) {
    if !runstate.gamestate.is(GameState::StartMenu) {
        if input.just_pressed(KeyCode::Back) {
            runstate.gamestate.transit_to(GameState::StartMenu);
        }
    }
    if runstate.gamestate.is(GameState::Game) {
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
