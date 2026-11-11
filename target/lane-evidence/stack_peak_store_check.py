import json,shutil,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from run_lane import child_environment
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
assert free>=25*1024**3
mp=r/'workspaces/worth-store/Cargo.toml';env=application_build_environment(child_environment());runner=CargoRunner(r,env)
(e/'stack-peak-store-application-build.json').write_text(json.dumps({'WORTH_IMPLEMENTATION_BUILD_ID':env['WORTH_IMPLEMENTATION_BUILD_ID']}))
result=runner(['cargo','check','--manifest-path',str(mp),'-p','worth-store','--tests','--keep-going'],mp.parent,e/'stack-peak-store-check.log')
messages=[]
for line in result.stdout.splitlines():
 try:row=json.loads(line)
 except ValueError:continue
 if row.get('reason')=='compiler-message' and row['message']['level'] in ('warning','error'):messages.append(row['message']['rendered'])
print('DIAGNOSTICS',json.dumps(messages),flush=True)
raise SystemExit(result.returncode)
