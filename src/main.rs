mod catch_stack;
mod constants;
mod core;
mod fish;
mod hook;
mod physics;

use bevy::prelude::*;
use bevy::diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::window::WindowResolution;

use catch_stack::*;
use constants::*;
use core::*;
use fish::*;
use hook::*;
use physics::*;


fn main() {
    App::new()
    .add_event::<FishLandedInStack>()
    .add_event::<FishKnockedOutOfStack>()
    .add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GO FISH".to_string(),
                resolution: WindowResolution::new(WINDOW_SIZE.x, WINDOW_SIZE.y).with_scale_factor_override(0.4),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }), 
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default(),
        CorePlugin,
        PhysicsPlugin,
        FishPlugin,
        HookPlugin,
        CatchStackPlugin
    ))
    .add_systems(Startup,
        add_bear)
    .add_systems(Update, (
        bevy::window::close_on_esc,
    ))
    .run();
}

fn add_bear(
    handles: Res<ImageHandles>,
    mut commands: Commands
) {
    let atlas_handle = handles.bear_atlas_handle.as_ref().expect("Images should be loaded");
    commands.spawn(SpriteSheetBundle {
        texture_atlas: atlas_handle.clone(),
        sprite: TextureAtlasSprite::new(0),
        transform: Transform::from_translation(Vec3::new(BEAR_POS.x, BEAR_POS.y, 10.0)), 
        ..default()
    });
}
