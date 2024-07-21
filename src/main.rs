mod camera;
mod level;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use camera::CameraManagementPlugin;
use level::LevelManagementPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .add_plugins((
            CameraManagementPlugin,
            LevelManagementPlugin,
        ))
        .run();
}
