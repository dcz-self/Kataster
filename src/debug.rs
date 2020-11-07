/*! Contains stuff useful for debugging the rest of the code.
 * Not connected to anything */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */


/*
pub fn debug1(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &GlobalTransform)>,
    entities: Query<(&RigidBodyHandleComponent, Without<AttachedToEntity, &GlobalTransform>)>,
) {
    for (target_entity, looks_at, mut transform) in query.iter() {
        if let Ok((body_handle, parent_transform)) = entities.get(target_entity.0) {
            let body = bodies.get(body_handle.handle()).unwrap();
            println!("render with ");
            println!("p {:?}", parent_transform.translation);
            println!("pb {:?}", body.position.translation.vector);
            println!("c {:?}", transform.translation);
        }
    }
}

pub fn debug2(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform)>,
    entities: Query<(&RigidBodyHandleComponent, Without<AttachedToEntity, &Transform>)>,
) {
    for (target_entity, looks_at, mut transform) in query.iter() {
        if let Ok((body_handle, parent_transform)) = entities.get(target_entity.0) {
            let body = bodies.get(body_handle.handle()).unwrap();
            println!("postupdate with ");
            println!("p {:?}", parent_transform.translation);
            println!("pb {:?}", body.position.translation.vector);
            println!("c {:?}", transform.translation);
        }
    }
}

pub fn debug3(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform)>,
    entities: Query<(&RigidBodyHandleComponent, Without<AttachedToEntity, &Transform>)>,
) {
    for (target_entity, looks_at, mut transform) in query.iter() {
        if let Ok((body_handle, parent_transform)) = entities.get(target_entity.0) {
            let body = bodies.get(body_handle.handle()).unwrap();
            println!("update with ");
            println!("p {:?}", parent_transform.translation);
            println!("pb {:?}", body.position.translation.vector);
            println!("c {:?}", transform.translation);
        }
    }
}

pub fn debug4(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform)>,
    entities: Query<(&RigidBodyHandleComponent, Without<AttachedToEntity, &Transform>)>,
) {
    for (target_entity, looks_at, mut transform) in query.iter() {
        if let Ok((body_handle, parent_transform)) = entities.get(target_entity.0) {
            let body = bodies.get(body_handle.handle()).unwrap();
            println!("finish with ");
            println!("p {:?}", parent_transform.translation);
            println!("pb {:?}", body.position.translation.vector);
            println!("c {:?}", transform.translation);
        }
    }
}

pub fn debug5(
    bodies: Res<RigidBodySet>,
    query: Query<(&AttachedToEntity, &LooksAt, &Transform)>,
    entities: Query<(&RigidBodyHandleComponent, Without<AttachedToEntity, &Transform>)>,
) {
    for (target_entity, looks_at, mut transform) in query.iter() {
        if let Ok((body_handle, parent_transform)) = entities.get(target_entity.0) {
            let body = bodies.get(body_handle.handle()).unwrap();
            println!("swivel with ");
            println!("p {:?}", parent_transform.translation);
            println!("pb {:?}", body.position.translation.vector);
            println!("c {:?}", transform.translation);
        }
    }
}*/

// TODO: this should be a plugin
/*        .add_stage_before(render::stage::RENDER, "prerender")
        .add_stage_before(stage::POST_UPDATE, "prepost")
        .add_stage_before(stage::UPDATE, "preupd")
        .add_stage_before(stage::LAST, "prefinal")
        .add_system_to_stage("prerender", components::debug1.system())
        .add_system_to_stage("prepost", components::debug2.system())
        .add_system_to_stage("preupd", components::debug3.system())
        .add_system_to_stage("prefinal", components::debug4.system())
        .add_stage_before("SWIVEL", "preswivel")
        .add_system_to_stage("preswivel", components::debug5.system()) */
