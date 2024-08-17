//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use crate::demo::level::SawBody;
use avian2d::prelude::LayerMask;
use avian2d::prelude::CollisionLayers;
use avian2d::prelude::{PhysicsLayer, Sensor};
use avian2d::prelude::Gravity;
use crate::demo::level::{Plank, Saw};
use crate::screens::Screen;
use avian2d::collision::Collider;
use avian2d::math::Scalar;
use avian2d::position::Position;
use avian2d::prelude::{RevoluteJoint, Rotation, Joint};
use avian2d::prelude::{
    AngularVelocity, CollidingEntities, ExternalForce, ExternalImpulse, LinearVelocity, RigidBody,
};
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use rand::Rng;

pub mod level;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((level::plugin,));
    app.insert_resource(Gravity(Vec2::NEG_Y * 100.));
    app.insert_resource(PrevMousePos { pos: Vec2::ZERO});
    app.add_systems(FixedUpdate, move_saw.run_if(in_state(Screen::Gameplay)));
    app.add_systems(FixedUpdate, split.run_if(in_state(Screen::Gameplay)));
    app.add_systems(Update, selection_system.run_if(in_state(Screen::Gameplay)));
    app.add_systems(Update, move_with_mouse.run_if(in_state(Screen::Gameplay)));
    app.add_systems(Update, nail_system);
    app.observe(spawn_plank);
    app.observe(deselect_all);
}

#[derive(Component)]
pub struct Selected;

#[derive(Event)]
pub struct DeselectAll;

#[derive(Resource)]
pub struct PrevMousePos {
    pos: Vec2
}

#[derive(PhysicsLayer)]
pub enum GameLayers {
    Objects,
    Ground
}

#[derive(Component)]
pub struct Nail;

fn nail_system(input: Res<ButtonInput<KeyCode>>, mut nail: Query<(Entity, &mut Position, &mut Rotation), With<Nail>>, mut commands: Commands
               ,
               window: Query<&Window>, camera: Query<(&Camera, &GlobalTransform)>
) {
    if input.just_pressed(KeyCode::KeyN) {
        if nail.is_empty() {
            commands.spawn(
                (Nail, Transform::from_xyz(0., 200., 0.),
                 Sensor,
                 Collider::segment(Vec2::Y * 2.5, -Vec2::Y * 7.5)),
            );
            // Spawn Nail, at cursor
        } else {
            let (e,_,_) = nail.get_single().unwrap();
            commands.entity(e).despawn();

        }
    }
    if let Ok((e, mut nail, mut rotation)) = nail.get_single_mut() {
        let window = window.get_single().unwrap();
        let (camera, gt) = camera.get_single().unwrap();
        if let Some(p) = get_world_pos(window, camera, gt) {
            nail.0 = p.truncate()
        }
        if input.pressed(KeyCode::KeyR) {
            *rotation = rotation.add_angle(-1f32.to_radians());
        }
    }
}

fn deselect_all(_: Trigger<DeselectAll>, selected: Query<Entity, With<Selected>>, mut commands: Commands) {
    for e in selected.iter() {
        commands.entity(e).despawn();
    }
}

fn move_with_mouse(window: Query<&Window>, camera: Query<(&Camera, &GlobalTransform)>, mut selected: Query<(&Position, &mut LinearVelocity), With<Selected>>, _prev_pos: ResMut<PrevMousePos>) {
    let window = window.get_single().unwrap();
    let (camera, gt) = camera.get_single().unwrap();

    if let Some(p) = get_world_pos(window, camera, gt) {
        if let Ok((obj_pos, mut s)) = selected.get_single_mut() {
            s.0 = p.truncate() - obj_pos.0;
            s.0 *= 4.;
        }
    }
}

fn get_world_pos(window: &Window, camera: &Camera, gt: &GlobalTransform) -> Option<Vec3> {
    window.cursor_position().and_then(|c| camera.viewport_to_world(gt, c)).map(|r| r.origin)
}

fn selection_system(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    selectables: Query<(Entity, &Collider, &Position, &Rotation), With<Selectable>>,
    input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands
) {
    if input.just_released(MouseButton::Left) {
        commands.trigger(DeselectAll);
    }
    if !input.just_pressed(MouseButton::Left) {
        return;
    }
    let (camera, transform) = camera.get_single().unwrap();
    let window = window.get_single().unwrap();
    if let Some(pos) = get_world_pos(window, camera, transform)
    {
        for (e, c, p, r) in selectables.iter() {
            if c.contains_point(*p, *r, pos.truncate()) {
                let new_ent = commands.spawn(
                    (
                        RigidBody::Dynamic,
                        Collider::circle(2.),
                        Transform::from_translation(pos),
                        CollisionLayers::new(GameLayers::Objects, GameLayers::Ground),
                    Selected)
                ).id();
                let mut local_pos = pos.truncate() - p.0;
                local_pos = (*r).inverse() * local_pos;
                commands.spawn((Selected, RevoluteJoint::new(e, new_ent).with_local_anchor_1(local_pos).with_compliance(0.)));
            }
        }
    }
}

pub const SAW_HEIGHT: f32 = 10.;

fn move_saw(
    mut query: Query<(&mut LinearVelocity, &mut Position, &mut Saw), Without<SawBody>>,
    mut saw_body: Query<(&mut LinearVelocity, &mut Position), (With<SawBody>, Without<Saw>)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if let (Ok((mut velocity, mut position, mut saw)), Ok((mut saw_body_vel, mut saw_body_pos))) = (query.get_single_mut(), saw_body.get_single_mut()) {
        let prev_velocity = *velocity;
        let prev_position = *position;
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

        if position.y < SAW_HEIGHT + 11. {
            velocity.0.x = 0.;
        } else if position.y < SAW_HEIGHT + 17. {
            velocity.0.x *= 0.1;
        }

        if position.y < 5. {
            position.y = 5.;
        } else if position.y >= SAW_HEIGHT + 19. {
            if !saw.active {
                info!("Saw activated");
            }
            saw.active = true;
        }
        if position.y > SAW_HEIGHT + 20.5 {
            if velocity.y > 0. {
                velocity.y = 0.;
            }
            position.y = SAW_HEIGHT + 20.;
        }

        if input.pressed(KeyCode::Space) {
            let jitter = if position.y > SAW_HEIGHT + 13. { 0.6 } else { 0.1 };
            if position.y > 10.5 {
                position.x += rand::thread_rng().gen_range(-jitter..jitter);
            }
            if position.y > 10. {
                velocity.y -= 0.2;
            } else {
                velocity.y = 0.;
            }
        } else if position.y < SAW_HEIGHT + 19.5 {
            velocity.y += 0.5;
        }
        let vel_change = velocity.0 - prev_velocity.0;
        let pos_change = position.0 - prev_position.0;
        saw_body_vel.0.x += vel_change.x;
        saw_body_pos.0.x += pos_change.x;
        saw_body_vel.0.y += vel_change.y * 0.5;
        if saw_body_pos.0.y < 17. + SAW_HEIGHT {
            saw_body_pos.0.y = 17. + SAW_HEIGHT;
        }
    }
}

#[derive(Event)]
pub struct SpawnPlank {
    width: f32,
    height: f32,
    position: Vec2,
    rotation: Rotation,
    l_vel: Option<LinearVelocity>,
    a_vel: Option<AngularVelocity>,
    color: Color,
}

#[derive(Component, Default)]
pub struct Selectable;

#[derive(Component)]
pub struct ColorInfo(Color);

const CUT_SIZE: f32 = 1.;

fn spawn_plank(trigger: Trigger<SpawnPlank>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    let ev = trigger.event();
    let collider = Collider::rectangle(ev.width, ev.height);
    let mut ent_commands = commands.spawn((
        RigidBody::Dynamic,
        collider,
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(ev.width, ev.height))),
            transform: Transform::from_translation(ev.position.extend(0.)),
            material: materials.add(ev.color),
            ..default()
        },
        Plank,
        ColorInfo(ev.color),
        Selectable::default(),
        CollisionLayers::new(GameLayers::Objects, LayerMask::ALL),
        StateScoped(Screen::Gameplay)
    ));
    if let Some(l) = ev.l_vel {
        ent_commands.insert(l);
    }
    if let Some(a) = ev.a_vel {
        ent_commands.insert(a);
    }
}

fn split(
    mut saw: Query<(&Position, &CollidingEntities, &mut Saw)>,
    planks: Query<(&Collider, &Position, &ColorInfo), With<Plank>>,
    mut commands: Commands,
) {
    if saw.is_empty() {
        return;
    }
    let (saw_pos, collisions, mut saw) = saw.get_single_mut().unwrap();
    if !saw.active {
        return;
    }
    if saw_pos.y > 11. {
        return;
    }
    for collision in &collisions.0 {
        if let Ok((c, p, color_info)) = planks.get(*collision) {
            saw.active = false;
            let relative_x = saw_pos.x - p.x;
            let cuboid = c.shape().as_cuboid().unwrap();
            let old_collider_width = cuboid.half_extents.x as f32 * 2.;
            let collider_height = cuboid.half_extents.y as f32 * 2.;
            let new_collider_left_width = old_collider_width / 2. + relative_x - CUT_SIZE;
            let new_collider_right_width = old_collider_width / 2. - relative_x - CUT_SIZE;
            if new_collider_left_width > 0. {
                let pos_left = Vec2::new(saw_pos.x - new_collider_left_width / 2. - CUT_SIZE, p.y);
                let angular = AngularVelocity(1. / new_collider_left_width.log2());
                let linear = LinearVelocity(Vec2::new(
                    -10. / new_collider_left_width.log2(),
                    10. / new_collider_left_width.log2(),
                ));
                let command_left = SpawnPlank {
                    width: new_collider_left_width,
                    height: collider_height,
                    position: pos_left,
                    rotation: Rotation::default(),
                    l_vel: Some(linear),
                    a_vel: Some(angular),
                    color: color_info.0
                };
                commands.trigger(command_left);
            }

            if new_collider_right_width > 0. {
                let pos_left = Vec2::new(saw_pos.x + new_collider_right_width / 2. - CUT_SIZE, p.y);
                let angular = AngularVelocity(-1. / new_collider_right_width.log2());
                let linear = LinearVelocity(Vec2::new(
                    10. / new_collider_right_width.log2(),
                    10. / new_collider_right_width.log2(),
                ));
                let command_left = SpawnPlank {
                    width: new_collider_right_width,
                    height: collider_height,
                    position: pos_left,
                    rotation: Rotation::default(),
                    l_vel: Some(linear),
                    a_vel: Some(angular),
                    color: color_info.0
                };
                commands.trigger(command_left);
            }
            commands.entity(*collision).despawn();
        }
    }
}
