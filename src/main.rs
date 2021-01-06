use bevy::ecs::SystemStage;
use bevy::prelude::*;
use bevy::ui::entity::CameraUiBundle;
use bevy_rapier2d::na::Vector2;
use bevy_rapier2d::physics::RapierConfiguration;
use bevy_rapier2d::physics::RapierPhysicsPlugin;


mod arena;
mod assets;
mod brain;
mod components;
mod contact;
mod debug;
mod explosion;
mod fps;
mod geometry;
mod laser;
mod mob;
//mod paq;
mod player;
mod shooter;
mod state;
//mod tga;
//mod treeb;
mod ui;
#[macro_use]
mod util;
mod viewer;

use arena::*;
use components::*;
use debug::Plugin as DebugPlugin;
use explosion::*;
use laser as projectile;
use player::*;
use state::*;
use ui::*;


use bevy::ecs::IntoSystem;


fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Breedmatic".to_string(),
            width: WINDOW_WIDTH as f32,
            height: WINDOW_HEIGHT as f32,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb_u8(5, 5, 10)))
        .add_event::<AsteroidSpawnEvent>()
        .add_event::<ExplosionSpawnEvent>()
        .add_event::<shooter::BrainFed>()
        .add_plugin(RapierPhysicsPlugin)
        .add_plugin(fps::Plugin)
        //.add_plugin(viewer::Plugin)
        .add_plugins(DefaultPlugins)
        //.init_asset_loader::<paq::Loader>()
        .add_resource(RapierConfiguration {
            gravity: Vector2::zeros(),
            ..Default::default()
        })
        // Following another entity needs to take place
        // after Rapier had its go updating the parent's position.
        .add_stage_after(stage::POST_UPDATE, "FOLLOW", SystemStage::parallel())
        .add_stage_after("FOLLOW", "SHOOT", SystemStage::parallel())
        // Stage added after add_default_plugins, else something messes up CLEANUP
        .add_stage_after(stage::POST_UPDATE, "HANDLE_CONTACT", SystemStage::parallel())
        .add_stage_after("HANDLE_CONTACT", "HANDLE_EXPLOSION", SystemStage::parallel())
        .add_stage_after("HANDLE_EXPLOSION", "HANDLE_EXIT", SystemStage::parallel())
        .add_stage_after("HANDLE_EXIT", "HANDLE_RUNSTATE", SystemStage::parallel())
        .add_stage_after("HANDLE_RUNSTATE", "CLEANUP", SystemStage::parallel()) // CLEANUP stage required by RapierUtilsPlugin
        .add_system_to_stage(stage::POST_UPDATE, arena::check_end.system())
        .add_system(hold_borgs.system())
        .add_system(mob::count_lifetime.system())
        .add_system_to_stage(stage::POST_UPDATE, user_input_system.system())
        .add_system_to_stage(stage::POST_UPDATE, arena::end_ai_round.system())
        .add_system_to_stage(stage::POST_UPDATE, arena::start_ai_round.system())
        .add_system(player::point_at_mouse.system())
        .add_system(player::keyboard_walk.system())
        .add_system_to_stage("FOLLOW", components::swivel_at.system())
        .add_system_to_stage("FOLLOW", components::follow.system())
        .add_system_to_stage("SHOOT", player::mouse_shoot.system())
        // TODO: those should both operate on a copy of mob positions,
        // otherwise one will use updated values.
        // Maybe use Transform and update Body.
        .add_system(mob::think.system())
        .add_system(shooter::think.system())
        .add_system(components::weapon_repeat.system())
        .add_system(projectile::despawn_laser_system.system())
        .add_system(explosion::handle.system())
        .add_system(setup_arena.system())
        .add_system(arena_spawn.system())
        .add_system(start_menu.system())
        .add_system(game_ui_spawn.system())
        .add_system(ui::score.system())
        .add_system(life_ui_system.system())
        .add_system(gameover_menu.system())
        .add_system(pause_menu.system())
        //.add_system(draw_blink_system.system())
        .add_startup_system(assets::setup.system())
        .add_startup_system(setup.system())
        .add_system_to_stage(stage::POST_UPDATE, contact::contact_system.system())
        .add_system_to_stage("HANDLE_CONTACT", spawn_asteroid_system.system())
        .add_system_to_stage("HANDLE_EXPLOSION", spawn_explosion.system())
        .add_system_to_stage("HANDLE_RUNSTATE", runstate_fsm.system())
        .add_system_to_stage("HANDLE_EXIT", state_exit_despawn.system())
        .add_resource(RunState::new(GameState::MainMenu))
        //.add_plugin(DebugPlugin)
        .run();
}

/// UiCamera and Camera2d are spawn once and for all.
/// Despawning them does not seem to be the way to go in bevy.
pub fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_scale(Vec3::splat(CAMERA_SCALE)),
            ..Default::default()
        })
        .spawn(CameraUiBundle::default());
    let texture_handle = asset_server
        .load("diffus.png");
    //let paq_handle: Handle<Paq> = asset_server.load("crimson.paq");
    //let texture_handle = asset_server.load("crimson.paq:panel.tga");
    commands.spawn(SpriteBundle {
        transform: {
            Transform::from_translation(Vec3::new(0.0, 0.0, -10.0))
                .mul_transform(Transform::from_scale(Vec3::splat(CAMERA_SCALE)))
        },
        material: materials.add(texture_handle.into()),
        ..Default::default()
    });
}
