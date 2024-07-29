use super::{DeathCount, MenuCameraMarker, SpeedrunTimer};
use crate::state::{LevelLoadingState, TargetLevel};
use bevy::prelude::*;

#[derive(Component)]
pub struct MainMenuNode;

#[derive(Component)]
pub struct StartGameButtonMarker;

#[derive(Component)]
pub struct LevelSelectButtonMarker;

pub fn create_main_menu(mut commands: Commands, asset_server: Res<AssetServer>,) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(MenuCameraMarker);
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
        .insert(MainMenuNode)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(125.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    border_radius: BorderRadius::MAX,
                    background_color: Color::BLACK.into(),
                    ..default()
                })
                .insert(StartGameButtonMarker)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Start Game",
                        TextStyle {
                            font: monocraft.clone(),
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    border_radius: BorderRadius::MAX,
                    background_color: Color::BLACK.into(),
                    ..default()
                })
                .insert(LevelSelectButtonMarker)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Level Select",
                        TextStyle {
                            font: monocraft.clone(),
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

pub fn handle_main_menu_clicks(
    start_game_query: Query<&Interaction, (Changed<Interaction>, With<StartGameButtonMarker>)>,
    level_select_query: Query<&Interaction, (Changed<Interaction>, With<LevelSelectButtonMarker>)>,
    mut next_state: ResMut<NextState<LevelLoadingState>>,
    mut speedrun_timer: ResMut<SpeedrunTimer>,
    mut target_level: ResMut<TargetLevel>,
    mut death_counter: ResMut<DeathCount>,
) {
    for interaction in start_game_query.iter() {
        if *interaction != Interaction::Pressed {
            return;
        }
        speedrun_timer.0.reset();
        death_counter.0 = 0;
        target_level.0 = 0;
        next_state.set(LevelLoadingState::Loading);
    }
    for interaction in level_select_query.iter() {
        if *interaction != Interaction::Pressed {
            return;
        }
        next_state.set(LevelLoadingState::LevelSelect);
    }
}

pub fn cleanup_main_menu(
    mut commands: Commands,
    query_main_menu: Query<Entity, Or<(With<MainMenuNode>, With<MenuCameraMarker>)>>,
) {
    for entity in query_main_menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
