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
const WATER_DRAG_Y: f32 = 4000.0;
const WATER_DRAG_X: f32 = 100.0;
const LINE_START_POS: Vec2 = Vec2::new(-290.0, 638.0);
const CAST_TARGET_POS: Vec2 = Vec2::new(300.0, 0.0);
const BEAR_POS: Vec2 = Vec2::new(-520.0, 540.0);
const FISH_STACK_HEIGHT: f32 = 15.0;
const STACK_POS: Vec3 = Vec3::new(-1100.0, 300.0, -1.0);
const FISH_PER_LEVEL: usize = 10;
const WINDOW_SIZE: Vec2 = Vec2::new(1100.0, 800.0);
const BITE_DISTANCE: f32 = 30.0;
const FISH_VELOCITY: f32 = 500.0;
const CRITICAL_TIME: f32 = 0.07;
const FISH_ATLAS_SIZES: [usize; 10] = [
    10, 5,
    9,  4,
    8,  3,
    7,  2,
    6,  1,
];

fn main() {
    App::new()
    .add_event::<FishLandedInStack>()
    .add_event::<FishCaught>()
    .add_event::<ReeledToSurface>()
    .add_event::<HookedFish>()
    .add_event::<StackCompleted>()
    .add_event::<FishKnockedOutOfStack>()
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .insert_resource(ImageHandles::default())
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
        interpolate_returning_to_water_arcs,
        interpolate_casting_arc,
        fish_bite_hook,
        apply_fish_movement,
        apply_velocity,
        apply_fish_boundaries,
        apply_fish_animation,
        move_hook,
        cast_hook,
        turn_hook_pink,
        reel_in,
        draw_fishing_line,
    ).before(catch_fish))
    .add_systems(Update,(
        catch_fish,
        handle_fish_landed_in_stack,
        handle_fish_caught,
        handle_fish_on_bite,
        handle_fish_reeled_to_surface,
        handle_hook_caught_fish,
        handle_hook_on_bite,
        handle_hook_reeled_to_surface,
        handle_fish_knocked_out_of_stack,
        handle_fish_landed
    ))
    .add_systems(Update, (
        reset_fish,
        reset_stack,
    ).after(catch_fish))
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

#[derive(Component, Debug)]
pub struct CastingHook {
    pub start_vel: Vec2,
    pub gravity: f32,
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub start_time_s: f32,
    pub end_time_s: f32
}

#[derive(Component, Debug)]
pub struct HookInWater;

#[derive(Component, Debug)]
pub struct WaitingToBeCast;

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

#[derive(Component, Default)]
pub struct CatchStack {
    pub total_fish: usize,
    pub fish: [Option<(Entity, usize, f32)>; FISH_PER_LEVEL]
}

#[derive(Component)]
pub struct InCatchStack;

#[derive(Event)]
pub struct FishLandedInStack {
    pub entity: Entity,
    pub fish_size: usize,
    pub position: Vec2,
    pub return_lane_y: f32
}

#[derive(Event, Default)]
pub struct StackCompleted;

fn reset_fish(
    mut completed_events: EventReader<StackCompleted>,
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

fn reset_stack(
    mut completed_events: EventReader<StackCompleted>,
    mut stack_query: Query<&mut CatchStack>,
) {
    if !completed_events.is_empty() {
        completed_events.clear();
        let mut catch_stack = stack_query.single_mut();
        catch_stack.total_fish = 0;
        for item in catch_stack.fish.iter_mut() {
            *item = None;
        }
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
    let mut fish_and_sort: Vec<(usize, usize)> = (0..FISH_PER_LEVEL).map(|f| (f, (rng.gen::<f32>() * FISH_PER_LEVEL as f32 * 1000.0) as usize)).collect();
    fish_and_sort.sort_by_key(|(_, key)| *key);
    for (pos_index, fish_index) in fish_and_sort.iter().map(|(item, _)| item).enumerate() {
        let fish_size = FISH_ATLAS_SIZES[*fish_index];
        let fish_half_width = (fish_size - 1) as f32 * 20.0 + 30.0;
        let pos_x = rng.gen::<f32>() * box_width - (box_width / 2.0) + WATER_POS.x;
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
                    translation: Vec3::new(LINE_START_POS.x, LINE_START_POS.y, 0.0),
                    scale: Vec3::new(2.0, 2.0, 1.0),
                    ..default()
                },
            ..default()
        },
        Hook {
            move_speed: 300.0
        },
        WaitingToBeCast
    ));
}

fn cast_hook(
    hook_query: Query<Entity, (With<Hook>, With<WaitingToBeCast>)>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands
) {
    for entity in &hook_query {
        if input.just_released(KeyCode::Space) {
            commands.entity(entity).remove::<WaitingToBeCast>();
            let (initial_vel, arc_time) = calculate_time_and_initial_vel_for_arc(
                LINE_START_POS.x,
                LINE_START_POS.y,
                CAST_TARGET_POS.x,
                CAST_TARGET_POS.y,
                GRAVITY,
                900.0 //TODO: K: make constant
            );
            commands.entity(entity).insert(CastingHook {
                start_vel: initial_vel,
                gravity: GRAVITY,
                start_pos: LINE_START_POS,
                end_pos: CAST_TARGET_POS,
                start_time_s: time.elapsed_seconds(),
                end_time_s: time.elapsed_seconds() + arc_time,
            });
        }
    }
}

fn move_hook(
    mut query: Query<(&mut Transform, &Hook), (With<HookInWater>, Without<NearFish>)>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>
) {
    let up_pressed = input.pressed(KeyCode::W) || input.pressed(KeyCode::Up);
    let down_pressed = input.pressed(KeyCode::S) || input.pressed(KeyCode::Down);
    for (mut transform, hook) in &mut query {
        let y_vel = (if up_pressed { 1.0 } else { 0.0 } + if down_pressed { -1.0 } else { 0.0 });
        let y_del = y_vel * time.delta_seconds();
        let new_y = transform.translation.y + y_del * hook.move_speed;
        let water_top = WATER_POS.y + WATER_SIZE.y / 2.0 - 100.0;
        let water_bottom = WATER_POS.y - WATER_SIZE.y / 2.0;
        transform.translation.y = new_y.clamp(water_bottom, water_top);
    }
}

#[derive(Event)]
pub struct HookedFish {
    pub hook_entity: Entity,
    pub fish_entity: Entity,
}

fn fish_bite_hook(
    fish_query: Query<(Entity, &Transform, &FishMouthPosition), Without<Hooked>>,
    hook_query: Query<(Entity, &Transform), (With<Hook>, Without<NearFish>)>,
    mut on_hook: EventWriter<HookedFish>,
) {
    for (hook_entity, hook) in &hook_query {
        for (fish_entity, fish, mouth_pos) in &fish_query {
            let distance = get_distance_to_fish_mouth(
                &hook.translation,
                &fish.translation,
                fish.scale.x,
                mouth_pos);
            if distance < BITE_DISTANCE {
                on_hook.send(HookedFish { hook_entity, fish_entity });
                break;
            }
        }
    }
}

fn handle_hook_on_bite(
    mut on_hook: EventReader<HookedFish>,
    mut commands: Commands
) {
    for event in on_hook.iter() {
        commands.entity(event.hook_entity).insert(NearFish);
    }
}

fn handle_fish_on_bite(
    time: Res<Time>,
    mut on_hook: EventReader<HookedFish>,
    mut commands: Commands
) {
    for event in on_hook.iter() {
        commands.entity(event.fish_entity).insert(Hooked { hook_time_s: time.elapsed_seconds() });
    }
}

#[derive(Event)]
pub struct ReeledToSurface {
    pub entity: Entity
}

fn reel_in(
    mut reelable_query: Query<(Entity, &mut Transform), (With<Reeling>, Without<CatchStack>)>,
    time: Res<Time>,
    mut on_reeled: EventWriter<ReeledToSurface>
) {
    let reel_speed = 600.0;
    let upper_boundary = WATER_POS.y + WATER_SIZE.y / 2.0;
    for (entity, mut pos) in &mut reelable_query {
        let mut hit_surface = false;
        let mut new_y = pos.translation.y + reel_speed * time.delta_seconds();
        if new_y > upper_boundary {
            hit_surface = true;
            new_y = upper_boundary;
        }
        pos.translation.y = new_y;
        if hit_surface {
            on_reeled.send(ReeledToSurface { entity });
        }
    }
}

fn handle_hook_reeled_to_surface(
    mut on_reeled: EventReader<ReeledToSurface>,
    mut hook_query: Query<(Entity, &mut Transform), With<Hook>>,
    mut commands: Commands
) {
    for event in on_reeled.iter() {
        if let Ok((hook_entity, mut hook_pos)) = hook_query.get_single_mut() {
            if event.entity == hook_entity {
                commands.entity(hook_entity).remove::<Reeling>();
                commands.entity(hook_entity).insert(WaitingToBeCast);
                hook_pos.translation = Vec3::new(LINE_START_POS.x, LINE_START_POS.y, 0.0);
            }
        }
    }
}

fn handle_fish_reeled_to_surface(
    mut on_reeled: EventReader<ReeledToSurface>,
    catch_stack: Query<(&Transform, &CatchStack)>,
    fish_query: Query<(Entity, &Transform), With<FishSize>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let (catch_stack_pos, catch_stack) = catch_stack.single();
    for event in on_reeled.iter() {
        for (fish_entity, fish_pos) in &fish_query {
            if fish_entity == event.entity {
                let catch_stack_pos = catch_stack_pos.translation;
                let catch_stack_pos = Vec3::new(
                    catch_stack_pos.x, 
                    catch_stack_pos.y + (catch_stack.total_fish as f32) * FISH_STACK_HEIGHT,
                    catch_stack_pos.z);
                commands.entity(event.entity).remove::<Reeling>();
                send_fish_to_stack(fish_pos.translation, catch_stack_pos, GRAVITY, time.elapsed_seconds(), &mut commands, event.entity);
            }
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

#[derive(Event)]
pub struct FishCaught {
    pub fish_entity: Entity,
    pub hook_entity: Entity,
    pub is_critical: bool
}

fn catch_fish(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    fish_query: Query<(Entity, &Hooked)>,
    hook_query: Query<Entity, (With<Hook>, With<NearFish>)>,
    mut on_catch: EventWriter<FishCaught>,
    mut on_critical: EventWriter<ReeledToSurface>
) {
    if let Ok(hook_entity) = hook_query.get_single() {
        if let Ok((fish_entity, hooked)) = fish_query.get_single() {
            if input.just_pressed(KeyCode::Space) {
                let react_time = time.elapsed_seconds() - hooked.hook_time_s;
                let is_critical = react_time < CRITICAL_TIME;
                on_catch.send(FishCaught { fish_entity, hook_entity, is_critical });
                if react_time < CRITICAL_TIME {
                    on_critical.send(ReeledToSurface { entity: fish_entity });
                    on_critical.send(ReeledToSurface { entity: hook_entity });
                }
            }
        }
    }
}

fn handle_fish_caught(
    mut on_caught: EventReader<FishCaught>,
    mut commands: Commands
) {
    for event in on_caught.iter() {
        commands.entity(event.fish_entity).remove::<(Hooked, FishMovement, FishMouthPosition, Velocity)>();
        if !event.is_critical {
            commands.entity(event.fish_entity).insert(Reeling);
        }
    }
}

fn handle_hook_caught_fish(
    mut on_caught: EventReader<FishCaught>,
    mut commands: Commands
) {
    for event in on_caught.iter() {
        commands.entity(event.hook_entity).remove::<(NearFish, HookInWater)>();
        if !event.is_critical {
            commands.entity(event.hook_entity).insert(Reeling);
        }
    }
}

fn interpolate_flying_arc(
    mut commands: Commands,
    time: Res<Time>,
    mut on_land: EventWriter<FishLandedInStack>,
    mut flying_query: Query<(Entity, &mut Transform, &Flying, &FishSize, &FishLanePos)>
) {
    for (entity, mut transform, flying, size, lane_pos) in &mut flying_query {
        if time.elapsed_seconds() > flying.end_time_s {
            commands.entity(entity).remove::<Flying>();
            on_land.send(FishLandedInStack { 
                entity, 
                fish_size: size.size, 
                position: flying.end_pos,
                return_lane_y: lane_pos.pos_y 
            });
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

fn interpolate_casting_arc(
    mut commands: Commands,
    time: Res<Time>,
    mut casting_query: Query<(Entity, &mut Transform, &CastingHook)>
) {
    for (entity, mut transform, casting) in &mut casting_query {
        if time.elapsed_seconds() > casting.end_time_s {
            commands.entity(entity).remove::<CastingHook>();
            commands.entity(entity).insert(HookInWater);
            transform.translation = Vec3::new(casting.end_pos.x, casting.end_pos.y, transform.translation.z);
        } else {
            let elapsed = time.elapsed_seconds() - casting.start_time_s;
            let new_pos_x = casting.start_pos.x + (elapsed * casting.start_vel.x);
            let new_pos_y = casting.start_pos.y + (elapsed * casting.start_vel.y) - (casting.gravity * elapsed * elapsed / 2.0 );
            transform.translation = Vec3::new(new_pos_x, new_pos_y, transform.translation.z);
        }
    }
}

fn interpolate_returning_to_water_arcs(
    mut commands: Commands,
    time: Res<Time>,
    mut returning_query: Query<(Entity, &mut Transform, &ReturningToWater, &FishSize)>,
    images: Res<ImageHandles>
) {
    for (entity, mut transform, returning, size) in &mut returning_query {
        let mut rng = rand::thread_rng();
        if time.elapsed_seconds() > returning.end_time_s {
            commands.entity(entity).remove::<(ReturningToWater, Handle<TextureAtlas>)>();
            commands.entity(entity).insert((
                Velocity { 
                    x: returning.start_vel.x,
                    y: 0.0,
                    drag_x: WATER_DRAG_X,
                    drag_y: WATER_DRAG_Y
                },
                FishMovement {
                    next_move_time: build_fish_movement_timer(&mut rng),
                    vel_to_apply: FISH_VELOCITY
                },
                build_fish_mouth_position(size.size),
                images.fish_atlas_handle.as_ref().expect("Images should be loaded").clone()
            ));
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

fn handle_fish_landed(
    mut on_land: EventReader<FishLandedInStack>,
    images: Res<ImageHandles>,
    mut commands: Commands
) {
    for event in on_land.iter() {
        commands.entity(event.entity).remove::<Handle<TextureAtlas>>();
        commands.entity(event.entity).insert((
            images.stack_atlas_handle.as_ref().expect("Images should be loaded").clone(),
            InCatchStack
        ));
    }
}

#[derive(Event)]
pub struct FishKnockedOutOfStack {
    pub fish_entity: Entity,
    pub stack_position: Vec2
}

fn handle_fish_knocked_out_of_stack(
    mut on_knocked_out: EventReader<FishKnockedOutOfStack>,
    fish_query: Query<(Entity, &FishLanePos), With<InCatchStack>>,
    time: Res<Time>,
    mut commands: Commands
) {
    for event in on_knocked_out.iter() {
        for (fish_entity, lane_pos) in &fish_query {
            let water_y = WATER_POS.y + WATER_SIZE.y / 2.0;
            let return_pos = calculate_return_position(lane_pos.pos_y);
            let return_val = calculate_return_path(
                event.stack_position.x, 
                event.stack_position.y, 
                return_pos.x, 
                return_pos.y, 
                water_y, 
                GRAVITY, 
                WATER_DRAG_Y, 
                time.elapsed_seconds());
            commands.entity(fish_entity).remove::<InCatchStack>();
            commands.entity(fish_entity).insert(return_val);
        }
    }
}

fn handle_fish_landed_in_stack(
    mut on_land: EventReader<FishLandedInStack>,
    mut catch_stack_query: Query<(&Transform, &mut CatchStack)>,
    mut on_fish_kod: EventWriter<FishKnockedOutOfStack>,
    mut on_complete: EventWriter<StackCompleted>,
) {
    for event in on_land.iter() {
        let (catch_stack_pos, mut catch_stack) = catch_stack_query.single_mut();
        let mut indexes_to_remove = Vec::<usize>::new();
        for (item_index, item) in catch_stack.fish.iter().enumerate() {
            if let Some((fish_entity, size, _)) = item.as_ref() {
                if *size < event.fish_size {
                    let start_pos_y = catch_stack_pos.translation.y + item_index as f32 * FISH_STACK_HEIGHT;
                    on_fish_kod.send(FishKnockedOutOfStack { 
                        fish_entity: *fish_entity,
                        stack_position: Vec2::new(catch_stack_pos.translation.x, start_pos_y) 
                    });
                    indexes_to_remove.push(item_index);
                }
            }
        }
        catch_stack.total_fish = catch_stack.total_fish + 1 - indexes_to_remove.len();
        for index in indexes_to_remove {
            catch_stack.fish[index] = None;
        }
        let mut insert_offset = 0;
        let mut first_empty_slot = 0;
        for i in 0..catch_stack.fish.len() {
            if catch_stack.fish[i].is_none() {
                insert_offset += 1;
            } else if insert_offset > 0 {
                debug_assert!(insert_offset <= i);
                catch_stack.fish[i - insert_offset] = catch_stack.fish[i];
                catch_stack.fish[i] = None;
                first_empty_slot = i + 1;
            } else {
                first_empty_slot = i + 1;
            }
        }
        catch_stack.fish[first_empty_slot] = Some((event.entity, event.fish_size, event.return_lane_y));

        if catch_stack.total_fish == FISH_PER_LEVEL {
            on_complete.send_default();
        }
    }
}

fn calculate_return_position(
    lane_y: f32,
) -> Vec2 {
    const WATER_SHRINK: f32 = 0.8; //factor to shrink the landing zone by, centered at WATER_POS

    const MIN_LEFT_X: f32 = WATER_POS.x - WATER_SIZE.x / 2.0 * WATER_SHRINK;
    const WATER_TOP_Y: f32 = WATER_POS.y + WATER_SIZE.y / 2.0;
    let del_y = WATER_TOP_Y - lane_y ;
    let del_x = WATER_SIZE.x * WATER_SHRINK * del_y / WATER_SIZE.y;
    Vec2::new(del_x + MIN_LEFT_X, lane_y)
}

fn calculate_return_path(
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    water_y: f32,
    gravity_y: f32,
    water_drag_y: f32,
    start_time_s: f32
) -> ReturningToWater {
    debug_assert_ne!(0.0, gravity_y);
    debug_assert_ne!(0.0, water_drag_y);
    //we need to hit the lane_y, so we need to know what the vel_y is at the water's surface, 
    //so then we can know how high it arcs and what vel we need to start with to hit that apex.
    //working backwards from the final position...
    let time_from_water_to_lane = (2.0 / water_drag_y * (water_y - end_y)).sqrt();
    let water_entrance_vel_y = water_drag_y * time_from_water_to_lane;
    let time_from_apex_to_water = water_entrance_vel_y / gravity_y;
    let apex_pos_y = water_y + time_from_apex_to_water * time_from_apex_to_water * gravity_y / 2.0;
    let time_to_apex = (2.0 / gravity_y * (apex_pos_y - start_y)).sqrt();

    let total_time = 
        time_to_apex 
        + time_from_apex_to_water 
        + time_from_water_to_lane;
    let start_vel_x = (end_x - start_x) / total_time;
    let start_vel_y = gravity_y * time_to_apex;
    let water_pos_x = end_x - (time_from_water_to_lane * start_vel_x);
    ReturningToWater { 
        start_vel: Vec2::new(start_vel_x, start_vel_y), 
        water_entrance_vel: Vec2::new(start_vel_x, -water_entrance_vel_y),
        start_pos: Vec2::new(start_x, start_y),
        water_entrance_pos: Vec2::new(water_pos_x, water_y),
        end_pos: Vec2::new(end_x, end_y),
        start_time_s,
        water_entrance_time_s: start_time_s + time_to_apex + time_from_apex_to_water,
        end_time_s: start_time_s + total_time,
        gravity: gravity_y,
        water_drag: water_drag_y
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
