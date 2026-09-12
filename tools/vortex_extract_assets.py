#!/usr/bin/env python3
"""Extract embedded Vortex web-relevant assets from Vortex.exe.

This is intentionally conservative: it extracts only self-describing container
formats found inside the supplied executable and writes a manifest with hashes.
No code is executed and no PE sections are modified.
"""
from __future__ import annotations
import argparse, hashlib, json, struct
from pathlib import Path

def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def find_all(data: bytes, sig: bytes):
    pos = 0
    while True:
        pos = data.find(sig, pos)
        if pos < 0:
            return
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
    try:
        json.loads(data[p + 8:p + 8 + chunk_len].rstrip(b' \0\t\r\n'))
    except Exception:
        return None
    return data[off:end]

def extract_ktx2(data, off):
    if off + 68 > len(data): return None
    # KTX2 header contains levelCount at byte 40 and DFD/KVD/SGD ranges after it.
    level_count = struct.unpack_from('<I', data, off + 40)[0]
    if level_count == 0 or level_count > 4096: return None
    end = off + 68 + level_count * 24
    if end > len(data): return None
    # Include all bytes through the largest declared payload range.
    for base in (off + 48,):
        for i in range(4):
            pass
    max_end = end
    # level index begins at byte 68; each entry is byteOffset, byteLength, uncompressedLength.
    for i in range(level_count):
        bo, bl, _ = struct.unpack_from('<QQQ', data, off + 68 + i * 24)
        max_end = max(max_end, bo + bl)
    if max_end <= len(data):
        return data[off:max_end]
    return None

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('exe', type=Path)
    ap.add_argument('-o', '--out', type=Path, default=Path('vortex-wasm/assets'))
    args = ap.parse_args()
    data = args.exe.read_bytes()
    args.out.mkdir(parents=True, exist_ok=True)
    manifest = []
    seen = set()
    specs = [(b'glTF', 'glb', extract_glb), (b'\x89PNG\r\n\x1a\n', 'png', extract_png), (b'OggS', 'ogg', None), (b'RIFF', 'riff', extract_riff), (b'KTX 20\xbb\r\n\x1a\n', 'ktx2', extract_ktx2)]
    for sig, ext, extractor in specs:
        for off in find_all(data, sig):
            if extractor is None:
                # Ogg pages are streamed containers; capture a conservative page and
                # let the manifest expose the offset for a later stream-aware pass.
                if off + 27 > len(data): continue
                segs = data[off + 26]
                if off + 27 + segs > len(data): continue
                size = 27 + segs + sum(data[off + 27:off + 27 + segs])
                blob = data[off:off + size]
            else:
                blob = extractor(data, off)
            if not blob: continue
            h = sha256(blob)
            key = (ext, h)
            if key in seen: continue
            seen.add(key)
            name = f'{ext}_{len([m for m in manifest if m["format"] == ext]):04d}.{ext}'
            path = args.out / name
            path.write_bytes(blob)
            manifest.append({'format':ext,'path':str(path),'offset':off,'size':len(blob),'sha256':h})
    (args.out / 'manifest.json').write_text(json.dumps(manifest, indent=2), encoding='utf-8')
    print(f'extracted {len(manifest)} objects to {args.out}')

if __name__ == '__main__': main()
