use bevy::prelude::*;
use crate::constants::*;
use crate::fish::*;
use crate::physics::*;

pub struct HookPlugin;
impl Plugin for HookPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<ReeledToSurface>()
        .add_event::<FishCaught>()
        .add_event::<HookedFish>()
        .add_event::<HookLandedInWater>()
        .add_event::<HookCast>()
        .add_systems(Startup, add_hook)
        .add_systems(Update, (
            reel_in,
            interpolate_casting_arc,
            move_hook,
            cast_hook,
            fish_bite_hook,
            turn_hook_pink,
            catch_fish,
            draw_fishing_line,
        ))
        .add_systems(PostUpdate, (
            handle_hook_reeled_to_surface,
            handle_fish_on_bite,
            handle_hook_on_bite,
            handle_hook_landed_in_water,
            handle_hook_caught_fish,
            handle_fish_caught,
        ));
    }
}

#[derive(Event)]
pub struct ReeledToSurface {
    pub entity: Entity
}

#[derive(Event)]
pub struct HookLandedInWater {
    pub hook_entity: Entity
}

#[derive(Event)]
pub struct HookedFish {
    pub hook_entity: Entity,
    pub fish_entity: Entity,
}

#[derive(Event)]
pub struct HookCast {
    pub hook_entity: Entity
}

#[derive(Event)]
pub struct FishCaught {
    pub fish_entity: Entity,
    pub hook_entity: Entity,
    pub is_critical: bool
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

fn cast_hook(
    hook_query: Query<Entity, (With<Hook>, With<WaitingToBeCast>)>,
    input: Res<Input<KeyCode>>,
    mut on_cast: EventWriter<HookCast>,
    time: Res<Time>,
    mut commands: Commands
) {
    for entity in &hook_query {
        if input.just_pressed(KeyCode::Space) {
            on_cast.send(HookCast{ hook_entity: entity });
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

fn fish_bite_hook(
    fish_query: Query<(Entity, &Transform, &FishMouthPosition), Without<Hooked>>,
    hook_query: Query<(Entity, &Transform), (With<HookInWater>, Without<NearFish>)>,
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

fn reel_in(
    mut reelable_query: Query<(Entity, &mut Transform), With<Reeling>>,
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
        commands.entity(event.fish_entity).remove::<(Velocity, FishMovement)>();
    }
}

fn interpolate_casting_arc(
    mut casting_query: Query<(Entity, &mut Transform, &CastingHook)>,
    time: Res<Time>,
    mut on_landed: EventWriter<HookLandedInWater>,
) {
    for (entity, mut transform, casting) in &mut casting_query {
        if time.elapsed_seconds() > casting.end_time_s {
            on_landed.send(HookLandedInWater { hook_entity: entity });
            transform.translation = Vec3::new(casting.end_pos.x, casting.end_pos.y, transform.translation.z);
        } else {
            let elapsed = time.elapsed_seconds() - casting.start_time_s;
            let new_pos_x = casting.start_pos.x + (elapsed * casting.start_vel.x);
            let new_pos_y = casting.start_pos.y + (elapsed * casting.start_vel.y) - (casting.gravity * elapsed * elapsed / 2.0 );
            transform.translation = Vec3::new(new_pos_x, new_pos_y, transform.translation.z);
        }
    }
}

fn handle_hook_landed_in_water(
    mut on_landed: EventReader<HookLandedInWater>,
    hook_query: Query<Entity, With<CastingHook>>,
    mut commands: Commands
) {
    for event in on_landed.iter() {
        if let Ok(hook_entity) = hook_query.get_single() {
            if hook_entity == event.hook_entity {
                commands.entity(hook_entity).remove::<CastingHook>();
                commands.entity(hook_entity).insert(HookInWater);
            }
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