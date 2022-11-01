use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

mod camera;
use camera::*;

mod physics;
use physics::*;

mod player;
use player::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -30.0,
            ..default()
        })
        .add_plugin(CameraPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_physics)
        .add_system(player_input)
        .run();
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the ground */
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(RigidBody::Fixed)
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(200.0, 0.2, 200.0))),
            material: materials.add(Color::rgb(0.67, 0.84, 0.92).into()),
            transform: Transform::from_xyz(0.0, -4.0, 0.0),
            ..default()
        });

    commands
        .spawn()
        .insert(Collider::cuboid(10.0, 0.1, 20.0))
        .insert(RigidBody::Fixed)
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(20.0, 0.2, 40.0))),
            transform: Transform::from_xyz(5.0, -4.0, 0.0)
                .with_rotation(Quat::from_rotation_z(0.3)),
            material: materials.add(Color::rgb(0.37, 0.34, 0.32).into()),
            ..default()
        });

    commands
        .spawn()
        .insert(Collider::cuboid(1.0, 1.0, 1.0))
        .insert(RigidBody::Dynamic)
        .insert(Velocity::default())
        .insert(ExternalForce::default())
        .insert(Hover::default())
        .insert(Deceleration(1.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(6.0, 0.0, 6.0)));
}
