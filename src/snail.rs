use bevy::prelude::*;
use crate::{core::*, constants::*, physics::Velocity, catch_stack::StackCompleted, hook::HookCast};

pub struct SnailPlugin;
impl Plugin for SnailPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<SnailStarted>()
        .add_event::<SnailReachedEnd>()
        .add_systems(Startup, (
            add_snail,
        ))
        .add_systems(Update, (
            start_snail,
            update_snail,
            animate_snail,
            update_lifespan,
        ))
        .add_systems(PostUpdate, (
            handle_snail_on_reset,
            create_snail_particles,
            handle_snail_on_stack_complete
        ));
    }
}

#[derive(Event, Default)]
pub struct SnailStarted;

#[derive(Event, Default)]
pub struct SnailReachedEnd;

#[derive(Component)]
pub struct Snail {
    pub speed: f32
}

#[derive(Component)]
pub struct ParticleTimer {
    pub timer: Timer
}

//maybe move this to core
#[derive(Component)]
pub struct Lifespan {
    pub timer: Timer
}

#[derive(Component)]
pub struct Stopped;

fn add_snail(
    images: Res<ImageHandles>,
    mut commands: Commands
) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: images.misc_atlas_handle.as_ref().expect("Images should be loaded").clone(),
            sprite: TextureAtlasSprite::new(7),
            transform: 
                Transform::from_translation(
                    SNAIL_START_POS.extend(0.0)),
            ..default()
        },
        Snail { speed: SNAIL_SPEED },
        Stopped,
        ParticleTimer {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating)
        }
    ));
}

fn animate_snail(
    mut snail_query: Query<&mut Transform, With<Snail>>,
    time: Res<Time>,
) {
    const SNAIL_ANIM_PERIOD_MS: u128 = 1500;
    const SNAIL_ANIM_AMOUNT: f32 = 0.1;
    if let Ok(mut snail) = snail_query.get_single_mut() {
        let factor = 
            std::f32::consts::PI * 2.0 
            * (time.elapsed().as_millis() % SNAIL_ANIM_PERIOD_MS) as f32 
            / SNAIL_ANIM_PERIOD_MS as f32;
        let factor_sin = factor.sin();
        let stretch_x = factor_sin * SNAIL_ANIM_AMOUNT + 1.0;
        let stretch_y = factor_sin * -SNAIL_ANIM_AMOUNT + 1.0;
        snail.scale = Vec3::new(stretch_x, stretch_y, 1.0);
    }
}

fn start_snail(
    mut on_cast: EventReader<HookCast>,
    snail_query: Query<Entity, (With<Snail>, With<Stopped>)>,
    mut commands: Commands,
    mut on_start: EventWriter<SnailStarted>
) {
    if !on_cast.is_empty() {
        on_cast.clear();
        if let Ok(entity) = snail_query.get_single() {
            commands.entity(entity).remove::<Stopped>();
            on_start.send_default();
        }
    }
}

fn update_snail(
    mut snail_query: Query<(&mut Transform, &Snail), Without<Stopped>>,
    time: Res<Time>,
    mut on_end: EventWriter<SnailReachedEnd>
) {
    if let Ok((mut snail_pos, snail)) = snail_query.get_single_mut() {
        if snail_pos.translation != SNAIL_END_POS.extend(0.0) {
            let new_x = snail_pos.translation.x + snail.speed * time.delta_seconds();
            if new_x >= SNAIL_END_POS.x {
                snail_pos.translation = SNAIL_END_POS.extend(0.0);
                on_end.send_default();
            } else {
                snail_pos.translation.x = new_x;
            }
        }
    }
}

fn create_snail_particles(
    mut snail_query: Query<(&mut ParticleTimer, &Transform), (With<Snail>, Without<Stopped>)>,
    time: Res<Time>,
    images: Res<ImageHandles>,
    mut commands: Commands
) {
    const PANIC_THRESHOLD: f32 = 400.0;
    if let Ok((mut timer, snail_pos)) = snail_query.get_single_mut() {
        if SNAIL_END_POS.x - snail_pos.translation.x < PANIC_THRESHOLD {
            timer.timer.tick(time.delta());
            if timer.timer.just_finished() {
                let particle_pos = Vec3::new(
                    snail_pos.translation.x + 50.0,
                    snail_pos.translation.y,
                    snail_pos.translation.z + 1.0,
                );
                commands.spawn((
                    SpriteSheetBundle {
                        texture_atlas: images.misc_atlas_handle.as_ref().expect("Images should be loaded").clone(),
                        sprite: TextureAtlasSprite::new(8),
                        transform: Transform::from_translation(
                            particle_pos),
                        ..default()
                    },
                    Lifespan {
                        timer: Timer::from_seconds(1.0, TimerMode::Once) 
                    },
                    Velocity {
                        x: -100.0,
                        y: 200.0,
                        drag_x: 100.0,
                        drag_y: 200.0
                    }
                ));
            }
        }
    }
}

fn update_lifespan(
    mut lifespan_query: Query<(Entity, &mut Lifespan)>,
    time: Res<Time>,
    mut commands: Commands
) {
    for (entity, mut lifespan) in &mut lifespan_query {
        lifespan.timer.tick(time.delta());
        if lifespan.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_snail_on_reset(
    mut on_level_end: EventReader<ResetLevel>,
    mut snail_query: Query<(Entity, &mut Transform), With<Snail>>,
    mut commands: Commands
) {
    if !on_level_end.is_empty() {
        on_level_end.clear();
        if let Ok((entity, mut snail_pos)) = snail_query.get_single_mut() {
            snail_pos.translation = SNAIL_START_POS.extend(0.0);
            commands.entity(entity).insert(Stopped);
        }
    }
}

fn handle_snail_on_stack_complete(
    mut on_stack_complete: EventReader<StackCompleted>,
    snail_query: Query<Entity, With<Snail>>,
    mut commands: Commands,
) {
    if !on_stack_complete.is_empty() {
        on_stack_complete.clear();
        if let Ok(entity) = snail_query.get_single() {
            commands.entity(entity).insert(Stopped);
        }
    }
}