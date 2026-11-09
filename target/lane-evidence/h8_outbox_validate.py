from pathlib import Path
import subprocess,sys
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
code=subprocess.call([py,str(e/'run_evidence.py'),'h8-outbox-query-round2-check-driver',py,str(e/'pa_merge_owned_cargo.py'),'h8-outbox-query-round2-check','workspaces/worth-query/Cargo.toml','check','-p','worth-query-execution','--tests','--message-format=json','--keep-going'])
if code:raise SystemExit('Inspect ALL Query diagnostics before any new round')
raise SystemExit(subprocess.call([py,str(e/'run_evidence.py'),'h8-outbox-focused-driver',py,str(e/'h8_outbox_verify.py')]))
