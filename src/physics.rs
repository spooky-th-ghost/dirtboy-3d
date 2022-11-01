use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hover>()
            .register_type::<Movement>()
            .add_system(handle_hover)
            .add_system(handle_movement)
            .add_system(dumb_drag);
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Movement {
    pub direction: Vec3,
    pub acceleration: f32,
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Hover {
    pub ray_length: f32,
    pub ride_height: f32,
    pub strength: f32,
    pub damper: f32,
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Deceleration(pub f32);

impl Default for Deceleration {
    fn default() -> Self {
        Deceleration(2.5)
    }
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

pub fn dumb_drag(time: Res<Time>, mut body_query: Query<(&mut Velocity, &Deceleration)>) {
    for (mut velo, deceleration) in &mut body_query {
        let lerped_vector = velo
            .linvel
            .lerp(Vec3::ZERO, time.delta_seconds() * deceleration.0);
        velo.linvel.x = lerped_vector.x;
        velo.linvel.z = lerped_vector.z;
    }
}
