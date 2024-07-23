use bevy::{prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

use crate::camera::{PlayerCameraMarker, PLAYER_RENDER_LAYER};

pub struct LevelManagementPlugin;

impl Plugin for LevelManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            .insert_resource(LevelSelection::index(0))
            .register_ldtk_entity::<PlayerBundle>("Player")
            .register_ldtk_int_cell::<TerrainBundle>(1)
            .add_systems(Startup, spawn_level)
            .add_systems(Update, add_collider)
            .add_systems(Update, update_player_grounded)
            .add_systems(Update, move_player)
            .add_systems(Update, loop_player);
    }
}

fn add_collider(mut commands: Commands, query: Query<Entity, Added<PlayerMarker>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Collider::round_cuboid(4., 2., 2.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                TransformBundle::from_transform(Transform::from_xyz(0., -4.2, 0.)),
                PlayerJumpColliderMarker,
            ));
        });
    }
}

fn update_player_grounded(
    query_player_jump_collider: Query<Entity, With<PlayerJumpColliderMarker>>,
    mut query_player: Query<&mut PlayerStatus, With<PlayerMarker>>,
    rapier_context: Res<RapierContext>,
) {
    if let Ok(player_jump_controller_entity) = query_player_jump_collider.get_single() {
        if let Ok(mut player_status) = query_player.get_single_mut() {
            player_status.grounded = rapier_context
                .intersection_pairs_with(player_jump_controller_entity)
                .peekable()
                .peek()
                != None;
        }
    }
}

fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("level.ldtk"),
        ..default()
    });
}

fn move_player(
    mut query_player: Query<
        (
            Entity,
            &mut Velocity,
            &mut ExternalForce,
            &Transform,
            &mut PlayerStatus,
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
        transform,
        mut player_status,
    )) = query_player.get_single_mut()
    {
        // spring force added here so that the screen does not shake when the character walks over
        // grid boundaries
        const SPRING_CONSTANT: f32 = 15000.0;
        let ray_pos = transform.translation.xy();
        let ray_dir = -1. * Vec2::Y;
        let max_toi = 10.;
        let solid = true;
        let filter = QueryFilter::default()
            .exclude_sensors()
            .exclude_collider(player_entity);
        if rapier_context
            .cast_ray(ray_pos, ray_dir, max_toi, solid, filter)
            .is_some()
            && player_status.grounded
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
            player_status.grounded = false;
        }
        // player_velocity.linvel = Vec2::ZERO;
        if keys.pressed(KeyCode::ArrowRight) {
            player_velocity.linvel += 35. * Vec2::X;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            player_velocity.linvel -= 35. * Vec2::X;
        }
        if keys.pressed(KeyCode::ArrowUp) && player_status.grounded {
            // ugly but i wrote it like this so i can print debug messages
            if player_status.jump_cooldown.finished() {
                player_velocity.linvel = 100. * Vec2::Y;
                spring_force.force = Vec2::ZERO;
                player_status.grounded = false;
                player_status.jump_cooldown.reset();
            }
        }
        player_velocity.linvel.x /= 1.6;
        if player_velocity.linvel.x.abs() < 0.1 {
            player_velocity.linvel.x = 0.;
        }
    }
}

fn loop_player(
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
                    } else if player_transform.translation.x > width {
                        player_transform.translation.x -= width;
                        camera_transform.translation.x -= width;
                    }
                    // player_transform.translation.x = ((player_transform.translation.x % width) + width) % width;
                }
            }
        }
    }
}

#[derive(Component)]
struct PlayerJumpColliderMarker;

#[derive(Default, Component)]
pub struct PlayerMarker;

#[derive(Component)]
struct PlayerStatus {
    grounded: bool,
    jump_cooldown: Timer,
    air_jumps: usize,
    max_air_jumps: usize,
}

#[derive(Bundle, LdtkEntity)]
struct PlayerBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    render_layer: RenderLayers,
    player_marker: PlayerMarker,
    player_status: PlayerStatus,
    rigid_body: RigidBody,
    collider: Collider,
    mass: AdditionalMassProperties,
    velocity: Velocity,
    friction: Friction,
    restitution: Restitution,
    spring_force: ExternalForce,
    locked_axes: LockedAxes,
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
                grounded: true,
                jump_cooldown: jump_cooldown_timer,
                air_jumps: 1,
                max_air_jumps: 1,
            },
            rigid_body: RigidBody::Dynamic,
            // collider: Collider::cuboid(5., 5.),
            collider: Collider::round_cuboid(5., 5., 2.),
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
            spring_force: ExternalForce {
                // force: Vec2::Y * 100.,
                ..default()
            },
            locked_axes: LockedAxes::ROTATION_LOCKED,
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
