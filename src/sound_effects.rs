use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

pub struct SoundEffectsManagementPlugin;

impl Plugin for SoundEffectsManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoundEffectEvent>()
            .insert_resource(AudioMuted(false))
            .add_systems(Startup, start_music)
            .add_systems(
                Update,
                (
                    play_sound_effect,
                    update_muted,
                    delete_finished_audio_bundles,
                ),
            );
    }
}

#[derive(Resource)]
struct AudioMuted(bool);

#[derive(Component, PartialEq, Eq)]
pub enum SoundEffectType {
    Jump,
    SmallPowerup,
    BigPowerup,
    Door,
    Lever,
    Key,
    Death,
    WaterDeath,
}

#[derive(Component)]
struct BackgroundMusicMarker;

#[derive(Event)]
pub struct SoundEffectEvent(pub SoundEffectType);

const SOUND_EFFECT_MAP: [(SoundEffectType, &str); 8] = [
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
    (SoundEffectType::WaterDeath, "sound_effects/water.wav"),
];

fn play_sound_effect(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut event_reader: EventReader<SoundEffectEvent>,
    muted: Res<AudioMuted>,
) {
    if muted.0 {
        return;
    }
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
    commands
        .spawn(AudioBundle {
            source: asset_server.load("music/far_from_shore.wav"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(0.5),
                ..default()
            },
        })
        .insert(BackgroundMusicMarker);
}

fn update_muted(
    mut muted: ResMut<AudioMuted>,
    keys: Res<ButtonInput<KeyCode>>,
    query_bgm: Query<&AudioSink, With<BackgroundMusicMarker>>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        muted.0 = !muted.0;
        let Ok(bgm) = query_bgm.get_single() else {
            return;
        };
        if muted.0 {
            bgm.set_volume(0.);
        } else {
            bgm.set_volume(0.5);
        }
    }
}

fn delete_finished_audio_bundles(
    mut commands: Commands,
    query_sfx: Query<(Entity, &AudioSink), Without<BackgroundMusicMarker>>,
) {
    for (e, audio_sink) in query_sfx.iter() {
        if audio_sink.empty() {
            commands.entity(e).despawn_recursive();
        }
    }
}
