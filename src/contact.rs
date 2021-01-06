use bevy::ecs::QueryError;
use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{EventQueue, RigidBodyHandleComponent},
    rapier::{
        dynamics::RigidBodySet,
        geometry::{ContactEvent, Proximity},
    },
};
use super::components::Borg;
use super::components::*;
use super::shooter;
use super::state::*;


enum Contacts {
    ShipAsteroid(Entity, Entity),
    LaserAsteroid(Entity, Entity),
}

pub fn contact_system(
    commands: &mut Commands,
    mut explosion_spawn_events: ResMut<Events<ExplosionSpawnEvent>>,
    mut runstate: ResMut<RunState>,
    events: Res<EventQueue>,
    bodies: ResMut<RigidBodySet>,
    damages: Query<&Damage>,
    genotypes: Query<&shooter::Genotype>,
    mut ships: Query<Mut<Borg>>,
    mut lasers: Query<Mut<Laser>>,
    mut mobs: Query<Mut<Mob>>,
    handles: Query<&RigidBodyHandleComponent>,
) {
    if !runstate.gamestate.current().is_arena() {
        return;
    }
     
    let mut contacts = vec![];
    while let Ok(contact_event) = events.contact_events.pop() {
        match contact_event {
            ContactEvent::Started(h1, h2) => {
                let b1 = bodies.get(h1).unwrap();
                let b2 = bodies.get(h2).unwrap();
                let e1 = Entity::from_bits(b1.user_data as u64);
                let e2 = Entity::from_bits(b2.user_data as u64);
                if ships.get_mut(e1).is_ok() && damages.get(e2).is_ok() {
                    contacts.push(Contacts::ShipAsteroid(e1, e2));
                } else if ships.get_mut(e2).is_ok() && damages.get(e1).is_ok() {
                    contacts.push(Contacts::ShipAsteroid(e2, e1));
                }
            }
            _ => (),
        };
    }
    while let Ok(proximity_event) = events.proximity_events.pop() {
        let asteroids = &mut mobs;
        if proximity_event.new_status == Proximity::Intersecting {
            let b1 = bodies.get(proximity_event.collider1).unwrap();
            let b2 = bodies.get(proximity_event.collider2).unwrap();
            let e1 = Entity::from_bits(b1.user_data as u64);
            let e2 = Entity::from_bits(b2.user_data as u64);
            if asteroids.get_mut(e2).is_ok() && lasers.get_mut(e1).is_ok() {
                contacts.push(Contacts::LaserAsteroid(e1, e2));
            } else if asteroids.get_mut(e1).is_ok() && lasers.get_mut(e2).is_ok() {
                contacts.push(Contacts::LaserAsteroid(e2, e1));
            }
        }
    }
    for contact in contacts.into_iter() {
        match contact {
            Contacts::LaserAsteroid(e1, e2) => {
                let asteroids = &mut mobs;
                let laser_handle = handles
                    .get(e1)
                    .unwrap()
                    .handle();
                let asteroid = asteroids.get_mut(e2).unwrap();
                runstate.score = runstate.score.map(|score| {
                    score
                        + match asteroid.size {
                            AsteroidSize::Small => 40,
                            AsteroidSize::Medium => 20,
                            AsteroidSize::Big => 10,
                        }
                });
                {
                    let laser_body = bodies.get(laser_handle).unwrap();

                    explosion_spawn_events.send(ExplosionSpawnEvent {
                        kind: ExplosionKind::LaserOnAsteroid,
                        x: laser_body.position().translation.x,
                        y: laser_body.position().translation.y,
                    });
                }
                commands.despawn(e1);
                commands.despawn(e2);
            }
            Contacts::ShipAsteroid(e1, e2) => {
                let player_body = bodies
                    .get(
                        handles
                            .get(e1)
                            .unwrap()
                            .handle(),
                    )
                    .unwrap();
                let mut borg = ships.get_mut(e1).unwrap();
                let damage = damages.get(e2).unwrap();
                borg.life = borg.life.saturating_sub(damage.value);
                if borg.life <= 0 {
                    explosion_spawn_events.send(ExplosionSpawnEvent {
                        kind: ExplosionKind::ShipDead,
                        x: player_body.position().translation.x,
                        y: player_body.position().translation.y,
                    });
                    commands.despawn_recursive(e1);
                    // FIXME: despawn LookAts
                    // This is kind of flaky... There could be a separate system to catch brainful despawns.
                    let score = runstate.score.unwrap_or(0);
                    match genotypes.get(e1) {
                        Ok(genotype) => runstate.shooter_gene_pool.preserve(
                            genotype.clone(),
                            score as f64,
                        ),
                        Err(QueryError::NoSuchEntity) => {},
                        Err(e) => println!("Borg unuseable genotype {:?}", e),
                    }
                } else {
                    explosion_spawn_events.send(ExplosionSpawnEvent {
                        kind: ExplosionKind::ShipContact,
                        x: player_body.position().translation.x,
                        y: player_body.position().translation.y,
                    });
                }
                let mob = mobs.get_mut(e2).unwrap();
                runstate.mob_gene_pool.preserve(mob.genotype().clone());
                commands.despawn(e2);
            }
        }
    }
}
