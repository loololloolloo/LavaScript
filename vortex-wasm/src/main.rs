mod manifest;

use bevy::asset::AssetPlugin;
use bevy::gltf::{Gltf, GltfAssetLabel};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use manifest::{VortexManifest, VortexManifestLoader};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const MANIFEST: &str = "manifest.json";
const MAX_SPEED: f32 = 5.5;
const GROUND_ACCEL: f32 = 28.0;
const AIR_ACCEL: f32 = 8.0;
const GROUND_FRICTION: f32 = 18.0;
const GRAVITY: f32 = 24.0;
const JUMP_SPEED: f32 = 8.0;
const CAPSULE_RADIUS: f32 = 0.38;
const CAPSULE_HALF_HEIGHT: f32 = 0.96;
const CAMERA_DISTANCE: f32 = 5.2;
const CAMERA_HEIGHT: f32 = 1.15;

#[derive(Resource)]
struct ManifestHandle(Handle<VortexManifest>);

#[derive(Resource)]
struct AvatarGltfHandle(Handle<Gltf>);

#[derive(Resource, Default)]
struct RuntimeState {
    avatar_spawned: bool,
    velocity: Vec3,
    grounded: bool,
}

#[derive(Resource)]
struct CameraState {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

#[derive(Component)]
struct VortexAvatar;

#[derive(Component)]
struct HumanoidRootPart;

#[derive(Component)]
struct VortexCamera;

#[derive(Component, Clone, Copy)]
struct CapsuleController {
    radius: f32,
    half_height: f32,
}

#[derive(Component)]
struct AvatarAnimationState {
    idle: Option<AnimationNodeIndex>,
    run: Option<AnimationNodeIndex>,
    jump: Option<AnimationNodeIndex>,
    current: Option<AnimationNodeIndex>,
}

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
        .insert_resource(CameraState {
            yaw: 0.0,
            pitch: -0.10,
            distance: CAMERA_DISTANCE,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_recovered_avatar,
                capture_mouse,
                look_camera,
                humanoid_controller,
                follow_third_person_camera,
                update_avatar_animation,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ManifestHandle(asset_server.load(MANIFEST)));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 7.0),
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

    let path = entry.path.strip_prefix("vortex-wasm/assets/").unwrap_or(&entry.path);
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(path));

    commands.spawn((
        SceneRoot(scene),
        Transform::from_xyz(0.0, CAPSULE_HALF_HEIGHT, 0.0),
        VortexAvatar,
        HumanoidRootPart,
        CapsuleController {
            radius: CAPSULE_RADIUS,
            half_height: CAPSULE_HALF_HEIGHT,
        },
    ));
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
    camera.pitch = (camera.pitch - delta.y * 0.0025)
        .clamp(-FRAC_PI_2 + 0.12, FRAC_PI_2 - 0.12);
}

fn approach(current: f32, target: f32, amount: f32) -> f32 {
    if current < target {
        (current + amount).min(target)
    } else {
        (current - amount).max(target)
    }
}

fn approach_vec3(current: Vec3, target: Vec3, amount: f32) -> Vec3 {
    let delta = target - current;
    let length = delta.length();
    if length <= amount || length == 0.0 {
        target
    } else {
        current + delta / length * amount
    }
}

fn humanoid_controller(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Res<CameraState>,
    mut state: ResMut<RuntimeState>,
    mut avatars: Query<(&mut Transform, &CapsuleController), With<VortexAvatar>>,
) {
    let Ok((mut transform, capsule)) = avatars.single_mut() else { return; };
    let dt = time.delta_secs().min(0.05);

    let mut input = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { input.y += 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { input.y -= 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { input.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { input.x += 1.0; }
    if input.length_squared() > 1.0 { input = input.normalize(); }

    let forward = Vec3::new(-camera.yaw.sin(), 0.0, -camera.yaw.cos());
    let right = Vec3::new(camera.yaw.cos(), 0.0, -camera.yaw.sin());
    let desired = (right * input.x + forward * input.y) * MAX_SPEED;

    let was_grounded = state.grounded;
    state.grounded = transform.translation.y <= capsule.half_height + 0.002;

    let accel = if state.grounded { GROUND_ACCEL } else { AIR_ACCEL };
    state.velocity.x = approach(state.velocity.x, desired.x, accel * dt);
    state.velocity.z = approach(state.velocity.z, desired.z, accel * dt);

    if input.length_squared() == 0.0 && state.grounded {
        let horizontal = Vec3::new(state.velocity.x, 0.0, state.velocity.z);
        let slowed = approach_vec3(horizontal, Vec3::ZERO, GROUND_FRICTION * dt);
        state.velocity.x = slowed.x;
        state.velocity.z = slowed.z;
    }

    if state.grounded {
        transform.translation.y = capsule.half_height;
        if keyboard.just_pressed(KeyCode::Space) {
            state.velocity.y = JUMP_SPEED;
            state.grounded = false;
        } else {
            state.velocity.y = -0.5;
        }
    } else {
        state.velocity.y -= GRAVITY * dt;
    }

    transform.translation += state.velocity * dt;

    let floor_y = capsule.half_height;
    if transform.translation.y <= floor_y {
        transform.translation.y = floor_y;
        state.velocity.y = 0.0;
        state.grounded = true;
    }

    let horizontal_velocity = Vec3::new(state.velocity.x, 0.0, state.velocity.z);
    if horizontal_velocity.length_squared() > 0.01 {
        let target_yaw = horizontal_velocity.x.atan2(-horizontal_velocity.z);
        let (_, current_yaw, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let mut delta = target_yaw - current_yaw;
        while delta > PI { delta -= TAU; }
        while delta < -PI { delta += TAU; }
        let turn_rate = if was_grounded { 10.0 } else { 5.0 };
        let next = current_yaw + delta.clamp(-turn_rate * dt, turn_rate * dt);
        transform.rotation = Quat::from_rotation_y(next);
    }
}

fn follow_third_person_camera(
    time: Res<Time>,
    camera_state: Res<CameraState>,
    avatars: Query<&Transform, With<VortexAvatar>>,
    mut cameras: Query<&mut Transform, (With<VortexCamera>, Without<VortexAvatar>)>,
) {
    let Ok(avatar) = avatars.single() else { return; };
    let Ok(mut camera) = cameras.single_mut() else { return; };

    let rotation = Quat::from_euler(EulerRot::YXZ, camera_state.yaw, camera_state.pitch, 0.0);
    let target = avatar.translation + Vec3::Y * CAMERA_HEIGHT;
    let desired_position = target + rotation * Vec3::new(0.0, 0.0, camera_state.distance);
    let smoothing = 1.0 - (-14.0 * time.delta_secs()).exp();
    camera.translation = camera.translation.lerp(desired_position, smoothing);
    camera.look_at(target, Vec3::Y);
}

fn update_avatar_animation(
    gltfs: Res<Assets<Gltf>>,
    gltf_handle: Option<Res<AvatarGltfHandle>>,
    state: Res<RuntimeState>,
    mut players: Query<(&mut AnimationPlayer, &mut AvatarAnimationState)>,
) {
    let Some(handle) = gltf_handle else { return; };
    let Some(gltf) = gltfs.get(&handle.0) else { return; };
    let _ = (&state, &mut players, gltf);
    // AnimationPlayer ownership is established by the glTF scene loader. Once the
    // recovered GLB's named clips are indexed here, transitions are driven from
    // grounded/speed state rather than from a replacement character rig.
}
