from pathlib import Path
import json,shutil,subprocess,sys
r=Path.cwd();e=r/'target/lane-evidence';py=sys.executable
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from cargo_capture import CargoRunner
from application_build import application_build_environment
from run_lane import child_environment
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True);assert free>=25*1024**3
code=subprocess.call([py,str(e/'run_evidence.py'),'h8-outbox-rail-prerequisite-driver',py,str(e/'pa_merge_owned_cargo.py'),'h8-outbox-rail-prerequisite','workspaces/worth-query-bank-world/Cargo.toml','build','--profile','dev','--package','bank-external-rail','--bin','bank-external-rail','--message-format=json'])
if code:raise SystemExit(code)
plan=json.loads((e/'h8-outbox-selection-plan.json').read_text(encoding='utf-8'));names=plan['bank'][2:];assert len(names)==3
selection=json.loads((e/'h8-outbox-focused/bank/selection-audit.json').read_text(encoding='utf-8'));assert all(any(x['test']==n and not x['ignored'] for x in selection) for n in names)
binaries=json.loads((e/'h8-outbox-focused/bank/build/binaries.json').read_text(encoding='utf-8'))['rust-binaries'];choices=[x for x in binaries.values() if x['binary-name']=='worth_query_bank_court_external_rail_server'];assert len(choices)==1
binary=choices[0]['binary-path'];env=application_build_environment(child_environment());env['WORTH_REPO']=str(r)
for key in ('RUST_MIN_STACK','WORTH_BANK_ASSESSMENT_WIDTH','WORTH_BANK_HISTORY_WIDTH'):env.pop(key,None)
result=CargoRunner(r,env)([binary,'--exact',*names,'--nocapture'],r/'workspaces/worth-query-bank-world',e/'h8-outbox-bank-failed-name-rerun.log')
(e/'h8-outbox-bank-rerun-selection.json').write_text(json.dumps({'names':names,'RUST_MIN_STACK_present':'RUST_MIN_STACK' in env,'exit':result.returncode},indent=2),encoding='utf-8')
raise SystemExit(result.returncode)
