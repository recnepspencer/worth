from pathlib import Path
import json,re,sys,collections
r=Path.cwd();e=r/'target/lane-evidence';sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from logs import read_lane
specs=[]
def add(p,kind,source,counts=None): specs.append((p,kind,source,counts))
for n in ['unclean-store-check','unclean-store-round2-check','unclean-store-round3-check']:add(n+'.log','check','d27064d826')
for d in ['unclean-focused','unclean-final-focused']:
 add(d+'/build/binaries-build.log','focused-build','d27064d826');add(d+'/run.log','test','d27064d826')
for n in ['graceful-court-check','graceful-court-round2-check']:add(n+'.log','check','0df3280458')
for d in ['graceful-focused','graceful-final-focused']:
 add(d+'/build/binaries-build.log','focused-build','0df3280458');add(d+'/run.log','test','0df3280458')
for n in ['shared-root-check','shared-query-check']:add(n+'.log','check','bb3e73655b')
add('shared-checkpoint/build.log','focused-build','bb3e73655b');add('shared-checkpoint/main.log','standalone-main','bb3e73655b',{'pass':1,'fail':0,'ignored':0})
add('topology-check.log','check','5894655937');add('topology-funding/build/binaries-build.log','focused-build','5894655937');add('topology-funding/run.log','test','5894655937')
add('principal-charge-debug.log','diagnostic-no-body','5894655937')
for n in ['principal-charge-debug2.log','principal-charge-debug3.log']:add(n,'diagnostic-test','5894655937',{'pass':0,'fail':1,'ignored':0})
add('principal-charge-pdb.log','offline-pdb','5894655937')
# Keep all failed as well as final enforcement invocations; no failure erased.
for pattern in ['unclean*format*.log','unclean*line-caps*.log','unclean*boundary*.log','unclean*context*.log','graceful-format.log','graceful-line-caps.log','graceful-boundary.log','graceful-context.log','shared-format*.log','shared-line-caps.log','shared-boundary.log','shared-context.log','topology-format*.log','topology-line-caps.log','topology-boundary.log','topology-context.log']:
 for p in sorted(e.glob(pattern)):
  if 'driver' not in p.name:add(p.relative_to(e).as_posix(),'gate',None)
rows=[]
for name,kind,source,counts in specs:
 p=e/name
 if not p.exists():raise RuntimeError('Missing named lane '+name)
 text=p.read_text(encoding='utf-8-sig');lines=text.splitlines();header={}
 for line in lines:
  for prefix,key in [('Command: ','command'),('COMMAND: ','command'),('HEAD: ','head'),('Git status --short empty: ','clean'),('CLEAN: ','clean'),('Start: ','start'),('START: ','start'),('End: ','end'),('END: ','end'),('Exit code: ','exit'),('EXIT: ','exit')]:
   if line.startswith(prefix):header[key]=line[len(prefix):]
 assert set(header)=={'command','head','clean','start','end','exit'},(name,header.keys())
 header['command']=json.loads(header['command']);header['exit']=int(header['exit'])
 if kind=='test':
  observed=read_lane(p); c=collections.Counter(t['event'] for t in observed['tests']);counts={'pass':c['ok'],'fail':c['failed'],'ignored':c['ignored']}
 rows.append({'log':'target/lane-evidence/'+name,'kind':kind,'source_commit':source,**header,'counts':counts})
(e/'h3-lane-index.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
print('Retained lanes',len(rows))
for row in rows:
 if row['counts'] is not None: print(row['log'],row['counts'],'exit',row['exit'])
print('COUNTS',dict(collections.Counter(row['kind'] for row in rows)))
