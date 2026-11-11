import json,shutil,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';out=e/'whole-frame-release';out.mkdir(exist_ok=True)
sys.path.insert(0,str(r/'scripts'));sys.path.insert(0,str(r/'scripts/test_cost'))
from application_build import application_build_environment
from cargo_capture import CargoRunner
from run_lane import child_environment
from logs import json_objects
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
if free<25*1024**3:raise SystemExit(125)
manifest=r/'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml'
command=['cargo','build','--release','-p','worth-query-topology-entry','--example','invariant_reads','--example','required_feature','--example','shared_execution','--manifest-path',str(manifest),'--message-format=json']
env=application_build_environment(child_environment());assert 'RUST_MIN_STACK' not in env
result=CargoRunner(r,env)(command,manifest.parent,out/'build.log')
if result.returncode:raise SystemExit(result.returncode)
inputs={}
for row in json_objects(result.stdout):
 if row.get('reason')=='compiler-artifact' and row.get('executable') and row['target']['name'] in ('invariant_reads','required_feature','shared_execution'):
  assert not row['profile']['test'];inputs[row['target']['name']]=row['executable']
assert len(inputs)==3
(e/'whole-frame-release-inputs.json').write_text(json.dumps(inputs,indent=2))
