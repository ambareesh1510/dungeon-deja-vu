use super::{CycleCount, DeathCount, MenuCameraMarker, SpeedrunTimer, UI_RENDER_LAYER};
use crate::state::{LevelLoadingState, TargetLevel};
use bevy::prelude::*;

#[derive(Component)]
pub struct MainMenuNode;

#[derive(Component)]
pub struct StartGameButtonMarker;

#[derive(Component)]
pub struct LevelSelectButtonMarker;

#[derive(Component)]
pub struct BackgroundMenuTileMarker;

pub fn create_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut camera = Camera2dBundle::default();
    camera.camera.order = 11;
    commands
        .spawn(camera)
        .insert(UI_RENDER_LAYER)
        .insert(MenuCameraMarker);
    let background_sprite_handle = asset_server.load("backgroundnowindows.png");
    let background_sprite_size = 128.;
    for x in -10..10 {
        for y in -10..10 {
            commands
                .spawn(SpriteBundle {
                    transform: Transform::from_xyz(
                        background_sprite_size * x as f32,
                        background_sprite_size * y as f32,
                        0.,
                    ),
                    texture: background_sprite_handle.clone(),
                    ..default()
                })
                .insert(BackgroundMenuTileMarker)
                .insert(UI_RENDER_LAYER);
        }
    }
    let monocraft = asset_server.load("Monocraft.ttf");
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        // .insert(UI_RENDER_LAYER)
        .insert(MainMenuNode)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(25.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                // .insert(StartGameButtonMarker)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Dungeon\nDéjà Vu",
                            TextStyle {
                                font: monocraft.clone(),
                                font_size: 60.0,
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ))
                        .insert(UI_RENDER_LAYER);
                })
                .insert(UI_RENDER_LAYER);
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(20.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .insert(StartGameButtonMarker)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Start Game",
                            TextStyle {
                                font: monocraft.clone(),
                                font_size: 40.0,
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ))
                        .insert(UI_RENDER_LAYER);
                })
                .insert(UI_RENDER_LAYER);
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(20.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .insert(LevelSelectButtonMarker)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Level Select",
                            TextStyle {
                                font: monocraft.clone(),
                                font_size: 40.0,
                                color: Color::srgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ))
                        .insert(UI_RENDER_LAYER);
                })
                .insert(UI_RENDER_LAYER);
        });
}

pub fn handle_main_menu_clicks(
    start_game_query: Query<&Interaction, (Changed<Interaction>, With<StartGameButtonMarker>)>,
    level_select_query: Query<&Interaction, (Changed<Interaction>, With<LevelSelectButtonMarker>)>,
    mut next_state: ResMut<NextState<LevelLoadingState>>,
    mut speedrun_timer: ResMut<SpeedrunTimer>,
    mut target_level: ResMut<TargetLevel>,
    mut death_counter: ResMut<DeathCount>,
    mut cycle_counter: ResMut<CycleCount>,
) {
    for interaction in start_game_query.iter() {
        if *interaction != Interaction::Pressed {
            return;
        }
        speedrun_timer.0.reset();
        death_counter.0 = 0;
        cycle_counter.0 = 0;
        target_level.0 = 0;
        next_state.set(LevelLoadingState::Loading);
    }
    for interaction in level_select_query.iter() {
        if *interaction != Interaction::Pressed {
            return;
        }
        next_state.set(LevelLoadingState::LevelSelect);
        println!("entering level select");
    }
}

pub fn cleanup_main_menu(
    mut commands: Commands,
    query_main_menu: Query<
        Entity,
        Or<(
            With<MainMenuNode>,
            With<MenuCameraMarker>,
            With<BackgroundMenuTileMarker>,
        )>,
    >,
) {
    for entity in query_main_menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
