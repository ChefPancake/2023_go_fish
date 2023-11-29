use bevy::prelude::*;
use crate::constants::*;

pub struct CorePlugin;
impl Plugin for CorePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
        .add_event::<ResetLevel>()
        .insert_resource(ImageHandles::default())
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(PreStartup, (
            load_images,
        ))
        .add_systems(Startup, (
            add_camera,
            add_bg,
            add_water,
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

    pub bear_atlas_handle: Option<Handle<TextureAtlas>>,
    pub fish_atlas_handle: Option<Handle<TextureAtlas>>,
    pub stack_atlas_handle: Option<Handle<TextureAtlas>>,
    pub misc_atlas_handle: Option<Handle<TextureAtlas>>
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
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -1000.0)),
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
            transform: Transform::from_translation(Vec3::new(WATER_POS.x, WATER_POS.y - 65.0, 1.0)),
            ..default()
        }
    );
}