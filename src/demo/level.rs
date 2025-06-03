//! Spawn the main level.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    demo::player::{PlayerAssets, player},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            player(400.0, &player_assets, &mut texture_atlas_layouts),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            )
        ],
    ));

    // Spawn static boxes for chain interaction
    spawn_static_boxes(&mut commands);
}

/// Spawns static boxes around the level that chains can interact with
fn spawn_static_boxes(commands: &mut Commands) {
    let box_positions = [
        Vec2::new(200.0, 100.0),
        Vec2::new(-150.0, 50.0),
        Vec2::new(100.0, -100.0),
        Vec2::new(-200.0, -150.0),
        Vec2::new(0.0, 200.0),
        Vec2::new(300.0, -50.0),
    ];

    for (i, &position) in box_positions.iter().enumerate() {
        commands.spawn((
            Name::new(format!("Static Box {}", i)),
            // Physics components
            RigidBody::Static,               // Static means it won't move
            Collider::rectangle(40.0, 40.0), // 40x40 pixel box
            Restitution::new(0.2),           // Slight bounciness
            Friction::new(0.7),              // Good friction for chain interaction
            // Visual components
            Sprite {
                color: Color::srgb(0.8, 0.8, 0.8), // Light gray color
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            Transform::from_translation(position.extend(0.0)),
            Visibility::default(),
            StateScoped(Screen::Gameplay), // Clean up when leaving gameplay
        ));
    }
}
