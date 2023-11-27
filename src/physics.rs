use bevy::prelude::*;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_velocity);
    }
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub drag_x: f32,
    pub drag_y: f32,
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


pub fn calculate_time_and_initial_vel_for_arc(
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