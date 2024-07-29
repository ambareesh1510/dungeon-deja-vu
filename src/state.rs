use bevy::prelude::*;

pub struct StateManagementPlugin;

impl Plugin for StateManagementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(LevelLoadingState::MainMenu)
            .insert_resource(TargetLevel(0))
            .add_systems(
                Update,
                (change_level,).run_if(in_state(LevelLoadingState::Loaded)),
            );
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

fn change_level(
    mut next_state: ResMut<NextState<LevelLoadingState>>,
    mut target_level: ResMut<TargetLevel>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        target_level.0 += 1;
        next_state.set(LevelLoadingState::Loading)
    }
}
