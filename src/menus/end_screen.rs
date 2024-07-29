use super::{level_select::BackButtonMarker, CycleCount, DeathCount, MenuCameraMarker, SpeedrunTimer};
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
                    ));
                });
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
                    ));
                });
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
