/*! Contains stuff useful for debugging the rest of the code.
 * Not connected to anything */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use bevy::app;
use bevy::app::stage;
use bevy::ecs::{ Query, Res, SystemStage, Without };
use bevy::render;
use bevy::transform::components::{ GlobalTransform, Transform };

use bevy_rapier2d::{
    physics::RigidBodyHandleComponent,
    rapier::dynamics::RigidBodySet,
};
use super::components::{ AttachedToEntity, LooksAt };


use bevy::ecs::IntoSystem;


pub struct Plugin;


impl app::Plugin for Plugin {
    fn build(&self, app: &mut app::AppBuilder) {
        app.add_stage_before(render::stage::RENDER, "prerender", SystemStage::parallel())
            .add_stage_before(stage::POST_UPDATE, "prepost", SystemStage::parallel())
            .add_stage_before(stage::UPDATE, "preupd", SystemStage::parallel())
            .add_stage_before(stage::LAST, "prefinal", SystemStage::parallel())
            .add_system_to_stage("prerender", debug1.system())
            .add_system_to_stage("prepost", debug2.system())
            .add_system_to_stage("preupd", debug3.system())
            .add_system_to_stage("prefinal", debug4.system())
            .add_stage_before("FOLLOW", "prefollow", SystemStage::parallel())
            .add_system_to_stage("prefollow", debug5.system());
    }
}



pub fn debug_positions(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform, &GlobalTransform)>,
    entities: Query<(&RigidBodyHandleComponent, &Transform, &GlobalTransform), Without<AttachedToEntity>>,
    title: &str,
) {
    for (target_entity, _looks_at, transform, gtransform) in query.iter() {
        if let Ok((body_handle, parent_transform, parent_gtransform)) = entities.get(target_entity.0) {
            let body = bodies.get(body_handle.handle()).unwrap();
            println!("{}", title);
            println!("pb {:?}", body.position().translation.vector);
            println!("pt {:?}", parent_transform.translation);
            println!("pg {:?}", parent_gtransform.translation);
            println!("ct {:?}", transform.translation);
            println!("cg {:?}", gtransform.translation);
            
        }
    }
}


pub fn debug1(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform, &GlobalTransform)>,
    entities: Query<(&RigidBodyHandleComponent, &Transform, &GlobalTransform), Without<AttachedToEntity>>,
) {
    debug_positions(bodies, query, entities, "render with")
}

pub fn debug2(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform, &GlobalTransform)>,
    entities: Query<(&RigidBodyHandleComponent, &Transform, &GlobalTransform), Without<AttachedToEntity>>,
) {
    debug_positions(bodies, query, entities, "postupdate with")
}

pub fn debug3(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform, &GlobalTransform)>,
    entities: Query<(&RigidBodyHandleComponent, &Transform, &GlobalTransform), Without<AttachedToEntity>>,
) {
    debug_positions(bodies, query, entities, "update with")
}

pub fn debug4(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform, &GlobalTransform)>,
    entities: Query<(&RigidBodyHandleComponent, &Transform, &GlobalTransform), Without<AttachedToEntity>>,
) {
    debug_positions(bodies, query, entities, "finish with")
}

pub fn debug5(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform, &GlobalTransform)>,
    entities: Query<(&RigidBodyHandleComponent, &Transform, &GlobalTransform), Without<AttachedToEntity>>,
) {
    debug_positions(bodies, query, entities, "follow with")
}
