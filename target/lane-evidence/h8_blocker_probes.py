from pathlib import Path
import subprocess,json
r=Path.cwd();peer=r.parent/'m3-p3-lifecycle'
base='workspaces/worth-query/crates/worth-query-execution/src/'
paths=[base+'facade/application_home_integration.rs',base+'domain_computation/primary_graph/application_runtime/home_retirement.rs']
rows=[]
for path in paths:
 row={'path':path}
 for label,tail in [('diff',['diff','--quiet','e787b51b3e','--',path]),('status',['status','--porcelain','--',path])]:
  command=['git','-C',str(peer),*tail];result=subprocess.run(command,capture_output=True,text=True)
  row[label]={'command':command,'exit':result.returncode,'stdout':result.stdout,'stderr':result.stderr};print(json.dumps(row[label]))
 rows.append(row)
(r/'target/lane-evidence/h8-journal-held-probes.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
assert all(row['diff']['exit']==1 for row in rows)
print('CONFIRMED: both required owners are held; no peer files mutated')
