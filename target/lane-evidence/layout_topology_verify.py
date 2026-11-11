"""One complete constitutional repair batch: only reached unit owners/court."""
import json,re,shutil,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';out=e/'layout-topology';out.mkdir(exist_ok=True)
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from nextest_build import build,list_tests
from coverage import inventory
from reservations import configuration
from run_lane import child_environment
runner=CargoRunner(r,application_build_environment(child_environment()));rows=[]
def disk():
 free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
 if free<25*1024**3:raise SystemExit(125)
def record(label,code):
 rows.append([label,code]);(out/'results.json').write_text(json.dumps(rows,indent=2));print('RESULT',label,code,flush=True)
configs=[('root','Cargo.toml',['worth-relational','worth-runtime-bridge','worth-signal','worth-court-runtime-bridge']),('query','workspaces/worth-query/Cargo.toml',['worth-query-execution','worth-query-publication','worth-query']),('store','workspaces/worth-store/Cargo.toml',['worth-store'])]
for name,mp,packages in configs:
 disk();manifest=r/mp;d=out/name;d.mkdir(exist_ok=True)
 result=runner(['cargo','check','--manifest-path',str(manifest),*[x for p in packages for x in ['-p',p]],'--tests','--keep-going'],manifest.parent,d/'check.log');record(name+'-check',result.returncode)
 if result.returncode:raise SystemExit(result.returncode)
guard='inline_type_sizes::owned_carriers_stay_within_declared_inline_ceilings'
pure='source::async_declaration::lowering::storage_tests::repeated_instances_preserve_fresh_template_canonical_meaning'
plans=[('root-unit','Cargo.toml',['worth-relational','worth-runtime-bridge','worth-signal'],{'lib':True},[guard,pure],4),('bridge-court','Cargo.toml',['worth-court-runtime-bridge'],{'test':['worth_court_runtime_bridge']},['runtime_bridge::original_lowering::'+n for n in ['original_lowering_charge_lives_until_the_last_shared_template_drops','original_budget_refuses_lowering_backing_without_retaining_a_charge','cancelled_original_request_refuses_lowering_backing']],3),('query','workspaces/worth-query/Cargo.toml',['worth-query-execution','worth-query-publication','worth-query'],{'lib':True},[guard],3),('store','workspaces/worth-store/Cargo.toml',['worth-store'],{'lib':True},[guard],1)]
for name,mp,packages,targets,names,count in plans:
 disk();manifest=r/mp;d=out/name;d.mkdir(exist_ok=True)
 reuse=build(manifest,d/'build',runner,profile='dev',selection={'packages':packages,'targets':targets});record(name+'-build',0)
 expression=' + '.join('test(='+n+')' for n in names);chosen=[x for x in inventory(list_tests(reuse,d/'selection.json',runner,expression)) if x['selected']]
 assert len(chosen)==count and set(x['test'] for x in chosen)==set(names),(name,chosen)
 assert all(not x['ignored'] and not re.search(r'(^|::)(scale|query_scale|exhaustive|scenario|scenarios)::',x['test']) for x in chosen)
 (d/'selection-audit.json').write_text(json.dumps(chosen,indent=2));configuration(r/'.config/nextest.toml',d/'nextest.toml','ci',{},r/'scripts/test_cost/account.py')
 result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(d/'nextest.toml'),'--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','--success-output','immediate','-E',expression],manifest.parent,d/'run.log');record(name+'-run',result.returncode)
raise SystemExit(int(any(code for _,code in rows)))
