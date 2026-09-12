mod manifest;

use bevy::asset::AssetPlugin;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use manifest::{VortexManifest, VortexManifestLoader};
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::time::Duration;

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
const ANIMATION_CROSSFADE: Duration = Duration::from_millis(200);

#[derive(Resource)]
struct ManifestHandle(Handle<VortexManifest>);

#[derive(Resource)]
struct AvatarGltfHandle(Handle<Gltf>);

#[derive(Resource, Default)]
struct RuntimeState {
    avatar_spawned: bool,
    started: bool,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VortexAnimationState {
    Idle,
    Run,
    Jump,
    Fall,
}

#[derive(Resource)]
struct VortexAnimationLibrary {
    graph_handle: Handle<AnimationGraph>,
    states: HashMap<VortexAnimationState, AnimationNodeIndex>,
}

#[derive(Component)]
struct AvatarAnimationState {
    current: VortexAnimationState,
    idle: AnimationNodeIndex,
    run: AnimationNodeIndex,
    jump: AnimationNodeIndex,
    fall: AnimationNodeIndex,
}

#[derive(Component)]
struct StartOverlay;

#[derive(Component)]
struct StartButton;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_AUDIO_CONTEXT: std::cell::RefCell<Option<web_sys::AudioContext>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
fn resume_web_audio_context() {
    WEB_AUDIO_CONTEXT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            if let Ok(context) = web_sys::AudioContext::new() {
                let _ = context.resume();
                *slot = Some(context);
                return;
            }
        }
        if let Some(context) = slot.as_ref() {
            let _ = context.resume();
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn resume_web_audio_context() {}

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
        .add_systems(Startup, (setup, setup_start_overlay).chain())
        .add_systems(
            Update,
            (
                build_animation_library,
                spawn_recovered_avatar,
                bind_animation_players,
                start_overlay_input,
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
    let gltf_handle = asset_server.load::<Gltf>("avatar.glb");
    commands.insert_resource(AvatarGltfHandle(gltf_handle));
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

fn setup_start_overlay(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(14.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        StartOverlay,
        children![
            (
                Text::new("VORTEX"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ),
            (
                Text::new("Click to start"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ),
            (
                Button,
                StartButton,
                Node {
                    width: px(230.0),
                    height: px(64.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(2.0)),
                    border_radius: BorderRadius::all(px(10.0)),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
                children![
                    (
                        Text::new("CLICK TO START / LOCK MOUSE"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ),
                ],
            ),
        ],
    ));
}

fn start_overlay_input(
    mut commands: Commands,
    mut state: ResMut<RuntimeState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<StartButton>)>,
    overlays: Query<Entity, With<StartOverlay>>,
) {
    for (interaction, mut color) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                state.started = true;
                *color = Color::srgb(0.2, 0.2, 0.2).into();
                resume_web_audio_context();

                if let Ok(mut window) = windows.single_mut() {
                    window.cursor_options.visible = false;
                    window.cursor_options.grab_mode = CursorGrabMode::Locked;
                }

                for entity in &overlays {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.2, 0.2, 0.2).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.12, 0.12, 0.12).into();
            }
        }
    }
}

fn build_animation_library(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltf_handle: Res<AvatarGltfHandle>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    library: Option<Res<VortexAnimationLibrary>>,
) {
    if library.is_some() || !asset_server.is_loaded_with_dependencies(&gltf_handle.0) {
        return;
    }

    let Some(gltf) = gltfs.get(&gltf_handle.0) else { return; };
    if gltf.named_animations.is_empty() {
        return;
    }

    let clips: Vec<Handle<AnimationClip>> = gltf.named_animations.values().cloned().collect();
    let (graph, node_indices) = AnimationGraph::from_clips(clips);
    let graph_handle = graphs.add(graph);

    let names: Vec<String> = gltf.named_animations.keys().cloned().collect();
    let mut mapped = HashMap::new();
    for state in [
        VortexAnimationState::Idle,
        VortexAnimationState::Run,
        VortexAnimationState::Jump,
        VortexAnimationState::Fall,
    ] {
        let chosen = choose_animation_for_state(state, &names);
        let index = chosen
            .and_then(|name| names.iter().position(|candidate| candidate == name))
            .and_then(|position| node_indices.get(position).copied())
            .unwrap_or(node_indices[0]);
        mapped.insert(state, index);
    }

    info!("Vortex animations: {:?}", names);
    commands.insert_resource(VortexAnimationLibrary {
        graph_handle,
        states: mapped,
    });
}

fn choose_animation_for_state(state: VortexAnimationState, names: &[String]) -> Option<&String> {
    let keywords: &[&str] = match state {
        VortexAnimationState::Idle => &["idle", "stand", "rest", "breath"],
        VortexAnimationState::Run => &["run", "running", "sprint", "jog", "walk"],
        VortexAnimationState::Jump => &["jump", "jumping", "leap", "hop"],
        VortexAnimationState::Fall => &["fall", "falling", "air", "airborne", "descend"],
    };

    names.iter().max_by_key(|name| {
        let lower = name.to_ascii_lowercase();
        keywords
            .iter()
            .enumerate()
            .filter(|(_, keyword)| lower.contains(**keyword))
            .map(|(index, keyword)| 1000 - index as i32 * 10 + keyword.len() as i32)
            .max()
            .unwrap_or(0)
    })
    .filter(|name| {
        let lower = name.to_ascii_lowercase();
        keywords.iter().any(|keyword| lower.contains(keyword))
    })
}

fn spawn_recovered_avatar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    manifest_handle: Res<ManifestHandle>,
    manifests: Res<Assets<VortexManifest>>,
    gltfs: Res<Assets<Gltf>>,
    gltf_handle: Res<AvatarGltfHandle>,
    mut state: ResMut<RuntimeState>,
) {
    if state.avatar_spawned {
        return;
    }

    let Some(manifest) = manifests.get(&manifest_handle.0) else { return; };
    let Some(entry) = manifest.entries.iter().find(|entry| {
        entry.format == "glb" && entry.classification == "vortex-r7-avatar"
    }) else { return; };

    if gltfs.get(&gltf_handle.0).is_none() {
        let path = entry
            .path
            .strip_prefix("vortex-wasm/assets/")
            .unwrap_or(&entry.path);
        let loaded = asset_server.load::<Gltf>(path);
        commands.insert_resource(AvatarGltfHandle(loaded));
        return;
    }

    let path = entry
        .path
        .strip_prefix("vortex-wasm/assets/")
        .unwrap_or(&entry.path);
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

fn bind_animation_players(
    mut commands: Commands,
    library: Option<Res<VortexAnimationLibrary>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Without<AnimationGraphHandle>>,
) {
    let Some(library) = library else { return; };
    let Some(&idle) = library.states.get(&VortexAnimationState::Idle) else { return; };
    let Some(&run) = library.states.get(&VortexAnimationState::Run) else { return; };
    let Some(&jump) = library.states.get(&VortexAnimationState::Jump) else { return; };
    let Some(&fall) = library.states.get(&VortexAnimationState::Fall) else { return; };

    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        transitions.play(&mut player, idle, Duration::ZERO).repeat();
        commands.entity(entity).insert((
            AnimationGraphHandle(library.graph_handle.clone()),
            transitions,
            AvatarAnimationState {
                current: VortexAnimationState::Idle,
                idle,
                run,
                jump,
                fall,
            },
        ));
        break;
    }
}

fn capture_mouse(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    state: Res<RuntimeState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !state.started {
        return;
    }
    let Ok(mut window) = windows.single_mut() else { return; };
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
        resume_web_audio_context();
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
    state: Res<RuntimeState>,
    mut avatars: Query<(&mut Transform, &CapsuleController), With<VortexAvatar>>,
    mut runtime: ResMut<RuntimeState>,
) {
    if !state.started {
        return;
    }

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

    let was_grounded = runtime.grounded;
    runtime.grounded = transform.translation.y <= capsule.half_height + 0.002;

    let accel = if runtime.grounded { GROUND_ACCEL } else { AIR_ACCEL };
    runtime.velocity.x = approach(runtime.velocity.x, desired.x, accel * dt);
    runtime.velocity.z = approach(runtime.velocity.z, desired.z, accel * dt);

    if input.length_squared() == 0.0 && runtime.grounded {
        let horizontal = Vec3::new(runtime.velocity.x, 0.0, runtime.velocity.z);
        let slowed = approach_vec3(horizontal, Vec3::ZERO, GROUND_FRICTION * dt);
        runtime.velocity.x = slowed.x;
        runtime.velocity.z = slowed.z;
    }

    if runtime.grounded {
        transform.translation.y = capsule.half_height;
        if keyboard.just_pressed(KeyCode::Space) {
            runtime.velocity.y = JUMP_SPEED;
            runtime.grounded = false;
        } else {
            runtime.velocity.y = -0.5;
        }
    } else {
        runtime.velocity.y -= GRAVITY * dt;
    }

    transform.translation += runtime.velocity * dt;

    let floor_y = capsule.half_height;
    if transform.translation.y <= floor_y {
        transform.translation.y = floor_y;
        runtime.velocity.y = 0.0;
        runtime.grounded = true;
    }

    let horizontal_velocity = Vec3::new(runtime.velocity.x, 0.0, runtime.velocity.z);
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
    state: Res<RuntimeState>,
    mut players: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &mut AvatarAnimationState,
    )>,
) {
    if !state.started {
        return;
    }

    let speed = Vec3::new(state.velocity.x, 0.0, state.velocity.z).length();
    let desired = if !state.grounded {
        if state.velocity.y > 0.0 {
            VortexAnimationState::Jump
        } else {
            VortexAnimationState::Fall
        }
    } else if speed > 0.25 {
        VortexAnimationState::Run
    } else {
        VortexAnimationState::Idle
    };

    for (mut player, mut transitions, mut animation_state) in &mut players {
        if animation_state.current == desired {
            continue;
        }

        let node = match desired {
            VortexAnimationState::Idle => animation_state.idle,
            VortexAnimationState::Run => animation_state.run,
            VortexAnimationState::Jump => animation_state.jump,
            VortexAnimationState::Fall => animation_state.fall,
        };

        transitions
            .play(&mut player, node, ANIMATION_CROSSFADE)
            .repeat();
        animation_state.current = desired;
    }
}
