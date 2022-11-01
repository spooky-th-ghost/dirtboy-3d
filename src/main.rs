use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

mod camera;
use camera::*;

mod physics;
use physics::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(CameraPlugin)
        .add_plugin(PhysicsPlugin)
        .add_startup_system(setup_physics)
        .add_system(player_input)
        .run();
}

fn setup_physics(mut commands: Commands) {
    /* Create the ground */
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(RigidBody::Fixed)
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -4.0, 0.0)));

    commands
        .spawn()
        .insert(Collider::cuboid(10.0, 0.1, 20.0))
        .insert(RigidBody::Fixed)
        .insert_bundle(TransformBundle::from(
            Transform::from_xyz(5.0, -4.0, 0.0).with_rotation(Quat::from_rotation_z(0.3)),
        ));

    commands
        .spawn()
        .insert(Collider::cuboid(1.0, 1.0, 1.0))
        .insert(RigidBody::Dynamic)
        .insert(Velocity::default())
        .insert(ExternalForce::default())
        .insert(Hover::default())
        .insert(Deceleration(1.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(6.0, 0.0, 6.0)));

    /* Create the player */
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Velocity::default())
        .insert(ExternalForce::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Hover::default())
        .insert(Movement {
            direction: Vec3::ZERO,
            acceleration: 300.0,
        })
        .insert(Deceleration::default())
        .insert(Player)
        .insert(Collider::capsule_y(1.0, 1.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .insert(Name::new("Player"));
}

#[derive(Component)]
pub struct Player;

pub fn player_input(
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Movement, With<Player>>,
    camera_query: Query<&Transform, With<Camera3d>>,
) {
    let camera_transform = camera_query.single();

    for mut player_movement in &mut player_query {
        let mut x = 0.0;
        let mut z = 0.0;

        let mut forward = camera_transform.forward();
        forward.y = 0.0;
        forward = forward.normalize();

        let mut left = camera_transform.left();
        left.y = 0.0;
        left = left.normalize();

        if keyboard.pressed(KeyCode::W) {
            z += 1.0;
        }

        if keyboard.pressed(KeyCode::S) {
            z -= 1.0;
        }

        if keyboard.pressed(KeyCode::A) {
            x += 1.0;
        }

        if keyboard.pressed(KeyCode::D) {
            x -= 1.0;
        }

        let left_vec: Vec3 = x * left;
        let forward_vec: Vec3 = z * forward;

        let final_vec = left_vec + forward_vec;

        player_movement.direction = final_vec;
    }
}
