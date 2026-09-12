# Vortex asset recovery

The browser client must use recovered Vortex data rather than invented replacements.

Static analysis of the supplied `Vortex/Vortex.exe` found embedded self-describing assets, including:

- GLB/glTF 2.0 avatar models
- PNG textures
- Ogg audio streams
- RIFF media
- KTX2 textures

The strongest confirmed GLB at offset `115472112` is 79,456 bytes and contains the actual Vortex R7 avatar mesh set:

- `R7Head`
- `R7LArm`
- `R7LLeg`
- `R7RArm`
- `R7RLeg`
- `R7Torso.001`
- `R7Body`

Its node hierarchy also contains `Right Arm`, `Left Arm`, `Right Leg`, `Left Leg`, `Head`, `Torso`, `HumanoidRootPart`, `Armature.001`, and `Body`.

Additional valid GLB containers were found at executable offsets `115551578`, `137782874`, `138130716`, `138152916`, and `138178548`.

## Extraction

Run the read-only extractor against the supplied executable:

```bash
python3 tools/vortex_extract_assets.py /path/to/Vortex/Vortex.exe
```

It writes recovered containers into `vortex-wasm/assets/` and creates `manifest.json` with offsets, sizes, and SHA-256 hashes.

## Porting rule

Do not replace recovered Vortex models, materials, animation data, or assets with lookalikes when the original data can be recovered. The browser runtime should progressively consume the recovered data and reconstructed behavior.

The current primitive world renderer is only a temporary harness for validating browser input/rendering. It is not considered the Vortex implementation milestone.
