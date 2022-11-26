use crate::{Deceleration, Grounded, Hover, Movement, RotationDriver};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(player_input);
    }
}

#[derive(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Velocity::default())
        .insert(ExternalForce::default())
        .insert(ExternalImpulse::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Hover::default())
        .insert(Movement {
            goal_velocity: Vec3::ZERO,
            direction: Vec3::ZERO,
            acceleration: 500.0,
        })
        .insert(RotationDriver::default())
        .insert(Deceleration::default())
        .insert(Player)
        .insert(Collider::capsule_y(1.0, 1.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .insert(Name::new("Player"));
}

pub fn player_input(
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<
        (
            &mut ExternalImpulse,
            &mut Movement,
            &mut Velocity,
            Option<&Grounded>,
        ),
        With<Player>,
    >,
    camera_query: Query<&Transform, With<Camera3d>>,
) {
    let camera_transform = camera_query.single();

    for (mut impulse, mut player_movement, mut velocity, is_grounded) in &mut player_query {
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

        if keyboard.just_pressed(KeyCode::Space) {
            if let Some(_) = is_grounded {
                velocity.linvel.y = 0.0;
                impulse.impulse = Vec3::Y * 300.0;
            }
        }

        let left_vec: Vec3 = x * left;
        let forward_vec: Vec3 = z * forward;

        let final_vec = left_vec + forward_vec;

        player_movement.direction = final_vec;
    }
}
