use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::{PlayerColliderMarker, PlayerMarker, PlayerStatus};

#[derive(Component, Debug)]
pub struct GoalMarker;

#[derive(Component, Debug)]
pub struct GoalSensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct GoalBundle {
    #[sprite_sheet_bundle(no_grid)]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    goal_marker: GoalMarker,
}

impl Default for GoalBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            goal_marker: GoalMarker,
        }
    }
}

pub fn add_goal_sensor(mut commands: Commands, query_goals: Query<Entity, Added<GoalMarker>>) {
    for goal in query_goals.iter() {
        commands.entity(goal).with_children(|parent| {
            parent.spawn((
                Collider::cuboid(24., 16.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
                GoalSensorMarker,
            ));
        });
    }
}

pub fn check_goal_reached(
    rapier_context: Res<RapierContext>,
    query_goal_sensor: Query<Entity, With<GoalSensorMarker>>,
    mut query_player: Query<&mut PlayerStatus, With<PlayerMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
) {
    let Ok(mut player_status) = query_player.get_single_mut() else {
        return;
    };
    let Ok(player_collider) = query_player_collider.get_single() else {
        return;
    };
    let Ok(goal_sensor) = query_goal_sensor.get_single() else {
        return;
    };

    if rapier_context.intersection_pair(player_collider, goal_sensor) == Some(true) {
        player_status.level_finished = true;
    }
}
