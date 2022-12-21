use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hover>()
            .register_type::<Movement>()
            .register_type::<Spring>()
            .add_system(handle_hover)
            .add_system(handle_rotation)
            .add_system(handle_movement);
        //            .add_system(drive_rotation)
    }
}

#[derive(Component)]
pub struct Grounded;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Movement {
    pub goal_velocity: Vec3,
    pub direction: Vec3,
    pub acceleration: f32,
}

#[derive(Component)]
pub struct RotationDriver {
    pub up_vector: Vec3,
    pub look_target: Vec3,
}

impl Default for RotationDriver {
    fn default() -> Self {
        RotationDriver {
            up_vector: Vec3::Y,
            look_target: Vec3::Z,
        }
    }
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Hover {
    pub ray_length: f32,
    pub ride_height: f32,
    pub strength: f32,
    pub damper: f32,
}

impl Hover {
    pub fn calculate_spring_force(&self, distance: f32, linear_velocity: Vec3) -> f32 {
        let ray_direction = Vec3::Y * -1.0;
        let ray_direction_velocity = ray_direction.dot(linear_velocity);
        let opposite_relative = ray_direction.dot(Vec3::ZERO);
        let relative_velocity = ray_direction_velocity - opposite_relative;
        let force_direction = distance - self.ride_height;
        let up_force = force_direction * self.strength;
        let damping_force = relative_velocity * self.damper;
        let spring_force = up_force - damping_force;
        spring_force * -1.0
    }
}

impl Default for Hover {
    fn default() -> Self {
        Hover {
            ray_length: 4.0,
            ride_height: 2.8,
            strength: 900.0,
            damper: 60.0,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Spring(pub f32);

impl Default for Spring {
    fn default() -> Self {
        Spring(200.0)
    }
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Deceleration(pub f32);

impl Default for Deceleration {
    fn default() -> Self {
        Deceleration(2.5)
    }
}

pub fn handle_hover(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut hover_query: Query<(
        &mut ExternalForce,
        &mut ExternalImpulse,
        &mut Velocity,
        &Transform,
        &Hover,
        Option<&Grounded>,
        Entity,
    )>,
    springs: Query<(Entity, &Spring)>,
) {
    for (
        mut external_force,
        mut external_impulse,
        mut velocity,
        transform,
        hover,
        is_grounded,
        hover_entity,
    ) in &mut hover_query
    {
        let ray_pos = transform.translation;
        let ray_dir = Vec3::Y * -1.0;
        let max_distance = hover.ray_length;
        let solid = true;
        let filter = QueryFilter::exclude_dynamic().exclude_sensors();

        if let Some((entity, intersection)) =
            rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, max_distance, solid, filter)
        {
            if springs.contains(entity) {
                if let Ok((_, spring)) = springs.get(entity) {
                    // velocity.linvel.y = 0.0;
                    external_impulse.impulse = intersection.normal * spring.0;
                    return;
                }
            }
            external_force.force.y =
                hover.calculate_spring_force(intersection.toi, velocity.linvel);
            if intersection.toi <= hover.ride_height {
                if let None = is_grounded {
                    commands.entity(hover_entity).insert(Grounded);
                }
            }
        } else {
            external_force.force.y = 0.0;
            if let Some(_) = is_grounded {
                commands.entity(hover_entity).remove::<Grounded>();
            }
        }
    }
}

pub fn handle_movement(
    mut movement_query: Query<(
        &mut ExternalForce,
        &Movement,
        &Velocity,
        Option<&mut RotationDriver>,
    )>,
) {
    for (mut external_force, movement, velocity, should_rotate) in &mut movement_query {
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

        if let Some(mut rotation_driver) = should_rotate {
            let new_target = movement.direction.normalize_or_zero();
            if new_target != Vec3::ZERO {
                rotation_driver.look_target = new_target;
            }
        }
    }
}

fn handle_rotation(
    time: Res<Time>,
    mut query: Query<(&Movement, &mut Transform), With<RotationDriver>>,
) {
    for (movement, mut transform) in &mut query {
        if movement.direction != Vec3::ZERO {
            let target = transform.translation - movement.direction;
            transform.look_at(target, Vec3::Y);
        }
    }
}
