use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{player::{
    animation::AnimationTimer, PlayerColliderMarker, PlayerInventory, PlayerMarker,
    SetCheckpointEvent,
}, sound_effects::{SoundEffectEvent, SoundEffectType}};

use super::INTERACT_KEYCODE;

#[derive(Component, Debug)]
pub struct DoorMarker;

#[derive(Component, Debug)]
pub struct DoorState {
    unlocked: bool,
}

#[derive(Component, Debug)]
struct DoorColliderMarker;

#[derive(Component, Debug)]
pub struct DoorSensorMarker;

#[derive(Component, Debug, PartialEq, Eq)]
pub enum DoorAnimationState {
    Idle,
    Opening,
}

#[derive(Bundle, LdtkEntity)]
pub struct DoorBundle {
    #[sprite_sheet_bundle("../assets/spritesheets/dooranim.png", 16, 32, 5, 1, 0, 0, 0)]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    door_marker: DoorMarker,
    door_state: DoorState,
    animation_state: DoorAnimationState,
    animation_timer: AnimationTimer,
    // rigid_body: RigidBody,
    // collider: Collider,
}

impl Default for DoorBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            door_marker: DoorMarker,
            door_state: DoorState { unlocked: false },
            animation_state: DoorAnimationState::Idle,
            animation_timer: AnimationTimer(Timer::new(
                Duration::from_millis(50),
                TimerMode::Repeating,
            )),
            // rigid_body: RigidBody::Dynamic,
            // collider: Collider::cuboid(2., 16.),
            // collider: Collider::round_cuboid(5., 3., 2.),
        }
    }
}
pub fn animate_door(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut DoorAnimationState,
            &mut TextureAtlas,
        ),
        With<DoorMarker>,
    >,
) {
    for (mut timer, mut state, mut atlas) in query.iter_mut() {
        timer.tick(time.delta());
        // println!("atlas: {:?}", atlas.index);
        if timer.0.finished() {
            match *state {
                DoorAnimationState::Idle => {}
                DoorAnimationState::Opening => {
                    if atlas.index == 4 {
                        *state = DoorAnimationState::Idle;
                    } else {
                        atlas.index += 1
                    }
                    timer.set_duration(Duration::from_millis(50));
                }
            }
        }
    }
}
pub fn add_door_interaction(mut commands: Commands, query_doors: Query<Entity, Added<DoorMarker>>) {
    for door in query_doors.iter() {
        commands.entity(door).with_children(|parent| {
            parent.spawn((
                Collider::cuboid(16., 16.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                TransformBundle::from_transform(Transform::from_xyz(-4., 0., 0.)),
                DoorSensorMarker,
            ));
            parent.spawn((
                Collider::cuboid(2., 16.),
                ActiveEvents::COLLISION_EVENTS,
                TransformBundle::from_transform(Transform::from_xyz(-4., 0., 0.)),
                DoorColliderMarker,
            ));
        });
    }
}

pub fn check_door_interacting(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut query_doors: Query<(&mut Parent, Entity), With<DoorSensorMarker>>,
    mut query_player: Query<&mut PlayerInventory, With<PlayerMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
    mut query_door_state: Query<(Entity, &mut DoorAnimationState, &mut DoorState)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut checkpoint_event_writer: EventWriter<SetCheckpointEvent>,
    mut sound_effect_event_writer: EventWriter<SoundEffectEvent>,
) {
    let Ok(mut inventory) = query_player.get_single_mut() else {
        return;
    };
    let Ok(player_collider) = query_player_collider.get_single() else {
        return;
    };
    if !keys.just_pressed(INTERACT_KEYCODE) {
        return;
    }

    for (door, door_collider) in query_doors.iter_mut() {
        let (door_entity, mut animation_state, mut door_state) =
            query_door_state.get_mut(door.get()).unwrap();

        if rapier_context.intersection_pair(player_collider, door_collider) == Some(true) {
            if inventory.num_keys >= 1 {
                println!("UNLOCKING DOOR");
                sound_effect_event_writer.send(SoundEffectEvent(SoundEffectType::Door));
                door_state.unlocked = true;
                *animation_state = DoorAnimationState::Opening;
                commands.entity(door_entity).despawn_descendants();
                inventory.num_keys -= 1;
                // atlas.index = 1;
                checkpoint_event_writer.send(SetCheckpointEvent);
            } else {
                println!("NEED KEY FOR DOOR");
            }
        }
    }
}
