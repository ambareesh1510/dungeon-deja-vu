use bevy::prelude::*;

pub struct CameraManagementPlugin;

impl Plugin for CameraManagementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_camera)
            .add_systems(Update, control_camera);
    }
}

#[derive(Component)]
struct CameraMarker;

fn setup_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.5;
    // camera.transform.translation.x += 1280.0 / 4.0;
    // camera.transform.translation.y += 720.0 / 4.0;
    commands.spawn((camera, CameraMarker));
}

fn control_camera(mut query_camera: Query<(&mut Transform, &mut OrthographicProjection), With<CameraMarker>>, keys: Res<ButtonInput<KeyCode>>) {
    if let Ok((mut camera_transform, mut camera_projection)) = query_camera.get_single_mut() {
        if keys.pressed(KeyCode::KeyW) {
            camera_transform.translation.y += 1.;
        }
        if keys.pressed(KeyCode::KeyS) {
            camera_transform.translation.y -= 1.;
        }
        if keys.pressed(KeyCode::KeyA) {
            camera_transform.translation.x -= 1.;
        }
        if keys.pressed(KeyCode::KeyD) {
            camera_transform.translation.x += 1.;
        }
        if keys.pressed(KeyCode::Minus) {
            camera_projection.scale *= 1.1;
        }
        if keys.pressed(KeyCode::Equal) {
            camera_projection.scale /= 1.1;
        }
    }
}
