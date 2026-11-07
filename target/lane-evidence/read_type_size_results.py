"""Read captured successful test output, not rendered compiler diagnostic text."""
import json, sys
from pathlib import Path
e=Path('target/lane-evidence')
for name in sys.argv[1:] or ['root','query','store','consumer']:
 path=e/('type-size-'+name)/'run.log'
 if not path.exists():continue
 lines=path.read_text(encoding='utf8').splitlines(); events=[];rows=[]
 for line in lines:
  if line.startswith('{'):
   try:event=json.loads(line)
   except ValueError:continue
   events.append(event)
 chunks=[x.get('stdout','') for x in events]+lines
 for chunk in chunks:
  for line in chunk.splitlines():
   line=line.lstrip()
   if line.startswith(('TYPE_SIZE|','APPLICATION_TYPE_SIZE|')):rows.append(line.split('|'))
 unique={tuple(row):row for row in rows}
 rows=list(unique.values());(e/('type-size-'+name+'-rows.json')).write_text(json.dumps(rows,indent=2))
 types={row[4]:row for row in rows if row[0]=='TYPE_SIZE'}
 big=sorted((row for row in types.values() if int(row[5])>=1024),key=lambda row:-int(row[5]))
 print(name,'ROWS',len(rows),'UNIQUE TYPES',len(types))
 print('RESULTS',[(x.get('event'),x.get('name')) for x in events if x.get('type')=='test' and x.get('event') in ('ok','failed')])
 print('EXECUTED SUITES',[{k:x[k] for k in ('passed','failed','ignored','filtered_out') if k in x} for x in events if x.get('type')=='suite' and x.get('event') in ('ok','failed') and x.get('passed',0)+x.get('failed',0)])
 print('\n'.join(row[5]+' '+row[4] for row in big[:30]))
 for row in rows:
  if row[0]=='APPLICATION_TYPE_SIZE':print(row[2],row[1])
