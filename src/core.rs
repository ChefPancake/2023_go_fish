use bevy::{prelude::*, sprite::Anchor};
use crate::{constants::*, snail::{SnailReachedEnd, SnailStarted}, catch_stack::StackCompleted};

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_event::<ResetLevel>()
        .insert_resource(GameTimer::default())
        .insert_resource(ImageHandles::default())
        .insert_resource(FontHandles::default())
        .add_systems(PreStartup, (
            load_images,
            load_fonts
        ))
        .add_systems(Startup, (
            add_camera,
            add_bg,
            add_water,
        ))
        .add_systems(Update, (
            wait_to_reset,
            update_game_timer,
        ))
        .add_systems(PostUpdate, (
            handle_snail_start,
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

#[derive(Resource, Default)]
pub struct FontHandles {
    pub timer_font_handle: Option<Handle<Font>>
}

#[derive(Resource, Default)]
pub struct GameTimer{
    pub running: bool,
    pub total_time_s: f32
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

fn load_fonts(
    mut fonts: ResMut<FontHandles>,
    asset_server: Res<AssetServer>
) {
    fonts.timer_font_handle = Some(asset_server.load("timer_font.ttf"));
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
    text_query: Query<Entity, With<Text>>,
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
            if let Ok(text) = text_query.get_single() {
                commands.entity(text).despawn();
            }
        }
    }
}

fn handle_win(
    mut on_stack_completed: EventReader<StackCompleted>,
    mut game_timer: ResMut<GameTimer>,
    images: Res<ImageHandles>,
    fonts: Res<FontHandles>,
    mut commands: Commands
) {
    if !on_stack_completed.is_empty() {
        on_stack_completed.clear();
        println!("YOU WIN");
        game_timer.running = false;
        commands.spawn((
            SpriteBundle {
                texture: images.win_bubble_handle.as_ref().expect("Images should be loaded").clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0))
                    .with_scale(Vec3::ONE * 2.0),
                ..default()
            },
            PopupTimer {
                timer: Timer::from_seconds(1.0, TimerMode::Once)
            }
        ));
        let time_string = format!("{:.1} s", game_timer.total_time_s);
        commands.spawn(Text2dBundle {
            text: Text::from_section(time_string, TextStyle {
                font: fonts.timer_font_handle.as_ref().expect("Fonts should be loaded").clone(),
                font_size: 160.00,
                ..default()
            }),
            text_anchor: Anchor::CenterLeft,
            transform: Transform::from_translation(Vec3::new(20.0, -230.0, 101.0)),
            ..default()
        });
    }
} 

fn handle_snail_start(
    mut on_snail_start: EventReader<SnailStarted>,
    mut game_time: ResMut<GameTimer>
) {
    if !on_snail_start.is_empty() {
        on_snail_start.clear();
        game_time.running = true;
        game_time.total_time_s = 0.0;
    }
}

fn update_game_timer(
    mut game_time: ResMut<GameTimer>,
    time: Res<Time>
) {
    if game_time.running {
        game_time.total_time_s += time.delta_seconds();
    }
}