use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::{PlayerColliderMarker, PlayerInventory, PlayerMarker, SetCheckpointEvent};

#[derive(Component, Debug)]
pub struct DoubleJumpMarker;

#[derive(Component, Debug)]
pub struct DoubleJumpSensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct DoubleJumpBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    double_jump_marker: DoubleJumpMarker,
}

impl Default for DoubleJumpBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            double_jump_marker: DoubleJumpMarker,
        }
    }
}

pub fn add_double_jump_sensor(
    mut commands: Commands,
    query_jump_token: Query<Entity, Added<DoubleJumpMarker>>,
) {
    for token in query_jump_token.iter() {
        commands.entity(token).with_children(|parent| {
            parent.spawn((
                Collider::cuboid(7., 7.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                DoubleJumpSensorMarker,
                TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
            ));
        });
    }
}

pub fn check_double_jump_acquire(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut q_double_jump_sensor: Query<(&mut Parent, Entity), With<DoubleJumpSensorMarker>>,
    mut query_player: Query<&mut PlayerInventory, With<PlayerMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
    mut q_double_jump: Query<Entity>,
    mut checkpoint_event_writer: EventWriter<SetCheckpointEvent>,
) {
    let Ok(mut inventory) = query_player.get_single_mut() else {
        return;
    };
    let Ok(player_collider) = query_player_collider.get_single() else {
        return;
    };

    for (double_jump, dj_sensor_entity) in q_double_jump_sensor.iter_mut() {
        let dj_entity = q_double_jump.get_mut(double_jump.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, dj_sensor_entity) == Some(true) {
            inventory.max_extra_jumps += 1;
            println!("PLAYER EXTRA JUMPS {:?}", inventory.max_extra_jumps);
            commands.entity(dj_entity).despawn_recursive();
            checkpoint_event_writer.send(SetCheckpointEvent);
        }
    }
}
