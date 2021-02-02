/*! Buttons. */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use bevy::app;
use bevy::asset::{ Assets, Handle };
use bevy::ecs::{ Changed, Commands, Mut, Mutated, Query, Res, ResMut, Resources, With };
use bevy::render::color::Color;
use bevy::sprite::ColorMaterial;
use bevy::ui::Interaction;
use bevy::ui::widget::Button;


use bevy::ecs::FromResources;
use bevy::prelude::IntoSystem;


pub struct Plugin;


impl app::Plugin for Plugin {
    fn build(&self, app: &mut app::AppBuilder) {
        app.init_resource::<Materials>()
            .add_system(highlight.system());
    }
}


pub struct Materials {
    pub normal: Handle<ColorMaterial>,
    /// Hover or kb focus
    pub indicated: Handle<ColorMaterial>,
    pub pressed: Handle<ColorMaterial>,
}


impl FromResources for Materials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        Materials {
            normal: materials.add(Color::rgb(0.1, 0.1, 0.1).into()),
            indicated: materials.add(Color::rgb(0.1, 0.1, 0.3).into()),
            pressed: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
        }
    }
}

fn highlight(
    button_materials: Res<Materials>,
    mut interactions: Query<
        (&Interaction, Mut<Handle<ColorMaterial>>),
        (Mutated<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut material) in interactions.iter_mut() {
        *material = match interaction {
            Interaction::Clicked => button_materials.pressed.clone(),
            Interaction::Hovered => button_materials.indicated.clone(),
            Interaction::None => button_materials.normal.clone(),
        };
    }
}
