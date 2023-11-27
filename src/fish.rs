
use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::constants::*;
use crate::physics::*;
use crate::core::*;


pub struct FishPlugin;

impl Plugin for FishPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<FishReturnedToWater>()
        .add_systems(Startup, 
            add_fish
        )
        .add_systems(Update, (
            apply_fish_movement,
            apply_fish_boundaries,
            apply_fish_animation,
            interpolate_returning_to_water_arcs,
        ))
        .add_systems(PostUpdate, (
            handle_fish_returned_to_water,
            reset_fish
        ));
    }
}

#[derive(Event)]
pub struct FishReturnedToWater {
    pub fish_entity: Entity,
    pub end_vel: Vec2
}


#[derive(Component)]
pub struct FishMovement {
    pub next_move_time: Timer,
    pub vel_to_apply: f32
}


#[derive(Component)]
pub struct FishBoundaries {
    pub min_x: f32,
    pub max_x: f32
}

#[derive(Component)]
pub struct FishLanePos {
    pub pos_y: f32
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

#[derive(Component)]
pub struct FishSize {
    pub size: usize
}

#[derive(Component, Debug)]
pub struct ReturningToWater {
    pub start_vel: Vec2,
    pub water_entrance_vel: Vec2,
    pub start_pos: Vec2,
    pub water_entrance_pos: Vec2,
    pub end_pos: Vec2,
    pub start_time_s: f32,
    pub water_entrance_time_s: f32,
    pub end_time_s: f32,
    pub gravity: f32,
    pub water_drag: f32,
}

fn reset_fish(
    mut completed_events: EventReader<ResetLevel>,
    fish_query: Query<Entity, With<FishSize>>,
    images: Res<ImageHandles>,
    mut commands: Commands,
) {
    if !completed_events.is_empty() {
        completed_events.clear();
        for fish_entity in &fish_query {
            commands.entity(fish_entity).despawn();
        }
        add_fish(images, commands);
    }
}

fn add_fish(
    images: Res<ImageHandles>,
    mut commands: Commands,
) {
    let fish_atlas_handle = images.fish_atlas_handle.as_ref().expect("Images should be loaded");
    let height_offset = 100.0;

    let box_width = WATER_SIZE.x;
    let box_height = WATER_SIZE.y - height_offset;
    let lane_height = box_height / FISH_PER_LEVEL as f32;
    let mut rng = rand::thread_rng();
    let mut fish_and_sort: Vec<(usize, usize)> = 
        (0..FISH_PER_LEVEL)
        .map(|f| (f, (rng.gen::<f32>() * FISH_PER_LEVEL as f32 * 10000.0) as usize))
        .collect();
    fish_and_sort.sort_by_key(|(_, key)| *key);
    for (pos_index, fish_index) in fish_and_sort.iter().map(|(item, _)| item).enumerate() {
        let fish_size = FISH_ATLAS_SIZES[*fish_index];
        let fish_half_width = (fish_size - 1) as f32 * 20.0 + 30.0;
        let pos_x = rand::random::<f32>() * box_width - (box_width / 2.0) + WATER_POS.x;
        let pos_y = WATER_POS.y - (box_height - height_offset) / 2.0 - height_offset + lane_height * pos_index as f32 + rng.gen::<f32>() * lane_height * 0.8;
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: fish_atlas_handle.clone(),
                sprite: TextureAtlasSprite::new(*fish_index),
                transform: Transform::from_translation(
                    Vec3::new(
                        pos_x, 
                        pos_y, 
                        -(fish_size as f32))),
                ..default()
            },
            build_fish_mouth_position(fish_size),
            FishMovement {
                next_move_time: build_fish_movement_timer(&mut rng),
                vel_to_apply: FISH_VELOCITY
            },
            FishBoundaries {
                min_x: -WATER_SIZE.x / 2.0 + WATER_POS.x + fish_half_width,
                max_x: WATER_SIZE.x / 2.0 + WATER_POS.x - fish_half_width,
            },
            FishAnimation {
                base_scale: 1.0,
                max_scale_add_x: 0.3,
                max_scale_add_y: 0.3,
                charge_anim_time_s: 0.3,
                dash_anim_time_s: 0.2,
                reset_anim_time_s: 2.0,
            },
            FishLanePos {
                pos_y
            },
            FishSize {
                size: fish_size,
            },
            Velocity {
                x: 0.0,
                y: 0.0,
                drag_x: WATER_DRAG_X,
                drag_y: WATER_DRAG_Y
            }
        ));
    }
}


fn build_fish_movement_timer(rng: &mut ThreadRng) -> Timer {
    let mut timer = Timer::from_seconds(rng.gen::<f32>() * 6.0 + 3.0, TimerMode::Repeating);
    timer.tick(Duration::from_secs_f32(rng.gen::<f32>() * 9.0));
    timer
}

fn build_fish_mouth_position(fish_size: usize) -> FishMouthPosition {
    let fish_half_width = (fish_size - 1) as f32 * 20.0 + 30.0;
    FishMouthPosition {
        offset_x: fish_half_width,
        offset_y: 20.0
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
    mut query: Query<(&FishAnimation, &FishMovement, &mut Transform)>,
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

fn interpolate_returning_to_water_arcs(
    time: Res<Time>,
    mut returning_query: Query<(Entity, &mut Transform, &ReturningToWater)>,
    mut on_returned: EventWriter<FishReturnedToWater>
) {
    for (entity, mut transform, returning) in &mut returning_query {
        if time.elapsed_seconds() > returning.end_time_s {
            on_returned.send(FishReturnedToWater { 
                fish_entity: entity, 
                end_vel: Vec2::new(returning.start_vel.x, 0.0) 
            });
            transform.translation = Vec3::new(returning.end_pos.x, returning.end_pos.y, transform.translation.z);
            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        } else if time.elapsed_seconds() > returning.water_entrance_time_s {
            let elapsed = time.elapsed_seconds() - returning.water_entrance_time_s;
            let new_pos_x = returning.water_entrance_pos.x + (elapsed * returning.water_entrance_vel.x);
            let new_pos_y = returning.water_entrance_pos.y + (elapsed * returning.water_entrance_vel.y) + (returning.water_drag * elapsed * elapsed / 2.0);
            transform.translation = Vec3::new(new_pos_x, new_pos_y, transform.translation.z);
        } else {
            let elapsed = time.elapsed_seconds() - returning.start_time_s;
            let new_pos_x = returning.start_pos.x + (elapsed * returning.start_vel.x);
            let new_pos_y = returning.start_pos.y + (elapsed * returning.start_vel.y) - (returning.gravity * elapsed * elapsed / 2.0);
            transform.translation = Vec3::new(new_pos_x, new_pos_y, transform.translation.z);
        }
    }
}

fn handle_fish_returned_to_water(
    mut on_returned: EventReader<FishReturnedToWater>,
    images: Res<ImageHandles>,
    fish_query: Query<(Entity, &FishSize)>,
    mut commands: Commands,
) {
    for event in on_returned.iter() {
        for (fish_entity, fish_size) in &fish_query {
            if fish_entity == event.fish_entity {   
                let mut rng = rand::thread_rng();
                commands.entity(event.fish_entity).remove::<(ReturningToWater, Handle<TextureAtlas>)>();
                commands.entity(event.fish_entity).insert((
                    Velocity { 
                        x: event.end_vel.x,
                        y: event.end_vel.y,
                        drag_x: WATER_DRAG_X,
                        drag_y: WATER_DRAG_Y
                    },
                    FishMovement {
                        next_move_time: build_fish_movement_timer(&mut rng),
                        vel_to_apply: FISH_VELOCITY
                    },
                    build_fish_mouth_position(fish_size.size),
                    images.fish_atlas_handle.as_ref().expect("Images should be loaded").clone()
                ));
            }
        }
    }
}