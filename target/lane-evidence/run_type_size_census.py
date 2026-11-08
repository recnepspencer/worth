"""One owned, selected-target build and exact-name census run per workspace."""
import json, re, shutil, sys
from pathlib import Path
r=Path.cwd(); e=r/'target/lane-evidence'; workspace=sys.argv[1]
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from nextest_build import build,list_tests
from coverage import inventory
from reservations import configuration
from run_lane import child_environment
definitions={
 'root': ('Cargo.toml',['worth-signal','worth-runtime-bridge','worth-relational'],{'lib':True}),
 'query': ('workspaces/worth-query/Cargo.toml',['worth-query-execution','worth-query-publication','worth-query'],{'lib':True}),
 'store': ('workspaces/worth-store/Cargo.toml',['worth-store','worth-store-recovery-runtime'],{'lib':True}),
 'consumer': ('workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml',['worth-query-consumer-root'],{'bin':['worth-query-consumer-root']}),
}
manifest,packages,targets=definitions[workspace];manifest=r/manifest
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
if free<25*1024**3:raise SystemExit(125)
env=application_build_environment(child_environment());assert 'RUST_MIN_STACK' not in env
runner=CargoRunner(r,env);out=e/('type-size-'+workspace);out.mkdir(exist_ok=True)
try:reuse=build(manifest,out/'build',runner,profile='dev',selection={'packages':packages,'targets':targets})
except Exception:
 for log in sorted((out/'build').glob('*.log')):
  print('\nCHILD_LOG',log,flush=True);print(log.read_text(encoding='utf8',errors='replace'),flush=True)
 raise
name='application_carriers_stay_within_declared_inline_ceilings' if workspace=='consumer' else 'owned_carriers_stay_within_declared_inline_ceilings'
expression='test(=inline_type_sizes::'+name+')'
rows=inventory(list_tests(reuse,out/'selection.json',runner,expression));selected=[row for row in rows if row['selected']]
assert len(selected)==len(packages),(selected,packages)
assert all(not row['ignored'] and row['test']=='inline_type_sizes::'+name for row in selected)
assert all(not re.search(r'(^|::)(exhaustive|scale|scenario|scenarios)::',row['test']) for row in selected)
(out/'selection-audit.json').write_text(json.dumps(selected,indent=2))
configuration(r/'.config/nextest.toml',out/'nextest.toml','ci',{},r/'scripts/test_cost/account.py')
result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(out/'nextest.toml'),
 '--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','--success-output','immediate','-E',expression],manifest.parent,out/'run.log')
for log in [out/'build'/'binaries-build.log',out/'run.log']:
 print('\nCHILD_LOG',log,flush=True);print(log.read_text(encoding='utf8',errors='replace'),flush=True)
raise SystemExit(result.returncode)
