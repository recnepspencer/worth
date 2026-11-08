from pathlib import Path
import json,os,shutil,struct,subprocess,sys
r=Path.cwd();e=r/'target/lane-evidence';out=e/'shared-checkpoint';out.mkdir(exist_ok=True);py=sys.executable
checks=[('root','Cargo.toml','worth-runtime-world'),('query','workspaces/worth-query/Cargo.toml','worth-query-execution')]
for label,manifest,package in checks:
 free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True);assert free>=25*1024**3
 code=subprocess.call([py,str(e/'run_evidence.py'),'shared-'+label+'-check-driver',py,str(e/'pa_merge_owned_cargo.py'),'shared-'+label+'-check',manifest,'check','-p',package,'--tests','--message-format=json','--keep-going'])
 if code:raise SystemExit('Read all diagnostics: '+label)
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost'),str(r/'scripts/ci')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from run_lane import child_environment
from query_gate_environment import environment
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True);assert free>=25*1024**3
env=application_build_environment({**child_environment(),**environment(r)})
env.pop('RUST_MIN_STACK',None)
runner=CargoRunner(r,env)
manifest=r/'workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml'
result=runner(['cargo','build','--manifest-path',str(manifest),'-p','worth-query-topology-entry','--example','shared_execution','--message-format=json'],manifest.parent,out/'build.log')
assert result.returncode==0,'Read every build diagnostic'
artifacts=[]
for line in result.stdout.splitlines():
 try:v=json.loads(line)
 except ValueError:continue
 if v.get('reason')=='compiler-artifact' and v.get('executable') and v['target']['name']=='shared_execution':artifacts.append(v['executable'])
assert len(artifacts)==1,artifacts
with Path(artifacts[0]).open('rb') as f:
 f.seek(0x3c);pe=struct.unpack('<I',f.read(4))[0];f.seek(pe+24);assert struct.unpack('<H',f.read(2))[0]==0x20b;f.seek(pe+24+72);reserve=struct.unpack('<Q',f.read(8))[0]
assert reserve==1024*1024,reserve
assert 'RUST_MIN_STACK' not in env
result=runner([artifacts[0]],manifest.parent,out/'main.log')
(out/'result.json').write_text(json.dumps({'exit':result.returncode,'pe_main_stack_reserve':reserve,'rust_min_stack_present':False,'example_source_unchanged':True},indent=2),encoding='utf-8')
print('RESULT',result.returncode,flush=True);raise SystemExit(result.returncode)
