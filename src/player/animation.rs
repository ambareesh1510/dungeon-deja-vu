use bevy::prelude::*;
use std::time::Duration;

use crate::player::{PlayerMarker, PlayerState};

#[derive(Resource)]
pub struct AnimationInfo {
    moving_start: usize,
    moving_end: usize,
    jumping_start: usize,
    jumping_end: usize,
    falling_start: usize,
    falling_end: usize,
    falling_to_idle_start: usize,
    falling_to_idle_end: usize,

    moving_durations: Vec<u64>,
    jumping_durations: Vec<u64>,
    falling_durations: Vec<u64>,
    falling_to_idle_durations: Vec<u64>,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

impl Default for AnimationInfo {
    fn default() -> Self {
        Self {
            moving_start: 10,
            moving_end: 13,
            jumping_start: 0,
            jumping_end: 2,
            falling_start: 2,
            falling_end: 4,
            falling_to_idle_start: 6,
            falling_to_idle_end: 10,

            moving_durations: vec![100, 100, 100, 100],
            jumping_durations: vec![100, 100, 100],
            falling_durations: vec![100, 100, 100],
            falling_to_idle_durations: vec![50, 50, 50, 50, 50],
        }
    }
}

pub fn animate_player(
    time: Res<Time>,
    animation_info: Res<AnimationInfo>,
    mut query: Query<
        (&mut TextureAtlas, &mut PlayerState, &mut AnimationTimer),
        With<PlayerMarker>,
    >,
) {
    if let Ok((mut atlas, mut state, mut timer)) = query.get_single_mut() {
        timer.tick(time.delta());
        // println!("state: {:?}", *state);
        if timer.finished() {
            match *state {
                PlayerState::Idle => {
                    // no idle animation as of now

                    atlas.index = 0;
                }
                PlayerState::MovingLeft => {
                    if atlas.index < animation_info.moving_start
                        || atlas.index > animation_info.moving_end
                    {
                        atlas.index = animation_info.moving_start;
                    } else {
                        atlas.index = if atlas.index == animation_info.moving_end {
                            animation_info.moving_start + 2
                        } else {
                            atlas.index + 1
                        };
                    }

                    timer.set_duration(Duration::from_millis(
                        animation_info.moving_durations[atlas.index - animation_info.moving_start],
                    ));
                }
                PlayerState::MovingRight => {
                    if atlas.index < animation_info.moving_start
                        || atlas.index > animation_info.moving_end
                    {
                        atlas.index = animation_info.moving_start;
                    } else {
                        atlas.index = if atlas.index == animation_info.moving_end {
                            animation_info.moving_start + 2
                        } else {
                            atlas.index + 1
                        };
                    }

                    timer.set_duration(Duration::from_millis(
                        animation_info.moving_durations[atlas.index - animation_info.moving_start],
                    ));
                }
                PlayerState::Jumping => {
                    if atlas.index < animation_info.jumping_start
                        || atlas.index > animation_info.jumping_end
                    {
                        atlas.index = animation_info.jumping_start;
                    } else {
                        atlas.index = if atlas.index == animation_info.jumping_end {
                            atlas.index
                        } else {
                            atlas.index + 1
                        };
                    }

                    timer.set_duration(Duration::from_millis(
                        animation_info.jumping_durations
                            [atlas.index - animation_info.jumping_start],
                    ));
                }
                PlayerState::Falling => {
                    if atlas.index < animation_info.falling_start
                        || atlas.index > animation_info.falling_end
                    {
                        atlas.index = animation_info.falling_start;
                    } else {
                        atlas.index = if atlas.index == animation_info.falling_end {
                            atlas.index
                        } else {
                            atlas.index + 1
                        };
                    }

                    timer.set_duration(Duration::from_millis(
                        animation_info.falling_durations
                            [atlas.index - animation_info.falling_start],
                    ));
                }
                PlayerState::MovingToIdle => {
                    atlas.index = animation_info.moving_start + 1;
                    timer.set_duration(Duration::from_millis(50));

                    *state = PlayerState::Idle;
                }
                PlayerState::FallingToIdle => {
                    if atlas.index < animation_info.falling_to_idle_start
                        || atlas.index > animation_info.falling_to_idle_end
                    {
                        atlas.index = animation_info.falling_to_idle_start;
                    }
                    atlas.index = if atlas.index == animation_info.falling_to_idle_end {
                        *state = PlayerState::Idle;
                        atlas.index
                    } else {
                        atlas.index + 1
                    };

                    timer.set_duration(Duration::from_millis(
                        animation_info.falling_to_idle_durations
                            [atlas.index - animation_info.falling_to_idle_start],
                    ));
                }
            }
        }
    }
}
