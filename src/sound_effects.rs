use bevy::{audio::PlaybackMode, prelude::*};

pub struct SoundEffectsManagementPlugin;

impl Plugin for SoundEffectsManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEffectEvent>()
            .add_systems(Startup, start_music)
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

const SOUND_EFFECT_MAP: [(SoundEffectType, &str); 7] = [
    (SoundEffectType::Jump, "sound_effects/jump.wav"),
    (
        SoundEffectType::SmallPowerup,
        "sound_effects/small_powerup.wav",
    ),
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

fn start_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(AudioBundle {
        source: asset_server.load("music/far_from_shore.wav"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..default()
        }
    });
}
