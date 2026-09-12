#!/usr/bin/env python3
"""Recover symbol-like Rust/Vortex names from a stripped Vortex PE.

This does not claim to recover true symbols. It extracts embedded demangled
Rust paths, source paths, and module/type/function-looking strings and groups
them into a useful architecture inventory.
"""
from __future__ import annotations
import argparse, json, re
from pathlib import Path

RUST = re.compile(rb'(?:vortex_engine|bevy_[a-zA-Z0-9_]+|wgpu|winit|rapier3d|egui)(?:::[A-Za-z0-9_.$<>-]+){1,12}')
SOURCE = re.compile(rb'(?:/private/tmp/vortex[^\x00]{0,300}|/build/\.cargo/registry/src/[^\x00]{0,300})')

def strings(data, minlen=6):
    out=[]
    for m in re.finditer(rb'[ -~]{%d,}' % minlen, data): out.append(m.group().decode('ascii','replace'))
    return out

def classify(s):
    if s.startswith('vortex_engine::'): return 'vortex'
    if s.startswith('bevy_'): return 'bevy'
    if s.startswith('wgpu'): return 'render'
    if s.startswith('winit'): return 'window_input'
    if s.startswith('rapier'): return 'physics'
    if s.startswith('egui'): return 'ui'
    return 'other'

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('exe'); ap.add_argument('--json'); ap.add_argument('--markdown')
    a=ap.parse_args(); data=Path(a.exe).read_bytes()
    syms=sorted(set(x.decode('utf-8','replace') for x in RUST.findall(data)))
    src=sorted(set(x.decode('utf-8','replace') for x in SOURCE.findall(data)))
    groups={k:[] for k in ('vortex','bevy','render','window_input','physics','ui','other')}
    for s in syms: groups[classify(s)].append(s)
    result={'counts':{k:len(v) for k,v in groups.items()},'symbols':groups,'source_paths':src}
    if a.json: Path(a.json).write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    if a.markdown:
        lines=['# Vortex recovered symbol inventory','',f"Unique symbol-like paths: **{len(syms)}**",'']
        for k,v in groups.items():
            lines += [f'## {k}', ''] + [f'- `{s}`' for s in v] + ['']
        Path(a.markdown).write_text('\n'.join(lines),encoding='utf-8')
    if not a.json and not a.markdown: print(json.dumps(result,indent=2))
if __name__=='__main__': main()
