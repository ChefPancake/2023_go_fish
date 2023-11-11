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
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GO FISH".to_string(),
                ..default()
            }),
            ..default()
        }), 
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default()))
    .add_systems(Startup, (
        add_camera,
        add_fish,
        add_hook))
    .add_systems(Update, (
        bevy::window::close_on_esc,
        fish_bite_hook,
        apply_fish_movement,
        apply_velocity,
        apply_fish_boundaries,
        apply_fish_animation,
        move_hook,
        turn_hook_pink,
    ))
    .add_systems(Update,
        catch_fish)
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
pub struct Hook {
    pub move_speed: f32
}

#[derive(Component)]
pub struct NearFish;

#[derive(Component)]
pub struct Hooked;

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
    pub base_scale: f32,
    pub max_scale_add_x: f32,
    pub max_scale_add_y: f32,
    pub charge_anim_time_s: f32,
    pub dash_anim_time_s: f32,
    pub reset_anim_time_s: f32,
}

#[derive(Component)]
pub struct FishMouthPosition {
    pub offset_x: f32,
    pub offset_y: f32,
}

fn add_fish(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    for _ in 0..10 {
        let mut rng = rand::thread_rng();
        let pos_x = rng.gen::<f32>() * 1000.0 - 500.0;
        let pos_y = rng.gen::<f32>() * 500.0 - 250.0;
        let mut timer = Timer::from_seconds(rng.gen::<f32>() * 3.0 + 6.0, TimerMode::Repeating);
        timer.tick(Duration::from_secs_f32(rng.gen::<f32>() * 6.0));
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
            FishMouthPosition {
                offset_x: -60.0,
                offset_y: 20.0
            },
            FishMovement {
                next_move_time: timer,
                vel_to_apply: -250.0
            },
            FishBoundaries {
                min_x: -500.0,
                max_x: 500.0
            },
            FishAnimation {
                base_scale: 1.0,
                max_scale_add_x: 0.3,
                max_scale_add_y: 0.3,
                charge_anim_time_s: 0.3,
                dash_anim_time_s: 0.2,
                reset_anim_time_s: 2.0,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                drag_x: 25.0,
                drag_y: 1000.0
            }
        ));
    }
}

fn turn_hook_pink(
    mut hooks: Query<(&mut Sprite, Option<&NearFish>), With<Hook>>
) {
    for (mut hook_sprite, near_fish) in &mut hooks {
        if near_fish.is_some() {
            hook_sprite.color = Color::PINK;
        } else {
            hook_sprite.color = Color::WHITE;
        }
    }
}

fn add_hook(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("hook.png"),
            transform: 
                Transform { 
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(2.0, 2.0, 1.0),
                    ..default()
                },
            ..default()
        },
        Hook {
            move_speed: 300.0
        }
    ));
}

fn move_hook(
    mut query: Query<(&mut Transform, &Hook), Without<NearFish>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>
) {
    let left_pressed = input.pressed(KeyCode::A) || input.pressed(KeyCode::Left);
    let right_pressed = input.pressed(KeyCode::D) || input.pressed(KeyCode::Right);
    let up_pressed = input.pressed(KeyCode::W) || input.pressed(KeyCode::Up);
    let down_pressed = input.pressed(KeyCode::S) || input.pressed(KeyCode::Down);
    let x_vel = (if left_pressed { -1.0 } else { 0.0 } + if right_pressed { 1.0 } else { 0.0 });
    let y_vel = (if up_pressed { 1.0 } else { 0.0 } + if down_pressed { -1.0 } else { 0.0 });
    let vel_vec = Vec3::new(x_vel, y_vel, 0.0).normalize_or_zero() * time.delta_seconds();
    
    for (mut transform, hook) in &mut query {
        transform.translation += vel_vec * hook.move_speed;
    }
}

fn fish_bite_hook(
    fish_query: Query<(Entity, &Transform, &FishMouthPosition)>,
    hook_query: Query<(Entity, &Transform), With<Hook>>,
    mut commands: Commands
) {
    for (hook_entity, hook) in &hook_query {
        let mut is_near_fish = false;
        for (fish_entity, fish, mouth_pos) in &fish_query {
            let distance = get_distance_to_fish_mouth(
                &hook.translation,
                &fish.translation,
                fish.scale.x,
                mouth_pos);
            if distance < 30.0 {
                is_near_fish = true;
                commands.entity(fish_entity).insert(Hooked);
            } else {
                commands.entity(fish_entity).remove::<Hooked>();
            }
        }
        if is_near_fish {
            commands.entity(hook_entity).insert(NearFish);
        } else {
            commands.entity(hook_entity).remove::<NearFish>();
        }
    }
}

fn catch_fish(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    fish_query: Query<Entity, With<Hooked>>,
) {
    if input.just_pressed(KeyCode::Space) {   
        for entity in &fish_query {
            commands.entity(entity).despawn();
        }
    }
}

fn get_distance_to_fish_mouth(from: &Vec3, fish_pos: &Vec3, fish_scale_x: f32, fish_mouth: &FishMouthPosition) -> f32 {
    let mouth_offset_x = 
        if fish_scale_x < 0.0 {
            fish_mouth.offset_x
        } else {
            -fish_mouth.offset_x
        };
    let rel_mouth_pos = Vec3::new(mouth_offset_x, fish_mouth.offset_y, 0.0);
    (*from + rel_mouth_pos - *fish_pos).length()
}

fn apply_fish_movement(
    mut query: Query<(&mut Velocity, &mut FishMovement), Without<Hooked>>,
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
    mut query: Query<(&FishAnimation, &FishMovement, &mut Transform), Without<Hooked>>,
) {
    for (anim, movement, mut transform) in &mut query {
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
        //after stretching, relax it
        let time = movement.next_move_time.elapsed();
        let time_left = movement.next_move_time.duration() - time;
        let time_s = time.as_secs_f32();
        let time_left_s = time_left.as_secs_f32();
        
        let base = anim.base_scale;
        if time_left_s < anim.charge_anim_time_s {
            let anim_perc = 1.0 - (time_left_s / anim.charge_anim_time_s);
            let anim_perc = anim_perc.powf(0.25);
            if facing == Facing::Right {
                transform.scale = Vec3::new(
                    -base + anim_perc * anim.max_scale_add_x, 
                    base + anim_perc * anim.max_scale_add_y, 
                    1.0)
            } else {
                transform.scale = Vec3::new(
                    base - anim_perc * anim.max_scale_add_x, 
                    base + anim_perc * anim.max_scale_add_y, 
                    1.0)
            }
        } else if time_s < anim.dash_anim_time_s {
            let anim_perc = time_s / anim.dash_anim_time_s;
            if facing == Facing::Right {
                transform.scale = Vec3::new(
                    -base + anim.max_scale_add_x - anim_perc * 2.0 * anim.max_scale_add_x, 
                    base + anim.max_scale_add_y - anim_perc * 2.0 * anim.max_scale_add_y, 
                    1.0)
            } else {
                transform.scale = Vec3::new(
                    base - anim.max_scale_add_x + anim_perc * 2.0 * anim.max_scale_add_x, 
                    base + anim.max_scale_add_y - anim_perc * 2.0 * anim.max_scale_add_y, 
                    1.0)
            }
        } else if time_s < anim.reset_anim_time_s {
            let anim_perc = time_s / anim.reset_anim_time_s;
            let anim_perc = anim_perc.powf(0.5);
            if facing == Facing::Right {
                transform.scale = Vec3::new(
                    -base - anim.max_scale_add_x + anim_perc * anim.max_scale_add_x, 
                    base - anim.max_scale_add_y + anim_perc * anim.max_scale_add_y, 
                    1.0)
            } else {
                transform.scale = Vec3::new(
                    base + anim.max_scale_add_x - anim_perc * anim.max_scale_add_x, 
                    base - anim.max_scale_add_y + anim_perc * anim.max_scale_add_y, 
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
    mut query: Query<(&mut Transform, &mut Velocity), Without<Hooked>>,
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
