use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_mod_sprite3d::{Sprite3d, Sprite3dPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Sprite3dPlugin::<StandardMaterial>::default(),
        ))
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
    commands.spawn(Sprite3d {
        material: pokey_mat.clone(),
        ..Default::default()
    });

    // Pokey with specific size
    commands.spawn((
        Sprite3d { material: pokey_mat.clone(), custom_size: Some(Vec2::new(10.0, 5.0)), ..default() },
        Transform::from_xyz(1.0 * 32.0, 0.0, 0.0),
    ));

    // Pokey Rotated 45 degrees Z
    commands.spawn((
        Sprite3d { material: pokey_mat.clone(), ..Default::default() },
        Transform::from_xyz(2.0 * 32.0, 0.0, 0.0).with_rotation(Quat::from_rotation_z(PI/4.0)),
    ));

    // Pokey spinning around center
    commands.spawn((
        Sprite3d { material: pokey_mat.clone(), ..Default::default() },
        Transform::from_xyz(3.0 * 32.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(PI/3.0)),
        Spinner,
    ));

    // Pokey spinning around bottom-right
    commands.spawn((
        Sprite3d { material: pokey_mat.clone(), anchor: Anchor::BottomRight, ..default() },
        Transform::from_xyz(4.0 * 32.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(PI/3.0)),
        Spinner,
    ));

    // Health
    commands.spawn((
        Sprite3d { material: health_mat.clone(), ..Default::default() },
        Transform::from_xyz(0.0*64.0, 64.0, 0.0),
    ));

    // Health tinted
    commands.spawn((
        Sprite3d { material: health_mat.clone(), color: Color::linear_rgb(1.0, 0.0, 0.0), ..default() },
        Transform::from_xyz(1.0*64.0, 64.0, 0.0),
    ));

    // Health cropped
    commands.spawn((
        Sprite3d { material: health_mat.clone(), rect: Some(Rect::new(10.0, 10.0, 58.0 - 10.0, 52.0 - 10.0)), ..default() },
        Transform::from_xyz(2.0*64.0, 64.0, 0.0),
    ));

    // Health flipped
    commands.spawn((
        Sprite3d { material: health_mat.clone(), flip_x: false, flip_y: true, ..default() },
        Transform::from_xyz(3.0*64.0, 64.0, 0.0),
    ));
    
    // Light
    commands.spawn(DirectionalLight {
        illuminance: 5_000.0,
        ..Default::default()
    });

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 80.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spin(mut spinners: Query<&mut Transform, With<Spinner>>, time: Res<Time>) {
    for mut transf in &mut spinners {
        transf.rotate_y(1.0 / 5.0 * TAU * time.delta_secs());
    }
}

#[derive(Component)]
struct Spinner;
