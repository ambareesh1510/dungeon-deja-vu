use super::{main_menu::BackgroundMenuTileMarker, MenuCameraMarker, UI_RENDER_LAYER};
use crate::{
    level::{FromLevelSelect, LastAccessibleLevel, LEVEL_IIDS},
    state::{LevelLoadingState, TargetLevel},
};
use bevy::prelude::*;

#[derive(Component)]
pub struct LevelSelectMenuNode;

#[derive(Component)]
pub struct LevelButtonMarker(usize);

#[derive(Component)]
pub struct BackButtonMarker;

pub fn create_level_select_menu(
    mut commands: Commands,
    last_accessible_level: Res<LastAccessibleLevel>,
    asset_server: Res<AssetServer>,
) {
    let mut camera = Camera2dBundle::default();
    camera.camera.order = 11;
    commands
        .spawn(camera)
        .insert(UI_RENDER_LAYER)
        .insert(MenuCameraMarker);
    let background_sprite_handle = asset_server.load("backgroundwindows.png");
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
        .insert(LevelSelectMenuNode)
        .with_children(|parent| {
            for i in 0..LEVEL_IIDS.len() {
                parent
                    .spawn(ButtonBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(10.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(LevelButtonMarker(i))
                    .with_children(|parent| {
                        parent
                            .spawn(TextBundle::from_section(
                                format!(
                                    "Level {} {}",
                                    i + 1,
                                    if i > last_accessible_level.0 {
                                        "[LOCKED]"
                                    } else {
                                        ""
                                    }
                                ),
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
            }
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
                .insert(BackButtonMarker)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Back to main menu",
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

pub fn handle_level_select_menu_clicks(
    level_select_query: Query<(&Interaction, &LevelButtonMarker), Changed<Interaction>>,
    back_button_query: Query<&Interaction, (Changed<Interaction>, With<BackButtonMarker>)>,
    mut next_state: ResMut<NextState<LevelLoadingState>>,
    mut from_level_select: ResMut<FromLevelSelect>,
    mut target_level: ResMut<TargetLevel>,
    last_accessible_level: Res<LastAccessibleLevel>,
) {
    for (interaction, level_button_marker) in level_select_query.iter() {
        if *interaction != Interaction::Pressed {
            return;
        }
        if level_button_marker.0 > last_accessible_level.0 {
            return;
        }
        target_level.0 = level_button_marker.0;
        next_state.set(LevelLoadingState::Loading);
        from_level_select.0 = true;
    }
    for interaction in back_button_query.iter() {
        if *interaction != Interaction::Pressed {
            return;
        }
        next_state.set(LevelLoadingState::MainMenu);
    }
}

pub fn cleanup_level_select_menu(
    mut commands: Commands,
    query_main_menu: Query<Entity, Or<(With<LevelSelectMenuNode>, With<MenuCameraMarker>)>>,
) {
    for entity in query_main_menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
