use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::{PlayerColliderMarker, PlayerInventory, PlayerMarker};

#[derive(Component, Debug)]
pub struct JumpTokenMarker;

#[derive(Component, Debug)]
pub struct JumpTokenSensorMarker;

#[derive(Component, Debug)]
pub struct JumpTokenStatus {
    pub active: bool,
    pub timer: Timer,
}

#[derive(Bundle, LdtkEntity)]
pub struct JumpTokenBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    token_marker: JumpTokenMarker,
    token_status: JumpTokenStatus,
}

impl Default for JumpTokenBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            token_marker: JumpTokenMarker,
            token_status: JumpTokenStatus {
                timer: Timer::from_seconds(5., TimerMode::Once),
                active: true,
            },
        }
    }
}

pub fn add_jump_token_sensor(
    mut commands: Commands,
    query_jump_token: Query<Entity, Added<JumpTokenMarker>>,
) {
    for token in query_jump_token.iter() {
        commands.entity(token).with_children(|parent| {
            parent.spawn((
                Collider::cuboid(7., 7.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                JumpTokenSensorMarker,
                TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
            ));
        });
    }
}

pub fn check_jump_token_acquire(
    rapier_context: Res<RapierContext>,
    mut query_token_sensor: Query<(&mut Parent, Entity), With<JumpTokenSensorMarker>>,
    mut query_player: Query<&mut PlayerInventory, With<PlayerMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
    mut query_token: Query<(&mut Visibility, &mut JumpTokenStatus)>,
    time: Res<Time>,
) {
    let Ok(mut inventory) = query_player.get_single_mut() else {
        return;
    };
    let Ok(player_collider) = query_player_collider.get_single() else {
        return;
    };

    for (token, token_sensor_entity) in query_token_sensor.iter_mut() {
        let (mut token_visibility, mut token_status) = query_token.get_mut(token.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, token_sensor_entity) == Some(true)
            && token_status.active
        {
            inventory.air_jumps += 1;
            // println!("ADDED JUMP");
            token_status.active = false;
            *token_visibility = Visibility::Hidden;
        } else if !token_status.active {
            token_status.timer.tick(time.delta());
            if token_status.timer.just_finished() {
                // println!("TOKEN RESETTING");
                token_status.timer.reset();
                token_status.active = true;
                *token_visibility = Visibility::Inherited;
            }
        }
    }
}
