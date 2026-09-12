#!/usr/bin/env python3
"""Lightweight static scanner for the Vortex Windows PE.

No third-party packages are required. The scanner is intentionally read-only:
it extracts PE section metadata and printable ASCII/UTF-16 strings, then groups
strings that are useful for reconstructing a browser/WASM port.
"""
from __future__ import annotations

import argparse
import re
import struct
from collections import Counter
from pathlib import Path


def u16(b, o):
    return struct.unpack_from("<H", b, o)[0]


def u32(b, o):
    return struct.unpack_from("<I", b, o)[0]


def u64(b, o):
    return struct.unpack_from("<Q", b, o)[0]


def pe_sections(data: bytes):
    if data[:2] != b"MZ":
        raise ValueError("not a PE file")
    pe = u32(data, 0x3C)
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("invalid PE signature")
    coff = pe + 4
    machine = u16(data, coff)
    count = u16(data, coff + 2)
    opt_size = u16(data, coff + 16)
    opt = coff + 20
    magic = u16(data, opt)
    image_base = u64(data, opt + 24) if magic == 0x20B else u32(data, opt + 28)
    entry_rva = u32(data, opt + 16)
    section_off = opt + opt_size
    sections = []
    for i in range(count):
        o = section_off + i * 40
        name = data[o:o + 8].split(b"\0", 1)[0].decode("ascii", "replace")
        vsize, va, raw_size, raw_ptr = struct.unpack_from("<IIII", data, o + 8)
        sections.append((name, va, vsize, raw_ptr, raw_size))
    return machine, image_base, entry_rva, sections


def ascii_strings(data: bytes, minimum=6):
    pat = re.compile(rb"[ -~]{%d,}" % minimum)
    return [m.group().decode("ascii", "replace") for m in pat.finditer(data)]


def utf16_strings(data: bytes, minimum=6):
    pat = re.compile(rb"(?:[ -~]\x00){%d,}" % minimum)
    out = []
    for m in pat.finditer(data):
        out.append(m.group().decode("utf-16le", "replace"))
    return out


def classify(strings):
    rules = {
        "vortex_modules": r"(?:^|::)vortex_engine::|Vortex::",
        "bevy": r"\bbevy[_:]|bevy-",
        "render_wgpu": r"wgpu|WGPU|WebGPU|WebGL|DirectX|Vulkan|OpenGL|shader|\.wgsl",
        "assets": r"\.gltf|\.glb|\.png|\.jpg|\.jpeg|\.ogg|\.wav|\.ron|\.json|\.toml",
        "network": r"connect\.playvortex\.io|playvortex\.io|socket|websocket|http",
        "windows": r"\.dll$|Win32|Windows|Registry|CreateWindow|CreateProcess|WinRT|USER32|KERNEL32",
        "gameplay": r"Player|physics|health|chat|clothing|avatar|camera|input|touch|sfx|leaderboard",
        "source_paths": r"\.cargo/registry/src/|/engine/src/|/src/[^ ]+\.rs",
    }
    return {k: sorted({s for s in strings if re.search(v, s, re.I)}) for k, v in rules.items()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("exe", type=Path)
    ap.add_argument("-o", "--output", type=Path)
    args = ap.parse_args()
    data = args.exe.read_bytes()
    machine, image_base, entry_rva, sections = pe_sections(data)
    strings = sorted(set(ascii_strings(data) + utf16_strings(data)))
    groups = classify(strings)
    dlls = sorted({s for s in strings if s.lower().endswith(".dll")})

    lines = [
        f"# Vortex static scan: {args.exe.name}",
        "",
        f"- Size: {len(data):,} bytes",
        f"- PE machine: 0x{machine:04x} ({'x86-64' if machine == 0x8664 else 'other'})",
        f"- Image base: 0x{image_base:x}",
        f"- Entry RVA: 0x{entry_rva:x}",
        "",
        "## Sections",
        "",
        "| Name | RVA | Virtual size | Raw offset | Raw size |",
        "|---|---:|---:|---:|---:|",
    ]
    for name, va, vs, rp, rs in sections:
        lines.append(f"| `{name}` | `0x{va:x}` | {vs:,} | `0x{rp:x}` | {rs:,} |")
    lines += ["", "## Imported DLL names recovered from strings", ""]
    lines += [f"- `{x}`" for x in dlls]
    for title, values in groups.items():
        lines += ["", f"## {title}", ""]
        # Keep the committed report manageable while preserving representative evidence.
        for value in values[:250]:
            lines.append(f"- `{value}`")
    report = "\n".join(lines) + "\n"
    if args.output:
        args.output.write_text(report, encoding="utf-8")
    else:
        print(report)


if __name__ == "__main__":
    main()
