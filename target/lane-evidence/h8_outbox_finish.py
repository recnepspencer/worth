from pathlib import Path
import json,subprocess,sys
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
lanes=[
 ('h8-outbox-fmt',['cargo','fmt','--manifest-path','workspaces/worth-query/Cargo.toml','-p','worth-query-execution','--','--check']),
 ('h8-outbox-bank-fmt',['cargo','fmt','--manifest-path','workspaces/worth-query-bank-world/Cargo.toml','-p','worth-query-bank-court-external-rail-server','--','--check']),
 ('h8-outbox-line-caps',['C:/Program Files/Git/bin/bash.exe','scripts/ci/check_workspace_rust_line_caps.sh','dirty']),
 ('h8-outbox-boundary-driver',[py,str(e/'pa_merge_owned_cargo.py'),'h8-outbox-boundary','tools/boundary-check/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','--root',str(r)]),
 ('h8-outbox-context-driver',[py,str(e/'pa_merge_owned_cargo.py'),'h8-outbox-context','tools/agent-context/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','check'])]
results={}
for label,command in lanes:
 code=subprocess.call([py,str(e/'run_evidence.py'),label,*command]);print('RESULT',label,code,flush=True);results[label]=code

(e/'h8-outbox-phase-results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
raise SystemExit(1 if any(results.values()) else 0)
