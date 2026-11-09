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
for n in ['h8-outbox-query-check','h8-outbox-query-round2-check']:
 rows.append(record(e/(n+'.log'),'check'))
for lane in ['query','bank']:
 d=e/'h8-outbox-focused'/lane
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
for name in ['fmt','bank-fmt','line-caps','boundary','context']:
 path=e/('h8-outbox-'+name+'.log')
 if path.exists():rows.append(record(path,'phase-gate'))
row=record(e/'h8-outbox-rail-prerequisite.log','prerequisite-build');rows.append(row)
row=record(e/'h8-outbox-bank-failed-name-rerun.log','focused-tests-rerun')
s=(e/'h8-outbox-bank-failed-name-rerun.log').read_text(encoding='utf-8')
summary=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',s);assert len(summary)==1
row['counts']=dict(zip(('passed','failed','ignored'),map(int,summary[0])))
row['selected_names']=json.loads((e/'h8-outbox-bank-rerun-selection.json').read_text(encoding='utf-8'))['names'];rows.append(row)
rows.append(record(e/'h8-outbox-query-round4-check.log','check'))
d=e/'h8-outbox-fixture-final/query'
rows.append(record(d/'build/binaries-build.log','focused-build'))
row=record(d/'run.log','focused-tests-rerun');s=(d/'run.log').read_text(encoding='utf-8')
selection=json.loads((d/'selection-audit.json').read_text(encoding='utf-8'));assert len(selection)==4 and all(not x['ignored'] for x in selection)
selected={x['test'] for x in selection};events=[];decoder=json.JSONDecoder()
for m in re.finditer(r'\{"type"\s*:\s*"test"',s):
 try:x,_=decoder.raw_decode(s,m.start());events.append(x)
 except ValueError:pass
outcomes={x['name'].split('$')[-1]:x['event'] for x in events if x['event'] in ('ok','failed','ignored') and x['name'].split('$')[-1] in selected}
assert set(outcomes)==selected
row['selected_names']=sorted(selected);row['counts']={k:sum(v==event for v in outcomes.values()) for k,event in [('passed','ok'),('failed','failed'),('ignored','ignored')]};rows.append(row)
for name in ['fmt','line-caps','boundary','context']:
 path=e/('h8-outbox-fixture-'+name+'.log')
 if path.exists():rows.append(record(path,'phase-gate'))
for row in rows:
 row['source_manifest']='target/lane-evidence/h8-outbox-source-inputs.json' if 'round4' in row['log'] or 'fixture' in row['log'] else 'target/lane-evidence/h8-outbox-round2-source-inputs.json'
(e/'h8-outbox-lane-index.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
for row in rows: print(row['kind'],row['log'],row['exit'],row['counts'])
