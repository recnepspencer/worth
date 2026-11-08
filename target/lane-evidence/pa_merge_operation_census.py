from pathlib import Path
import json,re,collections
r=Path.cwd();e=r/'target/lane-evidence';rows={}
files=list(e.glob('pa-merge-*.log'))
for p in e.glob('pa-merge-*'):
 if p.is_dir():files.extend(p.rglob('*.log'))
for p in files:
 if '-unit-plan' in p.name:continue  # Declaration logs retain the planned command, not a compiler run.
 with p.open(encoding='utf-8',errors='replace') as f:
  start=None;head=None
  for line in f:
   if line.startswith('Start: '):start=line[7:].strip()
   if line.startswith('HEAD: '):head=line[6:].strip()
   if not line.startswith('{'):continue
   try:x=json.loads(line)
   except ValueError:continue
   if x.get('type')!='cargo-invocation':continue
   cmd=x['command'];key=(start,json.dumps(cmd),x.get('cwd'))
   row=rows.setdefault(key,{'start':start,'head':head,'command':cmd,'logs':[]})
   row['logs'].append(str(p.relative_to(r)))
counts=collections.Counter()
for x in rows.values():
 c=x['command'];kind='other'
 if c[:1]==['cargo']:
  sub=c[1]
  if sub in ('check','clippy','build','run'):kind=sub
  elif sub=='test':kind='doc-test' if '--doc' in c else 'cargo-test'
  elif sub=='nextest':kind='nextest-build' if '--list-type' in c and c[c.index('--list-type')+1]=='binaries-only' else 'nextest-'+c[2]
  else:kind='cargo-'+sub
 else:kind='native-execution'
 x['kind']=kind;counts[kind]+=1
(e/'pa-merge-actual-operation-census.json').write_text(json.dumps({'counts':counts,'invocations':list(rows.values())},indent=2));print(json.dumps(counts,indent=2));print('canonical logs',len(files),'distinct invocation records',len(rows))
