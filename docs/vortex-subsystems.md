# Vortex subsystem reconstruction map

This document records the current binary-derived architecture map for the Vortex Windows build. Names are evidence from embedded Rust strings/source paths; they are not a claim that stripped native symbols have been perfectly recovered.

## Core

`vortex_engine::GamePlugin` appears to be the main engine/plugin composition point. The recovered namespace also exposes `Player`, gameplay state, settings, chat, health, physics, sound/effects, avatar extensions, and rendering materials.

## Portable candidates

| Subsystem | Evidence | WASM status |
|---|---|---|
| ECS / game loop | Bevy 0.19.1 + GamePlugin | High |
| Player/entity model | `vortex_engine::Player`, `Instance`, project/avatar types | High |
| Physics | `vortex_engine::physics::Humanoid` | High; browser build needs a WASM-compatible physics backend/config |
| Health | `vortex_engine::health` | High |
| Chat | `vortex_engine::chat` | High; transport layer must be browser-safe |
| Settings | `vortex_engine::settings` | High after replacing native persistence |
| Audio/SFX | `vortex_engine::sfx` | Medium-high |
| Avatar materials | `ClothingExtension`, `FaceDecalExtension`, `StudExtension` | High |
| Rendering | Bevy render/PBR/mesh/transform + WGPU | High; browser backend is WebGPU/WebGL depending on target |
| Custom shaders | `lensflare.wgsl`, `stud_material.wgsl`, `face_decal_material.wgsl` | High |
| UI | `bevy_egui::EguiPlugin` | High with browser integration |
| Asset system | `bevy_asset`, embedded assets, GLTF-related strings | High |
| Windows platform | Win32/NT/COM/WinRT/raw input/Winsock imports | Replace, not port literally |

## Native-only boundaries

The executable imports Windows facilities for process creation, registry/system configuration, window creation, raw input, clipboard/UI automation, Winsock, COM/WinRT, and related OS services. Those calls should become a thin browser-host boundary rather than being translated instruction-for-instruction.

## Rendering path

The binary contains WGPU/WebGPU markers plus backend names for DirectX 12, Vulkan, OpenGL, browser/WebGPU, and a noop backend. That is strong evidence that the rendering abstraction already separates much of the game from the platform graphics API.

## Network boundary

The build embeds `connect.playvortex.io:7777`. The WASM client should isolate networking behind a browser-compatible transport interface. Native socket calls cannot be reused directly inside browser WASM.

## Next reconstruction pass

1. Parse `.pdata` exception/unwind records to enumerate native function ranges.
2. Correlate function ranges with nearby Rust/source-path strings and code references.
3. Build a dependency graph around `GamePlugin`, player/avatar, physics, render, UI, and network code.
4. Identify protocol/message type names and serialization evidence.
5. Extract embedded custom assets/shaders where legally and technically appropriate for reconstruction.
6. Start a minimal WASM crate containing only independently reconstructed portable systems; keep platform adapters separate.
