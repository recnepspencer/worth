import json,sys,re,collections
from pathlib import Path
e=Path(__file__).resolve().parent;sys.path.insert(0,str(e))
from whole_binary_frames import canonical,rust_linkage,defining_crate
p=Path(sys.argv[1]);data=json.loads(p.read_text());group={};failures=[]
for binary in data['binaries']:
 for row in binary['large_functions']:
  linkage=[n for n in row.get('public_linkage_names',[]) if rust_linkage(n)]
  for symbol in linkage or row['names']:
   name,error=canonical(symbol)
   if error:failures.append(symbol);continue
   crate=defining_crate(symbol,name)
   if not crate.startswith(('worth_','bank','invariant_reads','shared_execution','required_feature')):continue
   v=group.setdefault(name,{'symbol_path':name,'defining_crate':crate,'max_frame':0,'binaries':[],'symbols':[]})
   v['max_frame']=max(v['max_frame'],row['frame_bytes'])
   if binary['binary'] not in v['binaries']:v['binaries'].append(binary['binary'])
   v['symbols'].append({'binary':binary['binary'],'rva':row['rva'],'bytes':row['frame_bytes'],'name':symbol})
data['functions']=sorted(group.values(),key=lambda r:(-r['max_frame'],r['symbol_path']))
data['demangle_failures']=failures
data['unresolved']=[{'binary':b['binary'],**r} for b in data['binaries'] for r in b['large_functions'] if not r['names']]
data['bands']={'>=128KiB':sum(v['max_frame']>=131072 for v in group.values()),'64-128KiB':sum(65536<=v['max_frame']<131072 for v in group.values()),'32-64KiB':sum(32768<=v['max_frame']<65536 for v in group.values())}
p.with_name(p.stem+'-dedup.json').write_text(json.dumps(data,indent=2))
print(len(group),data['bands'],'demangle failures',len(failures))
print('CRATES',collections.Counter(v['symbol_path'].lstrip('<').split('::')[0] for v in group.values() if v['max_frame']>=65536))
for v in data['functions'][:65]:print(v['max_frame'],v['symbol_path'])
