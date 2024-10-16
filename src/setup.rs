use bevy::app::{App, Startup};
use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::states::{AppState, GameState};
use crate::ui::WINDOW_SIZE;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::from(WINDOW_SIZE),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_sub_state::<GameState>()
        .add_systems(Startup, setup_camera);
    }
}
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
