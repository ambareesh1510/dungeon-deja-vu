use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{camera::hud::OpenTextBoxEvent, player::PlayerColliderMarker};

#[derive(Component, Debug)]
pub struct SignMarker;

#[derive(Component, Debug)]
pub struct SignState {
    text: String,
    reading: bool,
}

#[derive(Component, Debug)]
pub struct SignSensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct SignBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    sign_marker: SignMarker,
    #[with(sign_initial_state)]
    sign_state: SignState,
}

impl Default for SignBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            sign_marker: SignMarker,
            sign_state: SignState {
                text: "".to_string(),
                reading: false,
            },
        }
    }
}

fn sign_initial_state(ei: &EntityInstance) -> SignState {
    SignState {
        text: ei.get_string_field("text").unwrap().clone(),
        reading: false,
    }
}

pub fn add_sign_interaction(
    mut commands: Commands,
    mut query_signs: Query<Entity, Added<SignMarker>>,
) {
    for sign in query_signs.iter_mut() {
        commands.entity(sign).with_children(|parent| {
            parent.spawn((
                Collider::cuboid(24., 24.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
                SignSensorMarker,
            ));
        });
    }
}

pub fn check_sign_interacting(
    rapier_context: Res<RapierContext>,
    mut query_sign_sensor: Query<(&mut Parent, Entity), With<SignSensorMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
    mut query_sign: Query<&mut SignState>,
    mut textbox_event_writer: EventWriter<OpenTextBoxEvent>,
) {
    let Ok(player_collider) = query_player_collider.get_single() else {
        return;
    };

    for (sign, sign_sensor) in query_sign_sensor.iter_mut() {
        let mut sign_state = query_sign.get_mut(sign.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, sign_sensor) == Some(true) {
            if !sign_state.reading {
                sign_state.reading = true;
                textbox_event_writer.send(OpenTextBoxEvent {
                    text: sign_state.text.clone(),
                });
            }
        } else if sign_state.reading {
            sign_state.reading = false;
            textbox_event_writer.send(OpenTextBoxEvent {
                text: "".to_string(),
            });
        }
    }

    // for sign in query_signs.iter() {
    //     println!("{:#?}", sign);
    // }
}
