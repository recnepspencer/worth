from pathlib import Path
import subprocess,sys,json
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
commands=[('topology-format-owner',['cargo','fmt','--manifest-path','workspaces/worth-query/Cargo.toml','-p','worth-query-testkit-host','--','--check']),('topology-format-fixture',['cargo','fmt','--manifest-path','workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml','-p','worth-query-topology-entry','--','--check']),('topology-line-caps',['C:/Program Files/Git/bin/bash.exe','scripts/ci/check_workspace_rust_line_caps.sh','dirty']),('topology-boundary-driver',[py,str(e/'pa_merge_owned_cargo.py'),'topology-boundary','tools/boundary-check/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','--root',str(r)]),('topology-context-driver',[py,str(e/'pa_merge_owned_cargo.py'),'topology-context','tools/agent-context/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','check'])]
results=[]
for label,cmd in commands:
 code=subprocess.call([py,str(e/'run_evidence.py'),label,*cmd]);results.append([label,code])
(e/'topology-phase-results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
