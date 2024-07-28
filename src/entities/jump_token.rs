use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

use crate::player::{animation::AnimationTimer, PlayerColliderMarker, PlayerInventory, PlayerMarker};

#[derive(Component, Debug)]
pub struct JumpTokenMarker;

#[derive(Component, Debug)]
pub struct JumpTokenSensorMarker;

#[derive(Component, Debug)]
pub struct JumpTokenStatus {
    active: bool,
    timer: Timer,
}

#[derive(Bundle, LdtkEntity)]
pub struct JumpTokenBundle {
    #[sprite_sheet_bundle("../assets/spritesheets/walljump.png", 16, 16, 4, 1, 0, 0, 0)]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    token_marker: JumpTokenMarker,
    token_status: JumpTokenStatus,
    animation_timer: AnimationTimer,
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
            animation_timer: AnimationTimer(Timer::new(
                Duration::from_millis(300),
                TimerMode::Repeating,
            )),
        }
    }
}

pub fn attach_timer(mut commands: Commands, query: Query<Entity, Added<JumpTokenMarker>>) {
    let mut rng = rand::thread_rng();
    for entity in query.iter() {
        commands.entity(entity).insert(AnimationTimer(Timer::new(
            Duration::from_millis(rng.gen_range(200..400)),
            TimerMode::Repeating,
        )));
    }
}

pub fn animate_jump_token(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlas), With<JumpTokenMarker>>,
) {
    for (mut timer, mut atlas) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.0.finished() {
            atlas.index = (atlas.index + 1) % 4;
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
