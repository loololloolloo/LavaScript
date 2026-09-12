#!/usr/bin/env python3
"""Extract embedded Vortex assets from Vortex.exe.

This is intentionally deterministic and read-only: it never modifies the input
executable. It recovers actual binary assets embedded in the supplied client,
including GLB, PNG, Ogg and KTX2 payloads.
"""
from __future__ import annotations
import argparse, json, struct
from pathlib import Path

MAGICS = {
    "glb": b"glTF",
    "png": b"\x89PNG\r\n\x1a\n",
    "ogg": b"OggS",
    "ktx2": b"\xabKTX 20\xbb\r\n\x1a\n",
}

def find_all(data: bytes, magic: bytes):
    start = 0
    while True:
        off = data.find(magic, start)
        if off < 0:
            return
        yield off
        start = off + 1

def glb_end(data, off):
    if off + 12 > len(data): return None
    version, total = struct.unpack_from("<II", data, off + 4)
    if version != 2 or total < 20 or off + total > len(data): return None
    return off + total

def png_end(data, off):
    p = off + 8
    while p + 12 <= len(data):
        n = struct.unpack_from(">I", data, p)[0]
        end = p + 12 + n
        if end > len(data): return None
        if data[p+4:p+8] == b"IEND": return end
        p = end
    return None

def ogg_end(data, off):
    p = off
    last = None
    while p + 27 <= len(data) and data[p:p+4] == b"OggS":
        segs = data[p+26]
        if p + 27 + segs > len(data): return None
        size = sum(data[p+27:p+27+segs])
        end = p + 27 + segs + size
        if end > len(data): return None
        last = end
        if data[p+5] & 0x04: return end
        p = end
    return last

def ktx2_end(data, off):
    # KTX2 header is 80 bytes followed by level index entries (24 bytes each).
    if off + 80 > len(data): return None
    level_count = struct.unpack_from("<I", data, off + 40)[0]
    if level_count == 0 or level_count > 10000: return None
    table_end = off + 80 + level_count * 24
    if table_end > len(data): return None
    end = table_end
    for i in range(level_count):
        byte_offset, byte_length, _ = struct.unpack_from("<QQQ", data, off + 80 + i * 24)
        if byte_offset == 0: continue
        candidate = off + byte_offset + byte_length
        if candidate > len(data): return None
        end = max(end, candidate)
    return end

def extract(data, out):
    records=[]
    seen=set()
    specs=[("glb",glb_end), ("png",png_end), ("ogg",ogg_end), ("ktx2",ktx2_end)]
    for kind,end_fn in specs:
        for off in find_all(data,MAGICS[kind]):
            end=end_fn(data,off)
            if not end or end<=off or (off,end) in seen: continue
            seen.add((off,end))
            blob=data[off:end]
            idx=len([r for r in records if r["type"]==kind])
            name=f"{kind}_{idx:03d}.{kind if kind!='ktx2' else 'ktx2'}"
            path=out/name
            path.write_bytes(blob)
            records.append({"type":kind,"offset":off,"size":len(blob),"path":str(path)})
    return records

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("exe",type=Path)
    ap.add_argument("-o","--out",type=Path,default=Path("vortex-wasm/assets/recovered"))
    args=ap.parse_args()
    data=args.exe.read_bytes(); args.out.mkdir(parents=True,exist_ok=True)
    records=extract(data,args.out)
    manifest=args.out/"manifest.json"; manifest.write_text(json.dumps(records,indent=2)+"\n")
    print(f"recovered {len(records)} assets -> {args.out}")
    for r in records: print(f"{r['type']:4} 0x{r['offset']:x} {r['size']:8} {r['path']}")

if __name__ == "__main__": main()
