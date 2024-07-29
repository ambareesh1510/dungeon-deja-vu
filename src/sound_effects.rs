use bevy::{prelude::*, utils::HashMap};

pub struct SoundEffectsManagementPlugin;

impl Plugin for SoundEffectsManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEffectEvent>()
            // .add_systems(Startup, create_sound_effect_handles)
            .add_systems(Update, play_sound_effect);
    }
}

#[derive(Component, PartialEq, Eq)]
pub enum SoundEffectType {
    Jump,
    SmallPowerup,
    BigPowerup,
    Door,
    Lever,
    Key,
    Death,
}

#[derive(Event)]
pub struct SoundEffectEvent(pub SoundEffectType);

// #[derive(Resource)]
// struct SoundEffectHandles([Entity; 5]);

fn create_sound_effect_handles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sound_effect_event_writer: EventWriter<SoundEffectEvent>,
) {
    // sound_effect_event_writer.send(SoundEffectEvent(SoundEffectType::Jump));
    // return;
    commands.spawn(AudioBundle {
        source: asset_server.load("sound_effects/jump.wav"),
        ..default()
    }).insert(SoundEffectType::Jump);
    commands.spawn(AudioBundle {
        source: asset_server.load("sound_effects/small_powerup.wav"),
        ..default()
    }).insert(SoundEffectType::SmallPowerup);
    commands.spawn(AudioBundle {
        source: asset_server.load("sound_effects/big_powerup.wav"),
        ..default()
    }).insert(SoundEffectType::BigPowerup);
    commands.spawn(AudioBundle {
        source: asset_server.load("sound_effects/door_open.wav"),
        ..default()
    }).insert(SoundEffectType::Door);
    commands.spawn(AudioBundle {
        source: asset_server.load("sound_effects/lever_toggle.wav"),
        ..default()
    }).insert(SoundEffectType::Lever);
}

const SOUND_EFFECT_MAP: [(SoundEffectType, &str); 7] = [
    (SoundEffectType::Jump, "sound_effects/jump.wav"),
    (SoundEffectType::SmallPowerup, "sound_effects/small_powerup.wav"),
    (SoundEffectType::BigPowerup, "sound_effects/big_powerup.wav"),
    (SoundEffectType::Door, "sound_effects/door_open.wav"),
    (SoundEffectType::Lever, "sound_effects/lever_toggle.wav"),
    (SoundEffectType::Key, "sound_effects/key.wav"),
    (SoundEffectType::Death, "sound_effects/death.wav"),
];

fn play_sound_effect(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut event_reader: EventReader<SoundEffectEvent>,
) {
    for SoundEffectEvent(sound_effect) in event_reader.read() {
        for (sound_effect_type, path) in SOUND_EFFECT_MAP {
            if *sound_effect == sound_effect_type {
                commands.spawn(AudioBundle {
                    source: asset_server.load(path),
                    ..default()
                });
            }
        }
    }
}
