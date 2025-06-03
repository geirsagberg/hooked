//! Chain shooting mechanics with physics.

use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};

use crate::{AppSystems, PausableSystems, demo::player::Player, screens::Screen};

/// Collision layers for physics objects
#[derive(PhysicsLayer, Default)]
pub enum Layer {
    #[default]
    ChainLink,
    StaticObstacle,
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ChainLink>();
    app.register_type::<ChainRoot>();
    app.init_resource::<ChainState>();

    app.add_systems(
        Update,
        (handle_chain_input, update_chain_rendering)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Marker component for chain links
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChainLink {
    pub link_index: usize,
}

/// Marker component for the root of a chain (connected to player)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChainRoot;

/// Resource to track if a chain is currently active
#[derive(Resource, Default)]
pub struct ChainState {
    pub active: bool,
    pub links: Vec<Entity>,
}

fn handle_chain_input(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut chain_state: ResMut<ChainState>,
    player_query: Query<&Transform, With<Player>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        // Remove existing chain if one exists
        if chain_state.active {
            for entity in &chain_state.links {
                commands.entity(*entity).despawn();
            }
            chain_state.links.clear();
            chain_state.active = false;
        }

        // Always create new chain
        if let Ok(player_transform) = player_query.single() {
            if let Some(cursor_world_pos) = get_cursor_world_position(&windows, &camera_query) {
                spawn_chain(
                    &mut commands,
                    &mut chain_state,
                    player_transform.translation.truncate(),
                    cursor_world_pos,
                );
            }
        }
    }
}

fn get_cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    let (camera, camera_transform) = camera_query.single().ok()?;

    camera
        .viewport_to_world_2d(camera_transform, cursor_pos)
        .ok()
}

fn spawn_chain(
    commands: &mut Commands,
    chain_state: &mut ChainState,
    start_pos: Vec2,
    target_pos: Vec2,
) {
    let chain_direction = (target_pos - start_pos).normalize();
    let chain_length = (target_pos - start_pos).length();
    let link_size = 20.0; // Base link size for physics
    let capsule_half_length = link_size * 0.4; // Half-length of each capsule
    let actual_link_spacing = capsule_half_length * 2.0; // Actual distance between link centers
    let num_links = (chain_length / actual_link_spacing).max(1.0) as usize;

    let mut previous_entity = None;
    chain_state.links.clear();
    for i in 0..num_links {
        let link_progress = i as f32 / num_links.max(1) as f32;
        let link_pos = start_pos
            + chain_direction * link_progress * (actual_link_spacing * (num_links - 1) as f32);

        // Calculate rotation to align capsule with chain direction
        let link_rotation = Quat::from_rotation_z(chain_direction.y.atan2(chain_direction.x));

        let mut entity_commands = commands.spawn((
            Name::new(format!("Chain Link {}", i)),
            ChainLink { link_index: i },
            // Physics components
            RigidBody::Dynamic,
            Collider::capsule(capsule_half_length, 2.0), // Length, radius - elongated capsule
            Mass(2.0),                                   // Increased mass for better stability
            LinearDamping(0.2),                          // More air resistance for stability
            AngularDamping(0.3),                         // More rotational damping
            SweptCcd::default(), // Continuous Collision Detection to prevent tunneling
            Restitution::new(0.1), // Less bounciness for smoother collisions
            Friction::new(0.7),  // Higher friction for better interaction with obstacles
            // Collision groups to ensure proper detection
            CollisionLayers::new(
                [Layer::ChainLink],
                [Layer::ChainLink, Layer::StaticObstacle],
            ),
            // Visual components - elongated rectangle to match physics
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(link_size * 0.8, 4.0)), // Match capsule dimensions
                ..default()
            },
            Transform::from_translation(link_pos.extend(0.0)).with_rotation(link_rotation),
            Visibility::default(),
        ));

        // Add root marker to first link
        if i == 0 {
            entity_commands.insert(ChainRoot);
        }

        let current_entity = entity_commands.id();
        chain_state.links.push(current_entity); // Create joint to previous link or player
        if let Some(prev_entity) = previous_entity {
            commands.spawn((
                Name::new(format!("Chain Joint {}-{}", i - 1, i)),
                RevoluteJoint::new(prev_entity, current_entity)
                    .with_local_anchor_1(Vec2::new(capsule_half_length, 0.0)) // Right end of previous link
                    .with_local_anchor_2(Vec2::new(-capsule_half_length, 0.0)) // Left end of current link
                    .with_compliance(0.00001) // Soft constraint for natural movement
                    .with_angular_velocity_damping(0.1), // Add some rotational damping
            ));
        }

        previous_entity = Some(current_entity);
    }

    // Give the chain an initial impulse towards the target
    if let Some(&first_link) = chain_state.links.first() {
        let impulse_strength = 200.0; // Reduced impulse strength for better collision handling
        let impulse = chain_direction * impulse_strength;

        commands
            .entity(first_link)
            .insert(ExternalImpulse::new(impulse));
    }

    chain_state.active = true;
}

fn update_chain_rendering(
    mut gizmos: Gizmos,
    chain_links: Query<&Transform, With<ChainLink>>,
    chain_state: Res<ChainState>,
) {
    if !chain_state.active || chain_state.links.len() < 2 {
        return;
    }

    // Draw lines between consecutive links
    for window in chain_state.links.windows(2) {
        if let (Ok(transform1), Ok(transform2)) =
            (chain_links.get(window[0]), chain_links.get(window[1]))
        {
            gizmos.line_2d(
                transform1.translation.truncate(),
                transform2.translation.truncate(),
                Color::WHITE,
            );
        }
    }
}
