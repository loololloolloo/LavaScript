# Vortex → WASM reverse-engineering notes

Target: `Vortex/Vortex.exe` from `Vortex-Windows.zip`.

This branch is for static analysis and a future browser/WASM reconstruction. The original executable is not copied into the repository.

## Initial binary facts

- PE32+ Windows GUI executable
- x86-64 machine code
- 172,496,896 bytes
- 11 PE sections
- `.text`: 0x06717730 bytes (~103.1 MiB)
- `.rdata`: 0x03052558 bytes (~48.1 MiB)
- `.pdata`/`.xdata` contain substantial unwind metadata
- Imports include Win32/NT, Winsock, COM/WinRT, graphics/system DLLs, and CRT components

## Strong identification signals

Static strings identify the binary as **Vortex v0.5.3** and expose a large amount of Rust/Bevy type information despite stripped symbols.

Observed engine/framework markers include:

- `vortex_engine::GamePlugin`
- `vortex_engine::Player`
- `vortex_engine::physics::*`
- `vortex_engine::health::*`
- `vortex_engine::chat::*`
- `vortex_engine::settings::*`
- `vortex_engine::sfx::*`
- `bevy_egui::EguiPlugin`
- `bevy_window::WindowPlugin`
- `bevy_render`
- `bevy_pbr`
- `bevy_mesh`
- `bevy_transform`
- `bevy_input`
- `bevy_asset`
- `bevy_gizmos`
- `bevy_sprite_render`

The embedded registry source paths identify Bevy **0.19.1** in this build.

## Rendering evidence

The executable embeds WGPU configuration strings and explicitly contains:

`NoopVulkanMetalDx12GlBrowserWebGpu`

It also contains DirectX 12, Vulkan, OpenGL 3.3+, WebGPU/WGPU, and many WGSL shader/resource names. This is strong evidence that the rendering layer is already organized around WGPU and that browser GPU backends are conceptually close to the native renderer.

Observed custom shader/resource names include:

- `shaders/lensflare.wgsl`
- `shaders/stud_material.wgsl`
- `shaders/face_decal_material.wgsl`
- `animgraph.ron`

There are also extensive Bevy PBR, post-processing, atmosphere, OIT, bloom, motion blur, shadow, and mesh-processing shader strings.

## Networking evidence

The binary contains:

- `connect.playvortex.io:7777`
- `https://playvortex.io`
- client/server bootstrap strings
- leaderboard/chat/message-related systems

A browser port cannot use arbitrary native sockets. The network protocol therefore needs to be identified and mapped to a browser-compatible transport, most likely WebSocket/WebTransport or a web gateway.

## Windows-only surface area

Native imports and strings show dependencies on Win32/NT, COM/WinRT, registry/system configuration, process/thread primitives, filesystem, user input/windowing, Winsock, and graphics drivers.

These should be treated as platform adapters, not targets for machine-code translation.

## Portability assessment

| Subsystem | Initial assessment | Strategy |
|---|---|---|
| ECS/gameplay | High | Reconstruct Rust/Bevy systems |
| Player/entity model | High | Reconstruct from recovered type graph |
| Physics | High | Isolate pure simulation and compile to WASM |
| Health/chat/settings | High | Rebuild as portable systems |
| Asset loading | High | Browser asset pipeline |
| GLTF/model handling | High | Bevy/web-compatible assets |
| WGSL/custom materials | High | Reuse/adapt shaders |
| WGPU rendering | High | Browser WebGPU path |
| UI/egui | High | Browser-compatible window/event adapter |
| Audio/SFX | Medium-high | Browser audio backend |
| Networking | Medium | Identify protocol; add browser transport |
| Windows window/input | Low as-is | Replace with browser backend |
| Registry/system settings | Low as-is | Replace with web storage/settings |
| Process launching/updater | Low as-is | Replace with web deployment/update flow |

## Next reverse-engineering passes

1. Parse the PE import directory into a normalized API inventory.
2. Extract and deduplicate all `vortex_engine` module/type/function strings.
3. Recover function boundaries from `.pdata` and map addresses to the recovered symbol-like names where possible.
4. Extract embedded resources/assets and identify which are game-specific versus Bevy framework assets.
5. Build a subsystem dependency graph around `GamePlugin`, rendering, physics, networking, avatar/clothing, UI, and bootstrap code.
6. Identify the network message/protocol structures from nearby type names and call sites.
7. Create a minimal browser/WASM scaffold that can host the reconstructed portable pieces without attempting to execute the original PE.

## Important constraint

This is a reconstruction/porting project, not a literal PE-to-WASM conversion. Native x86-64 machine code, Windows APIs, and browser APIs are different execution environments. The useful output of this reverse-engineering work is a recoverable architecture and portable implementation plan.
