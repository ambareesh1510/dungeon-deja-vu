use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use tiles::spawn_wall_collision;

mod tiles;

use crate::camera::PlayerCameraMarker;
use crate::player::{PlayerMarker, PlayerStatus, SetCheckpointEvent};
use crate::state::{LevelLoadingState, TargetLevel};

pub struct LevelManagementPlugin;

impl Plugin for LevelManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            // .insert_resource(LevelSelection::index(0))
            .add_event::<SetCheckpointEvent>()
            .register_ldtk_int_cell::<TerrainBundle>(1)
            .register_ldtk_int_cell::<WaterBundle>(2)
            .register_ldtk_int_cell::<SpikeBundle>(4)
            .add_systems(Startup, spawn_ldtk_world)
            .add_systems(Update, spawn_wall_collision)
            .add_systems(OnEnter(LevelLoadingState::Loading), load_level)
            .add_systems(
                Update,
                (inter_level_pause,).run_if(in_state(LevelLoadingState::Loading)),
            )
            .add_systems(
                OnEnter(LevelLoadingState::Loaded),
                (spawn_backwards_barrier,),
            )
            .add_systems(OnExit(LevelLoadingState::Loaded), (cleanup_level_objects,))
            .add_systems(
                Update,
                (finish_level, update_backwards_barrier)
                    .run_if(in_state(LevelLoadingState::Loaded)),
            );
    }
}

#[derive(Component)]
struct InterLevelTimer(Timer);

fn load_level(
    mut commands: Commands,
    target_level: Res<TargetLevel>,
    mut query_level_set: Query<&mut LevelSet>,
) {
    commands.spawn(InterLevelTimer(Timer::from_seconds(0.7, TimerMode::Once)));
    if let Ok(mut level_set) = query_level_set.get_single_mut() {
        *level_set = LevelSet::from_iids([LEVEL_IIDS[target_level.0]]);
    }
}

fn inter_level_pause(
    mut commands: Commands,
    mut query_timer: Query<(Entity, &mut InterLevelTimer)>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<LevelLoadingState>>,
) {
    let Ok((e, mut timer)) = query_timer.get_single_mut() else {
        println!("did not find timer");
        return;
    };
    if timer.0.finished() {
        next_state.set(LevelLoadingState::Loaded);
        commands.entity(e).despawn();
    }
    timer.0.tick(time.delta());
}

fn cleanup_level_objects(
    query: Query<Entity, Or<(With<LevelIid>, With<BackwardsBarrier>)>>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

const LEVEL_IIDS: [&str; 4] = [
    "a4a8aaa0-25d0-11ef-8b42-cbb4af80c537",
    "410524d0-25d0-11ef-b3d7-db494d819bf6",
    "a56e81e0-25d0-11ef-a5a2-a938910d70c0",
    "dd650080-25d0-11ef-814d-6b1968b17386",
];

fn spawn_ldtk_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    target_level: Res<TargetLevel>,
) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("level.ldtk"),
        level_set: LevelSet::from_iids([LEVEL_IIDS[target_level.0]]),
        ..default()
    });
}

fn finish_level(
    mut query_player: Query<&mut PlayerStatus, With<PlayerMarker>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut player_status) = query_player.get_single_mut() else {
        return;
    };
    if keys.just_pressed(KeyCode::KeyF) {
        player_status.level_finished = true;
    }
}

#[derive(Default, Component)]
pub struct KillPlayerMarker;

#[derive(Default, Component)]
struct WaterMarker;

#[derive(Bundle, LdtkIntCell)]
struct WaterBundle {
    water_marker: WaterMarker,
    kill_player_marker: KillPlayerMarker,
    collider: Collider,
    sensor: Sensor,
}

impl Default for WaterBundle {
    fn default() -> Self {
        Self {
            water_marker: WaterMarker,
            kill_player_marker: KillPlayerMarker,
            collider: Collider::cuboid(8., 6.),
            sensor: Sensor,
        }
    }
}

#[derive(Default, Component)]
struct SpikeMarker;

#[derive(Bundle, LdtkIntCell)]
struct SpikeBundle {
    spike_marker: SpikeMarker,
    kill_player_marker: KillPlayerMarker,
    collider: Collider,
    sensor: Sensor,
}

impl Default for SpikeBundle {
    fn default() -> Self {
        Self {
            spike_marker: SpikeMarker,
            kill_player_marker: KillPlayerMarker,
            collider: Collider::cuboid(5., 5.),
            sensor: Sensor,
        }
    }
}

#[derive(Default, Component)]
struct TerrainMarker;

#[derive(Bundle, LdtkIntCell)]
struct TerrainBundle {
    terrain_marker: TerrainMarker,
    rigid_body: RigidBody,
    // collider: Collider,
}

impl Default for TerrainBundle {
    fn default() -> Self {
        Self {
            terrain_marker: TerrainMarker,
            rigid_body: RigidBody::Fixed,
            // collider: Collider::cuboid(8., 8.), // cuboid better because less points!!! (?)
        }
    }
}

#[derive(Component)]
struct BackwardsBarrier;

fn spawn_backwards_barrier(mut commands: Commands) {
    commands
        .spawn((Collider::cuboid(1., 1000.), BackwardsBarrier))
        .insert(TransformBundle::from_transform(Transform::from_xyz(
            0., 0., 0.,
        )));
}

fn update_backwards_barrier(
    query_level: Query<&LayerMetadata, With<LayerMetadata>>,
    query_camera: Query<
        (&Camera, &Transform, &GlobalTransform),
        (With<PlayerCameraMarker>, Without<BackwardsBarrier>),
    >,
    mut query_barrier: Query<&mut Transform, With<BackwardsBarrier>>,
) {
    let Ok(mut barrier) = query_barrier.get_single_mut() else {
        return;
    };
    let Ok((camera, camera_transform, camera_global_transform)) = query_camera.get_single() else {
        return;
    };

    let mut level_width = 0.;
    for level in query_level.iter() {
        if level.layer_instance_type == bevy_ecs_ldtk::ldtk::Type::IntGrid {
            level_width = level.c_wid as f32 * 16.;
        }
    }

    let w_end = camera
        .viewport_to_world_2d(
            camera_global_transform,
            camera.logical_viewport_size().unwrap(),
        )
        .unwrap()
        .x;
    let w_start = camera
        .viewport_to_world_2d(camera_global_transform, Vec2::new(0., 0.))
        .unwrap()
        .x;
    let width = w_end - w_start;

    let barrier_offset = 5.;
    let barrier_jitter_correction = 10.;
    barrier.translation.x = camera_transform.translation.x - width / 2. - barrier_offset;
    if barrier.translation.x < 0. {
        barrier.translation.x += level_width;
    }
    if barrier.translation.x > level_width - barrier_offset - barrier_jitter_correction {
        barrier.translation.x = level_width - barrier_offset - barrier_jitter_correction
    }
}
