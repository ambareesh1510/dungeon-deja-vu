use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use std::time::Duration;

#[derive(Component, Deref, DerefMut)]
pub struct ClockAnimationTimer(pub Timer);

#[derive(Component)]
pub struct ClockMarker;

#[derive(Bundle, LdtkEntity)]
pub struct ClockBundle {
    marker: ClockMarker,
    #[sprite_sheet_bundle("../assets/spritesheets/clock2.png", 32, 48, 8, 1, 0, 0, 0)]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    animation_timer: ClockAnimationTimer,
}
impl Default for ClockBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            marker: ClockMarker,
            animation_timer: ClockAnimationTimer(Timer::new(
                Duration::from_millis(5000),
                TimerMode::Repeating,
            )),
        }
    }
}

pub fn animate_clock(
    time: Res<Time>,
    mut query: Query<(&mut ClockAnimationTimer, &mut TextureAtlas), With<ClockMarker>>,
) {
    for (mut timer, mut atlas) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.0.finished() {
            atlas.index = (atlas.index + 1) % 8;
        }
    }
}
