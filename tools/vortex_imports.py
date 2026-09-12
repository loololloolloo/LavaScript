#!/usr/bin/env python3
"""Read-only PE import inventory for Vortex.exe.

Stdlib only. Run against the original Windows executable and emit a compact
JSON or Markdown inventory of imported DLLs/functions.
"""
from __future__ import annotations

import argparse
import json
import struct
from pathlib import Path


def u16(b, o): return struct.unpack_from('<H', b, o)[0]
def u32(b, o): return struct.unpack_from('<I', b, o)[0]
def u64(b, o): return struct.unpack_from('<Q', b, o)[0]


def rva_to_off(data, sections, rva):
    for s in sections:
        va, size, raw, raw_size = s
        span = max(size, raw_size)
        if va <= rva < va + span:
            return raw + (rva - va)
    return None


def parse(path):
    data = Path(path).read_bytes()
    if data[:2] != b'MZ': raise ValueError('not an MZ executable')
    pe = u32(data, 0x3c)
    if data[pe:pe+4] != b'PE\0\0': raise ValueError('bad PE signature')
    fh = pe + 4
    nsec = u16(data, fh + 2)
    opt = fh + 20
    magic = u16(data, opt)
    if magic != 0x20b: raise ValueError('expected PE32+')
    dd = opt + 112
    imp_rva = u32(data, dd + 8)
    imp_size = u32(data, dd + 12)
    sh = opt + u16(data, fh + 16)
    sections = []
    for i in range(nsec):
        o = sh + 40*i
        name = data[o:o+8].split(b'\0',1)[0].decode('ascii','replace')
        sections.append((u32(data,o+12),u32(data,o+8),u32(data,o+20),u32(data,o+16),name))
    mapped = [(a,b,c,d) for a,b,c,d,_ in sections]
    off = rva_to_off(data, mapped, imp_rva)
    if off is None: return {'dlls': [], 'import_directory': {'rva': imp_rva, 'size': imp_size}}
    out=[]
    while off + 20 <= len(data):
        oft, _, _, name_rva, ft = struct.unpack_from('<IIIII', data, off)
        if not any((oft, name_rva, ft)): break
        no = rva_to_off(data, mapped, name_rva)
        dll = ''
        if no is not None:
            end=data.find(b'\0',no)
            dll=data[no:end if end>=0 else len(data)].decode('ascii','replace')
        thunk_rva = oft or ft
        to = rva_to_off(data, mapped, thunk_rva)
        funcs=[]
        if to is not None:
            for i in range(100000):
                x=u64(data,to+i*8)
                if x==0: break
                if x & (1<<63): funcs.append({'ordinal': x & 0xffff})
                else:
                    no=rva_to_off(data,mapped,x & 0x7fffffff)
                    if no is None: funcs.append({'raw_rva':x & 0x7fffffff}); continue
                    hint=u16(data,no); end=data.find(b'\0',no+2)
                    name=data[no+2:end if end>=0 else len(data)].decode('ascii','replace')
                    funcs.append({'name':name,'hint':hint})
        out.append({'dll':dll,'functions':funcs})
        off += 20
    return {'import_directory': {'rva':imp_rva,'size':imp_size}, 'dlls':out}


def markdown(obj):
    lines=['# Vortex PE import inventory','',f"Import directory RVA: `0x{obj['import_directory']['rva']:x}`",'']
    for d in obj['dlls']:
        lines += [f"## `{d['dll']}`",'']
        for f in d['functions']:
            lines.append(f"- `{f.get('name', '#'+hex(f.get('ordinal',0)))}`")
        lines.append('')
    return '\n'.join(lines)


def main():
    ap=argparse.ArgumentParser(); ap.add_argument('exe'); ap.add_argument('--json'); ap.add_argument('--markdown')
    a=ap.parse_args(); obj=parse(a.exe)
    if a.json: Path(a.json).write_text(json.dumps(obj,indent=2)+'\n',encoding='utf-8')
    if a.markdown: Path(a.markdown).write_text(markdown(obj),encoding='utf-8')
    if not a.json and not a.markdown: print(json.dumps(obj,indent=2))

if __name__=='__main__': main()
