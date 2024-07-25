use crate::level::{loop_player, PlayerMarker};
use bevy::{
    prelude::*,
    render::{camera::ScalingMode, view::RenderLayers},
};
use bevy_ecs_ldtk::LayerMetadata;

const CAMERA_UNIT_HEIGHT: f32 = 250.;

pub struct CameraManagementPlugin;

impl Plugin for CameraManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, setup_camera)
            // .add_systems(Update, control_camera)
            .add_systems(Update, attach_player_camera_to_player)
            .add_systems(Update, autoscroll_camera.after(loop_player))
            .add_systems(Update, loop_main_cameras);
    }
}

pub const PLAYER_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);
const PLAYER_CAMERA_ORDER: isize = 1;

#[derive(Component)]
struct CameraMarker;

#[derive(Component)]
pub struct PlayerCameraMarker;

#[derive(Component)]
struct MainCameraMarker;

fn setup_camera(mut commands: Commands, query_level: Query<&LayerMetadata, Added<LayerMetadata>>) {
    let scaling_mode = ScalingMode::FixedVertical(CAMERA_UNIT_HEIGHT);
    for level in query_level.iter() {
        if level.layer_instance_type == bevy_ecs_ldtk::ldtk::Type::IntGrid {
            let level_width = level.c_wid as f32 * 16.;
            let mut player_camera = Camera2dBundle::default();
            player_camera.projection.scaling_mode = scaling_mode;
            player_camera.camera.order = PLAYER_CAMERA_ORDER;
            commands.spawn((
                player_camera,
                PlayerCameraMarker,
                CameraMarker,
                PLAYER_RENDER_LAYER,
            ));

            let mut main_camera = Camera2dBundle::default();
            main_camera.projection.scaling_mode = scaling_mode;
            commands.spawn((main_camera, MainCameraMarker, CameraMarker));

            let mut main_camera_2 = Camera2dBundle::default();
            main_camera_2.projection.scaling_mode = scaling_mode;
            main_camera_2.transform.translation.x = level_width;
            main_camera_2.camera.order = -1;
            commands.spawn((main_camera_2, MainCameraMarker, CameraMarker));
        }
    }
}

fn control_camera(
    mut query_camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        With<PlayerCameraMarker>,
    >,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((mut camera_transform, mut camera_projection)) = query_camera.get_single_mut() {
        if keys.pressed(KeyCode::KeyW) {
            camera_transform.translation.y += 1.;
        }
        if keys.pressed(KeyCode::KeyS) {
            camera_transform.translation.y -= 1.;
        }
        if keys.pressed(KeyCode::KeyA) {
            camera_transform.translation.x -= 1.;
        }
        if keys.pressed(KeyCode::KeyD) {
            camera_transform.translation.x += 1.;
        }
        if keys.pressed(KeyCode::Minus) {
            camera_projection.scale *= 1.1;
        }
        if keys.pressed(KeyCode::Equal) {
            camera_projection.scale /= 1.1;
        }
    }
}

// TODO: make this use delta time!
fn attach_player_camera_to_player(
    mut query_player_camera: Query<
        &mut Transform,
        (With<PlayerCameraMarker>, Without<PlayerMarker>),
    >,
    mut query_main_camera: Query<
        &mut Transform,
        (
            With<MainCameraMarker>,
            (Without<PlayerMarker>, Without<PlayerCameraMarker>),
        ),
    >,
    query_player: Query<&Transform, With<PlayerMarker>>,
) {
    // the lowest possible position of the camera such that the part outside of the level is not
    // shown
    let low_pos = CAMERA_UNIT_HEIGHT / 2.;
    if let (Ok(mut player_camera_transform), Ok(player_transform)) = (
        query_player_camera.get_single_mut(),
        query_player.get_single(),
    ) {
        let delta = (player_transform.translation.y - 10.0) - player_camera_transform.translation.y;
        player_camera_transform.translation.y += delta / 3.;
        if player_camera_transform.translation.y < low_pos {
            player_camera_transform.translation.y = low_pos;
        }
        for mut main_camera_transform in query_main_camera.iter_mut() {
            main_camera_transform.translation.y += delta / 3.;
            if main_camera_transform.translation.y < low_pos {
                main_camera_transform.translation.y = low_pos;
            }
        }
    }
}

fn loop_main_cameras(
    mut query_main_cameras: Query<&mut Transform, With<MainCameraMarker>>,
    query_level: Query<&LayerMetadata>,
) {
    let mut level_width = 1000. * 16.;
    for level in query_level.iter() {
        if level.layer_instance_type == bevy_ecs_ldtk::ldtk::Type::IntGrid {
            level_width = level.c_wid as f32 * 16.;
        }
    }
    for mut camera_transform in query_main_cameras.iter_mut() {
        if camera_transform.translation.x > 3. * level_width / 2. {
            camera_transform.translation.x -= 2. * level_width;
        }
    }
}

fn autoscroll_camera(
    mut query_main_cameras: Query<&mut Transform, With<MainCameraMarker>>,
    mut query_player_camera: Query<&mut Transform, (With<CameraMarker>, Without<MainCameraMarker>)>,
    query_player: Query<
        &mut Transform,
        (
            With<PlayerMarker>,
            Without<CameraMarker>,
            Without<MainCameraMarker>,
        ),
    >,
    query_level: Query<&LayerMetadata>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = query_player.get_single() {
        let mut level_width = 1000. * 16.;
        for level in query_level.iter() {
            if level.layer_instance_type == bevy_ecs_ldtk::ldtk::Type::IntGrid {
                level_width = level.c_wid as f32 * 16.;
            }
        }

        // println!("camera positions:");
        // for mut camera_transform in query_cameras.iter_mut() {
        if let Ok(mut player_camera_transform) = query_player_camera.get_single_mut() {
            // println!("{}", player_camera_transform.translation.x);
            let new_transform = player_camera_transform.translation.x/*  % level_width */;
            let new_player_transform =
                ((player_transform.translation.x % level_width) + level_width) % level_width;
            let delta = new_player_transform - new_transform;
            if delta <= 0. {
                // println!("^ ignored");
                return;
            }
            // NOTE: if the delta is greater than 0, that means the player's world transform is
            // greater than the camera's, which is in the center of the screen. So the camera will
            // only scroll if the player is past the halfway point
            let modded_delta = ((delta % level_width) + level_width) % level_width;

            // println!("modded delta is {modded_delta}");
            player_camera_transform.translation.x += modded_delta;
            for mut camera_transform in query_main_cameras.iter_mut() {
                camera_transform.translation.x += modded_delta;
            }
        }
    }
}
