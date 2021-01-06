/*! Live brain viewer */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */
use bevy::app;
use bevy::app::{ Events, EventReader };
use bevy::asset::{ Assets, AssetServer };
use bevy::ecs::{ Commands, Entity, Local, Query, Res, ResMut, SystemStage, With };
use bevy::math::{ Rect, Size, Vec3 };
use bevy::render::color::Color;
use bevy::render::mesh::Mesh;
use bevy::sprite::ColorMaterial;
use bevy::transform::hierarchy::ChildBuilder;
use bevy::ui::entity::NodeBundle;
use bevy::ui::{ AlignItems, Node, PositionType, Style, Val };
use bevy_prototype_lyon;
use bevy_prototype_lyon::prelude::{ point, primitive, FillOptions, PathBuilder, ShapeType, StrokeOptions, TessellationMode };
use crate::components::Borg;
use crate::shooter;
use crate::shooter::BrainFed;
use std::collections::HashMap;


use bevy::prelude::BuildChildren;
use bevy::ecs::IntoSystem;
use std::iter::FromIterator;


pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut app::AppBuilder) {
        app.add_stage_after(app::stage::UPDATE, "draw_imm", SystemStage::parallel())
            .add_system_to_stage("draw_imm", draw_preview.system())
            .add_system_to_stage(app::stage::UPDATE, kill_preview.system());
    }
}


struct Preview;

fn kill_preview(
    commands: &mut Commands,
    preview_ui: Query<Entity, With<Preview>>,
) {
    for e in preview_ui.iter() {
        commands.despawn(e);
    }
}


#[derive(Default)]
pub struct FedEvents {
    event_reader: EventReader<BrainFed>,
}

fn val_to_color(val: f32) -> Color {
    let calm = Vec3::new(0.1, 0.1, 0.1);
    let high = Vec3::new(1.0, 1.0, 0.3);
    let anti = Vec3::new(0.3, 0.8, 1.0);
    let norm = val / (1.0 + val.abs()); // softsign. Starts fast and doesn't stop increasing much after 1.0.
    let color = calm
        + if norm > 0.0 {
            high - calm
        } else {
            calm - anti
        }
        * norm;
    // No good reason. Ideally it should be linear in respect to how it is perceived.
    Color::rgb_linear(color[0], color[1], color[2])
}

fn draw_brain(
    commands: &mut ChildBuilder,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    brain: &shooter::Brain,
    inputs: &shooter::Inputs,
) {
    let vert_space = 50.0;
    let horz_space = 50.0;
    let pad = 20.0;
    
    let layers = brain.get_layers();

    let node_positions: HashMap<shooter::NodeId, _>
        = HashMap::from_iter(
            layers.into_iter().enumerate()
                .flat_map(|(lidx, nodes)| {
                    nodes.into_iter().enumerate()
                        .map(move |(nidx, id)| (id, (nidx, lidx)))
                })
        );

    let signals = brain.find_signals(inputs.clone());

    // Draw lines underneath
    for signal in &signals {
        match signal {
            shooter::Signal::Synapse { value, from, to } => {
                let (from_num, from_layer) = node_positions.get(from).unwrap();
                let (to_num, to_layer) = node_positions.get(to).unwrap();
                let mut builder = PathBuilder::new();
                builder.line_to(point(
                    horz_space * *to_num as f32,
                    -vert_space * *to_layer as f32,
                ));
                builder.line_to(point(
                    horz_space * *from_num as f32,
                    -vert_space * *from_layer as f32,
                ));
                let path = builder.build();
                commands
                    .spawn(path.stroke(
                        materials.add(val_to_color(*value).into()).clone(),
                        &mut meshes,
                        Vec3::new(0.0, 0.0, 0.0),
                        &StrokeOptions::default().with_line_width(2.0),
                    ))
                    .with(Preview)
                    .with(Node::default())
                    // Line anchored at top left now
                    .with(Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            top: Val::Px(pad),
                            left: Val::Px(pad),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
            },
            _ => {}, // draw later
        }
    }

    // Draw nodes on top.
    for signal in &signals {
        match signal {
            shooter::Signal::Input { id, value } => {
                let (num, layer) = node_positions.get(id).unwrap();
                commands
                    .spawn(primitive(
                        materials.add(val_to_color(*value).into()).clone(),
                        &mut meshes,
                        ShapeType::Circle(10.0),
                        TessellationMode::Fill(&FillOptions::default()),
                        Vec3::new(0.0, 0.0, 0.0),
                    ))
                    .with(Preview)
                    .with(Node::default())
                    // anchored at the center
                    .with(Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            top: Val::Px(pad + *layer as f32 * vert_space),
                            left: Val::Px(pad + *num as f32 * horz_space),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
            },
            shooter::Signal::Neuron { id, raw_value: _, activation_value } => {
                let (num, layer) = node_positions.get(id).unwrap();
                commands
                    .spawn(primitive(
                        materials.add(val_to_color(*activation_value).into()).clone(),
                        &mut meshes,
                        ShapeType::Circle(10.0),
                        TessellationMode::Fill(&FillOptions::default()),
                        Vec3::new(0.0, 0.0, 0.0),
                    ))
                    .with(Preview)
                    .with(Node::default())
                    // anchored at the center
                    .with(Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            top: Val::Px(pad + *layer as f32 * vert_space),
                            left: Val::Px(pad + *num as f32 * horz_space),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
            },
            _ => {}, // Already drawn.
        }
    }
}


fn draw_preview(
    commands: &mut Commands,
    mut state: Local<FedEvents>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    fed_events: Res<Events<BrainFed>>,
    brains: Query<&shooter::Brain, With<Borg>>,
) {
    let brain_feed = state.event_reader.iter(&fed_events)
        .next()
        .and_then(|fed|
            brains.get(fed.entity).ok().map(|brain| (brain, &fed.inputs))
        );
    
    if let Some((brain, inputs)) = brain_feed {
        commands
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(200.0), Val::Px(200.0)),
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Baseline,
                    //align_content: AlignContent::Baseline,
                    padding: Rect::all(Val::Px(20.0)),
                    ..Default::default()
                },
                material: materials.add(Color::rgb(0.08, 0.08, 0.08).into()),
                ..Default::default()
            })
            .with(Preview)
            .with_children(|parent| draw_brain(
                parent,
                &mut materials,
                &mut meshes,
                brain,
                &inputs,
            ));
    }
}
