use bevy::{prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

use crate::camera::{PlayerCameraMarker, PLAYER_RENDER_LAYER};
use crate::state::{LevelLoadingState, TargetLevel};

pub struct LevelManagementPlugin;

impl Plugin for LevelManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            // .insert_resource(LevelSelection::index(0))
            .insert_resource(AnimationInfo::default())
            .add_event::<SetCheckpointEvent>()
            .register_ldtk_entity::<PlayerBundle>("Player")
            .register_ldtk_int_cell::<TerrainBundle>(1)
            .register_ldtk_int_cell::<WaterBundle>(2)
            .register_ldtk_int_cell::<SpikeBundle>(4)
            .add_systems(Startup, spawn_ldtk_world)
            .add_systems(OnEnter(LevelLoadingState::Loading), (load_level,))
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
                (
                    add_collider,
                    update_player_grounded,
                    finish_level,
                    move_player,
                    loop_player,
                    update_backwards_barrier,
                    animate_player,
                    set_player_checkpoint,
                    kill_player,
                )
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

fn add_collider(mut commands: Commands, query: Query<(Entity, &Transform), Added<PlayerMarker>>) {
    if let Ok((entity, player_transform)) = query.get_single() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Collider::round_cuboid(3., 2., 2.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                TransformBundle::from_transform(Transform::from_xyz(0., -4.2, 0.)),
                PlayerJumpColliderMarker,
            ));
        });
        commands.entity(entity).insert(PlayerCheckpoint(player_transform.translation.xy()));
    }
}

fn update_player_grounded(
    query_player_jump_collider: Query<Entity, With<PlayerJumpColliderMarker>>,
    mut query_player: Query<(&mut PlayerState, &Velocity), With<PlayerMarker>>,
    rapier_context: Res<RapierContext>,
) {
    if let Ok(player_jump_controller_entity) = query_player_jump_collider.get_single() {
        if let Ok((mut player_state, velocity)) = query_player.get_single_mut() {
            if rapier_context
                .intersection_pairs_with(player_jump_controller_entity)
                .peekable()
                .peek()
                != None
            {
                // if on the ground
                if *player_state == PlayerState::Falling {
                    *player_state = PlayerState::FallingToIdle;
                }
            } else {
                if velocity.linvel.y < 0. {
                    *player_state = PlayerState::Falling;
                }
            }
        }
    }
}

const LEVEL_IIDS: [&str; 2] = [
    "410524d0-25d0-11ef-b3d7-db494d819bf6",
    "a56e81e0-25d0-11ef-a5a2-a938910d70c0",
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

fn move_player(
    mut query_player: Query<
        (
            Entity,
            &mut Velocity,
            &mut ExternalForce,
            &Transform,
            &mut Sprite,
            &mut PlayerStatus,
            &mut PlayerState,
        ),
        With<PlayerMarker>,
    >,
    keys: Res<ButtonInput<KeyCode>>,
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
) {
    if let Ok((
        player_entity,
        mut player_velocity,
        mut spring_force,
        player_transform,
        mut sprite,
        mut player_status,
        mut player_state,
    )) = query_player.get_single_mut()
    {
        // spring force added here so that the screen does not shake when the character walks over
        // grid boundaries
        const SPRING_CONSTANT: f32 = 15000.0;
        let ray_pos = player_transform.translation.xy();
        let ray_dir = -1. * Vec2::Y;
        let max_toi = 10.;
        let solid = true;
        let filter = QueryFilter::default()
            .exclude_sensors()
            .exclude_collider(player_entity);
        if rapier_context
            .cast_ray(ray_pos, ray_dir, max_toi, solid, filter)
            .is_some()
            && *player_state != PlayerState::Jumping
            && *player_state != PlayerState::Falling
        {
            let (_, toi) = rapier_context
                .cast_ray(ray_pos, ray_dir, max_toi, solid, filter)
                .unwrap();
            let dist = ray_dir.length() * (max_toi - toi);
            spring_force.force = dist * SPRING_CONSTANT * Vec2::Y
                - SPRING_CONSTANT / 5. * player_velocity.linvel.y * Vec2::Y;
        } else {
            spring_force.force = Vec2::ZERO;
        }

        if !player_status.jump_cooldown.finished() {
            player_status.jump_cooldown.tick(time.delta());
            spring_force.force = Vec2::ZERO;
            // player_status.grounded = false;
            // *player_state = PlayerState::Jumping;
        }
        // player_velocity.linvel = Vec2::ZERO;
        const VELOCITY: Vec2 = Vec2::new(55., 0.);
        let mut moved = false;
        if keys.pressed(KeyCode::ArrowRight) {
            player_velocity.linvel += VELOCITY;
            if *player_state == PlayerState::MovingLeft || *player_state == PlayerState::Idle {
                *player_state = PlayerState::MovingRight;
            }
            sprite.flip_x = false;
            moved = true;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            player_velocity.linvel -= VELOCITY;
            if *player_state == PlayerState::MovingRight || *player_state == PlayerState::Idle {
                *player_state = PlayerState::MovingLeft;
            }
            sprite.flip_x = true;
            moved = true
        }
        if !moved {
            if *player_state == PlayerState::MovingLeft || *player_state == PlayerState::MovingRight
            {
                *player_state = PlayerState::MovingToIdle;
            }
        }
        if keys.pressed(KeyCode::ArrowUp)
            && *player_state != PlayerState::Jumping
            && *player_state != PlayerState::Falling
        {
            // ugly but i wrote it like this so i can print debug messages
            if player_status.jump_cooldown.finished() {
                player_velocity.linvel = 130. * Vec2::Y;
                spring_force.force = Vec2::ZERO;
                *player_state = PlayerState::Jumping;
                player_status.jump_cooldown.reset();
            }
        }
        player_velocity.linvel.x /= 1.6;
        if player_velocity.linvel.x.abs() < 0.1 {
            player_velocity.linvel.x = 0.;
        }
    }
}

fn animate_player(
    time: Res<Time>,
    animation_info: Res<AnimationInfo>,
    mut query: Query<
        (&mut TextureAtlas, &mut PlayerState, &mut AnimationTimer),
        With<PlayerMarker>,
    >,
) {
    if let Ok((mut atlas, mut state, mut timer)) = query.get_single_mut() {
        timer.tick(time.delta());
        // println!("state: {:?}", *state);
        if timer.finished() {
            match *state {
                PlayerState::Idle => {
                    // no idle animation as of now

                    atlas.index = 0;
                }
                PlayerState::MovingLeft => {
                    if atlas.index < animation_info.moving_start
                        || atlas.index > animation_info.moving_end
                    {
                        atlas.index = animation_info.moving_start;
                    } else {
                        atlas.index = if atlas.index == animation_info.moving_end {
                            animation_info.moving_start + 2
                        } else {
                            atlas.index + 1
                        };
                    }

                    timer.set_duration(Duration::from_millis(
                        animation_info.moving_durations[atlas.index - animation_info.moving_start],
                    ));
                }
                PlayerState::MovingRight => {
                    if atlas.index < animation_info.moving_start
                        || atlas.index > animation_info.moving_end
                    {
                        atlas.index = animation_info.moving_start;
                    } else {
                        atlas.index = if atlas.index == animation_info.moving_end {
                            animation_info.moving_start + 2
                        } else {
                            atlas.index + 1
                        };
                    }

                    timer.set_duration(Duration::from_millis(
                        animation_info.moving_durations[atlas.index - animation_info.moving_start],
                    ));
                }
                PlayerState::Jumping => {
                    if atlas.index < animation_info.jumping_start
                        || atlas.index > animation_info.jumping_end
                    {
                        atlas.index = animation_info.jumping_start;
                    } else {
                        atlas.index = if atlas.index == animation_info.jumping_end {
                            atlas.index
                        } else {
                            atlas.index + 1
                        };
                    }

                    timer.set_duration(Duration::from_millis(
                        animation_info.jumping_durations
                            [atlas.index - animation_info.jumping_start],
                    ));
                }
                PlayerState::Falling => {
                    if atlas.index < animation_info.falling_start
                        || atlas.index > animation_info.falling_end
                    {
                        atlas.index = animation_info.falling_start;
                    } else {
                        atlas.index = if atlas.index == animation_info.falling_end {
                            atlas.index
                        } else {
                            atlas.index + 1
                        };
                    }

                    timer.set_duration(Duration::from_millis(
                        animation_info.falling_durations
                            [atlas.index - animation_info.falling_start],
                    ));
                }
                PlayerState::MovingToIdle => {
                    atlas.index = animation_info.moving_start + 1;
                    timer.set_duration(Duration::from_millis(50));

                    *state = PlayerState::Idle;
                }
                PlayerState::FallingToIdle => {
                    if atlas.index < animation_info.falling_to_idle_start
                        || atlas.index > animation_info.falling_to_idle_end
                    {
                        atlas.index = animation_info.falling_to_idle_start;
                    }
                    atlas.index = if atlas.index == animation_info.falling_to_idle_end {
                        *state = PlayerState::Idle;
                        atlas.index
                    } else {
                        atlas.index + 1
                    };

                    timer.set_duration(Duration::from_millis(
                        animation_info.falling_to_idle_durations
                            [atlas.index - animation_info.falling_to_idle_start],
                    ));
                }
            }
        }
    }
}

// TODO: split camera looping and player looping into separate systems
pub fn loop_player(
    mut query_player_camera: Query<
        &mut Transform,
        (With<PlayerCameraMarker>, Without<PlayerMarker>),
    >,
    mut query_player: Query<&mut Transform, With<PlayerMarker>>,
    query_level: Query<&LayerMetadata>,
) {
    if let Ok(mut player_transform) = query_player.get_single_mut() {
        if let Ok(mut camera_transform) = query_player_camera.get_single_mut() {
            for level in query_level.iter() {
                if level.layer_instance_type == bevy_ecs_ldtk::ldtk::Type::IntGrid {
                    let width = level.c_wid as f32 * 16.;
                    if player_transform.translation.x < 0. {
                        player_transform.translation.x += width;
                        camera_transform.translation.x += width;
                        println!(
                            "looped camera transform is {}",
                            camera_transform.translation.x
                        )
                    } else if player_transform.translation.x > width {
                        player_transform.translation.x -= width;
                        camera_transform.translation.x -= width;
                    }
                }
            }
        }
    }
}

fn set_player_checkpoint(
    mut query_player_checkpoint: Query<&mut PlayerCheckpoint, With<PlayerMarker>>,
    mut checkpoint_events: EventReader<SetCheckpointEvent>
) {
    let Ok(mut player_checkpoint) = query_player_checkpoint.get_single_mut() else {
        return;
    };
    for SetCheckpointEvent(coords) in checkpoint_events.read() {
        player_checkpoint.0 = *coords;
        println!("set player checkpoint to {}", player_checkpoint.0)
    }
}

fn kill_player(
    mut query_player: Query<(Entity, &mut PlayerStatus), With<PlayerMarker>>,
    query_hazards: Query<Entity, With<KillPlayerMarker>>,
    rapier_context: Res<RapierContext>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok((player_entity, mut player_status)) = query_player.get_single_mut() else {
        return;
    };
    let mut kill_player = false;
    if keys.just_pressed(KeyCode::KeyR) {
        kill_player = true;
    } else {
        for hazard in query_hazards.iter() {
            if rapier_context.intersection_pair(player_entity, hazard) == Some(true) {
                kill_player = true;
            }
        }
    }
    if kill_player {
        player_status.dead = true;
    }
}

#[derive(Component)]
struct PlayerJumpColliderMarker;

#[derive(Default, Component)]
pub struct PlayerMarker;

#[derive(Resource)]
struct AnimationInfo {
    moving_start: usize,
    moving_end: usize,
    jumping_start: usize,
    jumping_end: usize,
    falling_start: usize,
    falling_end: usize,
    falling_to_idle_start: usize,
    falling_to_idle_end: usize,

    moving_durations: Vec<u64>,
    jumping_durations: Vec<u64>,
    falling_durations: Vec<u64>,
    falling_to_idle_durations: Vec<u64>,
}
impl Default for AnimationInfo {
    fn default() -> Self {
        Self {
            moving_start: 10,
            moving_end: 13,
            jumping_start: 0,
            jumping_end: 2,
            falling_start: 2,
            falling_end: 4,
            falling_to_idle_start: 6,
            falling_to_idle_end: 10,

            moving_durations: vec![100, 100, 100, 100],
            jumping_durations: vec![100, 100, 100],
            falling_durations: vec![100, 100, 100],
            falling_to_idle_durations: vec![50, 50, 50, 50, 50],
        }
    }
}

#[derive(Component)]
pub struct PlayerStatus {
    jump_cooldown: Timer,
    pub level_finished: bool,
    pub dead: bool,
    // air_jumps: usize,
    // max_air_jumps: usize,
}

#[derive(Component, Debug, PartialEq, Eq)]
enum PlayerState {
    Idle,
    MovingLeft,
    MovingRight,
    Jumping,
    Falling,
    MovingToIdle,
    FallingToIdle,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Debug)]
pub struct PlayerInventory {
    num_keys: usize,
}

impl PlayerInventory {
    pub fn has_key(&self) -> bool {
        self.num_keys > 0
    }

    pub fn use_key(&mut self) {
        self.num_keys -= 1;
    }

    pub fn add_key(&mut self) {
        self.num_keys += 1;
    }
}

#[derive(Component, Debug)]
pub struct PlayerCheckpoint(pub Vec2);

#[derive(Event)]
pub struct SetCheckpointEvent(pub Vec2);

#[derive(Bundle, LdtkEntity)]
struct PlayerBundle {
    #[sprite_sheet_bundle("../assets/spritesheets/slimespritesheet.png", 16, 16, 11, 2, 1, 0, 0)]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    render_layer: RenderLayers,
    player_marker: PlayerMarker,
    player_status: PlayerStatus,
    player_inventory: PlayerInventory,
    rigid_body: RigidBody,
    collider: Collider,
    mass: AdditionalMassProperties,
    velocity: Velocity,
    friction: Friction,
    restitution: Restitution,
    spring_force: ExternalForce,
    locked_axes: LockedAxes,
    player_state: PlayerState,
    animation_timer: AnimationTimer,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        let mut jump_cooldown_timer = Timer::new(Duration::from_millis(300), TimerMode::Once);
        jump_cooldown_timer.tick(Duration::from_millis(300));
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            render_layer: PLAYER_RENDER_LAYER,
            player_marker: PlayerMarker,
            player_status: PlayerStatus {
                jump_cooldown: jump_cooldown_timer,
                level_finished: false,
                dead: false,
                // air_jumps: 1,
                // max_air_jumps: 1,
            },
            player_inventory: PlayerInventory { num_keys: 0 },
            rigid_body: RigidBody::Dynamic,
            // collider: Collider::cuboid(5., 5.),
            collider: Collider::round_cuboid(5., 3., 2.),
            mass: AdditionalMassProperties::Mass(50.),
            velocity: Velocity::default(),
            friction: Friction {
                coefficient: 0.,
                combine_rule: CoefficientCombineRule::Min,
            },
            restitution: Restitution {
                coefficient: 0.,
                combine_rule: CoefficientCombineRule::Min,
            },
            spring_force: ExternalForce::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            player_state: PlayerState::Idle,
            animation_timer: AnimationTimer(Timer::new(
                Duration::from_millis(100),
                TimerMode::Repeating,
            )),
        }
    }
}

#[derive(Default, Component)]
struct KillPlayerMarker;

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
            collider: Collider::cuboid(8., 8.),
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
    collider: Collider,
}

impl Default for TerrainBundle {
    fn default() -> Self {
        Self {
            terrain_marker: TerrainMarker,
            rigid_body: RigidBody::Fixed,
            collider: Collider::cuboid(8., 8.), // cuboid better because less points!!! (?)
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
