from pathlib import Path
import subprocess,sys,json
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
assert json.loads((e/'shared-checkpoint/result.json').read_text())['exit']==0
commands=[('shared-format-root',['cargo','fmt','--manifest-path','Cargo.toml','-p','worth-runtime-world','--','--check']),('shared-format-query',['cargo','fmt','--manifest-path','workspaces/worth-query/Cargo.toml','-p','worth-query-execution','--','--check']),('shared-line-caps',['C:/Program Files/Git/bin/bash.exe','scripts/ci/check_workspace_rust_line_caps.sh','dirty']),('shared-boundary-driver',[py,str(e/'pa_merge_owned_cargo.py'),'shared-boundary','tools/boundary-check/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','--root',str(r)]),('shared-context-driver',[py,str(e/'pa_merge_owned_cargo.py'),'shared-context','tools/agent-context/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','check'])]
results=[]
for label,cmd in commands:
 code=subprocess.call([py,str(e/'run_evidence.py'),label,*cmd]);results.append([label,code])
(e/'shared-phase-results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
