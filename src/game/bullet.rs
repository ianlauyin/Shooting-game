use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

use crate::asset_loader::MeshHandles;
use crate::constants::WINDOW_SIZE;
use crate::game::Velocity;
use crate::states::GameState;
use crate::ui::ZIndexMap;

#[derive(Event)]
pub struct ShootBulletEvent {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Event)]
pub struct RemoveBulletEvent;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_bullet.run_if(in_state(GameState::InPlay)))
            .observe(shoot_bullet);
    }
}

#[derive(Component)]
struct Bullet;

#[derive(Bundle)]
struct BulletBundle {
    bullet: Bullet,
    velocity: Velocity,
    mesh_bundle: MaterialMesh2dBundle<ColorMaterial>,
}

impl BulletBundle {
    fn new(
        x: f32,
        y: f32,
        mesh_handle: Handle<Mesh>,
        material_handle: Handle<ColorMaterial>,
    ) -> Self {
        Self {
            bullet: Bullet,
            velocity: Velocity { x: 0., y: 10. },
            mesh_bundle: MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: material_handle,
                transform: Transform {
                    translation: Vec3::new(x, y, ZIndexMap::Bullet.value()),
                    scale: Vec3::new(5., 5., 0.),
                    ..default()
                },
                ..default()
            },
        }
    }
}

fn shoot_bullet(
    trigger: Trigger<ShootBulletEvent>,
    mut commands: Commands,
    mesh_handles: Res<MeshHandles>,
) {
    let ShootBulletEvent { x, y } = trigger.event();
    let (mesh, material) = mesh_handles.bullet.clone();
    commands.spawn(BulletBundle::new(
        x.clone() - 20.,
        y.clone(),
        mesh.clone(),
        material.clone(),
    ));
    commands.spawn(BulletBundle::new(
        x.clone() + 20.,
        y.clone(),
        mesh,
        material,
    ));
}

fn check_bullet(mut commands: Commands, bullet_queries: Query<(Entity, &Transform), With<Bullet>>) {
    for (entity, transform) in bullet_queries.iter() {
        if transform.translation.y > WINDOW_SIZE.y / 2. {
            commands.entity(entity).despawn();
        }
    }
}