use crate::Player;
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraController>()
            .add_startup_system(setup_camera)
            .add_system(update_camera_target_position)
            .add_system(lerp_to_camera_position.after(update_camera_target_position))
            .add_system(rotate_camera);
    }
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct CameraController {
    pub z_distance: f32,
    pub y_distance: f32,
    pub angle: f32,
    pub easing: f32,
    pub target_position: Vec3,
    pub player_position: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        CameraController {
            z_distance: 30.0,
            y_distance: 10.0,
            angle: 0.0,
            easing: 4.0,
            target_position: Vec3::ZERO,
            player_position: Vec3::ZERO,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(CameraController::default());
}

fn update_camera_target_position(
    mut camera_query: Query<&mut CameraController>,
    player_query: Query<&Transform, With<Player>>,
) {
    let mut camera = camera_query.single_mut();
    let player_transform = player_query.single();

    let mut starting_transform = player_transform.clone();
    starting_transform.rotate_y(camera.angle.to_radians());
    let dir = starting_transform.forward().normalize();
    camera.target_position =
        starting_transform.translation + (dir * camera.z_distance) + (Vec3::Y * camera.y_distance);
    camera.player_position = player_transform.translation;
}

fn lerp_to_camera_position(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &CameraController)>,
) {
    for (mut transform, camera_controller) in &mut camera_query {
        let lerped_position = transform.translation.lerp(
            camera_controller.target_position,
            time.delta_seconds() * camera_controller.easing,
        );
        transform.translation = lerped_position;
        transform.look_at(camera_controller.player_position, Vec3::Y);
    }
}

fn rotate_camera(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut CameraController>,
) {
    let mut camera = camera_query.single_mut();

    if keyboard.pressed(KeyCode::Q) {
        camera.angle -= 45.0 * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::E) {
        camera.angle += 45.0 * time.delta_seconds();
    }

    if camera.angle > 360.0 {
        camera.angle -= 360.0;
    }

    if camera.angle < -360.0 {
        camera.angle += 360.0;
    }
}
