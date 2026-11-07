"""Planned completion gates; one tool build at a time on the existing owned target."""
import json,shutil,subprocess,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable;rows=[]
assert (r/'target/constitution-tools').is_dir(),'reuse existing owned constitution target only'
ui=r/'workspaces/worth-ui/Cargo.lock';before=ui.read_bytes()
assert not subprocess.check_output(['git','diff','--name-only','HEAD','--','workspaces/worth-ui'],text=True).strip()
def run(name,*cmd):
 code=subprocess.call([py,str(e/'run_evidence.py'),name,*cmd]);rows.append((name,code));(e/'layout-topology-phase-results.json').write_text(json.dumps(rows,indent=2));return code
try:
 bash='C:/Program Files/Git/bin/bash.exe'
 run('layout-topology-caps-dirty',bash,'scripts/ci/check_workspace_rust_line_caps.sh','dirty')
 run('layout-topology-caps-slice',bash,'scripts/ci/check_workspace_rust_line_caps.sh','diff','3cbe5ad81f')
 for label,tool,args in [('boundary','boundary-check',['--root','.']),('context','agent-context',['check'])]:
  free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
  if free<25*1024**3:raise SystemExit(125)
  code=run('layout-topology-'+label+'-driver',py,str(e/'lifecycle_tool_gate.py'),'layout-topology-'+label,tool,*args)
  if label=='context' and code:
   log=(e/'lifecycle-layout-topology-context.log').read_text(encoding='utf8')
   if 'rerun agent-context generate' in log:
    run('layout-topology-context-generate-driver',py,str(e/'lifecycle_tool_gate.py'),'layout-topology-context-generate','agent-context','generate')
    run('layout-topology-context-recheck-driver',py,str(e/'lifecycle_tool_gate.py'),'layout-topology-context-recheck','agent-context','check')
finally:
 if ui.read_bytes()!=before:
  (e/'layout-topology-ui-metadata-only.patch').write_bytes(subprocess.check_output(['git','diff','--','workspaces/worth-ui/Cargo.lock']))
  ui.write_bytes(before)
 assert not subprocess.check_output(['git','diff','--name-only','HEAD','--','workspaces/worth-ui'],text=True).strip()
print('PHASE RESULTS',rows,flush=True)
raise SystemExit(int(any(code for name,code in rows)))
