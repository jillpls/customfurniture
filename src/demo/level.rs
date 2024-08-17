//! Spawn the main level.

use avian2d::collision::Sensor;
use avian2d::prelude::{Collider, RigidBody};
use bevy::{ecs::world::Command, prelude::*};

pub(super) fn plugin(_app: &mut App) {
    // No setup required for this plugin.
    // It's still good to have a function here so that we can add some setup
    // later if needed.
}

#[derive(Component)]
pub struct Plank;

#[derive(Component)]
pub struct Saw {
    pub active: bool
}

/// A [`Command`] to spawn the level.
/// Functions that accept only `&mut World` as their parameter implement [`Command`].
/// We use this style when a command requires no configuration.
pub fn spawn_level(world: &mut World) {
    let mut commands = world.commands();
    commands.spawn(
        (Transform::default(), RigidBody::Static, Collider::segment(Vec2::new(-100000., 0.), Vec2::new(100000.,0.)))
    );
    let mut transform = Transform::from_xyz(-50. ,5., 0.);
    commands.spawn(
        (transform, RigidBody::Dynamic, Collider::rectangle(200., 10.), Plank)
    );

    commands.spawn(
        (Transform::from_xyz(0., 20., 0.), RigidBody::Kinematic, Collider::triangle(-Vec2::X*0.05, Vec2::X * 0.05, -Vec2::Y * 10.), Saw { active: true }, Sensor)
    );
    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
}
