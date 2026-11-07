"""Match unnamed PE functions to retained COFF definitions, without building."""
import collections,gc,json,os,struct,sys
from pathlib import Path
e=Path(__file__).resolve().parents[1];sys.path.insert(0,str(e))
from lifecycle_compiled_frames import Frames
from whole_binary_frames import rust_linkage
from pdb_public_frames import Pdb
root=Path.cwd()
def long(path):return Path('\\\\?\\'+str(Path(path).resolve()))
data=json.loads(Path(sys.argv[1]).read_text())
targets={next(p for p in Path(b['exe']).parents if p.name=='target') for b in data['binaries']}
objects=collections.defaultdict(list)
for directory in [* [target/profile/'incremental' for target in targets for profile in ('debug','release')], e/'stack-peak-build/coff']:
 for directory,dirs,files in os.walk(long(directory)):
  for file in files:
   if file.endswith(('.o','.obj')):
    path=Path(directory)/file
    if path.stat().st_size==0:continue
    objects[path.stem].append(path)
    if '.rcgu.' in file and file.count('.')>=3:objects[file.split('.')[1]].append(path)
print('INDEXED_OBJECTS',sum(map(len,objects.values())),flush=True)
def read_object(path):
 raw=path.read_bytes()
 big=raw[:4]==b'\0\0\xff\xff'
 if big:count,sp,ns=struct.unpack_from('<III',raw,44);start=56;stride=20
 else:
  _,count,_,sp,ns,optional,_=struct.unpack_from('<HHIIIHH',raw);start=20+optional;stride=18
 sections=[struct.unpack_from('<8sIIIIIIHHI',raw,start+i*40) for i in range(count)]
 strings=sp+ns*stride;checksums={};functions=collections.defaultdict(list);index=0
 while index<ns:
  pos=sp+index*stride;encoded=raw[pos:pos+8]
  if encoded[:4]==b'\0'*4:
   so=struct.unpack_from('<I',encoded,4)[0]+strings;name=raw[so:raw.index(b'\0',so)].decode(errors='replace')
  else:name=encoded.split(b'\0')[0].decode(errors='replace')
  value=struct.unpack_from('<I',raw,pos+8)[0]
  if big:section,ty,storage,aux=struct.unpack_from('<iHBB',raw,pos+12)
  else:section,ty,storage,aux=struct.unpack_from('<hHBB',raw,pos+12)
  if section>0:
   if storage==3 and ty==0 and aux and name.startswith('.'):
    checksums[section]=struct.unpack_from('<I',raw,pos+stride+8)[0]
   if rust_linkage(name) and ty&0x20:functions[section].append((value,name))
  index+=aux+1
 return raw,sections,checksums,functions
cache={};unresolved=[];attributed=0
for binary in data['binaries']:
 exe=Path(binary['exe']);pdb=exe.with_suffix('.pdb')
 if not pdb.exists():pdb=exe.with_name(exe.stem.replace('-','_')+'.pdb')
 p=Pdb(pdb);dbi=p.read(3);modbytes,conbytes=struct.unpack_from('<II',dbi,24)
 modules=[];mi=64
 while mi<64+modbytes:
  end=dbi.index(b'\0',mi+64);name=dbi[mi+64:end].decode(errors='replace')
  end2=dbi.index(b'\0',end+1);obj=dbi[end+1:end2].decode(errors='replace')
  modules.append((name,obj));mi=(end2+4)&~3
 con=64+modbytes;version=struct.unpack_from('<I',dbi,con)[0]
 assert version==0xf12eba2d,hex(version)
 pe=Frames(str(exe));contributions=[]
 for pos in range(con+4,con+conbytes,28):
  section,offset,size,chars,module,crc,reloccrc=struct.unpack_from('<H2xIIIH2xII',dbi,pos)
  if section and section<=len(pe.sections):
   start=pe.sections[section-1][1]+offset
   contributions.append((start,start+size,module,crc,reloccrc))
 contributions.sort();import bisect
 starts=[v[0] for v in contributions];count=0
 for row in binary['large_functions']:
  if row['names']:continue
  rva=row['rva'];ci=bisect.bisect_right(starts,rva)-1
  if ci<0 or contributions[ci][1]<=rva:
   unresolved.append(dict(binary=binary['binary'],rva=rva,reason='no section contribution'));continue
  start,end,module,crc,reloccrc=contributions[ci];mn,mo=modules[module]
  # LLVM's module name includes the original crate hash and cgu stem. The
  # retained incremental file preserves the cgu stem before the link suffix.
  base=Path(mn.replace('\\','/')).name
  stem=base.split('.')[1] if '.rcgu.' in base and base.count('.')>=3 else Path(base).stem
  candidates=list(objects.get(stem,[]))
  for direct in (Path(mn),Path(mo)):
   if direct.suffix in ('.o','.obj') and long(direct).exists():candidates.append(long(direct))
  matches=[];failures=[]
  for candidate in dict.fromkeys(candidates):
   key=str(candidate)
   if key not in cache:
    try:cache[key]=read_object(candidate)
    except (ValueError,struct.error) as error:failures.append(str(error));continue
   raw,sections,crcs,functions=cache[key]
   for number,section in enumerate(sections,1):
    if section[3]!=end-start or crcs.get(number)!=crc:continue
    names=[name for value,name in functions.get(number,[]) if value==rva-start]
    if not names:continue
    reloc=section[5];nr=section[7];mask=set();unsupported=[]
    for i in range(nr):
     offset,symbol,kind=struct.unpack_from('<IIH',raw,reloc+i*10)
     width=8 if kind==1 else 4 if kind in (2,3,4,5,6,7,8,9,11) else 2 if kind==10 else 0
     if not width:unsupported.append(kind)
     mask.update(range(offset,offset+width))
    code=raw[section[4]:section[4]+section[3]];po=pe.offset(start);linked=pe.data[po:po+len(code)]
    different=sum(a!=b for i,(a,b) in enumerate(zip(code,linked)) if i not in mask)
    if different or unsupported:continue
    matches.append(dict(object=key,section=number,crc=crc,section_size=end-start,relocation_count=nr,non_relocated_mismatches=different,names=names))
  if matches:
   names=sorted({name for m in matches for name in m['names']})
   # Identical retained copies are acceptable. Different semantic symbols are
   # preserved as aliases rather than coalesced into one invented identity.
   row['names']=names;row['public_linkage_names']=names;row['coff_matches']=matches
   row['coff_module']=mn;count+=1;attributed+=1
  else:unresolved.append(dict(binary=binary['binary'],rva=rva,contribution=[start,end,module,crc,reloccrc],module=mn,object_path=mo,candidate_count=len(candidates),failures=failures))
 print(binary['binary'],'COFF attributed',count,'still unnamed',sum(not r['names'] for r in binary['large_functions']),flush=True)
 p.close();del pe;cache.clear();gc.collect()
Path(sys.argv[2]).write_text(json.dumps(data,indent=2))
(e/'stack-peak-coff-unresolved.json').write_text(json.dumps(unresolved,indent=2))
print('ATTRIBUTED',attributed,'UNRESOLVED',len(unresolved),flush=True)
