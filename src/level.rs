use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkPlugin, LdtkSpriteSheetBundle, LdtkEntity, LdtkWorldBundle, app::LdtkEntityAppExt, LevelSelection};

pub struct LevelManagementPlugin;

impl Plugin for LevelManagementPlugin {
    fn build(&self, app: &mut App) {
            app.add_plugins(LdtkPlugin)
                .insert_resource(LevelSelection::index(0))
                .register_ldtk_entity::<PlayerBundle>("Player")
                .add_systems(Startup, spawn_level);
    }
}

fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("level.ldtk"),
        ..default()
    });
}

#[derive(Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
}
