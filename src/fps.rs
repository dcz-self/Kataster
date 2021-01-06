use bevy::app;
use bevy::asset::AssetServer;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::ecs::{ Commands, Query, Res };
use bevy::render::color::Color;
use bevy::text::TextStyle;
use bevy::ui::entity::{ TextBundle, CameraUiBundle };
use bevy::ui::prelude::AlignSelf;
use bevy::ui::widget::{ Text };
use bevy::ui::Style;


use bevy::ecs::IntoSystem;


// A unit struct to help identify the FPS UI component, since there may be many Text components
struct FpsText;

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<(&mut Text, &FpsText)>) {
    for (mut text, _tag) in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.value = format!("FPS: {:.2}", average);
            }
        }
    }
}

fn setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands
        // 2d camera
        .spawn(CameraUiBundle::default())
        // texture
        .spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                value: "FPS:".to_string(),
                font: asset_server.load("kenvector_future.ttf"),
                style: TextStyle {
                    font_size: 10.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(FpsText);
}


pub struct Plugin;


impl app::Plugin for Plugin {
    fn build(&self, app: &mut app::AppBuilder) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(setup.system())
            .add_system(text_update_system.system());
    }
}
