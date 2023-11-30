use bevy::prelude::*;
use crate::{constants::*, snail::SnailReachedEnd};

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_event::<ResetLevel>()
        .insert_resource(ImageHandles::default())
        .add_systems(PreStartup, (
            load_images,
        ))
        .add_systems(Startup, (
            add_camera,
            add_bg,
            add_water,
        ))
        .add_systems(Update, (
            wait_to_reset,
        ))
        .add_systems(PostUpdate, (
            handle_loss,
            handle_win,
        ));
    }
}

#[derive(Event, Default)]
pub struct ResetLevel;

#[derive(Resource, Default)]
pub struct ImageHandles {
    pub fish_handle: Option<Handle<Image>>,
    pub hook_handle: Option<Handle<Image>>,
    pub bg_handle: Option<Handle<Image>>,
    pub bear_handle: Option<Handle<Image>>,
    pub stack_handle: Option<Handle<Image>>,
    pub water_handle: Option<Handle<Image>>,
    pub misc_handle: Option<Handle<Image>>,
    pub win_bubble_handle: Option<Handle<Image>>,
    pub lose_bubble_handle: Option<Handle<Image>>,

    pub bear_atlas_handle: Option<Handle<TextureAtlas>>,
    pub fish_atlas_handle: Option<Handle<TextureAtlas>>,
    pub stack_atlas_handle: Option<Handle<TextureAtlas>>,
    pub misc_atlas_handle: Option<Handle<TextureAtlas>>
}

#[derive(Component)]
pub struct PopupTimer {
    pub timer: Timer
}

fn load_images(
    mut images: ResMut<ImageHandles>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>
) {
    images.bg_handle = Some(asset_server.load("background.png"));
    images.hook_handle = Some(asset_server.load("hook.png"));
    images.water_handle = Some(asset_server.load("water.png"));

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

    let misc_handle = asset_server.load("misc.png");
    let misc_atlas = TextureAtlas::from_grid(
        misc_handle.clone(),
        Vec2::new(600.0, 200.0),
        2, 
        5,
        None,
        None,
    );
    let misc_atlas_handle = atlases.add(misc_atlas);
    images.misc_handle = Some(misc_handle);
    images.misc_atlas_handle = Some(misc_atlas_handle);

    images.win_bubble_handle = Some(asset_server.load("win_bubble.png"));
    images.lose_bubble_handle = Some(asset_server.load("loss_bubble.png"));
}

fn add_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn add_bg(
    images: Res<ImageHandles>,
    mut commands: Commands
) {
    let image_handle = images.bg_handle.as_ref().expect("images should be loaded");
    commands.spawn(
        SpriteBundle {
            texture: image_handle.clone(),
            transform: 
                Transform::from_translation(Vec3::new(0.0, 0.0, -1000.0)),
            ..default()
        }
    );
}

fn add_water(
    images: Res<ImageHandles>,
    mut commands: Commands
) {
    let image_handle = images.water_handle.as_ref().expect("images should be loaded");
    commands.spawn(
        SpriteBundle {
            sprite: Sprite { color: Color::Rgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.2 }, ..default() },
            texture: image_handle.clone(),
            transform: Transform::from_translation(Vec3::new(WATER_POS.x, WATER_POS.y - 65.0, 5.0)),
            ..default()
        }
    );
}

fn handle_loss(
    mut on_snail_end: EventReader<SnailReachedEnd>,
    images: Res<ImageHandles>,
    mut commands: Commands
) {
    if !on_snail_end.is_empty() {
        on_snail_end.clear();
        println!("GAME OVER");
        commands.spawn((
            SpriteBundle {
                texture: images.lose_bubble_handle.as_ref().expect("images should be loaded").clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0))
                    .with_scale(Vec3::ONE * 2.0),
                ..default()
            },
            PopupTimer {
                timer: Timer::from_seconds(0.7, TimerMode::Once)
            }
        ));
    }
}

fn wait_to_reset(
    mut popup_query: Query<(Entity, &mut PopupTimer)>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut on_reset: EventWriter<ResetLevel>
) {
    for (entity, mut timer) in &mut popup_query {
        timer.timer.tick(time.delta());
        if timer.timer.finished() && input.pressed(KeyCode::Space) {
            commands.entity(entity).despawn();
            on_reset.send_default();
        }
    }
    //with either bubble up, after timer X has expired
    //when player hits Space, reset the game
}

fn handle_win() {

}