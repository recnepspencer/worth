"""One serial implementation verification round, with one issued build identity."""
import json,os,re,shutil,sys,tomllib
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';base=e/'layout-r3';base.mkdir(exist_ok=True)
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from nextest_build import build,list_tests
from coverage import inventory
from reservations import configuration
from run_lane import child_environment
from standalone import execute
from logs import json_objects
assert 'RUST_MIN_STACK' not in os.environ
env=application_build_environment(child_environment());runner=CargoRunner(r,env)
(base/'application-build.json').write_text(json.dumps({'WORTH_IMPLEMENTATION_BUILD_ID':env['WORTH_IMPLEMENTATION_BUILD_ID']}))
configs={
 'query':('workspaces/worth-query/Cargo.toml',['worth-query-execution','worth-query-publication','worth-query']),
 'store':('workspaces/worth-store/Cargo.toml',['worth-store','worth-store-recovery-runtime']),
 'bank':('workspaces/worth-query-bank-world/Cargo.toml',['worth-query-bank-court-external-rail-server']),
 'consumer':('workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml',['worth-query-consumer-root','worth-query-topology-entry'])}
results=[]
def remember(name,code):
 results.append((name,code));(base/'results.json').write_text(json.dumps(results,indent=2));print('RESULT',name,code,flush=True)
def disk():
 free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',free,flush=True)
 if free<25*1024**3:raise SystemExit(125)
for name,(mp,packages) in configs.items():
 disk();manifest=r/mp;out=base/name;out.mkdir(exist_ok=True)
 command=['cargo','check','--manifest-path',str(manifest),*[x for p in packages for x in ('-p',p)],'--tests','--keep-going']
 result=runner(command,manifest.parent,out/'check.log');remember(name+'-check',result.returncode)
 if result.returncode:raise SystemExit(result.returncode)
# Ordinary production entry roles, not libtest replacements.
disk();mp,packages=configs['consumer'];manifest=r/mp;out=base/'ordinary';out.mkdir(exist_ok=True)
result=runner(['cargo','build','--manifest-path',str(manifest),*[x for p in packages for x in ('-p',p)],'--bins','--examples','--message-format=json'],manifest.parent,out/'build.log')
remember('ordinary-build',result.returncode)
if result.returncode:raise SystemExit(result.returncode)
expected={'worth-query-consumer-root','invariant_reads','required_feature','shared_execution'};programs={}
for row in json_objects(result.stdout):
 if row.get('reason')!='compiler-artifact' or not row.get('executable') or row['target']['name'] not in expected:continue
 assert not row['profile']['test'];target=row['target'];mp=Path(row['manifest_path']);kind=next(x for x in ('bin','example') if x in target['kind']);package=tomllib.loads(mp.read_text())['package']['name'];name=target['name']
 programs[name]=dict(binary=package+'::'+kind+'/'+name,test='<main>',ignored=False,package=package,target=name,kind=kind,lib=False,harness=False,exe=row['executable'],cwd=str(mp.parent),manifest=str(mp),profile_test=False)
assert set(programs)==expected
(out/'actual-entry-artifacts.json').write_text(json.dumps(programs,indent=2))
for name in ('worth-query-consumer-root','invariant_reads','required_feature','shared_execution'):
 remember(name+'-main',execute(programs[name],runner,out/(name+'.log'),env).returncode)
# Exact, audited bounded controls and retained owner guards.
guard='inline_type_sizes::owned_carriers_stay_within_declared_inline_ceilings'
new_tests=['source::async_declaration::lowering::storage_tests::'+n for n in [
 'original_lowering_charge_lives_until_the_last_shared_template_drops',
 'original_budget_refuses_lowering_backing_without_retaining_a_charge',
 'cancelled_original_request_refuses_lowering_backing',
 'repeated_instances_preserve_fresh_template_canonical_meaning']]
bank=json.loads((e/'head-closure-final-bank-plan.json').read_text())['names']
cleanup_case='oracle::certification::temporal_conditional_oracle_parity::managed_cleanup::conditional_reinstallation_retains_failed_cleanup_without_repeating_business'
handoff_base='domain_computation::primary_graph::tests::application_attempt::'
handoff_names=[handoff_base+'cleanup_dispatch::failed_cleanup_keeps_the_original_first_send_after_derived_windows_are_discarded',handoff_base+'terminal_failures::aftermath_sealing_interruption::native_preparation_control_stop_keeps_failed_cleanup_evidence_at_the_commit_door']
plans=[('query',[guard,cleanup_case,*handoff_names],6,['worth-query-court-query-decl-host','worth-query-court-execution-package-archive']),('store',[guard],2,[]),('bank',['cost::inline_type_sizes::bank_workflow_carriers_stay_within_declared_inline_ceilings'],1,[]),('consumer',['inline_type_sizes::application_carriers_stay_within_declared_inline_ceilings'],1,[])]
for name,names,count,extra in plans:
 disk();mp,packages=configs[name];manifest=r/mp;out=base/name
 reuse=build(manifest,out/'build',runner,profile='dev',selection={'packages':packages+extra})
 remember(name+'-build',0)
 expression=' + '.join('test(='+n+')' for n in names)
 rows=inventory(list_tests(reuse,out/'selection.json',runner,expression));chosen=[row for row in rows if row['selected']]
 assert len(chosen)==count,(name,chosen)
 assert set(x['test'] for x in chosen)==set(names)
 if name=='bank':
  parent_rows=inventory(list_tests(reuse,out/'bank-parent-selection.json',runner,' + '.join('test(='+n+')' for n in bank)))
  parents=[x for x in parent_rows if x['selected']]
  assert len(parents)==2 and all(not x['ignored'] for x in parents)
  (out/'bank-parent-selection-audit.json').write_text(json.dumps(parents,indent=2))
 assert all(not x['ignored'] and not re.search(r'(^|::)(scale|query_scale|exhaustive|scenario|scenarios)::',x['test']) for x in chosen)
 (out/'selection-audit.json').write_text(json.dumps(chosen,indent=2))
 configuration(r/'.config/nextest.toml',out/'nextest.toml','ci',{},r/'scripts/test_cost/account.py')
 result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(out/'nextest.toml'),'--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','--success-output','immediate','-E',expression],manifest.parent,out/'run.log')
 remember(name+'-run',result.returncode)
disk()
import subprocess
remember('bank-default-direct',subprocess.run([sys.executable,str(e/'layout_bank_default_direct_r3.py')],env=env).returncode)
raise SystemExit(int(any(code for name,code in results)))
