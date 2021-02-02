/*! Buttons. */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use bevy::app;
use bevy::asset::{ Assets, Handle };
use bevy::ecs::{ Commands, Res, ResMut, Resources };
use bevy::render::color::Color;
use bevy::sprite::ColorMaterial;


use bevy::ecs::FromResources;


pub struct Plugin;


impl app::Plugin for Plugin {
    fn build(&self, app: &mut app::AppBuilder) {
        //app.add_stage_before(render::stage::RENDER, "prerender", SystemStage::parallel())
        app.init_resource::<Materials>();
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
            indicated: materials.add(Color::rgb(0.1, 0.1, 0.5).into()),
            pressed: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
        }
    }
}
