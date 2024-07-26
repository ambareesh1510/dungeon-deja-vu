use bevy::{prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::camera::PLAYER_RENDER_LAYER;
use crate::level::{PlayerInventory, PlayerMarker};

#[derive(Component, Debug)]
pub struct KeyMarker;

#[derive(Component, Debug)]
pub struct KeyInfo {
    id: usize,
}

#[derive(Component, Debug)]
pub struct KeySensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct KeyBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    render_layer: RenderLayers,
    key_marker: KeyMarker,
    #[with(key_initial_info)]
    key_info: KeyInfo,
}

impl Default for KeyBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            render_layer: PLAYER_RENDER_LAYER,
            key_marker: KeyMarker,
            key_info: KeyInfo { id: 0 },
        }
    }
}

fn key_initial_info(ei: &EntityInstance) -> KeyInfo {
    KeyInfo {
        id: *ei.get_int_field("key_id").unwrap() as usize,
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
    mut query_key_info: Query<(Entity, &mut KeyInfo)>,
) {
    let Ok((mut inventory, player_collider)) = query_player.get_single_mut() else {
        return;
    };

    for (key, key_sensor_entity) in query_keys.iter_mut() {
        let (key_entity, key_info) = &mut query_key_info.get_mut(key.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, key_sensor_entity) == Some(true) {
            println!("GOT KEY {}", key_info.id);
            inventory.add_key(key_info.id);
            commands.entity(*key_entity).despawn_descendants();
        }
    }

    // for door in query_doors.iter() {
    //     println!("{:#?}", door);
    // }
}
