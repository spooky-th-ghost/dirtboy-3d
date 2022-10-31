use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;

mod camera;
use camera::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(CameraPlugin)
        .register_type::<Hover>()
        .register_type::<Movement>()
        .add_startup_system(setup_physics)
        .add_system(handle_hover)
        .add_system(handle_movement)
        .add_system(player_input)
        .add_system(dumb_drag)
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
            deceleration: 2.5,
        })
        .insert(Player)
        .insert(Collider::capsule_y(1.0, 1.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .insert(Name::new("Player"));
}

#[derive(Component)]
pub struct Player;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Movement {
    pub direction: Vec3,
    pub acceleration: f32,
    pub deceleration: f32,
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Hover {
    pub ray_length: f32,
    pub ride_height: f32,
    pub strength: f32,
    pub damper: f32,
}

impl Default for Hover {
    fn default() -> Self {
        Hover {
            ray_length: 4.0,
            ride_height: 2.8,
            strength: 700.0,
            damper: 50.0,
        }
    }
}

pub fn handle_hover(
    rapier_context: Res<RapierContext>,
    mut hover_query: Query<(&mut ExternalForce, &Velocity, &Transform, &Hover, Entity)>,
) {
    for (mut external_force, velocity, transform, hover, hover_entity) in &mut hover_query {
        let ray_pos = transform.translation;
        let ray_dir = Vec3::Y * -1.0;
        let max_toi = hover.ray_length;
        let solid = true;
        let filter = QueryFilter::exclude_dynamic();

        if let Some((entity, toi)) =
            rapier_context.cast_ray(ray_pos, ray_dir, max_toi, solid, filter)
        {
            if hover_entity != entity {
                let hit_point = ray_pos + ray_dir * toi;
                let distance = hit_point.distance(transform.translation);

                let ray_direction_velocity = ray_dir.dot(velocity.linvel);
                let opposite_relative = ray_dir.dot(Vec3::ZERO);

                let relative_velocity = ray_direction_velocity - opposite_relative;

                let force_direction = distance - hover.ride_height;
                let up_force = force_direction * hover.strength;
                let damping_force = relative_velocity * hover.damper;
                let spring_force = up_force - damping_force;
                external_force.force.y = spring_force * -1.0;
            } else {
                external_force.force.y = 0.0;
            }
        } else {
            external_force.force.y = 0.0;
        }
    }
}

pub fn handle_movement(mut movement_query: Query<(&mut ExternalForce, &Movement, &Velocity)>) {
    for (mut external_force, movement, velocity) in &mut movement_query {
        let mut flat_direction = movement.direction;
        let mut flat_velo = velocity.linvel;
        flat_direction.y = 0.0;
        flat_velo.y = 0.0;

        let acceleration_to_apply = if flat_velo != Vec3::ZERO && flat_direction != Vec3::ZERO {
            let angle_diff = flat_direction.angle_between(flat_velo).to_degrees();
            if angle_diff > 145.0 {
                movement.acceleration * 4.0
            } else if angle_diff > 90.0 && angle_diff < 145.0 {
                movement.acceleration * 3.0
            } else if angle_diff > 45.0 && angle_diff < 90.0 {
                movement.acceleration * 2.0
            } else {
                movement.acceleration
            }
        } else {
            movement.acceleration
        };

        let force_to_add = movement.direction.normalize_or_zero() * acceleration_to_apply;

        let y_force = external_force.force.y;
        external_force.force = Vec3::new(force_to_add.x, y_force, force_to_add.z);
    }
}

pub fn dumb_drag(time: Res<Time>, mut body_query: Query<(&mut Velocity, &Movement)>) {
    for (mut velo, movement) in &mut body_query {
        let lerped_vector = velo
            .linvel
            .lerp(Vec3::ZERO, time.delta_seconds() * movement.deceleration);
        velo.linvel.x = lerped_vector.x;
        velo.linvel.z = lerped_vector.z;
    }
}

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
