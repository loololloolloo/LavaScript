#!/usr/bin/env python3
"""Extract embedded Vortex web-relevant assets from Vortex.exe.

Only self-describing resource/container formats are extracted. The executable
is never executed or modified. GLB metadata is inspected so the recovered R7
avatar can be given a stable runtime filename.
"""
from __future__ import annotations
import argparse, hashlib, json, struct
from pathlib import Path

KTX2 = b'\xabKTX 20\xbb\r\n\x1a\n'

def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def find_all(data: bytes, sig: bytes):
    pos = 0
    while True:
        pos = data.find(sig, pos)
        if pos < 0: return
        yield pos
        pos += 1

def extract_png(data, off):
    end = data.find(b'\x00\x00\x00\x00IEND\xaeB`\x82', off + 8)
    return data[off:end + 12] if end >= 0 else None

def extract_riff(data, off):
    if off + 8 > len(data): return None
    size = struct.unpack_from('<I', data, off + 4)[0]
    end = off + 8 + size
    return data[off:end] if end <= len(data) else None

def parse_glb(blob: bytes):
    if len(blob) < 20: return {}
    try:
        chunk_len, chunk_type = struct.unpack_from('<II', blob, 12)
        if chunk_type != 0x4E4F534A: return {}
        doc = json.loads(blob[20:20 + chunk_len].rstrip(b' \0\t\r\n'))
    except Exception:
        return {}
    nodes = [n.get('name') for n in doc.get('nodes', []) if n.get('name')]
    meshes = [m.get('name') for m in doc.get('meshes', []) if m.get('name')]
    animations = [a.get('name') for a in doc.get('animations', []) if a.get('name')]
    return {'nodes': nodes, 'meshes': meshes, 'animations': animations}

def extract_glb(data, off):
    if off + 12 > len(data): return None
    version, total = struct.unpack_from('<II', data, off + 4)
    if version != 2 or total <= 12 or total > 64 * 1024 * 1024: return None
    end = off + total
    if end > len(data): return None
    p = off + 12
    if p + 8 > end: return None
    chunk_len, chunk_type = struct.unpack_from('<II', data, p)
    if chunk_type != 0x4e4f534a or p + 8 + chunk_len > end: return None
    try: json.loads(data[p + 8:p + 8 + chunk_len].rstrip(b' \0\t\r\n'))
    except Exception: return None
    return data[off:end]

def extract_ktx2(data, off):
    if off + 68 > len(data): return None
    level_count = struct.unpack_from('<I', data, off + 40)[0]
    if level_count == 0 or level_count > 4096: return None
    level_end = off + 68 + level_count * 24
    if level_end > len(data): return None
    max_end = level_end
    for i in range(level_count):
        bo, bl, _ = struct.unpack_from('<QQQ', data, off + 68 + i * 24)
        max_end = max(max_end, bo + bl)
    return data[off:max_end] if max_end <= len(data) else None

def extract_ogg_stream(data, off):
    if off + 27 > len(data): return None
    serial = struct.unpack_from('<I', data, off + 14)[0]
    pos = off
    last = None
    while pos + 27 <= len(data) and data[pos:pos + 4] == b'OggS':
        page_segments = data[pos + 26]
        if pos + 27 + page_segments > len(data): break
        body_len = sum(data[pos + 27:pos + 27 + page_segments])
        end = pos + 27 + page_segments + body_len
        if end > len(data) or struct.unpack_from('<I', data, pos + 14)[0] != serial: break
        last = end
        header_type = data[pos + 5]
        pos = end
        if header_type & 0x04: break
    return data[off:last] if last else None

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe', type=Path)
    ap.add_argument('-o', '--out', type=Path, default=Path('vortex-wasm/assets'))
    args = ap.parse_args()
    data = args.exe.read_bytes()
    args.out.mkdir(parents=True, exist_ok=True)
    manifest, seen = [], set()
    specs = [(b'glTF','glb',extract_glb),(b'\x89PNG\r\n\x1a\n','png',extract_png),(b'OggS','ogg',extract_ogg_stream),(b'RIFF','riff',extract_riff),(KTX2,'ktx2',extract_ktx2)]
    glb_index = 0
    for sig, ext, extractor in specs:
        for off in find_all(data, sig):
            blob = extractor(data, off)
            if not blob: continue
            h = sha256(blob); key = (ext, h)
            if key in seen: continue
            seen.add(key)
            meta = parse_glb(blob) if ext == 'glb' else {}
            r7 = ext == 'glb' and {'HumanoidRootPart','Torso','R7Body','R7Head'}.issubset(set(meta.get('nodes', [])))
            if ext == 'glb' and r7 and not any(m.get('classification') == 'vortex-r7-avatar' for m in manifest):
                name = 'vortex-r7.glb'
                classification = 'vortex-r7-avatar'
            else:
                index = len([m for m in manifest if m['format'] == ext])
                name = f'{ext}_{index:04d}.{ext}'
                classification = 'glb' if ext == 'glb' else ext
            path = args.out / name
            path.write_bytes(blob)
            item = {'format':ext,'path':str(path),'offset':off,'size':len(blob),'sha256':h,'classification':classification}
            if meta: item.update(meta)
            manifest.append(item)
            glb_index += ext == 'glb'
    (args.out / 'manifest.json').write_text(json.dumps(manifest, indent=2), encoding='utf-8')
    print(f'extracted {len(manifest)} objects to {args.out}')

if __name__ == '__main__': main()
