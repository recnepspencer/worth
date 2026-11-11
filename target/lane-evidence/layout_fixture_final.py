"""One post-review fixture check/build; rerun the sole failed named case."""
import json,re,shutil,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';out=e/'layout-fixture-final';out.mkdir(exist_ok=True)
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from nextest_build import build,list_tests
from coverage import inventory
from reservations import configuration
from run_lane import child_environment
env=application_build_environment(child_environment());runner=CargoRunner(r,env);manifest=r/'workspaces/worth-query/Cargo.toml'
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
if free<25*1024**3:raise SystemExit(125)
package='worth-query-court-execution-package-archive';fixture='worth-query-testkit-execution-primary-graph-application-contribution'
result=runner(['cargo','check','--manifest-path',str(manifest),'-p',fixture,'-p',package,'--tests','--keep-going'],manifest.parent,out/'check.log')
if result.returncode:raise SystemExit(result.returncode)
free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
if free<25*1024**3:raise SystemExit(125)
reuse=build(manifest,out/'build',runner,profile='dev',selection={'packages':[package,'worth-query-court-query-decl-host'],'targets':{'test':['worth_query_court_execution_package_archive','worth_query_court_query_decl_host']}})
name='domain_computation::primary_graph::tests::application_attempt::cleanup_dispatch::failed_cleanup_keeps_the_original_first_send_after_derived_windows_are_discarded';expression='test(='+name+')'
rows=inventory(list_tests(reuse,out/'selection.json',runner,expression));chosen=[x for x in rows if x['selected']]
assert len(chosen)==1 and chosen[0]['test']==name and not chosen[0]['ignored']
assert not re.search(r'(^|::)(scale|exhaustive|scenario|scenarios)::',name)
(out/'selection-audit.json').write_text(json.dumps(chosen,indent=2))
configuration(r/'.config/nextest.toml',out/'nextest.toml','ci',{},r/'scripts/test_cost/account.py')
result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(out/'nextest.toml'),'--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','--success-output','immediate','-E',expression],manifest.parent,out/'run.log')
raise SystemExit(result.returncode)
