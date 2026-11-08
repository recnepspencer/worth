from pathlib import Path
import subprocess,sys,json
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
commands=[('graceful-format',['cargo','fmt','--manifest-path','workspaces/worth-store/Cargo.toml','-p','worth-store-court-application','--','--check']),('graceful-line-caps',['C:/Program Files/Git/bin/bash.exe','scripts/ci/check_workspace_rust_line_caps.sh','dirty']),('graceful-boundary-driver',[py,str(e/'pa_merge_owned_cargo.py'),'graceful-boundary','tools/boundary-check/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','--root',str(r)]),('graceful-context-driver',[py,str(e/'pa_merge_owned_cargo.py'),'graceful-context','tools/agent-context/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','check'])]
results=[]
for label,cmd in commands:
 code=subprocess.call([py,str(e/'run_evidence.py'),label,*cmd]);results.append([label,code])
(e/'graceful-phase-results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
