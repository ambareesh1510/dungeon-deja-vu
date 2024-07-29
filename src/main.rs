mod camera;
mod entities;
mod level;
mod player;
mod state;
mod menus;
mod sound_effects;

use bevy::{
    asset::AssetMetaCheck, ecs::schedule::{LogLevel, ScheduleBuildSettings}, prelude::*
};
use bevy_rapier2d::prelude::*;
use camera::CameraManagementPlugin;
use entities::EntityManagementPlugin;
use level::LevelManagementPlugin;
use player::PlayerManagementPlugin;
use state::StateManagementPlugin;
use menus::MenuManagementPlugin;
use sound_effects::SoundEffectsManagementPlugin;

fn main() {
    App::new()
        // Enable ambiguity warnings for the Update schedule
        .edit_schedule(Startup, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })

        .add_plugins(DefaultPlugins
            .set( ImagePlugin::default_nearest())
            .set(AssetPlugin { meta_check: AssetMetaCheck::Never,..default()})
            
        )
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(24.))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins((
            CameraManagementPlugin,
            LevelManagementPlugin,
            StateManagementPlugin,
            EntityManagementPlugin,
            PlayerManagementPlugin,
            MenuManagementPlugin,
            SoundEffectsManagementPlugin,
        ))
        .run();
}
