# GitHub Copilot Guide for Bevy 0.16

This guide provides comprehensive guidance for developing with Bevy 0.16, incorporating key migration changes, common patterns, and best practices derived from analyzing real-world Bevy projects.

## Table of Contents

1. [Key Migration Changes from 0.15 to 0.16](#key-migration-changes-from-015-to-016)
2. [Essential Patterns and Best Practices](#essential-patterns-and-best-practices)
3. [Plugin Architecture](#plugin-architecture)
4. [State Management](#state-management)
5. [UI and Theming](#ui-and-theming)
6. [Asset Loading and Management](#asset-loading-and-management)
7. [Audio Systems](#audio-systems)
8. [System Scheduling](#system-scheduling)
9. [Common Migration Gotchas](#common-migration-gotchas)
10. [Code Examples and Snippets](#code-examples-and-snippets)

## Key Migration Changes from 0.15 to 0.16

### 1. Required Component Initialization

**Before (0.15):**

```rust
commands.spawn(SpriteBundle::default());
```

**After (0.16):**

```rust
commands.spawn((
    Sprite::default(),
    Transform::default(),
    Visibility::default(),
));
```

### 2. UI Node Changes

**Before (0.15):**

```rust
commands.spawn(NodeBundle {
    style: Style {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    },
    ..default()
});
```

**After (0.16):**

```rust
commands.spawn((
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    },
    // Other components as needed
));
```

### 3. System Parameter Changes

**Before (0.15):**

```rust
fn my_system(mut commands: Commands, query: Query<Entity, With<MyComponent>>) {
    // System logic
}
```

**After (0.16):**

```rust
fn my_system(mut commands: Commands, query: Query<Entity, With<MyComponent>>) {
    // System logic - mostly unchanged, but some parameter types may differ
}
```

### 4. Observer System Introduction

**New in 0.16:**

```rust
fn setup(mut commands: Commands) {
    commands.observe(on_enemy_death);
}

fn on_enemy_death(trigger: Trigger<EnemyDied>, mut commands: Commands) {
    let entity = trigger.entity();
    commands.entity(entity).despawn();
}
```

## Essential Patterns and Best Practices

### Plugin Organization

Organize your code into modular plugins for better maintainability:

```rust
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameplayState>()
            .add_systems(OnEnter(Screen::Gameplay), spawn_level)
            .add_systems(
                Update,
                (
                    player_movement,
                    handle_collisions,
                    update_score,
                ).run_if(in_state(Screen::Gameplay)),
            )
            .add_systems(OnExit(Screen::Gameplay), despawn_gameplay_entities);
    }
}
```

### Screen/State Management Pattern

Use enums for clear state management:

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Screen {
    Splash,
    Loading,
    Title,
    Gameplay,
    GameOver,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Menu {
    None,
    Main,
    Settings,
    Pause,
    Credits,
}
```

### Component-Driven Design

Prefer composition over inheritance:

```rust
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

// Spawn player with multiple components
commands.spawn((
    Player,
    Health(100.0),
    Velocity(Vec2::ZERO),
    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    Sprite::default(),
    Transform::default(),
));
```

## Plugin Architecture

### Main Plugin Structure

```rust
pub struct MyGamePlugin;

impl Plugin for MyGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AudioPlugin,
            DemoPlugin,
            DevToolsPlugin,
            ScreenPlugin,
            ThemePlugin,
        ));
    }
}
```

### Sub-Plugin Pattern

```rust
pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerPlugin,
            MovementPlugin,
            AnimationPlugin,
            LevelPlugin,
        ));
    }
}
```

## State Management

### Screen Transitions

```rust
fn check_for_screen_change(
    mut next_screen: ResMut<NextState<Screen>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        next_screen.set(Screen::Title);
    }
}
```

### Conditional System Execution

```rust
app.add_systems(
    Update,
    (
        handle_player_input,
        update_player_animation,
        move_player,
    ).run_if(in_state(Screen::Gameplay).and(in_state(Menu::None))),
);
```

## UI and Theming

### Theme-Based UI Components

```rust
#[derive(Component)]
pub struct ThemeColor(pub ColorPalette);

#[derive(Resource)]
pub struct ThemeData {
    pub button_text: TextFont,
    pub title_text: TextFont,
    pub label_text: TextFont,
}

fn spawn_button(
    commands: &mut Commands,
    theme: &ThemeData,
    text: &str,
) -> Entity {
    commands.spawn((
        Button,
        Node {
            padding: UiRect::all(Val::Px(16.0)),
            margin: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
    )).with_children(|parent| {
        parent.spawn((
            Text::new(text),
            theme.button_text.clone(),
            TextColor(Color::WHITE),
        ));
    }).id()
}
```

### Interaction Handling

```rust
fn handle_button_interactions(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ThemeColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background, theme_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = theme_color.0.primary().into();
            }
            Interaction::Hovered => {
                *background = theme_color.0.secondary().into();
            }
            Interaction::None => {
                *background = theme_color.0.neutral().into();
            }
        }
    }
}
```

## Asset Loading and Management

### Asset Tracking Pattern

```rust
#[derive(Resource, Debug, Deref, DerefMut)]
pub struct LoadingAssets(pub Vec<UntypedHandle>);

impl LoadingAssets {
    pub fn add<T: Asset>(&mut self, handle: &Handle<T>) -> &Handle<T> {
        self.push(handle.clone().untyped());
        handle
    }
}

fn check_loading_complete(
    mut commands: Commands,
    loading_assets: Res<LoadingAssets>,
    asset_server: Res<AssetServer>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if loading_assets.is_empty() {
        return;
    }

    let all_loaded = loading_assets
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle.id()));

    if all_loaded {
        commands.remove_resource::<LoadingAssets>();
        next_screen.set(Screen::Title);
    }
}
```

### Asset Loading System

```rust
fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let textures = MyTextures {
        player: loading_assets.add(&asset_server.load("images/player.png")),
        background: loading_assets.add(&asset_server.load("images/background.png")),
    };

    commands.insert_resource(textures);
}
```

## Audio Systems

### Audio Resource Management

```rust
#[derive(Resource)]
pub struct AudioHandles {
    pub music: Vec<Handle<AudioSource>>,
    pub sfx: SfxHandles,
}

#[derive(Resource)]
pub struct SfxHandles {
    pub button_hover: Handle<AudioSource>,
    pub button_click: Handle<AudioSource>,
    pub footsteps: Vec<Handle<AudioSource>>,
}

fn load_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let audio_handles = AudioHandles {
        music: vec![
            loading_assets.add(&asset_server.load("audio/music/background.ogg")),
        ],
        sfx: SfxHandles {
            button_hover: loading_assets.add(&asset_server.load("audio/sfx/button_hover.ogg")),
            button_click: loading_assets.add(&asset_server.load("audio/sfx/button_click.ogg")),
            footsteps: vec![
                loading_assets.add(&asset_server.load("audio/sfx/step1.ogg")),
                loading_assets.add(&asset_server.load("audio/sfx/step2.ogg")),
            ],
        },
    };

    commands.insert_resource(audio_handles);
}
```

### Audio Playback

```rust
fn play_button_sound(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
    sfx: Res<SfxHandles>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                commands.spawn((
                    AudioPlayer::new(sfx.button_hover.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
            Interaction::Pressed => {
                commands.spawn((
                    AudioPlayer::new(sfx.button_click.clone()),
                    PlaybackSettings::DESPAWN,
                ));
            }
            _ => {}
        }
    }
}
```

## System Scheduling

### System Sets and Ordering

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    Input,
    Movement,
    Collision,
    Animation,
    Cleanup,
}

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                GameplaySet::Input,
                GameplaySet::Movement,
                GameplaySet::Collision,
                GameplaySet::Animation,
                GameplaySet::Cleanup,
            ).chain(),
        )
        .add_systems(
            Update,
            (
                handle_input.in_set(GameplaySet::Input),
                apply_movement.in_set(GameplaySet::Movement),
                check_collisions.in_set(GameplaySet::Collision),
                update_animations.in_set(GameplaySet::Animation),
            ).run_if(in_state(Screen::Gameplay)),
        );
    }
}
```

## Common Migration Gotchas

### 1. Bundle Deconstruction

**Problem:** Bundles are no longer automatically deconstructed in 0.16.

**Solution:** Manually specify individual components:

```rust
// Instead of SpriteBundle
commands.spawn((
    Sprite::default(),
    Transform::default(),
    Visibility::default(),
    // Add other components as needed
));
```

### 2. UI Node Styling

**Problem:** `NodeBundle` style properties are now separate components.

**Solution:** Use individual components:

```rust
commands.spawn((
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
));
```

### 3. Observer System Usage

**New Feature:** Use observers for event-driven programming:

```rust
// Register observer
commands.observe(handle_collision);

// Trigger event
commands.trigger_targets(CollisionEvent { damage: 10.0 }, entity);

// Handle event
fn handle_collision(
    trigger: Trigger<CollisionEvent>,
    mut health_query: Query<&mut Health>,
) {
    if let Ok(mut health) = health_query.get_mut(trigger.entity()) {
        health.0 -= trigger.event().damage;
    }
}
```

## Code Examples and Snippets

### Complete Player Movement System

```rust
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

fn player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    speed_query: Query<&MovementSpeed, With<Player>>,
) {
    if let (Ok(mut transform), Ok(speed)) = (
        player_query.get_single_mut(),
        speed_query.get_single(),
    ) {
        let mut direction = Vec3::ZERO;

        if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
            transform.translation += direction * speed.0 * time.delta_secs();
        }
    }
}
```

### Animation System

```rust
#[derive(Component)]
pub struct AnimationTimer(pub Timer);

#[derive(Component)]
pub struct AnimationFrames {
    pub current: usize,
    pub frames: Vec<usize>,
}

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut AnimationFrames, &mut Sprite)>,
) {
    for (mut timer, mut animation, mut sprite) in &mut query {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            animation.current = (animation.current + 1) % animation.frames.len();
            if let Some(texture_atlas) = &mut sprite.texture_atlas {
                texture_atlas.index = animation.frames[animation.current];
            }
        }
    }
}
```

### Menu Navigation System

```rust
fn handle_menu_input(
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    input: Res<ButtonInput<KeyCode>>,
    current_screen: Res<State<Screen>>,
    current_menu: Res<State<Menu>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        match (*current_screen.get(), *current_menu.get()) {
            (Screen::Title, Menu::Main) => {
                next_menu.set(Menu::None);
            }
            (Screen::Gameplay, Menu::None) => {
                next_menu.set(Menu::Pause);
            }
            (Screen::Gameplay, Menu::Pause) => {
                next_menu.set(Menu::None);
            }
            _ => {}
        }
    }
}
```

## Best Practices Summary

1. **Use Composition**: Prefer small, focused components over large bundles
2. **State Management**: Use clear state enums and conditional system execution
3. **Plugin Organization**: Split functionality into logical plugins
4. **Asset Tracking**: Implement proper asset loading and dependency management
5. **Observer Events**: Use the new observer system for event-driven architecture
6. **System Ordering**: Use system sets to ensure proper execution order
7. **Theme Consistency**: Implement reusable theming systems for UI
8. **Error Handling**: Use proper error handling patterns with Results and Options

This guide should help you migrate to Bevy 0.16 and establish good patterns for your game development projects. Remember to check the official migration guide for the most up-to-date breaking changes and new features.
