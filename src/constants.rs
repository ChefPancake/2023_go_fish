use bevy::{math::{Vec2, Vec3}, render::color::Color};

pub fn background_color() -> Color {
    Color::hex(BACKGROUND_COLOR).expect("failed to parse as hex")
}
pub const BACKGROUND_COLOR: &str = "C9D1C4";
pub const BACKGROUND_SIZE: Vec2 = Vec2::new(2732.0, 2048.0);
pub const WATER_SIZE: Vec2 = Vec2::new(1450.0, 1200.0);
pub const WATER_POS: Vec2 = Vec2::new(377.0, -250.0);
pub const GRAVITY: f32 = 6000.0;
pub const WATER_DRAG_Y: f32 = 12000.0;
pub const WATER_DRAG_X: f32 = 50.0;
pub const CAST_TARGET_POS: Vec2 = Vec2::new(300.0, 220.0);
pub const BEAR_POS: Vec2 = Vec2::new(-520.0, 540.0);
pub const STACK_POS: Vec3 = Vec3::new(-1000.0, 435.0, -1.0);
pub const FISH_PER_LEVEL: usize = 10;
pub const FISH_VELOCITY: f32 = 500.0;
pub const CRITICAL_TIME: f32 = 0.07;
pub const FISH_ATLAS_SIZES: [usize; 10] = [
    10, 5,
    9,  4,
    8,  3,
    7,  2,
    6,  1,
];
pub const FISH_MOUTH_POSITIONS_AND_SIZES: [(Vec2, f32); 10] = [
    (Vec2::new(0.0, 0.0), 20.0),
    (Vec2::new(20.0, 0.0), 30.0),
    (Vec2::new(20.0, 0.0), 35.0),
    (Vec2::new(70.0, 0.0), 35.0),
    (Vec2::new(70.0, 0.0), 35.0),
    (Vec2::new(110.0, 0.0), 35.0),
    (Vec2::new(110.0, 0.0), 40.0),
    (Vec2::new(150.0, 0.0), 40.0),
    (Vec2::new(150.0, 0.0), 50.0),
    (Vec2::new(200.0, 0.0), 45.0),
];
pub const FISH_STACK_SIZES: [f32; 10] = [
    10.0,
    15.0,
    15.0,
    20.0,
    25.0,
    30.0,
    35.0,
    35.0,
    35.0,
    50.0,
];
pub const LEVEL_LENGTH_S: f32 = 100.0;
pub const SNAIL_START_POS: Vec2 = Vec2::new(-80.0, -850.0);
pub const SNAIL_END_POS: Vec2 = Vec2::new(875.0, SNAIL_START_POS.y);
pub const SNAIL_SPEED: f32 = (SNAIL_END_POS.x - SNAIL_START_POS.x) / LEVEL_LENGTH_S;
pub const CLOUD_END_X: f32 = 1500.0;
pub const CLOUD_START_X: f32 = -CLOUD_END_X;
pub const CLOUD_Y: f32 = 700.0;
