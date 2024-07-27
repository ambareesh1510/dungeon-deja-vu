use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::level::{PlayerInventory, PlayerMarker, SetCheckpointEvent};

#[derive(Component, Debug)]
pub struct KeyMarker;

#[derive(Component, Debug)]
pub struct KeySensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct KeyBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    key_marker: KeyMarker,
}

impl Default for KeyBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            key_marker: KeyMarker,
        }
    }
}

pub fn add_key_sensor(mut commands: Commands, query_keys: Query<Entity, Added<KeyMarker>>) {
    for key in query_keys.iter() {
        commands.entity(key).with_children(|parent| {
            parent.spawn((
                Collider::cuboid(5., 5.),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                KeySensorMarker,
                TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
            ));
        });
    }
}

pub fn check_key_interacting(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut query_keys: Query<(&mut Parent, Entity), With<KeySensorMarker>>,
    mut query_player: Query<(&mut PlayerInventory, Entity), With<PlayerMarker>>,
    mut query_key_entity: Query<Entity>,
    mut checkpoint_event_writer: EventWriter<SetCheckpointEvent>,
) {
    let Ok((mut inventory, player_collider)) = query_player.get_single_mut()
    else {
        return;
    };

    for (key, key_sensor_entity) in query_keys.iter_mut() {
        let key_entity = &mut query_key_entity.get_mut(key.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, key_sensor_entity) == Some(true) {
            println!("GOT KEY");
            inventory.add_key();
            commands.entity(*key_entity).despawn_recursive();
            checkpoint_event_writer.send(SetCheckpointEvent);
        }
    }
}
