#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod bear;
mod catch_stack;
mod clouds;
mod constants;
mod core;
mod fish;
mod hook;
mod physics;
mod snail;
mod window;

use bevy::prelude::*;
#[cfg(debug_assertions)]
use bevy::diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};

use audio::*;
use bear::*;
use catch_stack::*;
use clouds::*;
use core::*;
use fish::*;
use hook::*;
use physics::*;
use snail::*;
use window::*;

fn main() {
    App::new()
    .add_plugins((
        #[cfg(debug_assertions)]
        LogDiagnosticsPlugin::default(),
        #[cfg(debug_assertions)]
        FrameTimeDiagnosticsPlugin::default(),
        HSLWindowPlugin,
        CorePlugin,
        HSLAudioPlugin,
        SnailPlugin,
        CloudsPlugin,
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
