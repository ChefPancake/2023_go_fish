use std::time::Duration;
use bevy::diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::{app::App, DefaultPlugins, time::Time};
use rand::prelude::*;

const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const WATER_SIZE: Vec2 = Vec2::new(1450.0, 1200.0);
const WATER_POS: Vec2 = Vec2::new(365.0, -250.0);
const GRAVITY: f32 = 6000.0;
const LINE_START_POS: Vec2 = Vec2::new(-290.0, 638.0);
const BEAR_POS: Vec2 = Vec2::new(-520.0, 540.0);
const FISH_STACK_HEIGHT: f32 = 15.0;
const STACK_POS: Vec3 = Vec3::new(-1200.0, 300.0, -1.0);
const FISH_PER_LEVEL: usize = 10;
const WINDOW_SIZE: Vec2 = Vec2::new(1200.0, 850.0);
const BITE_DISTANCE: f32 = 30.0;

const FISH_ATLAS_SIZES: [usize; 10] = [
    10, 5,
    9,  4,
    8,  3,
    7,  2,
    6,  1,
];


fn main() {
    App::new()
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .insert_resource(ImageHandles::default())
    .add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GO FISH".to_string(),
                resolution: WindowResolution::new(WINDOW_SIZE.x, WINDOW_SIZE.y).with_scale_factor_override(0.4),
                ..default()
            }),
            ..default()
        }), 
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin::default()))
    .add_systems(Startup, 
        load_images)
    .add_systems(Startup, (
        add_camera,
        add_bg,
        add_bear,
        add_fish,
        add_hook,
        add_catch_stack)
        .after(load_images))
    .add_systems(Update, (
        bevy::window::close_on_esc,
        interpolate_flying_arc,
        fish_bite_hook,
        apply_fish_movement,
        apply_velocity,
        apply_fish_boundaries,
        apply_fish_animation,
        move_hook,
        turn_hook_pink,
        reel_in_fish,
        draw_fishing_line,
    ).before(catch_fish))
    .add_systems(Update,
        catch_fish)
    .add_systems(Update, 
        reset_level.after(catch_fish))
    .run();
}

fn add_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn load_images(
    mut images: ResMut<ImageHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>
) {
    images.bg_handle = Some(asset_server.load("background.png"));
    images.hook_handle = Some(asset_server.load("hook.png"));

    let fish_handle = asset_server.load("fish_atlas.png");
    let fish_atlas = TextureAtlas::from_grid(
        fish_handle.clone(),
        Vec2::new(600.0, 200.0),
        2,
        5,
        None, 
        None
    );
    images.fish_handle = Some(fish_handle);
    let fish_atlas_handle = atlases.add(fish_atlas);
    images.fish_atlas_handle = Some(fish_atlas_handle);

    let stack_handle = asset_server.load("stack_atlas.png");
    let stack_atlas = TextureAtlas::from_grid(
        stack_handle.clone(),
        Vec2::new(600.0, 200.0),
        2,
        5,
        None, 
        None
    );
    images.stack_handle = Some(stack_handle);
    let fish_atlas_handle = atlases.add(stack_atlas);
    images.stack_atlas_handle = Some(fish_atlas_handle);

    let bear_handle = asset_server.load("bear_atlas.png");
    let bear_atlas = TextureAtlas::from_grid(
        bear_handle.clone(), 
        Vec2::new(550.0, 450.0),
        3,
        2, 
        None, 
        None
    );
    let bear_atlas_handle = atlases.add(bear_atlas);
    images.bear_handle = Some(bear_handle);
    images.bear_atlas_handle = Some(bear_atlas_handle);
}

#[derive(Resource, Default)]
pub struct ImageHandles {
    pub fish_handle: Option<Handle<Image>>,
    pub hook_handle: Option<Handle<Image>>,
    pub bg_handle: Option<Handle<Image>>,
    pub bear_handle: Option<Handle<Image>>,
    pub stack_handle: Option<Handle<Image>>,

    pub bear_atlas_handle: Option<Handle<TextureAtlas>>,
    pub fish_atlas_handle: Option<Handle<TextureAtlas>>,
    pub stack_atlas_handle: Option<Handle<TextureAtlas>>,
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
pub struct Hooked {
    pub hook_time_s: f32
}

#[derive(Component, Debug)]
pub struct Flying {
    pub start_vel: Vec2,
    pub gravity: f32,
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub start_time_s: f32,
    pub end_time_s: f32
}

#[derive(Component, Debug)]
pub struct Reeling;

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

#[derive(Component, Default)]
pub struct CatchStack {
    pub total_fish: usize
}

fn reset_level(
    fish_query: Query<(), With<FishMovement>>,
    images: Res<ImageHandles>,
    commands: Commands,
) {
    if fish_query.is_empty() {
        add_fish(images, commands);
    }
}

fn add_bg(
    images: Res<ImageHandles>,
    mut commands: Commands
) {
    let image_handle = images.bg_handle.as_ref().expect("images should be loaded");
    commands.spawn(
        SpriteBundle {
            texture: image_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1000.0)),
            ..default()
        }
    );
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

fn add_catch_stack(
    mut commands: Commands
){
    commands.spawn(
        (
        Transform::from_translation(STACK_POS),
        CatchStack::default())
    );
}

fn add_fish(
    images: Res<ImageHandles>,
    mut commands: Commands,
) {
    let fish_atlas_handle = images.fish_atlas_handle.as_ref().expect("Images should be loaded");
    let box_width = WATER_SIZE.x * 0.9;
    let box_height = WATER_SIZE.y * 0.9;
    let lane_height = box_height / FISH_PER_LEVEL as f32;
    let mut rng = rand::thread_rng();
    let mut fish_indexes: Vec<usize> = (0..FISH_PER_LEVEL).collect();
    fish_indexes.sort_by_key(|_| (rng.gen::<f32>() * FISH_PER_LEVEL as f32 * 1000.0) as usize);
    println!("{:?}", fish_indexes);
    for (pos_index, fish_index) in fish_indexes.iter().enumerate() {
        println!("{:?}", pos_index);
        println!("{:?}", fish_index);
        let fish_half_width = (FISH_ATLAS_SIZES[*fish_index] - 1) as f32 * 20.0 + 30.0;
        let pos_x = rng.gen::<f32>() * box_width - (box_width / 2.0) + WATER_POS.x;
        let pos_y = WATER_POS.y - box_height / 2.0 + lane_height * pos_index as f32 + rng.gen::<f32>() * lane_height * 0.8;
        let mut timer = Timer::from_seconds(rng.gen::<f32>() * 6.0 + 3.0, TimerMode::Repeating);
        timer.tick(Duration::from_secs_f32(rng.gen::<f32>() * 9.0));
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: fish_atlas_handle.clone(),
                sprite: TextureAtlasSprite::new(*fish_index),
                transform: Transform::from_translation(
                    Vec3::new(
                        pos_x, 
                        pos_y, 
                        -(FISH_ATLAS_SIZES[*fish_index] as f32))),
                ..default()
            },
            FishMouthPosition {
                offset_x: fish_half_width,
                offset_y: 20.0
            },
            FishMovement {
                next_move_time: timer,
                vel_to_apply: 250.0
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
            Velocity {
                x: 0.0,
                y: 0.0,
                drag_x: 25.0,
                drag_y: 1000.0
            }
        ));
    }
}

fn draw_fishing_line(
    hook_query: Query<&Transform, With<Hook>>,
    mut gizmos: Gizmos
) {
    if let Ok(hook_pos) = hook_query.get_single() {
        let hook_pos = hook_pos.translation;
        let visual_surface_y = WATER_POS.y + WATER_SIZE.y / 2.0 - 40.0;
        let distance_to_hook_x = LINE_START_POS.x - hook_pos.x;
        let distance_to_surface_y = LINE_START_POS.y - visual_surface_y;

        let node_near_pole = Vec2::new(
            hook_pos.x + 0.9 * distance_to_hook_x, 
            visual_surface_y + 0.3 * distance_to_surface_y,
        );
        let node_near_surface = Vec2::new(
            hook_pos.x + 0.4 * distance_to_hook_x, 
            visual_surface_y + 0.1 * distance_to_surface_y,
        );
        let node_at_surface = Vec2::new(hook_pos.x, visual_surface_y);
        let points = [[
            LINE_START_POS, 
            node_near_pole,
            node_near_surface,
            node_at_surface,
        ]];
        let bezier = Bezier::new(points);
        gizmos.linestrip_2d(bezier.to_curve().iter_positions(50), Color::GRAY);
        gizmos.line_2d(node_at_surface, Vec2::new(hook_pos.x, hook_pos.y + 25.0), Color::GRAY);
    }
}

fn calculate_time_and_initial_vel_for_arc(
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    gravity_y: f32,
    max_y: f32,
) -> (Vec2, f32) {
    debug_assert!(start_y < max_y);
    debug_assert!(end_y < max_y);
    debug_assert_ne!(0.0, gravity_y);

    //calculate these times as trajectory functions originating from the apex
    //their sum is the total arc time, from which we can derive start vels

    let time_to_apex = (2.0 / gravity_y * (max_y - start_y)).sqrt();
    let time_from_apex = (2.0 / gravity_y * (max_y - end_y)).sqrt();
    let total_time = time_to_apex + time_from_apex;

    let vel_x = (end_x - start_x) / total_time;
    let vel_y = gravity_y * time_to_apex;

    return (Vec2::new(vel_x, vel_y), total_time);
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
    time: Res<Time>,
    fish_query: Query<(Entity, &Transform, &FishMouthPosition), Without<Hooked>>,
    hook_query: Query<(Entity, &Transform), (With<Hook>, Without<NearFish>)>,
    mut commands: Commands
) {
    for (hook_entity, hook) in &hook_query {
        for (fish_entity, fish, mouth_pos) in &fish_query {
            let distance = get_distance_to_fish_mouth(
                &hook.translation,
                &fish.translation,
                fish.scale.x,
                mouth_pos);
            if distance < BITE_DISTANCE {
                commands.entity(fish_entity).insert(Hooked { hook_time_s: time.elapsed_seconds() });
                commands.entity(hook_entity).insert(NearFish);
                break;
            } else {
                commands.entity(fish_entity).remove::<Hooked>();
            }
        }
    }
}

fn reel_in_fish(
    mut commands: Commands,
    time: Res<Time>,
    mut fish_query: Query<(Entity, &mut Transform), (With<Reeling>, Without<CatchStack>)>,
    mut catch_stack: Query<(&Transform, &mut CatchStack), With<CatchStack>>
) {
    let reel_speed = 600.0;
    let upper_boundary = WATER_POS.y + WATER_SIZE.y / 2.0;
    for (entity, mut fish_pos) in &mut fish_query {
        //move the fish straight up at reel_speed
        //if the fish hits the surface, send it flying
        let mut hit_surface = false;

        let mut new_y = fish_pos.translation.y + reel_speed * time.delta_seconds();
        if new_y > upper_boundary {
            hit_surface = true;
            new_y = upper_boundary;
        }
        fish_pos.translation.y = new_y;
        if hit_surface {
            let (catch_stack_pos, mut catch_stack) = catch_stack.single_mut();
            catch_stack.total_fish += 1;
            let catch_stack_pos = catch_stack_pos.translation;
            let catch_stack_pos = Vec3::new(
                catch_stack_pos.x, 
                catch_stack_pos.y + (catch_stack.total_fish as f32) * FISH_STACK_HEIGHT,
                catch_stack_pos.z);
            commands.entity(entity).remove::<Reeling>();
            send_fish_to_stack(fish_pos.translation, catch_stack_pos, GRAVITY, time.elapsed_seconds(), &mut commands, entity);
        }
    }
}

fn send_fish_to_stack(fish_pos: Vec3, catch_stack_pos: Vec3, gravity: f32, elapsed_time: f32, commands: &mut Commands, entity: Entity) {
    let (arc_vel, arc_time) = calculate_time_and_initial_vel_for_arc(fish_pos.x, fish_pos.y, catch_stack_pos.x, catch_stack_pos.y, gravity, 900.0);
    let flying = Flying {
        start_vel: arc_vel,
        gravity,
        start_pos: Vec2::new(fish_pos.x, fish_pos.y),
        end_pos: Vec2::new(catch_stack_pos.x, catch_stack_pos.y),
        start_time_s: elapsed_time,
        end_time_s: elapsed_time + arc_time,
    };
    commands.entity(entity).insert(flying);
}

fn catch_fish(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    fish_query: Query<(Entity, &Hooked, &Transform)>,
    mut catch_stack: Query<(&Transform, &mut CatchStack)>,
    hook_query: Query<Entity, (With<Hook>, With<NearFish>)>,
) {
    let critical_time = 0.07;
    if let Ok(hook_entity) = hook_query.get_single() {
        if input.just_pressed(KeyCode::Space) {
            commands.entity(hook_entity).remove::<NearFish>();
            let (catch_stack_pos, mut catch_stack) = catch_stack.single_mut();
            for (entity, hooked, fish_pos) in &fish_query {
                commands.entity(entity).remove::<(Hooked, FishMovement, FishMouthPosition, Velocity)>();
                let react_time = time.elapsed_seconds() - hooked.hook_time_s;
                if react_time < critical_time {
                    catch_stack.total_fish += 1;
                    let catch_stack_pos = catch_stack_pos.translation;
                    let catch_stack_pos = Vec3::new(
                        catch_stack_pos.x, 
                        catch_stack_pos.y + (catch_stack.total_fish as f32) * FISH_STACK_HEIGHT,
                        catch_stack_pos.z);
                    send_fish_to_stack(fish_pos.translation, catch_stack_pos, GRAVITY, time.elapsed_seconds(), &mut commands, entity)
                } else {
                    commands.entity(entity).insert(Reeling);
                }
            }
        }
    }
}

fn interpolate_flying_arc(
    mut commands: Commands,
    time: Res<Time>,
    images: Res<ImageHandles>,
    mut flying_query: Query<(Entity, &mut Transform, &Flying)>
) {
    for (entity, mut transform, flying) in &mut flying_query {
        if time.elapsed_seconds() > flying.end_time_s {
            //TODO: K: move sprite change behind an event
            commands.entity(entity).remove::<(Flying, Handle<TextureAtlas>)>();
            commands.entity(entity).insert(images.stack_atlas_handle.as_ref().expect("Images should be loaded").clone());
            transform.translation = Vec3::new(flying.end_pos.x, flying.end_pos.y, transform.translation.z);
            transform.scale = Vec3::new(1.0, 1.0, 1.0);
        } else {
            let elapsed = time.elapsed_seconds() - flying.start_time_s;
            let new_pos_x = flying.start_pos.x + (elapsed * flying.start_vel.x);
            let new_pos_y = flying.start_pos.y + (elapsed * flying.start_vel.y) - (flying.gravity * elapsed * elapsed / 2.0 );
            transform.translation = Vec3::new(new_pos_x, new_pos_y, transform.translation.z);
        }
    }
}

fn get_distance_to_fish_mouth(
    from: &Vec3, 
    fish_pos: &Vec3, 
    fish_scale_x: f32, 
    fish_mouth: &FishMouthPosition
) -> f32 {
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
