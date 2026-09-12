mod manifest;

use bevy::asset::AssetPlugin;
use bevy::gltf::GltfAssetLabel;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use manifest::{VortexManifest, VortexManifestLoader};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const MANIFEST: &str = "manifest.json";
const MOVE_SPEED: f32 = 5.5;
const AIR_CONTROL: f32 = 0.35;
const GRAVITY: f32 = 24.0;
const JUMP_SPEED: f32 = 8.0;

#[derive(Resource)]
struct ManifestHandle(Handle<VortexManifest>);

#[derive(Resource, Default)]
struct RuntimeState {
    avatar_spawned: bool,
    velocity_y: f32,
}

#[derive(Resource)]
struct CameraState {
    yaw: f32,
    pitch: f32,
}

#[derive(Component)]
struct VortexAvatar;

#[derive(Component)]
struct VortexCamera;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_vortex() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Vortex".into(),
                        canvas: Some("#vortex-canvas".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .init_asset::<VortexManifest>()
        .register_asset_loader(VortexManifestLoader)
        .insert_resource(RuntimeState::default())
        .insert_resource(CameraState { yaw: 0.0, pitch: -0.08 })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_recovered_avatar,
                capture_mouse,
                look_camera,
                move_humanoid,
                follow_camera,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ManifestHandle(asset_server.load(MANIFEST)));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 7.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        VortexCamera,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_recovered_avatar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    manifest_handle: Res<ManifestHandle>,
    manifests: Res<Assets<VortexManifest>>,
    mut state: ResMut<RuntimeState>,
) {
    if state.avatar_spawned {
        return;
    }

    let Some(manifest) = manifests.get(&manifest_handle.0) else { return; };
    let Some(entry) = manifest.entries.iter().find(|entry| {
        entry.format == "glb" && entry.classification == "vortex-r7-avatar"
    }) else { return; };

    let path = entry
        .path
        .strip_prefix("vortex-wasm/assets/")
        .unwrap_or(&entry.path);
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(path));

    commands.spawn((SceneRoot(scene), Transform::default(), VortexAvatar));
    state.avatar_spawned = true;
}

fn capture_mouse(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = windows.single_mut() else { return; };

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

fn look_camera(
    mut motions: EventReader<MouseMotion>,
    mut camera: ResMut<CameraState>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = windows.single() else { return; };
    if window.cursor_options.grab_mode != CursorGrabMode::Locked {
        motions.clear();
        return;
    }

    let delta = motions.read().fold(Vec2::ZERO, |sum, event| sum + event.delta);
    camera.yaw -= delta.x * 0.0025;
    camera.pitch = (camera.pitch - delta.y * 0.0025).clamp(-FRAC_PI_2 + 0.05, FRAC_PI_2 - 0.05);
}

fn move_humanoid(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Res<CameraState>,
    mut state: ResMut<RuntimeState>,
    mut avatars: Query<&mut Transform, With<VortexAvatar>>,
) {
    let Ok(mut transform) = avatars.single_mut() else { return; };

    let mut input = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { input.y += 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { input.y -= 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { input.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { input.x += 1.0; }
    if input.length_squared() > 1.0 { input = input.normalize(); }

    let forward = Vec3::new(-camera.yaw.sin(), 0.0, -camera.yaw.cos());
    let right = Vec3::new(camera.yaw.cos(), 0.0, -camera.yaw.sin());
    let desired = (right * input.x + forward * input.y) * MOVE_SPEED;
    let grounded = transform.translation.y <= 0.001;
    let control = if grounded { 1.0 } else { AIR_CONTROL };

    transform.translation += desired * control * time.delta_secs();

    if grounded {
        transform.translation.y = 0.0;
        if keyboard.just_pressed(KeyCode::Space) { state.velocity_y = JUMP_SPEED; }
    } else {
        state.velocity_y -= GRAVITY * time.delta_secs();
    }

    transform.translation.y += state.velocity_y * time.delta_secs();
    if transform.translation.y < 0.0 {
        transform.translation.y = 0.0;
        state.velocity_y = 0.0;
    }

    if input.length_squared() > 0.0 {
        let (_, current_yaw, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let target = desired.x.atan2(-desired.z);
        let mut delta = target - current_yaw;
        while delta > PI { delta -= TAU; }
        while delta < -PI { delta += TAU; }
        let next = current_yaw + delta.clamp(-8.0 * time.delta_secs(), 8.0 * time.delta_secs());
        transform.rotation = Quat::from_rotation_y(next);
    }
}

fn follow_camera(
    camera_state: Res<CameraState>,
    avatars: Query<&Transform, With<VortexAvatar>>,
    mut cameras: Query<&mut Transform, (With<VortexCamera>, Without<VortexAvatar>)>,
) {
    let Ok(avatar) = avatars.single() else { return; };
    let Ok(mut camera) = cameras.single_mut() else { return; };

    let rotation = Quat::from_euler(EulerRot::YXZ, camera_state.yaw, camera_state.pitch, 0.0);
    camera.translation = avatar.translation + rotation * Vec3::new(0.0, 1.0, 6.0);
    camera.look_at(avatar.translation + Vec3::Y * 1.0, Vec3::Y);
}
