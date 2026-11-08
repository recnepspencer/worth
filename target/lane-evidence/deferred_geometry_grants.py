from pathlib import Path
import json,subprocess
r=Path.cwd();e=r/'target/lane-evidence';base=Path('C:/programming/worth worktees/worktrees')
paths=json.loads((e/'deferred-geometry-paths.json').read_text(encoding='utf-8'));rows=[]
for name in ['m3-h5-signal','m3-p3-lifecycle']:
 root=base/name;head=subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip()
 for path in paths:
  diff=['git','-C',str(root),'diff','--quiet','e787b51b3e','--',path]
  status=['git','-C',str(root),'status','--porcelain','--',path]
  d=subprocess.run(diff);s=subprocess.run(status,capture_output=True,text=True,check=True).stdout
  assert d.returncode in (0,1),(path,d.returncode)
  row={'path':path,'worktree':name,'head':head,'diff_command':diff,'diff_exit':d.returncode,'status_command':status,'status':s,'granted':d.returncode==0 and not s};rows.append(row)
  print(json.dumps(row),flush=True)
(e/'deferred-geometry-grants.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
print('Audit completed; held files:',json.dumps([x for x in rows if not x['granted']]),flush=True)
