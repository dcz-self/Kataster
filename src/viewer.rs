/*! Live brain viewer */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */
use bevy::app;
use bevy::app::{ Events, EventReader };
use bevy::asset::{ Assets, AssetServer };
use bevy::ecs::{ Commands, Entity, Local, Query, Res, ResMut, With };
use bevy::math::{ Rect, Size, Vec3 };
use bevy::render::color::Color;
use bevy::render::mesh::Mesh;
use bevy::sprite::ColorMaterial;
use bevy::transform::hierarchy::ChildBuilder;
use bevy::ui::entity::NodeComponents;
use bevy::ui::{ AlignItems, Node, PositionType, Style, Val };
use bevy_prototype_lyon;
use bevy_prototype_lyon::prelude::{ point, primitive, FillOptions, PathBuilder, ShapeType, StrokeOptions, TessellationMode };
use crate::brain::Neuron;
use crate::components::Borg;
use crate::shooter;
use crate::shooter::BrainFed;


use bevy::prelude::BuildChildren;
use bevy::prelude::IntoQuerySystem;


pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut app::AppBuilder) {
        app.add_stage_after(app::stage::UPDATE, "draw_imm")
            .add_system_to_stage("draw_imm", draw_preview.system())
            .add_system_to_stage(app::stage::UPDATE, kill_preview.system());
    }
}


struct Preview;

fn kill_preview(
    mut commands: Commands,
    preview_ui: Query<With<Preview, Entity>>,
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
    let calm = Vec3::new(0.2, 0.2, 0.2);
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
    Color::rgb_linear(color[0], color[1], color[2])
}

fn draw_brain(
    commands: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    brain: &shooter::Brain,
    inputs: &shooter::Inputs,
) {
    let vert_space = 50.0;
    let horz_space = 50.0;
    let pad = 20.0;
    
    let red = materials.add(Color::rgb(0.8, 0.0, 0.0).into());
    let blue = materials.add(Color::rgb(0.0, 0.4, 0.0).into());
    
    let mut draw_layer = |layer: &[Neuron], num| {
        for (outidx, neuron) in layer.iter().enumerate() {
            for (inidx, connection) in neuron.weights.iter().enumerate() {
                if connection != &0.0 {
                    let mut builder = PathBuilder::new();
                    builder.line_to(point(
                        horz_space * outidx as f32,
                        -vert_space * num as f32,
                    ));
                    builder.line_to(point(
                        horz_space * inidx as f32,
                        -vert_space * (num - 1) as f32,
                    ));
                    let path = builder.build();
                    commands
                        .spawn(path.stroke(
                            red.clone(),
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
                }
            }
            
            commands
                .spawn(primitive(
                    blue.clone(),
                    &mut meshes,
                    ShapeType::Circle(10.0),
                    TessellationMode::Fill(&FillOptions::default()),
                    Vec3::new(0.0, 0.0, 0.0),
                    /*Vec3::new(
                        outidx as f32 * horz_space,
                        num as f32 * vert_space,
                        0.0,
                    ),*/
                ))
                .with(Preview)
                .with(Node::default())
                // anchored at the center
                .with(Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(pad + num as f32 * vert_space),
                        left: Val::Px(pad + outidx as f32 * horz_space),
                        ..Default::default()
                    },
                    ..Default::default()
                });
        }
    };
    
    draw_layer(&brain.output_layer, 2);
    draw_layer(&brain.hidden_layer, 1);
    

    let mut draw_inputs = |inputs: &shooter::Inputs| {
        let inputs = shooter::Brain::normalize_inputs(inputs.clone());
        for (idx, input) in inputs.into_iter().enumerate() {
            commands
                .spawn(primitive(
                    materials.add(val_to_color(input).into()).clone(),
                    &mut meshes,
                    ShapeType::Circle(10.0),
                    TessellationMode::Fill(&FillOptions::default()),
                    Vec3::new(0.0, 0.0, 0.0),
                    /*Vec3::new(
                        outidx as f32 * horz_space,
                        num as f32 * vert_space,
                        0.0,
                    ),*/
                ))
                .with(Preview)
                .with(Node::default())
                // anchored at the center
                .with(Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(pad),
                        left: Val::Px(pad + idx as f32 * horz_space),
                        ..Default::default()
                    },
                    ..Default::default()
                });
        }
    };

    draw_inputs(&inputs);
}



fn draw_preview(
    mut commands: Commands,
    mut state: Local<FedEvents>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    fed_events: Res<Events<BrainFed>>,
    brains: Query<With<Borg, &shooter::Brain>>,
) {
    let brain_feed = state.event_reader.iter(&fed_events)
        .next()
        .and_then(|fed|
            brains.get(fed.entity).ok().map(|brain| (brain, &fed.inputs))
        );
    
    if let Some((brain, inputs)) = brain_feed {
        commands
            .spawn(NodeComponents {
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
                &asset_server,
                &mut materials,
                &mut meshes,
                brain,
                &inputs,
            ));
    }
}
