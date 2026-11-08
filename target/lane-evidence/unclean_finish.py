from pathlib import Path
import subprocess,sys,json
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable;results=[]
assert json.loads((e/'unclean-focused/result.json').read_text())['exit']==0
commands=[
('unclean-final-format-root',['cargo','fmt','--manifest-path','Cargo.toml','-p','worth-foundational','--','--check']),
('unclean-final-format-query',['cargo','fmt','--manifest-path','workspaces/worth-query/Cargo.toml','-p','worth-query-host','--','--check']),
('unclean-final-format-store',['cargo','fmt','--manifest-path','workspaces/worth-store/Cargo.toml','-p','worth-store-application','-p','worth-store-court-application','--','--check']),
('unclean-line-caps',['C:/Program Files/Git/bin/bash.exe','scripts/ci/check_workspace_rust_line_caps.sh','dirty']),
('unclean-boundary-driver',[py,str(e/'pa_merge_owned_cargo.py'),'unclean-boundary','tools/boundary-check/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','--root',str(r)]),
('unclean-context-driver',[py,str(e/'pa_merge_owned_cargo.py'),'unclean-context','tools/agent-context/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','check']),
]
for label,cmd in commands:
 code=subprocess.call([py,str(e/'run_evidence.py'),label,*cmd]);results.append([label,code]);(e/'unclean-phase-results.json').write_text(json.dumps(results,indent=2))
raise SystemExit(int(any(code for _,code in results)))
