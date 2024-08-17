//! Spawn the main level.

use rand::{Rng, thread_rng};
use crate::demo::{SAW_HEIGHT, Selectable, SpawnPlank};
use avian2d::collision::Sensor;
use avian2d::prelude::{Collider, MassPropertiesBundle, RigidBody, Rotation};
use bevy::{ecs::world::Command, prelude::*};
use crate::screens::Screen;

pub(super) fn plugin(_app: &mut App) {
    // No setup required for this plugin.
    // It's still good to have a function here so that we can add some setup
    // later if needed.
}

#[derive(Component)]
pub struct Plank;

#[derive(Component)]
pub struct Saw {
    pub active: bool,
}

#[derive(Component)]
pub struct SawBody;

/// A [`Command`] to spawn the level.
/// Functions that accept only `&mut World` as their parameter implement [`Command`].
/// We use this style when a command requires no configuration.
pub fn spawn_level(world: &mut World) {
    let mut commands = world.commands();
    commands.spawn((
        Transform::default(),
        RigidBody::Static,
        Collider::segment(Vec2::new(-100000., 0.), Vec2::new(100000., 0.)),
        StateScoped(Screen::Gameplay)
    ));
    let mut rng = thread_rng();
    for i in 0..10 {
        let pos_y = 5. + (i as f32) * 10. + rng.gen_range(1.5..3.5);
        let pos_x = -200. + rng.gen_range(-20f32..20.);
        let width = 200. + rng.gen_range(-25f32..5.);
        let rotation = Rotation::from_degrees(rng.gen_range(-5f32..5.));
        commands.trigger(SpawnPlank {
            width: width,
            height: 10.,
            position: Vec2::new(pos_x, pos_y),
            rotation,
            l_vel: None,
            a_vel: None,
            color: Color::srgba(0.3 + rng.gen_range(-0.05..0.05),0.15 + rng.gen_range(-0.03..0.03),0.01, 1.)
        });
    }


    commands.spawn(
        (
        Transform::from_xyz(0., 20.+SAW_HEIGHT, 0.),
        RigidBody::Kinematic,
        Collider::rectangle(30.,20.),
        Name::new("Saw body"),
            SawBody
        )
    );
     commands.spawn((
        Transform::from_xyz(0., 20. + SAW_HEIGHT, 0.),
        RigidBody::Kinematic,
        Collider::triangle(-Vec2::X * 0.05, Vec2::X * 0.05, -Vec2::Y * 10.),
        Saw { active: true },
        Sensor,
        Name::new("Saw"),
                             StateScoped(Screen::Gameplay)
    ));


    // The only thing we have in our level is a player,
    // but add things like walls etc. here.
}


pub fn spawn_banana(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut transform = Transform::from_xyz(-200., 600., 0.);
    transform.scale = Vec3::ONE * 0.02;

    let mut colliders = vec![];
    colliders.push((Vec2::new(-275.,-150.), 85f32.to_radians(), Collider::capsule(150.,250.)));
    colliders.push((Vec2::new(225.,-150.), 95f32.to_radians(), Collider::capsule(150.,250.)));
    colliders.push((Vec2::new(400.,0.), -55f32.to_radians(), Collider::capsule(150.,400.)));
    let collider = Collider::compound(colliders);
        commands.spawn(
        (SpriteBundle {
            texture: asset_server.load("images/banana.png"),
            transform,
            ..default()
        }, collider,
            RigidBody::Dynamic,
            Selectable,
            MassPropertiesBundle::new_computed(&Collider::circle(1.), 1.),
        StateScoped(Screen::Gameplay)
        )
    );

}