# Vortex asset recovery

The browser target is intended to use **the actual data embedded in the supplied Vortex client**, not substitute assets designed to resemble it.

## Confirmed embedded data

Static scanning of the supplied `Vortex/Vortex.exe` found embedded binary payloads with the signatures of:

- GLB/glTF 2.0 models
- PNG textures
- Ogg audio
- KTX2 textures

The recovered GLB JSON contains Vortex-specific avatar nodes including `HumanoidRootPart`, `Torso`, `Right Arm`, `Left Arm`, `Right Leg`, `Left Leg`, `R7Head`, `R7LArm`, `R7LLeg`, `R7RArm`, `R7RLeg`, `R7Torso`, and `R7Body`.

## Extraction

Run the extractor against the original executable:

```bash
python3 tools/extract_vortex_assets.py /path/to/Vortex/Vortex.exe
```

It writes recovered payloads to `vortex-wasm/assets/recovered/` and creates a `manifest.json` containing source offsets and sizes.

The extractor is read-only with respect to the executable. It uses the native container formats to determine asset boundaries instead of blindly slicing between magic signatures.

## Integration target

The recovered assets need to become the browser client's real asset inputs:

1. load the recovered GLB avatar instead of placeholder geometry;
2. preserve the original node/skeleton names and animation clips;
3. load recovered textures/material inputs;
4. reproduce the Vortex material/shader behavior from the embedded WGSL;
5. connect those assets to the recovered `Player`/`Humanoid` state;
6. replace native-only APIs with browser equivalents while preserving gameplay semantics.

The existing WebGL scene is only a bootstrap harness until the recovered Vortex data is wired in. The goal is to port Vortex itself, not make a visual recreation.
