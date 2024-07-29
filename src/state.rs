use bevy::prelude::*;

pub struct StateManagementPlugin;

impl Plugin for StateManagementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(LevelLoadingState::MainMenu)
            .insert_resource(TargetLevel(0));
    }
}

#[derive(Resource)]
pub struct TargetLevel(pub usize);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LevelLoadingState {
    MainMenu,
    LevelSelect,
    Loading,
    Loaded,
    EndScreen,
}


