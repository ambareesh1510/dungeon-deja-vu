use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    entities::platform::add_platform_colliders,
    player::{animation::AnimationTimer, PlayerColliderMarker, SetCheckpointEvent},
};

use super::platform::{PlatformInfo, PlatformMarker};

#[derive(Component, Debug)]
pub struct LeverMarker;

#[derive(Component, Debug)]
pub struct LeverState {
    id: usize,
    activated: bool,
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum LeverAnimationState {
    Idle,
    LeftToRight,
    RightToLeft,
    
} 

#[derive(Component, Debug)]
pub struct LeverSensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct LeverBundle {
    #[sprite_sheet_bundle("../assets/spritesheets/lever.png", 32, 16, 5, 1, 0, 0, 0)]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    lever_marker: LeverMarker,
    #[with(lever_initial_state)]
    lever_state: LeverState,
    animation_timer: AnimationTimer,
    animation_state: LeverAnimationState,
}

impl Default for LeverBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            lever_marker: LeverMarker,
            lever_state: LeverState {
                id: 0,
                activated: false,
            },
            animation_timer: AnimationTimer(Timer::new(
                Duration::from_millis(100),
                TimerMode::Repeating,
            )),
            animation_state: LeverAnimationState::Idle,
        }

    }
}


fn lever_initial_state(ei: &EntityInstance) -> LeverState {
    LeverState {
        id: *ei.get_int_field("lever_id").unwrap() as usize,
        activated: *ei.get_bool_field("activated").unwrap(),
    }
}

pub fn add_lever_interaction(
    mut commands: Commands,
    query_levers: Query<Entity, Added<LeverMarker>>,
) {
    for lever in query_levers.iter() {
        commands.entity(lever).with_children(|parent| {
            parent.spawn((
                Collider::cuboid(8., 8.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
                LeverSensorMarker,
            ));
        });
    }
}

pub fn animate_lever(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut LeverAnimationState, &mut TextureAtlas), With<LeverMarker>>,
) {
    for (mut timer, mut state, mut atlas) in query.iter_mut() {
        timer.tick(time.delta());
        
        if timer.finished() {
            
            match *state {
                LeverAnimationState::LeftToRight => {
                    if atlas.index == 4 {
                        *state = LeverAnimationState::Idle;
                    } else {
                        atlas.index += 1;
                    }
                    timer.set_duration(Duration::from_millis(100));
                }
                LeverAnimationState::RightToLeft => {
                    if atlas.index == 0 {
                        *state = LeverAnimationState::Idle;
                    } else {
                        atlas.index -= 1;
                    }
                    timer.set_duration(Duration::from_millis(100));
                }
                _ => {}
            }
        }
    }
}

pub fn check_lever_interacting(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut query_lever_sensor: Query<(&mut Parent, Entity), With<LeverSensorMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
    mut query_lever: Query<(&mut LeverState, &mut LeverAnimationState)>,
    mut query_platforms: Query<(&PlatformInfo, &mut TextureAtlas, Entity), With<PlatformMarker>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut checkpoint_event_writer: EventWriter<SetCheckpointEvent>,
) {
    let Ok(player_collider) = query_player_collider.get_single() else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyQ) {
        return;
    }

    for (lever, lever_sensor) in query_lever_sensor.iter_mut() {
        let (mut lever_state, mut animation_state) = query_lever.get_mut(lever.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, lever_sensor) != Some(true) {
            continue;
        }

        
        println!("SWITCHING LEVER {}", lever_state.id);
        if lever_state.activated {
            *animation_state = LeverAnimationState::LeftToRight;
        } else {
            *animation_state = LeverAnimationState::RightToLeft;
        }
        lever_state.activated = !lever_state.activated;

        for (platform_info, mut atlas, platform) in query_platforms.iter_mut() {
            if platform_info.id == lever_state.id {
                if lever_state.activated {
                    add_platform_colliders(&mut commands, platform);
                    atlas.index = 0;
                } else {
                    commands.entity(platform).despawn_descendants();
                    atlas.index = 1

                }
            }
        }
        checkpoint_event_writer.send(SetCheckpointEvent);
    }

    // for lever in query_levers.iter() {
    //     println!("{:#?}", lever);
    // }
}
