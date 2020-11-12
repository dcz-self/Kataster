/*! Long-lived asset storage */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use bevy::asset;
use bevy::asset::Handle;
use bevy::ecs::{ Commands, Res, ResMut };
use bevy::sprite::ColorMaterial;


const ASSET_DIR: &str = "./assets/";

pub struct Assets {
    pub projectile: Option<Handle<ColorMaterial>>,
    pub removal: Option<Handle<ColorMaterial>>,
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<asset::AssetServer>,
    mut materials: ResMut<asset::Assets<ColorMaterial>>,
) {
    let projectile = asset_server.load("laserRed07.png");
    let removal = asset_server.load("flash00.png");
    commands.insert_resource(Assets {
        projectile: Some(materials.add(projectile.into())),
        removal: Some(materials.add(removal.into())),
    });
}
