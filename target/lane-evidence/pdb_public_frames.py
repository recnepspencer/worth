"""Read every public CodeView record, including duplicate names omitted by DbgHelp."""
import json,sys,struct,mmap,collections,gc,hashlib
from pathlib import Path
e=Path(__file__).resolve().parent;sys.path.insert(0,str(e))
from lifecycle_compiled_frames import Frames
class Pdb:
 def __init__(self,path):
  self.file=open(path,'rb');self.data=mmap.mmap(self.file.fileno(),0,access=mmap.ACCESS_READ)
  self.block,self.free,self.count,self.directory_bytes,self.reserved,self.map=struct.unpack_from('<6I',self.data,32)
  count=(self.directory_bytes+self.block-1)//self.block
  blocks=struct.unpack_from('<'+str(count)+'I',self.data,self.map*self.block)
  directory=b''.join(self.data[n*self.block:(n+1)*self.block] for n in blocks)[:self.directory_bytes]
  stream_count=struct.unpack_from('<I',directory)[0];sizes=struct.unpack_from('<'+str(stream_count)+'I',directory,4)
  pos=4+4*stream_count;self.streams=[]
  for size in sizes:
   count=0 if size==0xffffffff else (size+self.block-1)//self.block
   blocks=struct.unpack_from('<'+str(count)+'I',directory,pos);pos+=4*count
   self.streams.append((size,blocks))
 def read(self,index):
  size,blocks=self.streams[index]
  return b''.join(self.data[n*self.block:(n+1)*self.block] for n in blocks)[:size]
 def close(self):self.data.close();self.file.close()

def main():
 data=json.loads(Path(sys.argv[1]).read_text())
 for b in data['binaries']:
  exe=Path(b['exe']);pdb=exe.with_suffix('.pdb')
  if not pdb.exists():pdb=exe.with_name(exe.stem.replace('-','_')+'.pdb')
  if not pdb.exists():raise ValueError(pdb)
  p=Pdb(pdb);dbi=p.read(3);symbols=struct.unpack_from('<H',dbi,20)[0];records=p.read(symbols)
  pe=Frames(str(exe));large={r['rva']:r for r in b['large_functions']};n=0;pos=0;tags=collections.Counter()
  header=pe.u32(0x3c)+24;debug,size=struct.unpack_from('<II',pe.data,header+112+6*8)
  signatures=[]
  for entry in range(pe.offset(debug),pe.offset(debug)+size,28):
   if pe.u32(entry+12)!=2:continue
   cv=pe.u32(entry+24)
   if pe.data[cv:cv+4]==b'RSDS':signatures.append((pe.data[cv+4:cv+20],pe.u32(cv+20)))
  info=p.read(1);signature=(info[12:28],struct.unpack_from('<I',info,8)[0])
  assert signature in signatures,('PE/PDB identity mismatch',exe,pdb)
  b['artifact_identity']={'exe_sha256':hashlib.sha256(pe.data).hexdigest(),'pdb_sha256':hashlib.sha256(p.data).hexdigest(),'pdb_guid_bytes':signature[0].hex(),'pdb_age':signature[1],'pdb_path':str(pdb)}
  while pos<len(records):
   length,tag=struct.unpack_from('<HH',records,pos);tags[tag]+=1
   if length<2:raise ValueError((pos,length))
   if tag==0x110e:
    flags,offset,section=struct.unpack_from('<IIH',records,pos+4)
    if section and section<=len(pe.sections):
     rva=pe.sections[section-1][1]+offset
     if rva in large:
      name=records[pos+14:pos+2+length].split(b'\0')[0].decode(errors='replace')
      large[rva].setdefault('public_linkage_names',[]).append(name)
      if name not in large[rva]['names']:large[rva]['names'].append(name);n+=1
   pos+=length+2
  modules=struct.unpack_from('<I',dbi,24)[0];mi=64;module_functions=0
  while mi<64+modules:
   stream=struct.unpack_from('<H',dbi,mi+34)[0];symbytes=struct.unpack_from('<I',dbi,mi+36)[0]
   mend=dbi.index(b'\0',mi+64)+1;mend=dbi.index(b'\0',mend)+1;mi=(mend+3)&~3
   if stream==0xffff or symbytes<4:continue
   mod=p.read(stream)[:symbytes];mp=4
   while mp+4<=len(mod):
    length,tag=struct.unpack_from('<HH',mod,mp)
    if length<2:raise ValueError(('module',stream,mp,length))
    if tag in (0x110f,0x1110,0x1146,0x1147,0x1155,0x1156):
     offset,section=struct.unpack_from('<IH',mod,mp+32)
     if section and section<=len(pe.sections):
      rva=pe.sections[section-1][1]+offset
      if rva in large:
       name=mod[mp+39:mp+2+length].split(b'\0')[0].decode(errors='replace')
       if name not in large[rva]['names']:large[rva]['names'].append(name);module_functions+=1
    mp+=length+2
  print(b['binary'],'PDB',pdb.stat().st_size,'extra publics',n,'module functions',module_functions,'unresolved',sum(not r['names'] for r in large.values()),flush=True)
  p.close();del pe,records;gc.collect()
 Path(sys.argv[2]).write_text(json.dumps(data,indent=2))
if __name__ == '__main__':main()
