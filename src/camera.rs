use crate::{
    level::{loop_player, PlayerCheckpoint, PlayerMarker, PlayerStatus},
    state::TargetLevel,
};
use bevy::{
    prelude::*,
    render::{camera::ScalingMode, view::RenderLayers},
};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::Velocity;

use crate::state::LevelLoadingState;

const CAMERA_UNIT_HEIGHT: f32 = 250.;

pub struct CameraManagementPlugin;

impl Plugin for CameraManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_dim_mesh)
            .add_systems(
                Update,
                (
                    // undim_camera,
                    setup_camera,
                    attach_player_camera_to_player,
                    autoscroll_camera.after(loop_player),
                    loop_main_cameras,
                    dim_camera
                        .before(loop_main_cameras)
                        .before(autoscroll_camera),
                )
                    .run_if(in_state(LevelLoadingState::Loaded)),
            )
            .add_systems(OnExit(LevelLoadingState::Loaded), (cleanup_cameras,));
    }
}

pub const PLAYER_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);
const PLAYER_CAMERA_ORDER: isize = 1;

#[derive(Component)]
pub struct CameraMarker;

#[derive(Component)]
pub struct PlayerCameraMarker;

#[derive(Component)]
struct MainCameraMarker;

#[derive(Component)]
struct DimCameraMarker;

#[derive(Component)]
struct DimMeshMarker;

fn cleanup_cameras(query: Query<Entity, With<CameraMarker>>, mut commands: Commands) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_dim_mesh(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(3000., 1000.)),
                color: Color::srgba(0.0, 0.0, 0.0, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(DimMeshMarker)
        .insert(RenderLayers::layer(10));

    let mut dim_camera = Camera2dBundle::default();
    dim_camera.camera.order = 10;
    commands.spawn((dim_camera, RenderLayers::layer(10), DimCameraMarker));
}

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

            return;
        }
    }
}

fn dim_camera(
    mut query_dim_sprite: Query<&mut Sprite, With<DimMeshMarker>>,
    mut query_player: Query<
        (&mut PlayerStatus, &PlayerCheckpoint, &mut Transform, &mut Velocity),
        With<PlayerMarker>,
    >,
    mut query_player_camera: Query<
        &mut Transform,
        (With<PlayerCameraMarker>, Without<PlayerMarker>),
    >,
    mut query_cameras: Query<
        &mut Transform,
        (
            With<MainCameraMarker>,
            Without<PlayerMarker>,
            Without<PlayerCameraMarker>,
        ),
    >,
    mut next_state: ResMut<NextState<LevelLoadingState>>,
    mut target_level: ResMut<TargetLevel>,
    time: Res<Time>,
    // mut virtual_time: ResMut<Time<Virtual>>
) {
    let Ok((mut player_status, player_checkpoint, mut player_transform, mut player_velocity)) =
        query_player.get_single_mut()
    else {
        return;
    };
    let Ok(mut dim_sprite) = query_dim_sprite.get_single_mut() else {
        return;
    };
    let Ok(mut player_camera_transform) = query_player_camera.get_single_mut() else {
        return;
    };
    let color_as_linear = dim_sprite.color.to_linear();
    let mut alpha = color_as_linear.alpha();
    if player_status.level_finished || player_status.dead {
        alpha += time.delta().as_secs_f32() * 2.;
        if player_status.dead {
            *player_velocity = Velocity::zero();
        }
        if alpha >= 1.5 {
            alpha = 1.5;
            if player_status.level_finished {
                target_level.0 += 1;
                next_state.set(LevelLoadingState::Loading);
            } else {
                player_status.dead = false;
                let new_translation = Vec3::new(player_checkpoint.transform.x, player_checkpoint.transform.y, 0.);
                let delta = new_translation - player_transform.translation;
                let camera_offset =
                    player_transform.translation.x - player_camera_transform.translation.x;
                if camera_offset < 0. {
                    player_camera_transform.translation.x += camera_offset
                };
                // player_transform.translation = Vec3::new(player_checkpoint.0.x, player_checkpoint.0.y, 0.);
                player_transform.translation += delta;
                player_camera_transform.translation += delta;
                for mut camera_transform in query_cameras.iter_mut() {
                    camera_transform.translation += delta;
                    if camera_offset < 0. {
                        camera_transform.translation.x += camera_offset
                    };
                }
                //
                // camera_transform.translation = Vec3::new(player_checkpoint.0.x, player_checkpoint.0.y, 0.);
            }
        }
    } else {
        alpha -= time.delta().as_secs_f32() * 2.;
        if alpha < 0. {
            alpha = 0.;
        }
    }
    dim_sprite.color = Color::LinearRgba(color_as_linear.with_alpha(alpha));
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
        if camera_transform.translation.x < -0.5 * level_width {
            camera_transform.translation.x += 2. * level_width;
        }
    }
}

fn autoscroll_camera(
    mut query_main_cameras: Query<&mut Transform, With<MainCameraMarker>>,
    mut query_player_camera: Query<
        &mut Transform,
        (With<PlayerCameraMarker>, Without<MainCameraMarker>),
    >,
    query_player: Query<
        &mut Transform,
        (
            With<PlayerMarker>,
            Without<CameraMarker>,
            Without<PlayerCameraMarker>,
            Without<MainCameraMarker>,
        ),
    >,
    query_level: Query<&LayerMetadata>,
) {
    if let Ok(player_transform) = query_player.get_single() {
        let mut level_width = 1000. * 16.;
        for level in query_level.iter() {
            if level.layer_instance_type == bevy_ecs_ldtk::ldtk::Type::IntGrid {
                level_width = level.c_wid as f32 * 16.;
            }
        }

        if let Ok(mut player_camera_transform) = query_player_camera.get_single_mut() {
            let new_transform = player_camera_transform.translation.x/*  % level_width */;
            let new_player_transform =
                ((player_transform.translation.x % level_width) + level_width) % level_width;
            let delta = new_player_transform - new_transform;
            if delta <= 0. {
                return;
            }
            // NOTE: if the delta is greater than 0, that means the player's world transform is
            // greater than the camera's, which is in the center of the screen. So the camera will
            // only scroll if the player is past the halfway point
            let modded_delta = ((delta % level_width) + level_width) % level_width;

            player_camera_transform.translation.x += modded_delta;
            for mut camera_transform in query_main_cameras.iter_mut() {
                camera_transform.translation.x += modded_delta;
            }
        }
    }
}
