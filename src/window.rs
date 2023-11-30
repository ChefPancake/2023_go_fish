use bevy::{prelude::*, window::{WindowResized, PrimaryWindow, WindowResolution }};
use crate::constants::*;

pub struct HSLWindowPlugin;
impl Plugin for HSLWindowPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(ClearColor(background_color()))
        .add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GO FISH".to_string(),
                fit_canvas_to_parent: true,
                resolution: WindowResolution::new(704.0, 527.0),
                ..default()
            }),
            ..default()
        }))
        .add_systems(PostUpdate, window_resize);
    }
}

fn window_resize(
    mut resize_events: EventReader<WindowResized>,
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
    mut query: Query<&mut OrthographicProjection>,
) {
    for resize_event in resize_events.iter() {
        if let Ok(entity) = primary_window_query.get_single() {   
            if resize_event.window == entity {
                let min_ratio = 
                    (resize_event.width / BACKGROUND_SIZE.x)
                    .min(resize_event.height / BACKGROUND_SIZE.y);
                query.single_mut().scale = 1.0 / min_ratio;
            }
        }
    }
}