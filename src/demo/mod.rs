//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use avian2d::collision::Collider;
use avian2d::math::Scalar;
use avian2d::position::Position;
use avian2d::prelude::{AngularVelocity, CollidingEntities, ExternalForce, ExternalImpulse, LinearVelocity, RigidBody};
use bevy::prelude::*;
use crate::demo::level::{Plank, Saw};

pub mod level;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
    ));
    app.add_systems(FixedUpdate, move_saw);
    app.add_systems(FixedUpdate, split);
}

fn move_saw(mut query: Query<(&mut LinearVelocity, &mut Position, &mut Saw)>, input: Res<ButtonInput<KeyCode>>) {
    if let Ok((mut velocity, mut position, mut saw)) = query.get_single_mut() {
        let mut movement = 0.;
        if input.pressed(KeyCode::ArrowLeft) {
            movement -= 1.;
        }
        if input.pressed(KeyCode::ArrowRight) {
            movement += 1.;
        }

        if movement != 0. {
            velocity.0.x += movement;
        } else {
            velocity.0.x *= 0.9;
            if velocity.0.x.abs() <= 0.1 {
                velocity.0.x = 0.;
            }
        }

        if position.y < 5. {
            position.y = 5.;
        } else if position.y >= 20. {
            if !saw.active { info!("Saw activated"); }
            saw.active = true;
            position.y = 20.;
        }

        if input.pressed(KeyCode::Space) {
            if position.y > 10. {
                velocity.y -= 0.2;
            } else {
                velocity.y = 0.;
            }
        } else if position.y < 20. {
            velocity.y += 0.5;
        }
    }
}

const CUT_SIZE: f32 = 1.;

fn split(mut saw: Query<(&Position, &CollidingEntities, &mut Saw)>, planks: Query<(&Collider, &Position), With<Plank>>, mut commands: Commands) {
    if saw.is_empty() { return; }
    let (saw_pos, collisions, mut saw) = saw.get_single_mut().unwrap();
    if !saw.active { return; }
    if saw_pos.y > 11. { return; }
    for collision in &collisions.0 {
        if let Ok((c, p)) = planks.get(*collision) {
            saw.active = false;
            let relative_x = saw_pos.x - p.x;
            let cuboid = c.shape().as_cuboid().unwrap();
            let old_collider_width = cuboid.half_extents.x as f32 * 2.;
            let collider_height = cuboid.half_extents.y as f32 * 2.;
            let new_collider_left_width = old_collider_width / 2. + relative_x - CUT_SIZE;
            let new_collider_right_width= old_collider_width / 2. - relative_x - CUT_SIZE;
            let new_collider_left = Collider::rectangle(new_collider_left_width, collider_height);
            let new_collider_right= Collider::rectangle(new_collider_right_width, collider_height);
            commands.entity(*collision).despawn();
            let pos_left = Vec3::new(saw_pos.x - new_collider_left_width / 2. - CUT_SIZE, p.y, 0.);
            let pos_right= Vec3::new(saw_pos.x + new_collider_right_width / 2. + CUT_SIZE, p.y, 0.);
            let mut left_velocity = LinearVelocity::default();
            let left_angular = AngularVelocity(1./new_collider_left_width.log2());
            let right_angular = AngularVelocity(-1./new_collider_right_width.log2());

            left_velocity.y = 10./new_collider_left_width.log2();
            left_velocity.x = -10./new_collider_left_width.log2();
            let mut right_velocity = LinearVelocity::default();
            right_velocity.y = 10./new_collider_right_width.log2();
            right_velocity.x = 10./new_collider_right_width.log2();

            commands.spawn((
                RigidBody::Dynamic,
                new_collider_left,
                Transform::from_translation(pos_left),
                left_velocity,
                left_angular,
                Plank,
            ));
            commands.spawn((
                RigidBody::Dynamic,
                new_collider_right,
                Transform::from_translation(pos_right),
                right_velocity,
                right_angular,
                Plank,
            ));
        }
    }

}