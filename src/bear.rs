use bevy::prelude::*;

use crate::{
    core::*, 
    constants::*,
    hook::*,
    catch_stack::*
};

pub struct BearPlugin;
impl Plugin for BearPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, (
            add_bear,
        ))
        .add_systems(Update, (
            update_critical_anim,
            animate_bear
        ))
        .add_systems(PostUpdate, (
            handle_bear_on_reeled_to_surface,
            handle_bear_on_cast,
            handle_bear_on_catch,
            handle_bear_on_fish_landed,
            handle_bear_on_hooked
        ));
    }
}


#[derive(Component)]
pub struct Bear;

#[derive(Component)]
pub struct BearCriticalFlash {
    pub anim_timer: Timer
}

#[derive(Component)]
pub struct BearAnimations {
    pub state: BearAnimationStates,
    pub timer: Timer,
    pub rate_multiplier: f32,
    pub stretch_x: f32,
    pub stretch_y: f32
}
impl BearAnimations {
    fn fishing() -> BearAnimations {
        BearAnimations {
            state: BearAnimationStates::Fishing,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            rate_multiplier: 1.0,
            stretch_x: 0.05,
            stretch_y: -0.05,
        }
    }

    fn waiting() -> BearAnimations {
        BearAnimations {
            state: BearAnimationStates::Waiting,
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            rate_multiplier: 1.0,
            stretch_x: 0.1,
            stretch_y: -0.1,
        }
    }

    fn hooking() -> BearAnimations {
        BearAnimations {
            state: BearAnimationStates::Hooking,
            timer: Timer::from_seconds(0.15, TimerMode::Once),
            rate_multiplier: 0.5,
            stretch_x: 0.0,
            stretch_y: 0.3,
        }
    }

    fn reeling() -> BearAnimations {
        BearAnimations {
            state: BearAnimationStates::Reeling,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            rate_multiplier: 0.5,
            stretch_x: 0.0,
            stretch_y: 0.02,
        }
    }

    fn catching() -> BearAnimations {
        BearAnimations {
            state: BearAnimationStates::Catching,
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            rate_multiplier: 1.0,
            stretch_x: 0.1,
            stretch_y: -0.1,
        }
    }

    fn casting() -> BearAnimations {
        BearAnimations { 
            state: BearAnimationStates::Casting, 
            timer: Timer::from_seconds(0.3, TimerMode::Once),
            rate_multiplier: 0.5,
            stretch_x: 0.3,
            stretch_y: 0.0,
        }
    }

    fn dancing() -> BearAnimations {
        BearAnimations {
            state: BearAnimationStates::Dancing,
            timer: Timer::from_seconds(0.15, TimerMode::Repeating),
            rate_multiplier: 1.0,
            stretch_x: 0.0,
            stretch_y: 0.05,
        }
    }
}

pub enum BearAnimationStates {
    Fishing,
    Hooking,
    Reeling,
    Catching,
    Waiting,
    Casting,
    Dancing
}

fn add_bear(
    handles: Res<ImageHandles>,
    mut commands: Commands
) {
    let atlas_handle = handles.bear_atlas_handle.as_ref().expect("Images should be loaded");
    commands.spawn(
        (SpriteSheetBundle {
            texture_atlas: atlas_handle.clone(),
            sprite: TextureAtlasSprite::new(BearStates::Casting.into()),
            transform: Transform::from_translation(Vec3::new(BEAR_POS.x, BEAR_POS.y, 10.0)), 
            ..default()
        },
        BearAnimations::waiting(),
        Bear
    ));
}

#[derive(PartialEq, Eq)]
pub enum BearStates {
    Fishing = 0,
    Casting = 1,
    Reeling = 2,
    Catching = 3,
    Critical1 = 4,
    Critical2 = 5
}

impl From<BearStates> for usize {
    fn from(val: BearStates) -> Self {
        use BearStates::*;
        match val {
            Fishing => 0,
            Casting => 1,
            Reeling => 2,
            Catching => 3,
            Critical1 => 4,
            Critical2 => 5,
        }
    }
}

impl Into<BearStates> for usize {
    fn into(self) -> BearStates {
        use BearStates::*;
        match self {
            0 => Fishing,
            1 => Casting,
            2 => Reeling,
            3 => Catching,
            4 => Critical1,
            5 => Critical2,
            _ => panic!("Invalid bear state: {}", self)
        }
    }
}

fn update_critical_anim(
    mut bear_query: Query<(&mut TextureAtlasSprite, &mut BearCriticalFlash), With<Bear>>,
    time: Res<Time>
) {
    if let Ok((mut bear_sprite, mut crit_flash)) = bear_query.get_single_mut() {
        crit_flash.anim_timer.tick(time.delta());
        if crit_flash.anim_timer.just_finished() {
            let bear_state: BearStates = bear_sprite.index.into();
            bear_sprite.index = 
            if bear_state == BearStates::Critical1 {
                BearStates::Critical2.into()
            } else {
                BearStates::Critical1.into()
            };
        }
    }
}

fn handle_bear_on_reeled_to_surface(
    mut on_reeled: EventReader<ReeledToSurface>,
    mut bear_query: Query<(&mut TextureAtlasSprite, &mut BearAnimations), With<Bear>>
) {
    if !on_reeled.is_empty() {
        on_reeled.clear();
        let (mut bear_sprite, mut animation) = bear_query.single_mut();
        let bear_state: BearStates = bear_sprite.index.into();
        if bear_state != BearStates::Critical1 
            && bear_state != BearStates::Critical2 
        {
            *animation = BearAnimations::catching();
            bear_sprite.index = BearStates::Catching.into();
        }
    }
}

fn handle_bear_on_cast(
    mut on_cast: EventReader<HookCast>,
    mut bear_query: Query<(Entity, &mut TextureAtlasSprite, &mut BearAnimations), With<Bear>>,
    mut commands: Commands
){
    if !on_cast.is_empty() {
        on_cast.clear();
        let (bear_entity, mut bear_sprite, mut animations) = bear_query.single_mut();
        bear_sprite.index = BearStates::Fishing.into();
        *animations = BearAnimations::casting();
        commands.entity(bear_entity).remove::<BearCriticalFlash>();
    }
}

fn handle_bear_on_fish_landed(
    mut on_land: EventReader<FishLandedInStack>,
    mut bear_query: Query<(Entity, &mut TextureAtlasSprite, &mut BearAnimations), With<Bear>>,
    mut commands: Commands
) {
    if !on_land.is_empty() {
        on_land.clear();
        let (bear_entity, mut bear_sprite, mut animation) = bear_query.single_mut();
        let bear_state: BearStates = bear_sprite.index.into();
        if bear_state != BearStates::Fishing {
            *animation = BearAnimations::waiting();
            bear_sprite.index = BearStates::Casting.into();
        }
        commands.entity(bear_entity).remove::<BearCriticalFlash>();
    }
}

fn handle_bear_on_hooked(
    mut on_hook: EventReader<HookedFish>,
    mut bear_query: Query<&mut BearAnimations, With<Bear>>
) {
    if !on_hook.is_empty() {
        on_hook.clear();
        let mut animation = bear_query.single_mut();
        *animation = BearAnimations::hooking();
    }
}

fn handle_bear_on_catch(
    mut on_reel: EventReader<FishCaught>,
    mut bear_query: Query<(Entity, &mut TextureAtlasSprite, &mut BearAnimations), With<Bear>>,
    mut commands: Commands
) {
    for event in on_reel.iter() {
        let (bear_entity, mut bear_sprite, mut animation) = bear_query.single_mut();
        if event.is_critical {
            *animation = BearAnimations::dancing();
            bear_sprite.index = BearStates::Critical1.into();
            commands.entity(bear_entity).insert(BearCriticalFlash {
                anim_timer: Timer::from_seconds(0.1, TimerMode::Repeating)
            });
        } else {
            *animation = BearAnimations::reeling();
            bear_sprite.index = BearStates::Reeling.into();
        }
    }
}

fn animate_bear(
    mut bear_query: Query<(&mut Transform, &mut BearAnimations), With<Bear>>,
    time: Res<Time>
) {
    if let Ok((mut bear_transform, mut animation)) = bear_query.get_single_mut() {
        animation.timer.tick(time.delta());
        bear_transform.scale = 
            interpolate_pulse_over_timer(
                &animation.timer,
                animation.rate_multiplier,
                animation.stretch_x,
                animation.stretch_y
            );
        //pulse slowly for now. sinusoidal, x and y axes 180deg out of phase
        use BearAnimationStates::*;
        //bear_transform.scale = 
        match animation.state {
            Casting => {
                if animation.timer.finished() {
                    *animation = BearAnimations::fishing();
                }
            },
            Hooking => {
                if animation.timer.finished() {
                    *animation = BearAnimations::fishing();
                }
            },
            _ => {}
        }
    }
}

fn interpolate_pulse_over_timer(
    timer: &Timer,
    rate_multiplier: f32,
    stretch_amount_x: f32,
    stretch_amount_y: f32
) -> Vec3 {
    if timer.finished() {
        Vec3::ONE
    } else {
        let stretch_factor = std::f32::consts::PI * 2.0 * timer.percent() * rate_multiplier;
        let stretch_sin = stretch_factor.sin();
        let stretch_x = stretch_sin * stretch_amount_x + 1.0;
        let stretch_y = stretch_sin * stretch_amount_y + 1.0;
        Vec3::new(stretch_x, stretch_y, 1.0)
    }
}