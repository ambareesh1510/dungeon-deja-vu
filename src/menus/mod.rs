use bevy::prelude::*;
use main_menu::{handle_main_menu_clicks, create_main_menu, cleanup_main_menu};
use level_select::{cleanup_level_select_menu, create_level_select_menu, handle_level_select_menu_clicks};

use crate::state::LevelLoadingState;

pub struct MenuManagementPlugin;

mod main_menu;
mod level_select;

impl Plugin for MenuManagementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(LevelLoadingState::MainMenu), create_main_menu)
            .add_systems(OnExit(LevelLoadingState::MainMenu), cleanup_main_menu)
            .add_systems(
                Update,
                (
                    handle_main_menu_clicks
                ).run_if(in_state(LevelLoadingState::MainMenu))
            )
            .add_systems(OnEnter(LevelLoadingState::LevelSelect), create_level_select_menu)
            .add_systems(OnExit(LevelLoadingState::LevelSelect), cleanup_level_select_menu)
            .add_systems(
                Update,
                (
                    handle_level_select_menu_clicks
                ).run_if(in_state(LevelLoadingState::LevelSelect))
            );
    }
}

#[derive(Component)]
struct MenuCameraMarker;

