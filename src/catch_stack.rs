use bevy::prelude::*;
use crate::constants::*;
use crate::core::*;
use crate::fish::*;
use crate::hook::*;
use crate::physics::*;

pub struct CatchStackPlugin;
impl Plugin for CatchStackPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<FishLandedInStack>()
        .add_event::<FishKnockedOutOfStack>()
        .add_event::<StackCompleted>()
        .add_systems(Startup, 
            add_catch_stack)
        .add_systems(Update, 
            interpolate_flying_arc)
        .add_systems(PostUpdate,(
            reset_stack,
            handle_fish_landed_in_stack,
            handle_fish_reeled_to_surface,
            handle_fish_knocked_out_of_stack,
            handle_fish_landed,
        ));
    }
}

#[derive(Component, Debug)]
pub struct FlyingToStack {
    pub start_vel: Vec2,
    pub gravity: f32,
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub start_time_s: f32,
    pub end_time_s: f32
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

fn add_catch_stack(
    mut commands: Commands
){
    commands.spawn(
        (
        Transform::from_translation(STACK_POS),
        CatchStack::default())
    );
}

fn reset_stack(
    mut completed_events: EventReader<ResetLevel>,
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
    let flying = FlyingToStack {
        start_vel: arc_vel,
        gravity,
        start_pos: Vec2::new(fish_pos.x, fish_pos.y),
        end_pos: Vec2::new(catch_stack_pos.x, catch_stack_pos.y),
        start_time_s: elapsed_time,
        end_time_s: elapsed_time + arc_time,
    };
    commands.entity(entity).insert(flying);
}

fn interpolate_flying_arc(
    mut flying_query: Query<(Entity, &mut Transform, &FlyingToStack, &FishSize, &FishLanePos)>,
    time: Res<Time>,
    mut on_land: EventWriter<FishLandedInStack>,
) {
    for (entity, mut transform, flying, size, lane_pos) in &mut flying_query {
        if time.elapsed_seconds() > flying.end_time_s {
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

fn handle_fish_landed(
    mut on_land: EventReader<FishLandedInStack>,
    images: Res<ImageHandles>,
    mut commands: Commands
) {
    for event in on_land.iter() {
        commands.entity(event.entity).remove::<(Handle<TextureAtlas>, FlyingToStack)>();
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
            if fish_entity == event.fish_entity {
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
}

fn handle_fish_landed_in_stack(
    mut on_land: EventReader<FishLandedInStack>,
    mut catch_stack_query: Query<(&Transform, &mut CatchStack)>,
    popup_query: Query<(), With<PopupTimer>>,
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

        if popup_query.is_empty() && catch_stack.total_fish == FISH_PER_LEVEL {
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
