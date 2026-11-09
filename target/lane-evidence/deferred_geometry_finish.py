from pathlib import Path
import json,subprocess,sys
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
lanes=[
 ('deferred-geometry-fmt',['cargo','fmt','--manifest-path','Cargo.toml','-p','worth-relational','--','--check']),
 ('deferred-geometry-line-caps',['C:/Program Files/Git/bin/bash.exe','scripts/ci/check_workspace_rust_line_caps.sh','dirty']),
 ('deferred-geometry-boundary-driver',[py,str(e/'pa_merge_owned_cargo.py'),'deferred-geometry-boundary','tools/boundary-check/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','--root',str(r)]),
 ('deferred-geometry-context-driver',[py,str(e/'pa_merge_owned_cargo.py'),'deferred-geometry-context','tools/agent-context/Cargo.toml','run','--target-dir',str(r/'target/constitution-tools'),'--','check'])]
results={}
for label,command in lanes:
 code=subprocess.call([py,str(e/'run_evidence.py'),label,*command]);print('RESULT',label,code,flush=True);results[label]=code

(e/'deferred-geometry-phase-results.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
raise SystemExit(1 if any(results.values()) else 0)
