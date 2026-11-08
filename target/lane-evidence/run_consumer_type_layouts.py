"""Measure real private generic layouts on the one required consumer unit build.

The committed compiler/profile remain unchanged. Bootstrap enables only this
diagnostic flag in this child; it declares no Cargo or product Rust feature.
"""
import copy,json,re,shutil,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';out=e/'type-size-consumer';out.mkdir(exist_ok=True)
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from nextest_build import list_tests
from coverage import inventory
from reservations import configuration
from run_lane import child_environment
from logs import json_objects
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
if free<25*1024**3:raise SystemExit(125)
env=application_build_environment(child_environment());assert 'RUST_MIN_STACK' not in env
env['RUSTC_BOOTSTRAP']='1';runner=CargoRunner(r,env)
manifest=r/'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml'
metadata=runner(['cargo','metadata','--offline','--format-version','1','--manifest-path',str(manifest)],manifest.parent,out/'cargo-metadata.log')
assert metadata.returncode==0
(out/'cargo-metadata.json').write_text(metadata.stdout,encoding='utf8')
command=['cargo','rustc','--manifest-path',str(manifest),'--package','worth-query-consumer-root',
 '--tests','--profile','dev','--','-Zprint-type-sizes']
plan=runner.unit_plan(command,manifest.parent,out/'selection-preflight.log')
roots=[plan['graph']['units'][index] for index in plan['graph']['roots']]
assert len(roots)==1 and roots[0]['mode']=='test' and roots[0]['target']['name']=='worth-query-consumer-root',roots
result=runner(command,manifest.parent,out/'compiler-layouts.log')
if result.returncode:raise SystemExit(result.returncode)
artifacts=[row for row in json_objects(result.stdout) if row.get('reason')=='compiler-artifact'
 and row['target']['name']=='worth-query-consumer-root' and row.get('executable') and row['profile']['test']]
assert len(artifacts)==1,artifacts
artifact=artifacts[0]
# Keep nextest's previously issued descriptor, but bind its executable path to
# this Cargo-issued artifact. Package, target, test mode, platform and profile
# must agree; neither the executable nor a test identity is guessed.
binaries=json.loads((e/'main-entry-consumer-courts/build/binaries.json').read_text())
identity='worth-query-consumer-root::bin/worth-query-consumer-root'
record=binaries['rust-binaries'][identity]
assert record['package-id']==artifact['package_id'] and record['kind']=='bin'
assert record['binary-name']==artifact['target']['name']
record['binary-path']=artifact['executable'];binaries['rust-binaries']={identity:record}
(out/'binaries.json').write_text(json.dumps(binaries,indent=2),encoding='utf8')
(out/'artifact-binding.json').write_text(json.dumps(dict(artifact=artifact,descriptor=record,roots=roots),indent=2))
reuse=['--cargo-metadata',str(out/'cargo-metadata.json'),'--binaries-metadata',str(out/'binaries.json')]
env.pop('RUSTC_BOOTSTRAP');runner.environment=env
name='inline_type_sizes::application_carriers_stay_within_declared_inline_ceilings'
expression='test(='+name+')'
rows=inventory(list_tests(reuse,out/'selection.json',runner,expression));selected=[row for row in rows if row['selected']]
assert len(selected)==1 and selected[0]['test']==name and not selected[0]['ignored']
(out/'selection-audit.json').write_text(json.dumps(selected,indent=2))
configuration(r/'.config/nextest.toml',out/'nextest.toml','ci',{},r/'scripts/test_cost/account.py')
result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(out/'nextest.toml'),
 '--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','--success-output','immediate','-E',expression],manifest.parent,out/'run.log')
for log in [out/'compiler-layouts.log',out/'run.log']:
 print('\nCHILD_LOG',log,flush=True);print(log.read_text(encoding='utf8',errors='replace'),flush=True)
raise SystemExit(result.returncode)
