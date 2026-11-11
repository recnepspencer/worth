from pathlib import Path
import subprocess,sys
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
lanes=[
 ('principal-fmt',['cargo','fmt','--manifest-path','Cargo.toml','-p','worth-relational','--','--check']),
 ('principal-line-caps',['C:/Program Files/Git/bin/bash.exe','scripts/ci/check_workspace_rust_line_caps.sh','dirty']),
 ('principal-boundary-driver',[py,str(e/'pa_merge_owned_cargo.py'),'principal-boundary','tools/boundary-check/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','--root',str(r)]),
 ('principal-context-driver',[py,str(e/'pa_merge_owned_cargo.py'),'principal-context','tools/agent-context/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','check'])]
for label,command in lanes:
 code=subprocess.call([py,str(e/'run_evidence.py'),label,*command]);print('RESULT',label,code,flush=True)
