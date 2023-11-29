use bevy::prelude::*;
use crate::core::*;
use crate::constants::*;

pub struct CloudsPlugin;
impl Plugin for CloudsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, add_clouds)
        .add_systems(Update, update_clouds);
    }
}

#[derive(Component)]
pub struct Cloud {
    pub speed: f32
}

fn add_clouds(
    images: Res<ImageHandles>,
    mut commands: Commands
) {
    let texture_handle = images.misc_atlas_handle.as_ref().expect("Images should be loaded").clone();
    commands.spawn_batch(
        (0..10)
        .map(move |i| (
            SpriteSheetBundle {
                texture_atlas: texture_handle.clone(),
                sprite: TextureAtlasSprite::new(i%6),
                transform: Transform::from_translation(
                    Vec3::new(
                        (CLOUD_END_X - CLOUD_START_X) * rand::random::<f32>() + CLOUD_START_X ,
                        rand::random::<f32>() * 300.0 + CLOUD_Y,
                        i as f32 - 20.0
                    )),
                ..default()
            },
            Cloud { speed: 50.0 + rand::random::<f32>() * 10.0 }
        ))
    );
}

fn update_clouds(
    mut clouds_query: Query<(&mut Transform, &Cloud)>,
    time: Res<Time>
) {
    for (mut cloud_pos, cloud) in &mut clouds_query {
        let mut new_x = cloud_pos.translation.x + cloud.speed * time.delta_seconds();
        if new_x > CLOUD_END_X {
            new_x -= CLOUD_END_X - CLOUD_START_X;
        }
        cloud_pos.translation.x = new_x;
    }
}