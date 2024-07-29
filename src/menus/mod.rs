use bevy::{prelude::*, time::Stopwatch};
use end_screen::{cleanup_end_screen, create_end_screen_menu, handle_end_screen_clicks};
use level_select::{
    cleanup_level_select_menu, create_level_select_menu, handle_level_select_menu_clicks,
};
use main_menu::{cleanup_main_menu, create_main_menu, handle_main_menu_clicks};

use crate::state::LevelLoadingState;

pub struct MenuManagementPlugin;

mod end_screen;
mod level_select;
mod main_menu;

impl Plugin for MenuManagementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpeedrunTimer(Stopwatch::new()))
            .insert_resource(DeathCount(0))
            .add_systems(OnEnter(LevelLoadingState::MainMenu), create_main_menu)
            .add_systems(OnExit(LevelLoadingState::MainMenu), cleanup_main_menu)
            .add_systems(
                Update,
                (handle_main_menu_clicks).run_if(in_state(LevelLoadingState::MainMenu)),
            )
            .add_systems(
                OnEnter(LevelLoadingState::LevelSelect),
                create_level_select_menu,
            )
            .add_systems(
                OnExit(LevelLoadingState::LevelSelect),
                cleanup_level_select_menu,
            )
            .add_systems(
                Update,
                (handle_level_select_menu_clicks).run_if(in_state(LevelLoadingState::LevelSelect)),
            )
            .add_systems(
                OnEnter(LevelLoadingState::EndScreen),
                create_end_screen_menu,
            )
            .add_systems(OnExit(LevelLoadingState::EndScreen), cleanup_end_screen)
            .add_systems(
                Update,
                (handle_end_screen_clicks).run_if(in_state(LevelLoadingState::EndScreen)),
            )
            .add_systems(Update, tick_speedrun_timer);
    }
}

#[derive(Component)]
struct MenuCameraMarker;

#[derive(Resource)]
pub struct SpeedrunTimer(pub Stopwatch);

#[derive(Resource)]
pub struct DeathCount(pub usize);

fn tick_speedrun_timer(time: Res<Time>, mut speedrun_timer: ResMut<SpeedrunTimer>) {
    speedrun_timer.0.tick(time.delta());
}
