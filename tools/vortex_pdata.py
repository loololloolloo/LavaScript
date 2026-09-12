#!/usr/bin/env python3
"""Enumerate x64 PE exception/unwind function ranges from .pdata."""
from __future__ import annotations
import argparse, json, struct
from pathlib import Path

def u16(b,o): return struct.unpack_from('<H',b,o)[0]
def u32(b,o): return struct.unpack_from('<I',b,o)[0]

def parse(path):
    b=Path(path).read_bytes(); pe=u32(b,0x3c); fh=pe+4; n=u16(b,fh+2); opt=fh+20
    if b[pe:pe+4]!=b'PE\0\0' or u16(b,opt)!=0x20b: raise ValueError('expected PE32+')
    entry=u32(b,opt+16); image_base=struct.unpack_from('<Q',b,opt+24)[0]
    sh=opt+u16(b,fh+16); secs=[]
    for i in range(n):
        o=sh+40*i; name=b[o:o+8].split(b'\0',1)[0].decode('ascii','replace')
        secs.append((name,u32(b,o+12),u32(b,o+8),u32(b,o+20),u32(b,o+16)))
    pdata=next((s for s in secs if s[0]=='.pdata'),None)
    if not pdata: return {'functions':[],'entry_rva':entry,'image_base':image_base}
    _,va,vs,raw,rs=pdata; size=min(vs,rs); count=size//12; funcs=[]
    for i in range(count):
        o=raw+i*12; begin,end,unwind=struct.unpack_from('<III',b,o)
        if begin < end: funcs.append({'begin_rva':begin,'end_rva':end,'size':end-begin,'unwind_rva':unwind})
    funcs.sort(key=lambda x:x['begin_rva'])
    return {'entry_rva':entry,'image_base':image_base,'pdata_rva':va,'pdata_size':size,'functions':funcs}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('exe'); ap.add_argument('--json'); ap.add_argument('--markdown'); a=ap.parse_args(); x=parse(a.exe)
    if a.json: Path(a.json).write_text(json.dumps(x,indent=2)+'\n',encoding='utf-8')
    if a.markdown:
        f=x['functions']; lines=['# Vortex `.pdata` function ranges','',f"Recovered **{len(f)}** unwind entries.",'','| Begin RVA | End RVA | Size | Unwind RVA |','|---:|---:|---:|---:|']
        for z in f: lines.append(f"| `0x{z['begin_rva']:x}` | `0x{z['end_rva']:x}` | `{z['size']}` | `0x{z['unwind_rva']:x}` |")
        Path(a.markdown).write_text('\n'.join(lines)+'\n',encoding='utf-8')
    if not a.json and not a.markdown: print(json.dumps(x,indent=2))
if __name__=='__main__': main()
