# Vortex WASM reconstruction

This directory is the browser-side reconstruction track for Vortex.

The first milestone is deliberately small: a browser-hosted playable movement shell that establishes the input/render/update loop without pretending the original Windows PE can execute directly in a browser.

## Current controls

- `WASD` moves the player
- browser canvas is the render surface
- the update loop runs independently of the Windows executable

## Next milestones

1. Replace the 2D shell with a WebGPU/Bevy scene.
2. Port the recovered player/entity state.
3. Add camera and humanoid movement.
4. Add physics/collision.
5. Integrate recovered assets/materials.
6. Add browser-compatible networking.
7. Reach a representative playable Vortex game scene.
