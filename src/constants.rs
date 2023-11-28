use bevy::{math::{Vec2, Vec3}, render::color::Color};

pub const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const WATER_SIZE: Vec2 = Vec2::new(1450.0, 1200.0);
pub const WATER_POS: Vec2 = Vec2::new(377.0, -250.0);
pub const GRAVITY: f32 = 6000.0;
pub const WATER_DRAG_Y: f32 = 4000.0;
pub const WATER_DRAG_X: f32 = 50.0;
pub const CAST_TARGET_POS: Vec2 = Vec2::new(300.0, 220.0);
pub const BEAR_POS: Vec2 = Vec2::new(-520.0, 540.0);
pub const FISH_STACK_HEIGHT: f32 = 15.0;
pub const STACK_POS: Vec3 = Vec3::new(-1000.0, 435.0, -1.0);
pub const FISH_PER_LEVEL: usize = 10;
pub const WINDOW_SIZE: Vec2 = Vec2::new(1100.0, 800.0);
pub const BITE_DISTANCE: f32 = 30.0;
pub const FISH_VELOCITY: f32 = 500.0;
pub const CRITICAL_TIME: f32 = 0.07;
pub const FISH_ATLAS_SIZES: [usize; 10] = [
    10, 5,
    9,  4,
    8,  3,
    7,  2,
    6,  1,
];