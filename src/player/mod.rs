use bevy::{prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

pub mod animation;

use crate::camera::{PlayerCameraMarker, PLAYER_RENDER_LAYER};
use crate::level::{BackwardsBarrier, KillPlayerMarker};
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
                    add_colliders,
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
pub struct PlayerColliderMarker;

#[derive(Component)]
pub struct PlayerJumpColliderMarker;

#[derive(Component)]
pub struct PlayerWallColliderMarker {
    dir: usize,
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
pub enum PlayerState {
    Idle,
    MovingLeft,
    MovingRight,
    Jumping,
    Falling,
    MovingToIdle,
    FallingToIdle,
    Sliding,
    SlidingToJump
}

#[derive(Component, Debug)]
pub struct PlayerInventory {
    pub num_keys: usize,
    pub max_extra_jumps: usize,
    pub extra_jumps: usize,
    pub air_jumps: usize,
    pub wall_jump_cd: [Timer; 2],
    pub on_wall: [bool; 2],
    pub has_wall_jump: bool,
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
                wall_jump_cd: [
                    Timer::from_seconds(0.8, TimerMode::Once),
                    Timer::from_seconds(0.8, TimerMode::Once),
                ],
                on_wall: [false; 2],
                has_wall_jump: false,
            },
            rigid_body: RigidBody::Dynamic,
            // collider: Collider::cuboid(5., 5.),
            collider: Collider::round_cuboid(6., 3., 2.),
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
            locked_axes: LockedAxes::ROTATION_LOCKED,
            player_state: PlayerState::Idle,
            animation_timer: AnimationTimer(Timer::new(
                Duration::from_millis(100),
                TimerMode::Repeating,
            )),
        }
    }
}

fn add_colliders(mut commands: Commands, query: Query<(Entity, &Transform), Added<PlayerMarker>>) {
    if let Ok((entity, player_transform)) = query.get_single() {
        commands.entity(entity).remove::<Collider>();
        commands.entity(entity).remove::<Friction>();
        commands.entity(entity).remove::<Restitution>();
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Collider::round_cuboid(6., 3., 2.),
                TransformBundle::from_transform(Transform::from_xyz(0., -2., 0.)),
                Friction {
                    coefficient: 0.,
                    combine_rule: CoefficientCombineRule::Min,
                },
                Restitution {
                    coefficient: 0.,
                    combine_rule: CoefficientCombineRule::Min,
                },
                PlayerColliderMarker,
            ));
            parent.spawn((
                Collider::round_cuboid(3., 2., 2.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                TransformBundle::from_transform(Transform::from_xyz(0., -4.2, 0.)),
                PlayerJumpColliderMarker,
            ));
            for i in 0..2 {
                let dir = 2. * i as f32 - 1.;
                parent.spawn((
                    Collider::round_cuboid(2., 1., 2.),
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                    TransformBundle::from_transform(Transform::from_xyz(dir * 4.3, -2., 0.)),
                    PlayerWallColliderMarker { dir: i },
                ));
            }
        });
        commands.entity(entity).insert(PlayerCheckpoint {
            transform: player_transform.translation.xy(),
        });
    }
}

fn update_player_grounded(
    query_player_jump_collider: Query<Entity, With<PlayerJumpColliderMarker>>,
    mut query_player_wall_collider: Query<
        (&mut PlayerWallColliderMarker, Entity),
        With<PlayerWallColliderMarker>,
    >,
    mut query_player: Query<
        (Entity, &mut PlayerInventory, &mut PlayerState, &Velocity),
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
    query_backwards_barrier: Query<Entity, With<BackwardsBarrier>>,
    rapier_context: Res<RapierContext>,
) {
    let Ok(player_jump_collider_entity) = query_player_jump_collider.get_single() else {
        return;
    };

    let Ok((player_entity, mut player_inventory, mut player_state, velocity)) =
        query_player.get_single_mut()
    else {
        return;
    };
    let Ok(backwards_barrier) = query_backwards_barrier.get_single() else {
        return;
    };

    // update if the player is touching the wall
    for (wall_cooldown, wall_collider) in query_player_wall_collider.iter_mut() {
        // check the collider to see if it is next to a wall
        player_inventory.on_wall[wall_cooldown.dir] = false;
        for (collider_1, collider_2, _) in rapier_context.intersection_pairs_with(wall_collider) {
            let other_entity = if collider_1 != wall_collider {
                collider_1
            } else {
                collider_2
            };
            if query_sensors.get(other_entity).is_err()
                && other_entity != player_entity
                && other_entity != backwards_barrier
            {
                player_inventory.on_wall[wall_cooldown.dir] = true;
                if player_inventory.has_wall_jump {
                    *player_state = PlayerState::Sliding;
                }
                // remove the air jumps if hit something
                player_inventory.air_jumps = 0;
            }
        }
    }

    let mut grounded = false;
    for (collider_1, collider_2, _) in
        rapier_context.intersection_pairs_with(player_jump_collider_entity)
    {
        let other_entity = if collider_1 != player_jump_collider_entity {
            collider_1
        } else {
            collider_2
        };
        if query_sensors.get(other_entity).is_err() {
            grounded = true;
            player_inventory.extra_jumps = player_inventory.max_extra_jumps;
            // remove the air jumps if hit something
            player_inventory.air_jumps = 0;
        }
    }

    if grounded && (*player_state == PlayerState::Falling || *player_state == PlayerState::Sliding) {
        println!("Resetting jump");
        *player_state = PlayerState::FallingToIdle;
    } else if !grounded && *player_state == PlayerState::Sliding && !(player_inventory.on_wall[0] || player_inventory.on_wall[1]) {
        *player_state = PlayerState::SlidingToJump;
    } else if !grounded && velocity.linvel.y < 0. && (*player_state != PlayerState::FallingToIdle && *player_state != PlayerState::Sliding && *player_state != PlayerState::SlidingToJump) {
        *player_state = PlayerState::Falling;
    }
}

fn move_player(
    mut query_player: Query<
        (
            &mut Velocity,
            &mut Sprite,
            &mut PlayerInventory,
            &mut PlayerStatus,
            &mut PlayerState,
        ),
        With<PlayerMarker>,
    >,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((
        mut player_velocity,
        mut sprite,
        mut player_inventory,
        mut player_status,
        mut player_state,
    )) = query_player.get_single_mut()
    {
        
        if !player_status.jump_cooldown.finished() {
            player_status.jump_cooldown.tick(time.delta());
            // player_status.grounded = false;
            // *player_state = PlayerState::Jumping;
        }
        let mut on_wall = false;
        for i in 0..2 {
            if !player_inventory.wall_jump_cd[i].finished() {
                player_inventory.wall_jump_cd[i].tick(time.delta());
            }
            if player_inventory.on_wall[i] {
                on_wall = true;
            }
        }
        // println!("state: {:?}", *player_state);
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
            if *player_state != PlayerState::SlidingToJump && *player_state != PlayerState::Sliding {
                sprite.flip_x = false;
            } else if *player_state == PlayerState::Sliding && !on_wall {
                *player_state = PlayerState::SlidingToJump;
            }
            moved = true;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            player_velocity.linvel -= VELOCITY;
            if *player_state == PlayerState::MovingRight || *player_state == PlayerState::Idle {
                *player_state = PlayerState::MovingLeft;
            }
            if *player_state != PlayerState::SlidingToJump && *player_state != PlayerState::Sliding {
                sprite.flip_x = true;
            } else if *player_state == PlayerState::Sliding && !on_wall {
                *player_state = PlayerState::SlidingToJump;
            }
            moved = true
        }

        // hack
        if player_inventory.on_wall[0] {
            sprite.flip_x = true;
        } else if player_inventory.on_wall[1] {
            sprite.flip_x = false;
        }

        if !moved {
            if *player_state == PlayerState::MovingLeft || *player_state == PlayerState::MovingRight
            {
                *player_state = PlayerState::MovingToIdle;
            }
        }
        if keys.just_pressed(KeyCode::ArrowUp) && player_status.jump_cooldown.finished() {
            let mut can_jump = false;
            let mut wall_jump = false;
            if *player_state != PlayerState::Jumping && *player_state != PlayerState::Falling {
                // jump from floor
                can_jump = true;
            } else if player_inventory.has_wall_jump
                && player_inventory.on_wall[0]
                && player_inventory.wall_jump_cd[0].finished()
            {
                // wall jump from left wall
                can_jump = true;
                wall_jump = true;
                player_inventory.wall_jump_cd[0].reset();
                // if they use the wall jump, reset their double jump
                player_inventory.extra_jumps = player_inventory.max_extra_jumps;
            } else if player_inventory.has_wall_jump
                && player_inventory.on_wall[1]
                && player_inventory.wall_jump_cd[1].finished()
            {
                // wall jump from right wall
                can_jump = true;
                wall_jump = true;
                player_inventory.wall_jump_cd[1].reset();
                // if they use the wall jump, reset their double jump
                player_inventory.extra_jumps = player_inventory.max_extra_jumps;
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
                if wall_jump {
                    *player_state = PlayerState::SlidingToJump;
                } else {
                    *player_state = PlayerState::Jumping;
                }
                player_status.jump_cooldown.reset();
                
            }
        }

        // allow player to slide down walls if they have wall jump
        if on_wall && player_inventory.has_wall_jump && player_velocity.linvel.y < -75. {
            player_velocity.linvel.y = -75.;
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
}

#[derive(Event)]
pub struct SetCheckpointEvent;

fn set_player_checkpoint(
    mut query_player: Query<(&mut PlayerCheckpoint, &Transform), With<PlayerMarker>>,
    mut checkpoint_events: EventReader<SetCheckpointEvent>,
) {
    let Ok((mut player_checkpoint, player_transform)) = query_player.get_single_mut() else {
        return;
    };
    for SetCheckpointEvent in checkpoint_events.read() {
        player_checkpoint.transform = player_transform.translation.xy();
        println!("set player checkpoint to {}", player_checkpoint.transform)
    }
}

fn kill_player(
    mut query_player: Query<&mut PlayerStatus, With<PlayerMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
    query_hazards: Query<Entity, With<KillPlayerMarker>>,
    rapier_context: Res<RapierContext>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut player_status) = query_player.get_single_mut() else {
        return;
    };
    let Ok(player_collider) = query_player_collider.get_single() else {
        return;
    };

    let mut kill_player = false;
    if keys.just_pressed(KeyCode::KeyR) {
        kill_player = true;
    } else {
        for hazard in query_hazards.iter() {
            if rapier_context.intersection_pair(player_collider, hazard) == Some(true) {
                kill_player = true;
            }
        }
    }
    if kill_player {
        player_status.dead = true;
        // time.pause();
    }
}
