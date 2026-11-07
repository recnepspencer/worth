"""Exact compiled entry lookup and native stack unwinding."""
import bisect,ctypes as c,json,struct,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from pdb_public_frames import Pdb
from lifecycle_compiled_frames import Frames
from whole_binary_frames import canonical
from windows_debug import d,H,Q,get
class Symbols:
 def __init__(self,exe,entry):
  self.exe=Path(exe);self.pe=Frames(exe);self.pdb=self.exe.with_suffix('.pdb');self.wanted=entry;self.entries={};self.exact={}
  if not self.pdb.exists():self.pdb=self.exe.with_name(self.exe.stem.replace('-','_')+'.pdb')
  p=Pdb(self.pdb);dbi=p.read(3);record_stream=struct.unpack_from('<H',dbi,20)[0];records=p.read(record_stream)
  self.scan(records,0)
  if 'entry' not in self.entries:
   mi=64;modules=struct.unpack_from('<I',dbi,24)[0]
   while mi<64+modules:
    stream=struct.unpack_from('<H',dbi,mi+34)[0];size=struct.unpack_from('<I',dbi,mi+36)[0]
    end=dbi.index(b'\0',mi+64)+1;end=dbi.index(b'\0',end)+1;mi=(end+3)&~3
    if stream!=0xffff and size>4:self.scan(p.read(stream)[:size],4)
  p.close();assert set(self.entries)=={'entry','probe'},self.entries
 def scan(self,data,pos):
  suffix=self.wanted.split('::')[-1].encode();needle=str(len(suffix)).encode()+suffix
  while pos+4<=len(data):
   length,tag=struct.unpack_from('<HH',data,pos)
   if length<2:raise ValueError(('bad PDB record',pos))
   if tag==0x110e:
    _,off,section=struct.unpack_from('<IIH',data,pos+4);name=data[pos+14:pos+length+2].split(b'\0')[0]
   elif tag in (0x110f,0x1110,0x1146,0x1147,0x1155,0x1156):
    off,section=struct.unpack_from('<IH',data,pos+32);name=data[pos+39:pos+length+2].split(b'\0')[0]
   else:pos+=length+2;continue
   if section and section<=len(self.pe.sections) and (((needle in name or (b'::'+suffix) in name) and self.wanted.split('::')[0].encode() in name) or b'chkstk' in name):
    rva=self.pe.sections[section-1][1]+off;text=name.decode(errors='replace');display,_=canonical(text)
    if display==self.wanted:self.entries['entry']=rva
    if text.lstrip('_')=='chkstk':self.entries['probe']=rva
   pos+=length+2
 def function(self,rva):
  i=bisect.bisect_right(self.pe.starts,rva)-1
  if i<0:return None
  begin,end,unwind=self.pe.functions[i]
  return (begin,end,self.pe.unwind(unwind)) if rva<end else None
 def capture(self,process,thread,ctx,base):
  d.SymRefreshModuleList(process);copy=c.create_string_buffer(ctx.raw,1232);frame=c.create_string_buffer(512)
  for off,value in [(0,get(ctx,'rip')),(32,get(ctx,'rbp')),(48,get(ctx,'rsp'))]:struct.pack_into('<Q',frame,off,value);struct.pack_into('<I',frame,off+12,3)
  rows=[];seen=set()
  for _ in range(256):
   pc=struct.unpack_from('<Q',frame,0)[0];sp=struct.unpack_from('<Q',frame,48)[0]
   if (pc,sp) not in seen and pc:
    seen.add((pc,sp));row={'pc':pc,'sp':sp};f=self.function(pc-base) if pc>=base else None
    symbol=c.create_string_buffer(16384);struct.pack_into('<I',symbol,0,88);struct.pack_into('<I',symbol,80,16000);disp=Q()
    if d.SymFromAddr(process,pc,c.byref(disp),symbol):
     addr=struct.unpack_from('<Q',symbol,56)[0];length=struct.unpack_from('<I',symbol,76)[0];raw=symbol.raw[84:84+length].decode(errors='replace')
     if f and addr==base+f[0]:row['symbol_path']=canonical(raw)[0] or raw;row['symbol_exact']=True
     elif not f:row['external_symbol']=raw;row['symbol_address']=addr
    if f:row.update(function_rva=f[0],rva=pc-base,static_frame=f[2])
    rows.append(row)
   if not d.StackWalk64(0x8664,process,thread,frame,copy,None,c.cast(d.SymFunctionTableAccess64,H),c.cast(d.SymGetModuleBase64,H),None):break
  for a,b in zip(rows,rows[1:]):a['live_frame_bytes']=b['sp']-a['sp']
  return rows
