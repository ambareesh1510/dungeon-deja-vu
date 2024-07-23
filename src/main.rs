mod camera;
mod level;

use bevy::{ecs::schedule::{LogLevel, ScheduleBuildSettings}, prelude::*};
use bevy_rapier2d::prelude::*;
use camera::CameraManagementPlugin;
use level::LevelManagementPlugin;

fn main() {
    App::new()
        // Enable ambiguity warnings for the Update schedule
        .edit_schedule(Startup, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        })
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(24.))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins((
            CameraManagementPlugin,
            LevelManagementPlugin,
        ))
        .run();
}
