use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::{
    animation::AnimationTimer, PlayerColliderMarker, PlayerInventory, PlayerMarker,
    SetCheckpointEvent,
};

#[derive(Component, Debug)]
pub struct WallJumpMarker;

#[derive(Component, Debug)]
pub struct WallJumpSensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct WallJumpBundle {
    #[sprite_sheet_bundle("../assets/spritesheets/walljump.png", 16, 16, 4, 1, 0, 0, 0)]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    wall_jump_marker: WallJumpMarker,
    animation_timer: AnimationTimer,
}

impl Default for WallJumpBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            wall_jump_marker: WallJumpMarker,
            animation_timer: AnimationTimer(Timer::new(
                Duration::from_millis(300),
                TimerMode::Repeating,
            )),
        }
    }
}

pub fn animate_wall_jump(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlas), With<WallJumpMarker>>,
) {
    for (mut timer, mut atlas) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.0.finished() {
            atlas.index = (atlas.index + 1) % 4;
        }
    }
}

pub fn add_wall_jump_sensor(
    mut commands: Commands,
    query_jump_token: Query<Entity, Added<WallJumpMarker>>,
) {
    for token in query_jump_token.iter() {
        commands.entity(token).with_children(|parent| {
            parent.spawn((
                Collider::cuboid(7., 7.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                WallJumpSensorMarker,
                TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
            ));
        });
    }
}

pub fn check_wall_jump_acquire(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut q_wall_jump_sensor: Query<(&mut Parent, Entity), With<WallJumpSensorMarker>>,
    mut query_player: Query<&mut PlayerInventory, With<PlayerMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
    mut q_wall_jump: Query<Entity>,
    mut checkpoint_event_writer: EventWriter<SetCheckpointEvent>,
) {
    let Ok(mut inventory) = query_player.get_single_mut() else {
        return;
    };
    let Ok(player_collider) = query_player_collider.get_single() else {
        return;
    };

    for (wall_jump, dj_sensor_entity) in q_wall_jump_sensor.iter_mut() {
        let dj_entity = q_wall_jump.get_mut(wall_jump.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, dj_sensor_entity) == Some(true) {
            inventory.has_wall_jump = true;
            println!("PLAYER ENABLED WALL JUMPS");
            commands.entity(dj_entity).despawn_recursive();
            checkpoint_event_writer.send(SetCheckpointEvent);
        }
    }
}
