use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_flycam::prelude::*;
use bevy_mod_sprite3d::{Sprite3d, Sprite3dBundle, Sprite3dPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Sprite3dPlugin::<StandardMaterial>::default(),
            NoCameraPlayerPlugin,
        ))
        .insert_resource(MovementSettings {
            speed: 150.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, spin)
        .run();
}


/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {

    let pokey_mat = materials.add(StandardMaterial {
        base_color_texture: Some(assets.load("pokey.png")),
        reflectance: 0.0,
        perceptual_roughness: 1.0,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    let health_mat = materials.add(StandardMaterial {
        base_color_texture: Some(assets.load("health.png")),
        reflectance: 0.0,
        perceptual_roughness: 1.0,
        cull_mode: None,
        ..default()
    });

    // Pokey with default size
    commands.spawn(Sprite3dBundle {
        material: pokey_mat.clone(),
        ..default()
    });

    // Pokey with specific size
    commands.spawn(Sprite3dBundle {
        sprite3d: Sprite3d {
            custom_size: Some(Vec2::new(10.0, 5.0)),
            ..default()
        },
        material: pokey_mat.clone(),
        transform: Transform::from_xyz(1.0 * 32.0, 0.0, 0.0),
        ..default()
    });

    // Pokey Rotated 45 degrees Z
    commands.spawn(Sprite3dBundle {
        material: pokey_mat.clone(),
        transform: Transform::from_xyz(2.0 * 32.0, 0.0, 0.0).with_rotation(Quat::from_rotation_z(PI/4.0)),
        ..default()
    });

    // Pokey spinning around center
    commands.spawn(
        (
            Sprite3dBundle {
                material: pokey_mat.clone(),
                transform: Transform::from_xyz(3.0 * 32.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(PI/3.0)),
                ..default()
            },
            Spinner,
        )
    );

    // Pokey spinning around bottom-right
    commands.spawn(
        (
            Sprite3dBundle {
                sprite3d: Sprite3d {
                    anchor: Anchor::BottomRight,
                    ..default()
                },
                material: pokey_mat.clone(),
                transform: Transform::from_xyz(4.0 * 32.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(PI/3.0)),
                ..default()
            },
            Spinner,
        )
    );

    // Health
    commands.spawn(Sprite3dBundle {
        material: health_mat.clone(),
        transform: Transform::from_xyz(0.0, 64.0, 0.0),
        ..default()
    });

    // Health tinted
    commands.spawn(Sprite3dBundle {
        sprite3d: Sprite3d {
            color: Color::linear_rgb(1.0, 0.0, 0.0),
            ..default()
        },
        material: health_mat.clone(),
        transform: Transform::from_xyz(64.0, 64.0, 0.0),
        ..default()
    });

    // Health cropped
    commands.spawn(Sprite3dBundle {
        sprite3d: Sprite3d {
            rect: Some(Rect::new(10.0, 10.0, 58.0 - 10.0, 52.0 - 10.0)),
            ..default()
        },
        material: health_mat.clone(),
        transform: Transform::from_xyz(128.0, 64.0, 0.0),
        ..default()
    });
    
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5_000.0,
            ..default()
        },
        ..default()
    });
    commands.spawn(
        (
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 80.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            FlyCam,
        )
    );
}

fn spin(
    mut spinners: Query<&mut Transform, With<Spinner>>,
    time: Res<Time>,
) {
    for mut transf in &mut spinners {
        transf.rotate_y(1.0 / 5.0 * TAU * time.delta_seconds());
    }
}

#[derive(Component)]
struct Spinner;