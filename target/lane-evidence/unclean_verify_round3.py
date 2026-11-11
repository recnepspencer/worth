from pathlib import Path
import json,subprocess,sys,shutil,re
r=Path.cwd();e=r/'target/lane-evidence';out=e/'unclean-final-focused';out.mkdir(exist_ok=True);py=sys.executable
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True);assert free>=25*1024**3
check=subprocess.call([py,str(e/'run_evidence.py'),'unclean-store-round3-check-driver',py,str(e/'pa_merge_owned_cargo.py'),'unclean-store-round3-check','workspaces/worth-store/Cargo.toml','check','-p','worth-store-application','--tests','--message-format=json','--keep-going'])
(out/'check-result.json').write_text(json.dumps({'exit':check},indent=2))
if check:raise SystemExit('Read every diagnostic before another check')
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from run_lane import child_environment
from nextest_build import build,list_tests
from coverage import inventory
names=['process::application::world_durable_release::unclean::'+n for n in ['unclean_native_tail_refuses_before_world_service_and_names_the_original_effect','quiescent_kill_reopens_the_acknowledged_native_and_world_heads','graceful_close_reopens_the_checkpointed_native_and_world_heads']]
assert all(not ({'exhaustive','scale','query_scale','scenario','scenarios'}&set(n.split('::'))) for n in names)
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True);assert free>=25*1024**3
manifest=r/'workspaces/worth-store/Cargo.toml';env=application_build_environment(child_environment());runner=CargoRunner(r,env)
reuse=build(manifest,out/'build',runner,profile='ci-test',selection={'packages':['worth-store-court-application'],'targets':{'test':['worth_store_court_application']}})
expr=' + '.join('test(='+n+')' for n in names)
selected=[row for row in inventory(list_tests(reuse,out/'selection.json',runner,expr)) if row['selected']]
assert len(selected)==len(names) and {row['test'] for row in selected}==set(names),selected
assert all(not row['ignored'] for row in selected)
(out/'selection-audit.json').write_text(json.dumps(selected,indent=2))
result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(r/'workspaces/worth-store/.config/nextest.toml'),'--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','--success-output','immediate','-E',expr],manifest.parent,out/'run.log')
(out/'result.json').write_text(json.dumps({'exit':result.returncode,'names':names,'note':'Owned committed Store/nextest configuration; no default-stack claim. Only the three exact nonignored process courts execute.'},indent=2));print('RESULT',result.returncode,flush=True);raise SystemExit(result.returncode)
