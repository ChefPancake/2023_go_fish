use bevy::prelude::*;
use crate::{core::*, constants::*};

pub struct SnailPlugin;
impl Plugin for SnailPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<SnailReachedEnd>()
        .add_systems(Startup, (
            add_snail,
        ))
        .add_systems(Update, (
            update_snail,
            animate_snail
        ))
        .add_systems(PostUpdate,
            handle_level_end);
    }
}

#[derive(Event, Default)]
pub struct SnailReachedEnd;

#[derive(Component)]
pub struct Snail {
    pub speed: f32
}

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
        Snail { speed: SNAIL_SPEED }
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

fn update_snail(
    mut snail_query: Query<(&mut Transform, &Snail)>,
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

fn handle_level_end(
    mut on_level_end: EventReader<ResetLevel>,
    mut snail_query: Query<&mut Transform, With<Snail>>
) {
    if !on_level_end.is_empty() {
        on_level_end.clear();
        if let Ok(mut snail_pos) = snail_query.get_single_mut() {
            snail_pos.translation = SNAIL_START_POS.extend(0.0);
        }
    }
}