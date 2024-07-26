use bevy::{prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::camera::PLAYER_RENDER_LAYER;
use crate::level::{PlayerInventory, PlayerMarker};

#[derive(Component, Debug)]
pub struct DoorMarker;

#[derive(Component, Debug)]
pub struct DoorState {
    id: usize,
    unlocked: bool,
}


#[derive(Component, Debug)]
struct DoorColliderMarker;

#[derive(Component, Debug)]
pub struct DoorSensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct DoorBundle {
    #[sprite_sheet_bundle("../assets/spritesheets/door.png", 16, 32, 2, 1, 0, 0, 0)]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    render_layer: RenderLayers,
    door_marker: DoorMarker,
    #[with(door_initial_status)]
    door_state: DoorState,
    // rigid_body: RigidBody,
    // collider: Collider,
}

impl Default for DoorBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            render_layer: PLAYER_RENDER_LAYER,
            door_marker: DoorMarker,
            door_state: DoorState {
                id: 0,
                unlocked: false,
            },
            // rigid_body: RigidBody::Dynamic,
            // collider: Collider::cuboid(2., 16.),
            // collider: Collider::round_cuboid(5., 3., 2.),
        }
    }
}

fn door_initial_status(ei: &EntityInstance) -> DoorState {
    DoorState {
        id: *ei.get_int_field("door_id").unwrap() as usize,
        unlocked: false,
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
    query_player: Query<(&PlayerInventory, Entity), With<PlayerMarker>>,
    mut query_door_state: Query<(Entity, &mut DoorState)>,
    mut query_door_texture: Query<&mut TextureAtlas, With <DoorMarker>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok((inventory, player_collider)) = query_player.get_single() else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyQ) {
        return;
    }

    for (door, door_collider) in query_doors.iter_mut() {
        let (door_entity, door_state) = &mut query_door_state.get_mut(door.get()).unwrap();
        let atlas = &mut query_door_texture.get_mut(door.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, door_collider) == Some(true) {
            if inventory.has_key(door_state.id) {
                println!("UNLOCKING DOOR {}", door_state.id);
                door_state.unlocked = true;
                commands.entity(*door_entity).despawn_descendants();
                atlas.index = 1;
            } else {
                println!("NEED KEY FOR DOOR {}", door_state.id);
            }
        }
    }

    // for door in query_doors.iter() {
    //     println!("{:#?}", door);
    // }
}
