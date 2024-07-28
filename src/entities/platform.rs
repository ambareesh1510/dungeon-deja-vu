use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component, Debug)]
pub struct PlatformMarker;

#[derive(Component, Debug)]
pub struct PlatformInfo {
    pub id: usize,
    pub active: bool,
}

#[derive(Component, Debug)]
pub struct PlatformColliderMarker;

#[derive(Bundle, LdtkEntity)]
pub struct PlatformBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    platform_marker: PlatformMarker,
    #[with(door_initial_status)]
    platform_state: PlatformInfo,
}

impl Default for PlatformBundle {
    fn default() -> Self {
        Self {
            sprite_sheet_bundle: LdtkSpriteSheetBundle::default(),
            platform_marker: PlatformMarker,
            platform_state: PlatformInfo {
                id: 0,
                active: true,
            },
        }
    }
}

fn door_initial_status(ei: &EntityInstance) -> PlatformInfo {
    PlatformInfo {
        id: *ei.get_int_field("platform_id").unwrap() as usize,
        active: *ei.get_bool_field("init_state").unwrap(),
    }
}

pub fn insert_platform_colliders(
    mut commands: Commands,
    query_doors: Query<(&PlatformInfo, Entity), Added<PlatformMarker>>,
) {
    for (platform_info, platform) in query_doors.iter() {
        if platform_info.active {
            add_platform_colliders(&mut commands, platform)
        }
    }
}

/// Called from lever.rs as well
pub fn add_platform_colliders(commands: &mut Commands, platform: Entity) {
    commands.entity(platform).with_children(|parent| {
        parent.spawn((
            Collider::cuboid(8., 8.),
            ActiveEvents::COLLISION_EVENTS,
            TransformBundle::default(),
            PlatformColliderMarker,
        ));
    });
}
