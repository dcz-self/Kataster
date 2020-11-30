/*! Long-lived asset storage */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use bevy::asset;
use bevy::asset::Handle;
use bevy::ecs::{ Commands, Res, ResMut };
use bevy::math::Vec2;
use bevy::sprite::{ ColorMaterial, TextureAtlas };


const ASSET_DIR: &str = "./assets/";

pub struct Assets {
    pub borg: Option<Handle<ColorMaterial>>,
    pub projectile: Option<Handle<ColorMaterial>>,
    pub louse: Option<Handle<TextureAtlas>>,
    pub removal: Option<Handle<ColorMaterial>>,
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<asset::AssetServer>,
    mut materials: ResMut<asset::Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<asset::Assets<TextureAtlas>>,
) {
    let borg = asset_server.load("survivor-shoot_rifle_0.png");
    let projectile = asset_server.load("laserRed07.png");
    let removal = asset_server.load("flash00.png");
    let louse_texture = asset_server.load("louse.png");
    let louse = TextureAtlas::from_grid(louse_texture, Vec2::new(64.0, 64.0), 1, 1);
    commands.insert_resource(Assets {
        borg: Some(materials.add(borg.into())),
        projectile: Some(materials.add(projectile.into())),
        removal: Some(materials.add(removal.into())),
        louse: Some(texture_atlases.add(louse)),
    });
}
