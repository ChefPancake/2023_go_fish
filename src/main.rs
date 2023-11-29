mod audio;
mod bear;
mod catch_stack;
mod constants;
mod core;
mod fish;
mod hook;
mod physics;

use bevy::prelude::*;
use bevy::window::WindowResolution;
#[cfg(debug_assertions)]
use bevy::diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use audio::*;
use bear::*;
use catch_stack::*;
use constants::*;
use core::*;
use fish::*;
use hook::*;
use physics::*;


fn main() {
    App::new()
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
        #[cfg(debug_assertions)]
        WorldInspectorPlugin::default(),
        #[cfg(debug_assertions)]
        LogDiagnosticsPlugin::default(),
        #[cfg(debug_assertions)]
        FrameTimeDiagnosticsPlugin::default(),
        CorePlugin,
        HSLAudioPlugin,
        PhysicsPlugin,
        FishPlugin,
        HookPlugin,
        CatchStackPlugin,
        BearPlugin
    ))
    .add_systems(Update, (
        bevy::window::close_on_esc,
    ))
    .run();
}
