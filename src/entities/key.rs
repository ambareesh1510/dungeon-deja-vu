use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    player::{
        animation::AnimationTimer, PlayerColliderMarker, PlayerInventory, PlayerMarker,
        SetCheckpointEvent,
    },
    sound_effects::{SoundEffectEvent, SoundEffectType},
};

#[derive(Component, Debug)]
pub struct KeyMarker;

#[derive(Component, Debug)]
pub struct KeySensorMarker;

#[derive(Bundle, LdtkEntity)]
pub struct KeyBundle {
    #[sprite_sheet_bundle("../assets/spritesheets/key.png", 16, 16, 8, 1, 0, 0, 0)]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    key_marker: KeyMarker,
    animation_timer: AnimationTimer,
}

impl Default for KeyBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            key_marker: KeyMarker,
            animation_timer: AnimationTimer(Timer::new(
                Duration::from_millis(100),
                TimerMode::Repeating,
            )),
        }
    }
}

pub fn animate_key(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlas), With<KeyMarker>>,
) {
    let durations = [450, 250, 200, 250, 450, 250, 200, 250];

    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = (sprite.index + 1) % 8;
            timer.set_duration(Duration::from_millis(durations[sprite.index as usize]));
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
    mut query_player: Query<&mut PlayerInventory, With<PlayerMarker>>,
    query_player_collider: Query<Entity, With<PlayerColliderMarker>>,
    mut query_key_entity: Query<Entity>,
    mut checkpoint_event_writer: EventWriter<SetCheckpointEvent>,
    mut sound_effect_event_writer: EventWriter<SoundEffectEvent>,
) {
    let Ok(mut inventory) = query_player.get_single_mut() else {
        return;
    };
    let Ok(player_collider) = query_player_collider.get_single() else {
        return;
    };

    for (key, key_sensor_entity) in query_keys.iter_mut() {
        let key_entity = &mut query_key_entity.get_mut(key.get()).unwrap();
        if rapier_context.intersection_pair(player_collider, key_sensor_entity) == Some(true) {
            // println!("GOT KEY");
            sound_effect_event_writer.send(SoundEffectEvent(SoundEffectType::Key));
            inventory.num_keys += 1;
            commands.entity(*key_entity).despawn_recursive();
            checkpoint_event_writer.send(SetCheckpointEvent);
        }
    }
}
