use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::prelude::{ EventReader, KeyCode };
use bevy::window::CursorMoved;
use bevy_rapier2d::{
    physics::{RapierConfiguration, RigidBodyHandleComponent},
    rapier::dynamics::RigidBodySet,
};
use bevy_rapier2d::na::{ Point2, Translation2, Vector2 };
use super::arena;
use super::assets;
use super::components::{weapon_trigger, Borg, LooksAt, Weapon};
use super::state::*;


/// Marks entities walking with the keyboard.
pub struct KeyboardWalk;


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
    assets: Res<assets::Assets>,
    audio_output: Res<Audio>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut weapons: Query<(&Transform, Mut<Weapon>)>,
) {
    if !runstate.gamestate.is(GameState::Game) {
        return;
    }
    if mouse_button_input.pressed(MouseButton::Left) {
        for (transform, mut weapon) in weapons.iter_mut() {
            weapon_trigger(&mut weapon, transform, &mut commands, &asset_server, &assets, &audio_output);
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
