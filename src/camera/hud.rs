use bevy::{
    prelude::*,
    sprite::Anchor,
    text::{BreakLineOn, Text2dBounds},
};

use crate::{
    camera::HUD_RENDER_LAYER,
    player::{PlayerInventory, PlayerMarker},
};

use super::{HudCameraMarker, CAMERA_UNIT_WIDTH};

#[derive(Component)]
pub struct HudIconMarker;

#[derive(Debug, Copy, Clone)]
pub enum HudIcon {
    None,
    Key,
    DoubleJump,
    WallJump,
    JumpToken,
    BgLeft,
    BgCenter,
    BgRight,
}

#[derive(Component)]
pub struct HudIconInfo {
    pub icon: HudIcon,
    pub index: usize,
    pub background: bool,
}

#[derive(Component)]
pub struct HudTextMarker;

/// Padding from the top left corner
const HUD_PADDING: Vec2 = Vec2::new(30., -30.);
const MAX_HUD_ICONS: usize = 15;

#[derive(Event)]
pub struct OpenTextBoxEvent {
    pub text: String,
}

pub fn show_textbox(
    mut textbox_events: EventReader<OpenTextBoxEvent>,
    mut q_textbox: Query<(&mut Text, &mut Text2dBounds, &mut Transform), With<HudTextMarker>>,
    q_hud_camera: Query<(&Camera, &GlobalTransform), With<HudCameraMarker>>,
    asset_server: Res<AssetServer>,
) {
    let Ok((mut text, mut text_2d_bounds, mut transform)) = q_textbox.get_single_mut() else {
        return;
    };
    let Ok((camera, camera_global_transform)) = q_hud_camera.get_single() else {
        return;
    };

    let screen_tl = camera
        .viewport_to_world_2d(camera_global_transform, Vec2::new(0., 0.))
        .unwrap();

    let screen_br = camera
        .viewport_to_world_2d(
            camera_global_transform,
            camera.logical_viewport_size().unwrap(),
        )
        .unwrap();
    let unit_height = screen_tl.y - screen_br.y;
    let unit_width = screen_br.x - screen_tl.x;
    let pixel_scaling = unit_width / CAMERA_UNIT_WIDTH;

    *text_2d_bounds = Text2dBounds {
        size: Vec2::new(150. * pixel_scaling, 600. * pixel_scaling),
    };
    *transform = Transform::from_xyz(0., unit_height / 2. + HUD_PADDING.y * pixel_scaling, 0.);

    for event in textbox_events.read() {
        // println!("SET TEXT TO {}", &event.text);
        text.sections.clear();
        text.sections.push(TextSection {
            value: event.text.clone(),
            style: TextStyle {
                font: asset_server.load("Monocraft.ttf"),
                font_size: 18.,
                ..default()
            },
        })
    }
}

pub fn spawn_hud(
    mut commands: Commands,
    q_hud_camera: Query<(&Camera, &GlobalTransform, Entity), Added<HudCameraMarker>>,
    asset_server: Res<AssetServer>,
) {
    let Ok((camera, camera_global_transform, camera_entity)) = q_hud_camera.get_single() else {
        return;
    };

    let screen_tl = camera
        .viewport_to_world_2d(camera_global_transform, Vec2::new(0., 0.))
        .unwrap();

    let screen_br = camera
        .viewport_to_world_2d(
            camera_global_transform,
            camera.logical_viewport_size().unwrap(),
        )
        .unwrap();
    let unit_height = screen_tl.y - screen_br.y;
    let unit_width = screen_br.x - screen_tl.x;
    let pixel_scaling = unit_width / CAMERA_UNIT_WIDTH;

    commands.entity(camera_entity).with_children(|parent| {
        for i in 0..MAX_HUD_ICONS {
            let mut transform = Transform {
                translation: Vec3::new(
                    -unit_width / 2.
                        + HUD_PADDING.x * pixel_scaling
                        + i as f32 * 16. * pixel_scaling,
                    unit_height / 2. + HUD_PADDING.y * pixel_scaling,
                    0.,
                ),
                scale: Vec3::new(pixel_scaling, pixel_scaling, 0.),
                rotation: Quat::default(),
            };

            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("slime.png"),
                    visibility: Visibility::Hidden,
                    ..default()
                })
                .insert(TransformBundle::from_transform(transform))
                .insert(HudIconMarker)
                .insert(HudIconInfo {
                    icon: HudIcon::None,
                    index: i,
                    background: false,
                })
                .insert(HUD_RENDER_LAYER);
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("slime.png"),
                    visibility: Visibility::Hidden,
                    ..default()
                })
                .insert(TransformBundle::from_transform({
                    transform.translation.z = -1.;
                    transform
                }))
                .insert(HudIconMarker)
                .insert(HudIconInfo {
                    icon: HudIcon::None,
                    index: i,
                    background: true,
                })
                .insert(HUD_RENDER_LAYER);
        }
        parent
            .spawn(Text2dBundle {
                text: {
                    let mut text = Text::from_section(
                        "",
                        TextStyle {
                            font: asset_server.load("Monocraft.ttf"),
                            font_size: 18.,
                            ..default()
                        },
                    );
                    text.linebreak_behavior = BreakLineOn::WordBoundary;
                    text
                }
                .with_justify(JustifyText::Center),
                text_2d_bounds: Text2dBounds {
                    size: Vec2::new(150. * pixel_scaling, 600. * pixel_scaling),
                },
                text_anchor: Anchor::TopCenter,
                ..default()
            })
            .insert(HudTextMarker)
            .insert(HUD_RENDER_LAYER)
            .insert(TransformBundle::from_transform(Transform::from_xyz(
                0.,
                unit_height / 2. + HUD_PADDING.y,
                0.,
            )));
    });
}

pub fn update_hud(
    mut q_hud_icons: Query<
        (
            &mut Visibility,
            &mut HudIconInfo,
            &mut Transform,
            &mut Handle<Image>,
        ),
        With<HudIconMarker>,
    >,
    q_player: Query<&PlayerInventory, With<PlayerMarker>>,
    q_hud_camera: Query<(&Camera, &GlobalTransform), With<HudCameraMarker>>,
    asset_server: Res<AssetServer>,
) {
    let Ok(player_inventory) = q_player.get_single() else {
        return;
    };
    let Ok((camera, camera_global_transform)) = q_hud_camera.get_single() else {
        return;
    };

    let screen_tl = camera
        .viewport_to_world_2d(camera_global_transform, Vec2::new(0., 0.))
        .unwrap();

    let screen_br = camera
        .viewport_to_world_2d(
            camera_global_transform,
            camera.logical_viewport_size().unwrap(),
        )
        .unwrap();
    let pixel_scaling = (screen_br.x - screen_tl.x) / CAMERA_UNIT_WIDTH;
    let unit_width = screen_br.x - screen_tl.x;
    let unit_height = screen_tl.y - screen_br.y;

    let mut player_hud: Vec<HudIcon> = vec![HudIcon::None];
    let mut last_ind = 1usize;
    if player_inventory.has_wall_jump {
        player_hud.push(HudIcon::WallJump);
        last_ind += 1;
    }

    for _ in 0..player_inventory.max_extra_jumps {
        player_hud.push(HudIcon::DoubleJump);
        last_ind += 1;
    }

    for _ in 0..player_inventory.num_keys {
        player_hud.push(HudIcon::Key);
        last_ind += 1;
    }

    for _ in 0..player_inventory.air_jumps {
        player_hud.push(HudIcon::JumpToken);
        last_ind += 1;
    }

    if last_ind < 2 {
        last_ind = 2;
    }

    while player_hud.len() < MAX_HUD_ICONS {
        player_hud.push(HudIcon::None);
    }
    assert!(player_hud.len() == MAX_HUD_ICONS);

    for (mut icon_visibility, mut info, mut transform, mut sprite) in q_hud_icons.iter_mut() {
        info.icon = if info.background == false {
            player_hud[info.index]
        } else if info.index == 0 {
            HudIcon::BgLeft
        } else if info.index == last_ind {
            HudIcon::BgRight
        } else if info.index < last_ind {
            HudIcon::BgCenter
        } else {
            HudIcon::None
        };

        *icon_visibility = Visibility::Visible;

        transform.translation = Vec3::new(
            -unit_width / 2.
                + HUD_PADDING.x * pixel_scaling
                + info.index as f32 * 16. * pixel_scaling,
            unit_height / 2. + HUD_PADDING.y * pixel_scaling,
            if info.background == false { 0. } else { -1. },
        );
        transform.scale = Vec3::new(pixel_scaling, pixel_scaling, 0.);

        match info.icon {
            HudIcon::None => *icon_visibility = Visibility::Hidden,
            HudIcon::Key => *sprite = asset_server.load("key_icon.png"),
            HudIcon::JumpToken => *sprite = asset_server.load("jump_token_icon.png"),
            HudIcon::WallJump => *sprite = asset_server.load("wall_jump_icon.png"),
            HudIcon::DoubleJump => *sprite = asset_server.load("double_jump_icon.png"),
            HudIcon::BgLeft => *sprite = asset_server.load("inventoryleft.png"),
            HudIcon::BgCenter => *sprite = asset_server.load("inventorymiddle.png"),
            HudIcon::BgRight => *sprite = asset_server.load("inventoryright.png"),
        }
    }
}
