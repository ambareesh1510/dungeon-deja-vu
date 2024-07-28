use bevy::prelude::*;

use crate::{
    camera::HUD_RENDER_LAYER,
    player::{PlayerInventory, PlayerMarker, PlayerStatus},
};

use super::HudCameraMarker;

#[derive(Component)]
pub struct HudIconMarker;

#[derive(Debug, Copy, Clone)]
pub enum HudIcon {
    None,
    Key,
    DoubleJump,
    WallJump,
    JumpToken,
}

#[derive(Component)]
pub struct HudIconInfo {
    pub icon: HudIcon,
    pub index: usize,
}

/// Padding from the top left corner
const HUD_PADDING: Vec2 = Vec2::new(10., -10.);
const MAX_HUD_ICONS: usize = 15;

#[derive(Event)]
pub struct OpenTextBoxEvent {
    text: str,
}

pub fn spawn_hud(
    mut commands: Commands,
    q_hud_camera: Query<(&Camera, &GlobalTransform, Entity), Added<HudCameraMarker>>,
) {
    let Ok((camera, camera_global_transform, camera_entity)) = q_hud_camera.get_single() else {
        return;
    };

    let screen_tl = camera
        .viewport_to_world_2d(camera_global_transform, Vec2::new(0., 0.))
        .unwrap();

    commands.entity(camera_entity).with_children(|parent| {
        for i in 0..MAX_HUD_ICONS {
            parent
                .spawn(SpriteBundle {
                    // texture: asset_server.load("slime.png"),
                    visibility: Visibility::Hidden,
                    ..default()
                })
                .insert(TransformBundle::from_transform(Transform::from_xyz(
                    screen_tl.x + HUD_PADDING.x + i as f32 * 16.,
                    screen_tl.y + HUD_PADDING.y,
                    0.,
                )))
                .insert(HudIconMarker)
                .insert(HudIconInfo {
                    icon: HudIcon::None,
                    index: i,
                })
                .insert(HUD_RENDER_LAYER);
        }
    });
}

pub fn update_hud(
    mut q_hud_icons: Query<
        (&mut Visibility, &mut HudIconInfo, &mut Handle<Image>),
        With<HudIconMarker>,
    >,
    q_player: Query<&PlayerInventory, With<PlayerMarker>>,
    asset_server: Res<AssetServer>,
) {
    let Ok(player_inventory) = q_player.get_single() else {
        return;
    };

    let mut player_hud: Vec<HudIcon> = vec![];
    if player_inventory.has_wall_jump {
        player_hud.push(HudIcon::WallJump);
    }

    for _ in 0..player_inventory.max_extra_jumps {
        player_hud.push(HudIcon::DoubleJump);
    }

    for _ in 0..player_inventory.air_jumps {
        player_hud.push(HudIcon::JumpToken);
    }

    for _ in 0..player_inventory.num_keys {
        player_hud.push(HudIcon::Key);
    }

    while player_hud.len() < MAX_HUD_ICONS {
        player_hud.push(HudIcon::None);
    }
    assert!(player_hud.len() == MAX_HUD_ICONS);
    // dbg!(&player_hud);

    for (mut icon_visibility, mut info, mut sprite) in q_hud_icons.iter_mut() {
        info.icon = player_hud[info.index];
        *icon_visibility = Visibility::Visible;
        match info.icon {
            HudIcon::None => *icon_visibility = Visibility::Hidden,
            HudIcon::Key => *sprite = asset_server.load("key_icon.png"),
            HudIcon::JumpToken => *sprite = asset_server.load("jump_token_icon.png"),
            HudIcon::WallJump => *sprite = asset_server.load("wall_jump_icon.png"),
            HudIcon::DoubleJump => *sprite = asset_server.load("double_jump_icon.png"),
        }
    }
}
