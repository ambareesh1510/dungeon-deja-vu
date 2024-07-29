use bevy::prelude::*;
use crate::state::LevelLoadingState;
use super::{level_select::BackButtonMarker, DeathCount, MenuCameraMarker, SpeedrunTimer};

#[derive(Component)]
pub struct EndScreenNode;

pub fn create_end_screen_menu(mut commands: Commands, speedrun_timer: Res<SpeedrunTimer>, death_count: Res<DeathCount>) {
    commands.spawn(Camera2dBundle::default()).insert(MenuCameraMarker);
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
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        format!("Congratulations! You finished the game in {} seconds with {} deaths.", speedrun_timer.0.elapsed_secs(), death_count.0),
                        TextStyle {
                            // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
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
                .insert(BackButtonMarker)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Back to main menu",
                        TextStyle {
                            // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });

}

pub fn handle_end_screen_clicks(
    back_button_query: Query<
        &Interaction,
        (Changed<Interaction>, With<BackButtonMarker>),
    >,
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
