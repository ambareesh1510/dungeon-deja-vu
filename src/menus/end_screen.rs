use super::{
    level_select::BackButtonMarker, main_menu::BackgroundMenuTileMarker, CycleCount, DeathCount,
    MenuCameraMarker, SpeedrunTimer, UI_RENDER_LAYER,
};
use crate::state::LevelLoadingState;
use bevy::prelude::*;

#[derive(Component)]
pub struct EndScreenNode;

pub fn create_end_screen_menu(
    mut commands: Commands,
    speedrun_timer: Res<SpeedrunTimer>,
    death_count: Res<DeathCount>,
    cycle_count: Res<CycleCount>,
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
        .insert(EndScreenNode)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Percent(90.0),
                        height: Val::Percent(35.0),
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
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        format!(
                            "Congratulations! You finished the game in {}m {:.2}s with a total of {} deaths and {} cycles.",
                            (speedrun_timer.0.elapsed_secs() / 60.) as isize,
                            speedrun_timer.0.elapsed_secs() % 60.,
                            death_count.0,
                            cycle_count.0,
                        ),
                        TextStyle {
                            font: monocraft.clone(),
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    )).insert(UI_RENDER_LAYER);
                }).insert(UI_RENDER_LAYER);
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Percent(90.0),
                        height: Val::Percent(25.0),
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
                .insert(BackButtonMarker)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Back to main menu",
                        TextStyle {
                            font: monocraft.clone(),
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    )).insert(UI_RENDER_LAYER);
                }).insert(UI_RENDER_LAYER);
        });
}

pub fn handle_end_screen_clicks(
    back_button_query: Query<&Interaction, (Changed<Interaction>, With<BackButtonMarker>)>,
    mut next_state: ResMut<NextState<LevelLoadingState>>,
) {
    for interaction in back_button_query.iter() {
        if *interaction != Interaction::Pressed {
            return;
        }
        next_state.set(LevelLoadingState::MainMenu);
    }
}

pub fn cleanup_end_screen(
    mut commands: Commands,
    query_main_menu: Query<Entity, Or<(With<EndScreenNode>, With<MenuCameraMarker>)>>,
) {
    for entity in query_main_menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
