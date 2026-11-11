"""Keep exact committed-compiler layouts, including private generic carriers."""
import json,re
from pathlib import Path
e=Path('target/lane-evidence'); rows=[];current=None
logs=[e/'type-size-consumer/compiler-layouts.log']+sorted(e.glob('type-size-*-layout/compiler-layouts.log'))
for line in (line.rstrip('\n') for log in logs for line in log.open(encoding='utf8')):
 match=re.fullmatch(r'print-type-size type: `(.+)`: (\d+) bytes, alignment: (\d+) bytes',line)
 if match:
  current=dict(type=match[1],bytes=int(match[2]),alignment=int(match[3]),members=[]);rows.append(current)
 elif current and line.startswith('print-type-size '):
  current['members'].append(line.removeprefix('print-type-size '))
by_type={}
for row in rows:
 key=(row['type'],row['bytes'],row['alignment'],tuple(row['members']))
 by_type[key]=row
rows=sorted(by_type.values(),key=lambda row:(-row['bytes'],row['type']))
(e/'compiler-type-layouts.json').write_text(json.dumps(rows,indent=2))
owned=[row for row in rows if row['bytes']>=1024 and row['type'].startswith(('worth_','bank','{closure@','{async fn'))]
print('ALL',len(rows),'OWNED >=1KiB',len(owned))
for row in owned[:35]:
 print(row['bytes'],row['type'].split('<',1)[0],row['members'][:3])
for row in owned:
 if re.search(r'history::edit::(?:Phase|Forking|Selecting)<',row['type']):
  print('PRIVATE',row['bytes'],row['type'].split('<',1)[0])
