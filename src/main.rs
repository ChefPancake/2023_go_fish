use bevy::diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::{app::App, DefaultPlugins, time::Time};

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
        apply_velocity))
    .run();
}

fn add_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
pub struct Fish;


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

fn add_fish(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("fish.png"),
            transform: Transform::from_translation(Vec3::new(400.0, 100.0, 0.0)),
            ..default()
        },
        Fish,
        FishMovement {
            next_move_time: Timer::from_seconds(2.0, TimerMode::Repeating),
            vel_to_apply: -200.0
        },
        Velocity {
            x: 0.0,
            y: 0.0,
            drag_x: 75.0,
            drag_y: 1000.0
        }
    ));
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

fn apply_velocity(
    mut query: Query<(&mut Transform, &mut Velocity)>,
    time: Res<Time>
) {
    for (mut transform, mut velocity) in &mut query {
        let to_subtract_from_vel_x =
            velocity.drag_x * time.delta_seconds();
        if to_subtract_from_vel_x > velocity.x.abs() {

        }
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








