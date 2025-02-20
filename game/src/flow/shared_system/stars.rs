use bevy::prelude::*;

use crate::constant::{ZIndex, MOBILE_WINDOW_SIZE};
use crate::res::ImageHandles;
use crate::states::AppState;
use crate::ui_component::{Blink, Velocity};

pub struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (check_stars_number, cleanup_stars)
                .run_if(in_state(AppState::Game).or(in_state(AppState::MainMenu))),
        );
    }
}

#[derive(Component)]
struct Stars;

fn check_stars_number(
    mut commands: Commands,
    stars_query: Query<&Transform, With<Stars>>,
    image_handles: Res<ImageHandles>,
) {
    let stars_handle = image_handles.stars.clone();
    let Some(first_star_transform) = stars_query.iter().next() else {
        spawn_star(&mut commands, stars_handle);
        return;
    };
    if stars_query.iter().len() == 1 && first_star_transform.translation.y < 0. {
        spawn_star(&mut commands, stars_handle);
    }
}

fn spawn_star(commands: &mut Commands, stars_handle: Handle<Image>) {
    commands.spawn((
        Stars,
        Blink::new(0.001, 0.1, 0.001),
        Velocity { x: 0., y: -2. },
        Sprite {
            image: stars_handle,
            ..default()
        },
        Transform {
            scale: Vec2::new(0.8, 0.8).extend(0.),
            translation: Vec3::new(0., MOBILE_WINDOW_SIZE.y, ZIndex::STARS.z_value()),
            ..default()
        },
    ));
}

fn cleanup_stars(mut commands: Commands, stars_query: Query<(Entity, &Transform), With<Stars>>) {
    for (entity, transform) in stars_query.iter() {
        if transform.translation.y <= -MOBILE_WINDOW_SIZE.y {
            commands.entity(entity).despawn();
        }
    }
}
