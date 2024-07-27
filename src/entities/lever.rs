use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    entities::platform::add_platform_colliders,
    player::{PlayerColliderMarker, SetCheckpointEvent},
};

use super::platform::{PlatformInfo, PlatformMarker};

#[derive(Component, Debug)]
pub struct LeverMarker;

#[derive(Component, Debug)]
pub struct LeverState {
    id: usize,
    activated: bool,
}

#[derive(Component, Debug)]
pub struct LeverSensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct LeverBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    lever_marker: LeverMarker,
    #[with(lever_initial_state)]
    lever_state: LeverState,
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

pub fn check_lever_interacting(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut query_lever_sensor: Query<(&mut Parent, Entity), With<LeverSensorMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
    mut query_lever: Query<&mut LeverState>,
    query_platforms: Query<(&PlatformInfo, Entity), With<PlatformMarker>>,
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
        let lever_state = &mut query_lever.get_mut(lever.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, lever_sensor) != Some(true) {
            continue;
        }

        println!("SWITCHING LEVER {}", lever_state.id);
        lever_state.activated = !lever_state.activated;
        for (platform_info, platform) in query_platforms.iter() {
            if platform_info.id == lever_state.id {
                if lever_state.activated {
                    add_platform_colliders(&mut commands, platform);
                } else {
                    commands.entity(platform).despawn_descendants();
                }
            }
        }
        checkpoint_event_writer.send(SetCheckpointEvent);
    }

    // for lever in query_levers.iter() {
    //     println!("{:#?}", lever);
    // }
}
