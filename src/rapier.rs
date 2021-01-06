/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use bevy::ecs::Commands;
use bevy_rapier2d::rapier::dynamics::RigidBodyBuilder;


pub trait WithBody {
    /// Adds the body to the entity while setting user data to entity number.
    fn with_body(&mut self, body_builder: RigidBodyBuilder) -> &mut Self;
}


impl WithBody for Commands {
    fn with_body(&mut self, body_builder: RigidBodyBuilder) -> &mut Self {
        let entity = self.current_entity().unwrap();
        let body_builder = body_builder.user_data(entity.to_bits() as u128);
        self.insert(entity, (body_builder, ))
    }
}
