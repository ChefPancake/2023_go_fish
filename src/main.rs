use std::time::Duration;

use bevy::diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::{app::App, DefaultPlugins, time::Time};
use rand::prelude::*;

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

fn main() {
    App::new()
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .add_plugins((
        DefaultPlugins, 
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default()))
    .add_systems(Startup, (add_camera, add_fish))
    .add_systems(Update, (
        bevy::window::close_on_esc,
        apply_fish_movement,
        apply_velocity,
        apply_fish_boundaries,
        apply_fish_animation))
    .run();
}

fn add_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
pub struct FishMovement {
    pub next_move_time: Timer,
    pub vel_to_apply: f32
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub drag_x: f32,
    pub drag_y: f32,
}

#[derive(Component)]
pub struct FishBoundaries {
    pub min_x: f32,
    pub max_x: f32
}

#[derive(Component)]
pub struct FishAnimation {
    pub base_scale: f32
}

fn add_fish(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for _ in 0..10 {
        //TODO K: apply direction randomly to both movement and transform.scale
        let mut rng = rand::thread_rng();
        let pos_x = rng.gen::<f32>() * 1000.0 - 500.0;
        let pos_y = rng.gen::<f32>() * 500.0 - 250.0;
        let mut timer = Timer::from_seconds(rng.gen::<f32>() * 2.0 + 2.0, TimerMode::Repeating);
        timer.tick(Duration::from_secs_f32(rng.gen::<f32>() * 2.0));
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("fish.png"),
                transform: Transform::from_translation(
                    Vec3::new(
                        pos_x, 
                        pos_y, 
                        0.0)),
                ..default()
            },
            FishMovement {
                next_move_time: timer,
                vel_to_apply: -400.0
            },
            FishBoundaries {
                min_x: -500.0,
                max_x: 500.0
            },
            FishAnimation {
                base_scale: 1.0
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                drag_x: 150.0,
                drag_y: 1000.0
            }
        ));
    }
}

fn apply_fish_movement(
    mut query: Query<(&mut Velocity, &mut FishMovement)>,
    time: Res<Time>
) {
    for (mut velocity, mut movement) in &mut query {
        movement.next_move_time.tick(time.delta());
        if movement.next_move_time.just_finished() {
            velocity.x = movement.vel_to_apply;
        }
    }
}

fn apply_fish_animation(
    mut query: Query<(&mut FishAnimation, &FishMovement, &mut Transform)>,
) {
    for (mut anim, movement, mut transform) in &mut query {
        #[derive(PartialEq, Eq)]
        enum Facing {
            Left,
            Right
        }

        let facing = 
            if transform.scale.x > 0.0 {
                Facing::Left
            } else {
                Facing::Right
            };

        //right before going off, squish it
        //right after going off, stretch it
        let perc_left = movement.next_move_time.percent_left();
        let perc = movement.next_move_time.percent(); 
        let base = anim.base_scale;
        if perc_left < 0.15 {
            let anim_perc = 1.0 - (perc_left / 0.15);
            let anim_perc = anim_perc.powf(0.25);
            if facing == Facing::Right {
                transform.scale = Vec3::new(
                    -base + anim_perc * 0.3, 
                    base + anim_perc * 0.3, 
                    1.0)
            } else {
                transform.scale = Vec3::new(
                    base - anim_perc * 0.3, 
                    base + anim_perc * 0.3, 
                    1.0)
            }
        } else if perc < 0.05 {
            let anim_perc = perc / 0.05;
            //let anim_perc = anim_perc.powf(0.25);
            if facing == Facing::Right {
                transform.scale = Vec3::new(
                    -base + 0.3 - anim_perc * 0.6, 
                    base + 0.3 - anim_perc * 0.6, 
                    1.0)
            } else {
                transform.scale = Vec3::new(
                    base - 0.3 + anim_perc * 0.6, 
                    base + 0.3 - anim_perc * 0.6, 
                    1.0)
            }
        } else if perc < 0.40 {
            let anim_perc = perc / 0.40;
            let anim_perc = anim_perc.powf(0.25);
            if facing == Facing::Right {
                transform.scale = Vec3::new(
                    -base - 0.3 + anim_perc * 0.3, 
                    base - 0.3 + anim_perc * 0.3, 
                    1.0)
            } else {
                transform.scale = Vec3::new(
                    base + 0.3 - anim_perc * 0.3, 
                    base - 0.3 + anim_perc * 0.3, 
                    1.0)
            }
        } else {
            transform.scale =
                if facing == Facing::Right {
                    Vec3::new(
                        -anim.base_scale,
                        anim.base_scale,
                        1.0)
                } else {
                    Vec3::new(
                        anim.base_scale,
                        anim.base_scale,
                        1.0)
                };
        }
    }
}

fn apply_fish_boundaries(
    mut query: Query<(&mut Transform, &FishBoundaries, &mut Velocity, &mut FishMovement)>
) {
    for (mut transform, boundaries, mut velocity, mut movement) in &mut query {
        if transform.translation.x > boundaries.max_x {
            let diff = transform.translation.x - boundaries.max_x;
            transform.translation.x -= diff;
            transform.scale.x *= -1.0;
            velocity.x *= -1.0;
            movement.vel_to_apply *= -1.0;
        } else if transform.translation.x < boundaries.min_x {
            let diff = boundaries.min_x - transform.translation.x;
            transform.translation.x += diff;
            transform.scale.x *= -1.0;
            velocity.x *= -1.0;
            movement.vel_to_apply *= -1.0;
        }
    }
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &mut Velocity)>,
    time: Res<Time>
) {
    for (mut transform, mut velocity) in &mut query {
        let (drag_x, drag_y) = (velocity.drag_x, velocity.drag_y);
        apply_vel_with_drag(
            &mut transform.translation.x,
            &mut velocity.x,
            drag_x,
            time.delta_seconds()
        );
        apply_vel_with_drag(
            &mut transform.translation.y,
            &mut velocity.y,
            drag_y,
            time.delta_seconds()
        );
    }
}

//TODO K: look at making more efficient
/// positive drag reduces vel.abs(), negative increases
fn apply_vel_with_drag(pos: &mut f32, vel: &mut f32, drag: f32, delta_s: f32) {
    if *vel == 0.0 {
        return;
    }
    let to_subtract_from_vel = 
        drag * delta_s;
    let new_vel =     
        if to_subtract_from_vel > vel.abs() {
            0.0
        } else if *vel > 0.0 {
            *vel - to_subtract_from_vel
        } else {
            *vel + to_subtract_from_vel
        };
    let pos_traveled =
        if new_vel == 0.0 { //vel intercepted 0 after applying drag
            let hit_zero_at_t = delta_s * vel.abs() / to_subtract_from_vel;
            let pos_traveled = hit_zero_at_t * vel.abs() / 2.0;
            if *vel > 0.0 {  
                pos_traveled
            } else {
                -pos_traveled
            }
        } else { //new_vel.abs() is > 0
            let pos_traveled =
                new_vel.abs() * delta_s + to_subtract_from_vel * delta_s / 2.0;
            if *vel > 0.0 {  
                pos_traveled
            } else {
                -pos_traveled
            }
        };
    *pos += pos_traveled;
    *vel = new_vel;
}






