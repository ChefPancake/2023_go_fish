use bevy::{prelude::*, audio::{VolumeLevel, Volume}};

use crate::{hook::*, catch_stack::{FishLandedInStack, FishKnockedOutOfStack}, fish::FishLandedInWater};

pub struct HSLAudioPlugin;
impl Plugin for HSLAudioPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(AudioHandles::default())
        .add_systems(PreStartup, (
            add_music,
            load_sounds,
        ))
        .add_systems(Update,(
            handle_reeling,
            handle_reeling_stop,
            handle_fish_landed,
            handle_fish_landed_in_water,
            handle_fish_knocked_out,
            handle_hook_landed,
            handle_fish_reeled_to_surface
        ));
    }
}

#[derive(Resource, Default)]
pub struct AudioHandles {
    reeling_sound: Option<Handle<AudioSource>>,
    landed_sound: Option<Handle<AudioSource>>,
    knocked_out_sound: Option<Handle<AudioSource>>,
    critical_sound: Option<Handle<AudioSource>>,
    small_splash_sound: Option<Handle<AudioSource>>,
    splash_sound: Option<Handle<AudioSource>>,
}

#[derive(Component)]
pub struct Music;

#[derive(Component)]
pub struct ReelingSound;

fn add_music(
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    commands.spawn(AudioBundle {
        source: asset_server.load("bg_music.wav"),
        settings: PlaybackSettings::LOOP,
        ..default()
    });
}

fn load_sounds(
    asset_server: Res<AssetServer>,
    mut audio_handles: ResMut<AudioHandles>
) {
    audio_handles.reeling_sound = Some(asset_server.load("reeling.wav"));
    audio_handles.landed_sound = Some(asset_server.load("landed.wav"));
    audio_handles.knocked_out_sound = Some(asset_server.load("knocked_out.wav"));
    audio_handles.critical_sound = Some(asset_server.load("critical.wav"));
    audio_handles.splash_sound = Some(asset_server.load("splash.wav"));
    audio_handles.small_splash_sound = Some(asset_server.load("small_splash.wav"));
}

fn handle_reeling(
    mut on_catch: EventReader<FishCaught>,
    audio: Res<AudioHandles>,
    mut commands: Commands
) {
    let mut is_critical = false;
    let has_events = !on_catch.is_empty();
    for event in on_catch.iter() {
        is_critical |= event.is_critical;
    }
    if has_events {
        if is_critical {
            commands.spawn((
                AudioBundle {
                    source: audio.critical_sound.as_ref().expect("Audio should be loaded").clone(),
                    settings: PlaybackSettings::DESPAWN,
                    ..default()
                },
                ReelingSound
            ));
        } else {
            commands.spawn((
                AudioBundle {
                    source: audio.reeling_sound.as_ref().expect("Audio should be loaded").clone(),
                    settings: PlaybackSettings::DESPAWN,
                    ..default()
                },
                ReelingSound
            ));
        }
    }
}

fn handle_reeling_stop(
    mut on_catch: EventReader<ReeledToSurface>,
    mut audio: Query<&mut AudioSink, With<ReelingSound>>,
) {
    if !on_catch.is_empty() {
        on_catch.clear();
        if let Ok(audio) = audio.get_single_mut() {
            audio.stop();
        }
    }
}

fn handle_fish_landed(
    mut on_land: EventReader<FishLandedInStack>,
    audio: Res<AudioHandles>,
    mut commands: Commands
) {
    if !on_land.is_empty() {
        on_land.clear();
        commands.spawn(AudioBundle {
            source: audio.landed_sound.as_ref().expect("Audio should be loaded").clone(),
            settings: PlaybackSettings::DESPAWN,
            ..default()
        });
    }
}

fn handle_fish_knocked_out(
    mut on_ko: EventReader<FishKnockedOutOfStack>,
    audio: Res<AudioHandles>,
    mut commands: Commands
) {
    if !on_ko.is_empty() {
        on_ko.clear();
        commands.spawn(AudioBundle {
            source: audio.knocked_out_sound.as_ref().expect("Audio should be loaded").clone(),
            settings: PlaybackSettings::DESPAWN,
            ..default()
        });
    }
}

fn handle_hook_landed(
    mut on_land: EventReader<HookLandedInWater>,
    audio: Res<AudioHandles>,
    mut commands: Commands
) {
    if !on_land.is_empty() {
        on_land.clear();
        commands.spawn(AudioBundle {
            source: audio.small_splash_sound.as_ref().expect("Audio should be loaded").clone(),
            settings: PlaybackSettings::DESPAWN,
            ..default()
        });
    }
}

fn handle_fish_reeled_to_surface(
    mut on_reeled: EventReader<ReeledToSurface>,
    audio: Res<AudioHandles>,
    mut commands: Commands
) {
    if !on_reeled.is_empty() {
        on_reeled.clear();
        commands.spawn(AudioBundle {
            source: audio.splash_sound.as_ref().expect("Audio should be loaded").clone(),
            settings: PlaybackSettings::DESPAWN.with_volume(Volume::Relative(VolumeLevel::new(0.8))),
            ..default()
        });
    }
}

fn handle_fish_landed_in_water(
    mut on_land: EventReader<FishLandedInWater>,
    audio: Res<AudioHandles>,
    mut commands: Commands
) {
    if !on_land.is_empty() {
        on_land.clear();
        commands.spawn(AudioBundle {
            source: audio.splash_sound.as_ref().expect("Audio should be loaded").clone(),
            settings: PlaybackSettings::DESPAWN.with_volume(Volume::Relative(VolumeLevel::new(0.8))),
            ..default()
        });
    }
}