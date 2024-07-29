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
}

#[derive(Component)]
pub struct HudIconInfo {
    pub icon: HudIcon,
    pub index: usize,
}

#[derive(Component)]
pub struct HudTextMarker;

/// Padding from the top left corner
const HUD_PADDING: Vec2 = Vec2::new(60., -60.);
const MAX_HUD_ICONS: usize = 15;

#[derive(Event)]
pub struct OpenTextBoxEvent {
    pub text: String,
}

pub fn show_textbox(
    mut textbox_events: EventReader<OpenTextBoxEvent>,
    mut q_textbox: Query<&mut Text, With<HudTextMarker>>,
    asset_server: Res<AssetServer>,
) {
    let Ok(mut text) = q_textbox.get_single_mut() else {
        return;
    };
    for event in textbox_events.read() {
        println!("SET TEXT TO {}", &event.text);
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
    let pixel_scaling = (screen_br.x - screen_tl.x) / CAMERA_UNIT_WIDTH;
    let unit_height = screen_tl.y - screen_br.y;

    commands.entity(camera_entity).with_children(|parent| {
        for i in 0..MAX_HUD_ICONS {
            let transform = Transform {
                translation: Vec3::new(
                    screen_tl.x + HUD_PADDING.x + i as f32 * 16. * pixel_scaling,
                    screen_tl.y + HUD_PADDING.y,
                    0.,
                ),
                scale: Vec3::new(pixel_scaling, pixel_scaling, 0.),
                rotation: Quat::default(),
            };

            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("slime.png"),
                    // visibility: Visibility::Hidden,
                    ..default()
                })
                .insert(TransformBundle::from_transform(transform))
                .insert(HudIconMarker)
                .insert(HudIconInfo {
                    icon: HudIcon::None,
                    index: i,
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
