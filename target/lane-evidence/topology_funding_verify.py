from pathlib import Path
import json,subprocess,sys,shutil,re
r=Path.cwd();e=r/'target/lane-evidence';out=e/'topology-funding';out.mkdir(exist_ok=True);py=sys.executable
manifest='workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml'
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True);assert free>=25*1024**3
code=subprocess.call([py,str(e/'run_evidence.py'),'topology-check-driver',py,str(e/'pa_merge_owned_cargo.py'),'topology-check',manifest,'check','-p','worth-query-topology-entry','--tests','--message-format=json','--keep-going'])
if code:raise SystemExit('Read every fixture/owner diagnostic')
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost'),str(r/'scripts/ci')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from run_lane import child_environment
from nextest_build import build,list_tests
from coverage import inventory
from reservations import configuration
from query_gate_environment import environment
env=application_build_environment({**child_environment(),**environment(r)});runner=CargoRunner(r,env)
names=sorted(row['name'].split('$',1)[1] for row in json.loads((e/'pa-merge-final-topology-blocks.json').read_text()))
assert len(names)==5 and all(not re.search(r'(^|::)(exhaustive|scale|query_scale|scenario|scenarios)::',n) for n in names)
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True);assert free>=25*1024**3
reuse=build(r/manifest,out/'build',runner,profile='dev',selection={'packages':['worth-query-topology-entry'],'targets':{'lib':True}})
expr=' + '.join('test(='+n+')' for n in names)
rows=[row for row in inventory(list_tests(reuse,out/'selection.json',runner,expr)) if row['selected']]
assert len(rows)==5 and {row['test'] for row in rows}==set(names) and all(not row['ignored'] for row in rows),rows
(out/'selection-audit.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
configuration(r/'.config/nextest.toml',out/'nextest.toml','ci',{},r/'scripts/test_cost/account.py')
result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(out/'nextest.toml'),'--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','--success-output','immediate','-E',expr],(r/manifest).parent,out/'run.log')
(out/'result.json').write_text(json.dumps({'exit':result.returncode,'names':names,'note':'Five ordinary nonignored names; owned fixture environment/configuration; no default-stack claim'},indent=2),encoding='utf-8')
print('RESULT',result.returncode,flush=True);raise SystemExit(result.returncode)
