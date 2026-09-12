use bevy::asset::AssetPlugin;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use wasm_bindgen::prelude::*;

const R7_GLTF: &str = "vortex-r7.glb";

#[derive(Resource)]
struct R7Scene(Handle<Scene>);

#[derive(Component)]
struct VortexAvatarRoot;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vortex WASM".into(),
                canvas: Some("#vortex-canvas".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }).set(AssetPlugin {
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, keep_avatar_grounded)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 8.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(R7_GLTF));
    commands.insert_resource(R7Scene(scene.clone()));
    commands.spawn((SceneRoot(scene), VortexAvatarRoot));
}

fn keep_avatar_grounded(
    mut roots: Query<&mut Transform, With<VortexAvatarRoot>>,
) {
    for mut transform in &mut roots {
        transform.translation.y = 0.0;
    }
}

#[wasm_bindgen]
pub fn start_vortex() {
    main();
}
