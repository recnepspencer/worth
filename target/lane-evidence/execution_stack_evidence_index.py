from pathlib import Path
import json,subprocess,re
r=Path.cwd();e=r/'target/lane-evidence'
records=[]
def add(label,path,counts=None):
 p=e/path
 if not p.exists():return
 h={}
 with p.open(encoding='utf8',errors='replace') as f:
  for i,line in enumerate(f):
   if i>12:break
   if line.startswith(('Command:','COMMAND:','HEAD:','Start:','START:','Git status --short empty:','CLEAN:')):
    k,v=line.split(':',1);h[k]=v.strip()
 if counts and isinstance(counts,dict) and 'exit' in counts:
  raw=p.read_text(encoding='utf8',errors='replace')
  match=re.search(r'test result: .*? (\d+) passed; (\d+) failed; (\d+) ignored',raw)
  if match:counts=dict(counts,passed=int(match[1]),failed=int(match[2]),ignored=int(match[3]),unit='test body')
  elif label.endswith('-main'):counts=dict(counts,passed=int(counts['exit']==0),failed=int(counts['exit']!=0),ignored=0,unit='genuine main process')
 records.append(dict(lane=label,log=p.relative_to(r).as_posix(),header=h,counts=counts))
for n in ['root','query','store','bank','fixture','wasm','round2-root','round2-query']:
 add(n+' check','execution-stack-'+n+'-check.log',dict(test_counts='not applicable; compile lane'))
for prefix in ['execution-stack-verification','execution-stack-round2-verification','execution-stack-final-transition']:
 result=e/prefix/'results.json'
 if not result.exists():continue
 for row in json.loads(result.read_text()):
  name=row['lane'];path=None
  if name.endswith('-main'):path=prefix+'/'+name+'.log'
  elif name=='consumer-mains-build':path=prefix+'/consumer-mains-build.log'
  elif name=='retention-default':path=prefix+'/retention-default-run.log'
  elif name.endswith('-tests'):path=prefix+'/'+name.removesuffix('-tests')+'/direct-run.log'
  if path:add(prefix+'/'+name,path,row)
for result_file in ['execution-stack-phase-results.json','execution-stack-final-correction-results.json']:
 p=e/result_file
 if p.exists():
  for label,code in json.loads(p.read_text()):
   if label=='execution-stack-final-transition-driver':continue
   actual=label.removesuffix('-driver')
   add(label,actual+'.log',dict(exit=code,test_counts='not applicable; gate lane'))
(e/'execution-stack-evidence-index.json').write_text(json.dumps(dict(index_head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),note='Exact observed HEADs/dirty bits remain raw. Counts refer to named test bodies or genuine mains as stated; compile/gate lanes are not tests. Source-input equivalence map ties dirty source to later commit. Held and cleanup exits stay red.',lanes=records),indent=2))
print('Indexed',len(records),'stack/check/gate evidence records')
