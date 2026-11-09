from pathlib import Path
import json,re,sys
r=Path.cwd();e=r/'target/lane-evidence'
sys.path.insert(0,str(r/'scripts/test_cost'))
from coverage import inventory

def record(p,kind):
 s=p.read_text(encoding='utf-8');row={'log':str(p.relative_to(r)).replace('\\','/'),'kind':kind}
 for key,pattern in [('command',r'(?im)^command: (.+)$'),('head',r'(?im)^head: (.+)$'),('clean',r'(?im)^(?:clean|git status --short empty): (.+)$'),('start',r'(?im)^start: (.+)$'),('end',r'(?im)^end: (.+)$'),('exit',r'(?im)^exit(?: code)?: (-?\d+)$')]:
  matches=re.findall(pattern,s);assert matches,(p,key);v=matches[-1].strip();row[key]=json.loads(v) if key=='command' else int(v) if key=='exit' else v
 row['counts']=None
 return row

rows=[]
for n in ['deferred-geometry-native-check','deferred-geometry-native-round2-check']:
 rows.append(record(e/(n+'.log'),'check'))
for lane in ['native','topology','store','bank']:
 d=e/'deferred-geometry-focused'/lane
 rows.append(record(d/'build/binaries-build.log','focused-build'))
 row=record(d/'run.log','focused-tests');s=(d/'run.log').read_text(encoding='utf-8')
 selection=json.loads((d/'selection-audit.json').read_text());assert all(not x['ignored'] for x in selection)
 full=inventory(json.loads((d/'selection.json').read_text()))
 row['selected_names']=[x['test'] for x in selection]
 row['unselected_ignored_inventory']=sum(x['ignored'] and not x['selected'] for x in full)
 if lane=='bank':
  summaries=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',s)
  assert len(summaries)==1,summaries
  a,b,c=map(int,summaries[0]);row['counts']={'passed':a,'failed':b,'ignored':c}
 else:
  events=[];decoder=json.JSONDecoder()
  for m in re.finditer(r'\{"type"\s*:\s*"test"',s):
   try:x,_=decoder.raw_decode(s,m.start());events.append(x)
   except ValueError:pass
  selected=set(row['selected_names'])
  outcomes={x['name'].split('$')[-1]:x['event'] for x in events if x['event'] in ('ok','failed','ignored') and x['name'].split('$')[-1] in selected}
  assert set(outcomes)==selected,(lane,outcomes,selected)
  row['counts']={k:sum(v==event for v in outcomes.values()) for k,event in [('passed','ok'),('failed','failed'),('ignored','ignored')]}
  row['raw_ignored_events']=len({x['name'] for x in events if x['event']=='ignored'})
 assert sum(row['counts'].values())==len(selection),(lane,row)
 rows.append(row)
for name in ['fmt','line-caps','boundary','context']:
 path=e/('deferred-geometry-'+name+'.log')
 if path.exists():rows.append(record(path,'phase-gate'))
(e/'deferred-geometry-lane-index.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
for row in rows: print(row['kind'],row['log'],row['exit'],row['counts'])
