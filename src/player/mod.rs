use bevy::{prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

pub mod animation;

use crate::camera::{PlayerCameraMarker, PLAYER_RENDER_LAYER};
use crate::level::KillPlayerMarker;
use crate::state::LevelLoadingState;

use animation::{animate_player, AnimationInfo, AnimationTimer};

pub struct PlayerManagementPlugin;

impl Plugin for PlayerManagementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnimationInfo::default())
            .add_event::<SetCheckpointEvent>()
            .register_ldtk_entity::<PlayerBundle>("Player")
            .add_systems(
                Update,
                (
                    add_jump_collider,
                    update_player_grounded,
                    move_player,
                    loop_player,
                    animate_player,
                    set_player_checkpoint,
                    kill_player,
                )
                    .run_if(in_state(LevelLoadingState::Loaded)),
            );
    }
}

#[derive(Default, Component)]
pub struct PlayerMarker;

#[derive(Component)]
pub struct PlayerJumpColliderMarker;

#[derive(Component)]
pub struct PlayerStatus {
    jump_cooldown: Timer,
    pub level_finished: bool,
    pub dead: bool,
    // air_jumps: usize,
    // max_air_jumps: usize,
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum PlayerState {
    Idle,
    MovingLeft,
    MovingRight,
    Jumping,
    Falling,
    MovingToIdle,
    FallingToIdle,
}

#[derive(Component, Debug)]
pub struct PlayerInventory {
    pub num_keys: usize,
    pub max_extra_jumps: usize,
    pub extra_jumps: usize,
    pub air_jumps: usize,
}

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
            player_inventory: PlayerInventory {
                num_keys: 0,
                max_extra_jumps: 0,
                extra_jumps: 0,
                air_jumps: 0,
            },
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

fn add_jump_collider(
    mut commands: Commands,
    query: Query<(Entity, &Transform), Added<PlayerMarker>>,
) {
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
        commands.entity(entity).insert(PlayerCheckpoint {
            transform: player_transform.translation.xy(),
            air_jumps: 0,
        });
    }
}

fn update_player_grounded(
    query_player_jump_collider: Query<Entity, With<PlayerJumpColliderMarker>>,
    mut query_player: Query<
        (&mut PlayerInventory, &mut PlayerState, &Velocity),
        With<PlayerMarker>,
    >,
    query_sensors: Query<
        Entity,
        (
            With<Sensor>,
            Without<PlayerMarker>,
            Without<PlayerJumpColliderMarker>,
        ),
    >,
    rapier_context: Res<RapierContext>,
) {
    if let Ok(player_jump_controller_entity) = query_player_jump_collider.get_single() {
        if let Ok((mut player_inventory, mut player_state, velocity)) =
            query_player.get_single_mut()
        {
            let mut grounded = false;
            for (collider_1, collider_2, _) in
                rapier_context.intersection_pairs_with(player_jump_controller_entity)
            {
                let other_entity = if collider_1 != player_jump_controller_entity {
                    collider_1
                } else {
                    collider_2
                };
                if query_sensors.get(other_entity).is_err() {
                    grounded = true;
                    player_inventory.extra_jumps = player_inventory.max_extra_jumps;
                }
            }
            if grounded && *player_state == PlayerState::Falling {
                *player_state = PlayerState::FallingToIdle;
            } else if !grounded
                && velocity.linvel.y < 0.
                && *player_state != PlayerState::FallingToIdle
            {
                *player_state = PlayerState::Falling;
            }
        }
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
            &mut PlayerInventory,
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
        mut player_inventory,
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
        if player_status.dead || player_status.level_finished {
            return;
        }
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
        if keys.pressed(KeyCode::ArrowUp) && player_status.jump_cooldown.finished() {
            let mut can_jump = false;
            if *player_state != PlayerState::Jumping && *player_state != PlayerState::Falling {
                // jump from floor
                can_jump = true;
            } else if player_inventory.extra_jumps >= 1 {
                // jump in air with double jump
                can_jump = true;
                player_inventory.extra_jumps -= 1;
            } else if player_inventory.air_jumps >= 1 {
                // jump in air with jump token
                can_jump = true;
                player_inventory.air_jumps -= 1;
            }
            // ugly but i wrote it like this so i can print debug messages
            if can_jump {
                player_velocity.linvel.y = 130.;
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

// TODO: split camera looping and player looping into separate systems
pub fn loop_player(
    mut query_player_camera: Query<
        &mut Transform,
        (With<PlayerCameraMarker>, Without<PlayerMarker>),
    >,
    mut query_player: Query<&mut Transform, With<PlayerMarker>>,
    query_level: Query<&LayerMetadata>,
) {
    let Ok(mut player_transform) = query_player.get_single_mut() else {
        return;
    };
    let Ok(mut camera_transform) = query_player_camera.get_single_mut() else {
        return;
    };
    for level in query_level.iter() {
        if level.layer_instance_type != bevy_ecs_ldtk::ldtk::Type::IntGrid {
            continue;
        }

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

#[derive(Component, Debug)]
pub struct PlayerCheckpoint {
    pub transform: Vec2,
    pub air_jumps: usize,
}

#[derive(Event)]
pub struct SetCheckpointEvent;

fn set_player_checkpoint(
    mut query_player: Query<
        (&mut PlayerCheckpoint, &PlayerInventory, &Transform),
        With<PlayerMarker>,
    >,
    mut checkpoint_events: EventReader<SetCheckpointEvent>,
) {
    let Ok((mut player_checkpoint, player_inventory, player_transform)) =
        query_player.get_single_mut()
    else {
        return;
    };
    for SetCheckpointEvent in checkpoint_events.read() {
        player_checkpoint.transform = player_transform.translation.xy();
        player_checkpoint.air_jumps = player_inventory.air_jumps;
        println!("set player checkpoint to {}", player_checkpoint.transform)
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
        // time.pause();
    }
}
