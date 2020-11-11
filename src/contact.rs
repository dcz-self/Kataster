use bevy::ecs::QueryError;
use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{EventQueue, RigidBodyHandleComponent},
    rapier::{
        dynamics::RigidBodySet,
        geometry::{ContactEvent, Proximity},
    },
};
use super::bobox::BodyHandleToEntity;
use super::components::Borg;
use super::components::*;
use super::shooter;
use super::state::*;


enum Contacts {
    ShipAsteroid(Entity, Entity),
    LaserAsteroid(Entity, Entity),
}

pub fn contact_system(
    mut commands: Commands,
    mut asteroid_spawn_events: ResMut<Events<AsteroidSpawnEvent>>,
    mut explosion_spawn_events: ResMut<Events<ExplosionSpawnEvent>>,
    mut runstate: ResMut<RunState>,
    events: Res<EventQueue>,
    bh_to_e: Res<BodyHandleToEntity>,
    bodies: ResMut<RigidBodySet>,
    damages: Query<&Damage>,
    genotypes: Query<&shooter::Genotype>,
    mut ships: Query<Mut<Borg>>,
    mut lasers: Query<Mut<Laser>>,
    mut mobs: Query<Mut<Mob>>,
    handles: Query<&RigidBodyHandleComponent>,
) {
    if runstate.gamestate.is(GameState::Game) {
        let mut contacts = vec![];
        while let Ok(contact_event) = events.contact_events.pop() {
            match contact_event {
                ContactEvent::Started(h1, h2) => {
                    let e1 = *(bh_to_e.0.get(&h1).unwrap());
                    let e2 = *(bh_to_e.0.get(&h2).unwrap());
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
            let mut asteroids = &mut mobs;
            if proximity_event.new_status == Proximity::Intersecting {
                let e1 = *(bh_to_e.0.get(&proximity_event.collider1).unwrap());
                let e2 = *(bh_to_e.0.get(&proximity_event.collider2).unwrap());
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
                    let mut asteroids = &mut mobs;
                    let laser_handle = handles
                        .get(e1)
                        .unwrap()
                        .handle();
                    let asteroid = asteroids.get_mut(e2).unwrap();
                    let asteroid_handle = handles
                        .get(e2)
                        .unwrap()
                        .handle();
                    runstate.score = runstate.score.and_then(|score| {
                        Some(
                            score
                                + match asteroid.size {
                                    AsteroidSize::Small => 40,
                                    AsteroidSize::Medium => 20,
                                    AsteroidSize::Big => 10,
                                },
                        )
                    });
                    {
                        let laser_body = bodies.get(laser_handle).unwrap();
                        let asteroid_body = bodies.get(asteroid_handle).unwrap();

                        explosion_spawn_events.send(ExplosionSpawnEvent {
                            kind: ExplosionKind::LaserOnAsteroid,
                            x: laser_body.position.translation.x,
                            y: laser_body.position.translation.y,
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
                    borg.life -= damage.value;
                    if borg.life <= 0 {
                        explosion_spawn_events.send(ExplosionSpawnEvent {
                            kind: ExplosionKind::ShipDead,
                            x: player_body.position.translation.x,
                            y: player_body.position.translation.y,
                        });
                        commands.despawn_recursive(e1);
                        // FIXME: despawn LookAts
                        // This is kind of flaky... There could be a separate system to catch brainful despawns.
                        match genotypes.get(e1) {
                            Ok(genotype) => runstate.shooter_gene_pool.preserve(
                                genotype.clone(),
                                borg.time_alive as f64,
                            ),
                            Err(QueryError::NoSuchEntity) => {},
                            Err(e) => println!("Borg unuseable genotype {:?}", e),
                        }
                        runstate.gamestate.transit_to(GameState::GameOver);
                    } else {
                        explosion_spawn_events.send(ExplosionSpawnEvent {
                            kind: ExplosionKind::ShipContact,
                            x: player_body.position.translation.x,
                            y: player_body.position.translation.y,
                        });
                    }
                    let mob = mobs.get_mut(e2).unwrap();
                    runstate.mob_gene_pool.preserve(mob.genotype().clone());
                    commands.despawn(e2);
                }
            }
        }
    }
}
