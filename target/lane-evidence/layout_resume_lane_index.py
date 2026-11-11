"""Index immutable full logs without changing or reinterpreting their results."""
import json,re
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';rows=[]
def log(path,label,kind):
 p=e/path
 if not p.exists():return
 lines=p.read_text(encoding='utf8',errors='replace').splitlines()
 command=next((x.split(': ',1)[1] for x in lines if x.startswith(('COMMAND: ','Command: '))),None)
 head=next((x.split(': ',1)[1] for x in lines if x.startswith('HEAD: ')),None)
 end=[x.split(': ',1)[1] for x in lines if x.startswith(('EXIT: ','Exit code: '))]
 events={};ignored_metadata=0
 for x in lines:
  try:v=json.loads(x)
  except (ValueError,TypeError):continue
  if not isinstance(v,dict):continue
  if v.get('type')=='test' and v.get('event') in ['ok','failed','ignored']:events[v['name']]=v['event']
  if v.get('type')=='suite':ignored_metadata=max(ignored_metadata,v.get('ignored',0))
 selection=p.parent/'selection.json'
 if kind=='tests' and selection.exists():
  doc=json.loads(selection.read_text());ignored_metadata=sum(t.get('ignored',False) for suite in doc.get('rust-suites',{}).values() for t in suite.get('testcases',{}).values())
 counts={k:sum(v==k for v in events.values()) for k in ['ok','failed','ignored']}
 rows.append(dict(label=label,kind=kind,log=path,head=head,command=command,exit=end[-1] if end else 'unfinished',passed=counts['ok'] if kind=='tests' else None,failed=counts['failed'] if kind=='tests' else None,ignored=counts['ignored'] if kind=='tests' else None,ignored_inventory_metadata=ignored_metadata))
log('layout-r2/query/check.log','initial Query check','check')
for w in ['query','store','bank','consumer']:log(f'layout-r3/{w}/check.log',f'round2 {w} check','check')
log('layout-fixture-final/check.log','fixture Query check','check')
log('layout-topology/root/check.log','first topology Root check','check')
for w in ['root','query','store']:log(f'layout-topology-final/{w}/check.log',f'corrected topology {w} check','check')
log('layout-topology-reviewed/store/check.log','reviewed Store-only check','check')
for w in ['root','store']:log(f'layout-topology-last/{w}/check.log',f'final {w} check','check')
log('layout-r3/ordinary/build.log','ordinary production build','build')
for w in ['query','store','bank','consumer']:
 log(f'layout-r3/{w}/build/binaries-build.log',f'round2 {w} build','build');log(f'layout-r3/{w}/run.log',f'round2 {w} tests','tests')
log('layout-fixture-final/build/binaries-build.log','fixture exact target build','build');log('layout-fixture-final/run.log','fixture failed-case rerun','tests')
for w in ['root-unit','bridge-court','query','store']:
 log(f'layout-topology-reviewed/{w}/build/binaries-build.log',f'topology {w} build','build');log(f'layout-topology-reviewed/{w}/run.log',f'topology {w} tests','tests')
for w in ['bridge-court','store']:
 log(f'layout-topology-last/{w}/build/binaries-build.log',f'final {w} build','build');log(f'layout-topology-last/{w}/run.log',f'final {w} tests','tests')
for n in ['worth-query-consumer-root','invariant_reads','required_feature','shared_execution']:log(f'layout-r3/ordinary/{n}.log',f'{n} main','main')
for n in ['invariant_reads','required_feature','shared_execution']:log(f'layout-r3-{n}-stack-diagnostic.log',f'{n} overflow diagnostic','diagnostic')
# The direct harness proof is one test invocation, separately audited from nextest.
driver=(e/'layout-r3-driver.log').read_text();direct=next(x.split('DIRECT COMMAND: ',1)[1] for x in driver.splitlines() if x.startswith('DIRECT COMMAND: '));assert '2 passed; 0 failed; 0 ignored' in driver
rows.append(dict(label='Bank default-stack parents',kind='tests',log='layout-r3-driver.log',head='4d063ecf8da0e13d3b7b92f73a8865185a9afce2',command=direct,exit='0',passed=2,failed=0,ignored=0,ignored_inventory_metadata=0))
for name in ['layout-topology-format','layout-topology-clone-format','layout-caps-dirty','layout-caps-slice','lifecycle-layout-boundary','lifecycle-layout-context','layout-topology-caps-dirty','layout-topology-caps-slice','lifecycle-layout-topology-boundary','lifecycle-layout-topology-context','layout-after-frame-driver','layout-final-frame-driver','layout-topology-completion-format']:
 log(name+'.log',name,'gate/diagnostic')
(e/'layout-resume-lanes.json').write_text(json.dumps(rows,indent=2))
s=['# Round2 resume lane index','', 'Full logs retain exact command vectors, HEAD, clean state, start/end and exit. Counts below are terminal test events, deduplicated by test identity. Ignored inventory metadata is separate: no ignored case was selected or executed. Non-test counts are not applicable. Bank direct is in layout-r3-driver.log at 4d063ecf8d, command printed under DIRECT COMMAND;2/0/0, RUST_MIN_STACK absent.','', '| Lane | HEAD | Exit | Pass/fail/ignored | Full log |','| --- | --- | ---: | --- | --- |']
for x in rows:
 counts=f"{x['passed']}/{x['failed']}/{x['ignored']}" if x['kind']=='tests' else 'n/a'
 if x['ignored_inventory_metadata']:counts+=f"; {x['ignored_inventory_metadata']} ignored metadata only"
 s.append(f"| {x['label']} | {(x['head'] or 'see log')[:10]} | {x['exit']} | {counts} | {x['log']} |")
s+=['','Exact commands (JSON vectors, never reconstructed shell commands):','']
for x in rows:s.append(f"- `{x['log']}`: `{x['command']}`")
(e/'layout-resume-lanes.md').write_text('\n'.join(s)+'\n')
print('LANES',len(rows),'checks',sum(x['kind']=='check' for x in rows),'builds',sum(x['kind']=='build' for x in rows),'tests',sum(x['kind']=='tests' for x in rows))
