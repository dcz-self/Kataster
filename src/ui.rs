use super::arena::START_LIFE;
use super::buttons;
use super::components::*;
use super::state::{ GameState, Mode, RunState, ValidStates };
use bevy::prelude::*;
use bevy::prelude::{ ChildBuilder, Handle, Font };
use bevy::ui::entity::{ ButtonBundle, ImageBundle, NodeBundle, TextBundle };

pub struct DrawBlinkTimer(pub Timer);


fn add_text_button<'a, 'b, 'c, 'd>(
    commands: &'a mut ChildBuilder<'c>,
    button_materials: &'b Res<buttons::Materials>,
    font_handle: &'d Handle<Font>,
    text: String,
) -> &'a mut ChildBuilder<'c> {
    commands
        .spawn(ButtonBundle {
            style: Style {
                // force some width
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    value: text,
                    font: font_handle.clone(),
                    style: TextStyle {
                        font_size: 50.0,
                        color: Color::rgb_u8(0x00, 0x44, 0x44),
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        })
        .with(ValidStates::from_func(|state| state == &GameState::MainMenu));
    commands
}

pub fn start_menu(
    commands: &mut Commands,
    runstate: ResMut<RunState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    button_materials: Res<buttons::Materials>,
) {
    if let Some(GameState::MainMenu) = runstate.gamestate.entering() {
        let font_handle = asset_server.load("kenvector_future.ttf");
        commands
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with(ValidStates::from_func(|state| state == &GameState::MainMenu))
            .with_children(|parent| {
                parent
                    .spawn(TextBundle {
                        text: Text {
                            value: "Breedmatic".to_string(),
                            font: font_handle.clone(),
                            style: TextStyle {
                                font_size: 100.0,
                                color: Color::rgb_u8(0x00, 0xAA, 0xAA),
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    })
                    .with(ValidStates::from_func(|state| state == &GameState::MainMenu));
                add_text_button(parent, &button_materials, &font_handle, "1: Start shooting".into());
                add_text_button(parent, &button_materials, &font_handle, "2: AI mode".into());
            });
    }
}

pub fn gameover_menu(
    commands: &mut Commands,
    runstate: ResMut<RunState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Some(GameState::ArenaOver(Mode::Player)) = runstate.gamestate.entering() {
        let font_handle = asset_server.load("kenvector_future.ttf");
        commands
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with(ValidStates::from_func(|state| state == &GameState::ArenaOver(Mode::Player)))
            .with_children(|parent| {
                parent
                    .spawn(TextBundle {
                        text: Text {
                            value: "Game Over".to_string(),
                            font: font_handle.clone(),
                            style: TextStyle {
                                font_size: 100.0,
                                color: Color::rgb_u8(0xAA, 0x22, 0x22),
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    })
                    .with(ValidStates::from_func(|state| state == &GameState::ArenaOver(Mode::Player)))
                    .spawn(TextBundle {
                        text: Text {
                            value: "enter".to_string(),
                            font: font_handle,
                            style: TextStyle {
                                font_size: 50.0,
                                color: Color::rgb_u8(0x44, 0x11, 0x11),
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    })
                    .with(DrawBlinkTimer(Timer::from_seconds(0.5, true)))
                    .with(ValidStates::from_func(|state| state == &GameState::ArenaOver(Mode::Player)));
            });
    }
}

pub fn pause_menu(
    commands: &mut Commands,
    runstate: ResMut<RunState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Some(GameState::ArenaPause(mode)) = runstate.gamestate.entering() {
        let mode = mode.clone();
        let states = ValidStates::from_func(move |state|
            state == &GameState::ArenaPause(mode)
        );
        let font_handle = asset_server.load("kenvector_future.ttf");
        commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with(states.clone())
            .with_children(|parent| {
                parent
                    .spawn(TextBundle {
                        text: Text {
                            value: "pause".to_string(),
                            font: font_handle,
                            style: TextStyle {
                                font_size: 100.0,
                                color: Color::rgb_u8(0xF8, 0xE4, 0x73),
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    })
                    .with(DrawBlinkTimer(Timer::from_seconds(0.5, true)))
                    .with(states.clone());
            });
    }
}

pub fn game_ui_spawn(
    commands: &mut Commands,
    runstate: ResMut<RunState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if runstate
        .gamestate
        .entering()
        .map(GameState::is_live_arena)
        .unwrap_or(false)
    {
        let font_handle = asset_server.load("kenvector_future.ttf");
        commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::FlexEnd,
                    flex_direction: FlexDirection::Row,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with(ValidStates::from_func(GameState::is_arena))
            .with_children(|parent| {
                parent
                    .spawn(TextBundle {
                        style: Style {
                            justify_content: JustifyContent::FlexEnd,
                            margin: Rect {
                                left: Val::Px(10.0),
                                right: Val::Px(10.0),
                                top: Val::Px(10.0),
                                bottom: Val::Px(10.0),
                            },
                            ..Default::default()
                        },
                        text: Text {
                            value: "00".to_string(),
                            font: font_handle,
                            style: TextStyle {
                                font_size: 50.0,
                                color: Color::rgb_u8(0x00, 0xAA, 0xAA),
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    })
                    .with(ValidStates::from_func(GameState::is_arena))
                    .with(UiScore {});
            })
            // Life counters
            // Not kept in 'GameOver' state, simplifying last counter removal.
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Row,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with(ValidStates::from_func(GameState::is_live_arena))
            .with_children(|parent| {
                for i in 1..(START_LIFE + 1) {
                    parent
                        .spawn(ImageBundle {
                            style: Style {
                                margin: Rect {
                                    left: Val::Px(10.0),
                                    right: Val::Px(10.0),
                                    top: Val::Px(10.0),
                                    bottom: Val::Px(10.0),
                                },
                                ..Default::default()
                            },
                            material: materials.add(
                                asset_server
                                    .load("playerLife1_red.png")
                                    .into(),
                            ),
                            ..Default::default()
                        })
                        .with(ValidStates::from_func(GameState::is_live_arena))
                        .with(UiLife { min: i });
                }
            });
    }
}

pub fn score(
    runstate: ChangedRes<RunState>, 
    mut query: Query<(Mut<Text>, &UiScore)>,
) {
    if !runstate.gamestate.current().is_arena() {
        return;
    }
    for (mut text, _uiscore) in query.iter_mut() {
        text.value = format!("{}", runstate.score.unwrap());
    }
}

pub fn life_ui_system(
    runstate: Res<RunState>,
    ship_query: Query<&Ship>,
    mut uilife_query: Query<(Mut<Draw>, &UiLife)>,
) {
    if !runstate.gamestate.current().is_arena() {
        return;
    }
    if let Some(player) = runstate.player {
        if let Ok(ship) = ship_query.get(player) {
            for (mut draw, uilife) in &mut uilife_query.iter_mut() {
                //draw.is_visible = ship.life >= uilife.min;
            }
        }
    }
}
