"""One declared correction batch for real constrained and protected layouts."""
import json,shutil,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence'
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from nextest_build import list_tests
from coverage import inventory
from reservations import configuration
from run_lane import child_environment
from logs import json_objects
environment=application_build_environment(child_environment());assert 'RUST_MIN_STACK' not in environment
plans=[
 dict(name='native-layout',manifest='Cargo.toml',package='worth-relational',flags=['--tests'],mode='test'),
 dict(name='world-layout',manifest='Cargo.toml',package='worth-runtime-world',flags=['--lib'],mode='build'),
 dict(name='legacy-layout',manifest='workspaces/worth-query/Cargo.toml',package='worth-query',flags=['--tests'],mode='test',
      prior='type-size-query/build/binaries.json',identity='worth-query',test='inline_type_sizes::owned_carriers_stay_within_declared_inline_ceilings'),
 dict(name='bank-layout',manifest='workspaces/worth-query-bank-world/Cargo.toml',package='worth-query-bank-court-external-rail-server',flags=['--test','worth_query_bank_court_external_rail_server'],mode='test',
      prior='main-entry-bank/build/binaries.json',identity='worth-query-bank-court-external-rail-server::worth_query_bank_court_external_rail_server',test='cost::inline_type_sizes::bank_workflow_carriers_stay_within_declared_inline_ceilings'),
 dict(name='consumer-layout',manifest='workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml',package='worth-query-consumer-root',flags=['--tests'],mode='test',
      prior='type-size-consumer/binaries.json',identity='worth-query-consumer-root::bin/worth-query-consumer-root',test='inline_type_sizes::application_carriers_stay_within_declared_inline_ceilings'),
]
results=[]
for plan in plans:
 name=plan['name'];out=e/('type-size-'+name);out.mkdir(exist_ok=True);manifest=r/plan['manifest']
 free=shutil.disk_usage('C:/').free;print('C_FREE_BYTES',name,free,flush=True)
 if free<25*1024**3:raise SystemExit(125)
 env={**environment,'RUSTC_BOOTSTRAP':'1'};runner=CargoRunner(r,env)
 try:
  metadata=runner(['cargo','metadata','--offline','--format-version','1','--manifest-path',str(manifest)],manifest.parent,out/'cargo-metadata.log')
  assert metadata.returncode==0
  (out/'cargo-metadata.json').write_text(metadata.stdout,encoding='utf8')
  command=['cargo','rustc','--manifest-path',str(manifest),'--package',plan['package'],*plan['flags'],'--profile','dev','--','-Zprint-type-sizes']
  declared=runner.unit_plan(command,manifest.parent,out/'selection-preflight.log')
  roots=[declared['graph']['units'][index] for index in declared['graph']['roots']]
  assert len(roots)==1 and roots[0]['mode']==plan['mode'],roots
  result=runner(command,manifest.parent,out/'compiler-layouts.log')
  if result.returncode:
   results.append((name,'compile',result.returncode));continue
  if 'test' not in plan:
   results.append((name,'compile-only',0));continue
  artifacts=[row for row in json_objects(result.stdout) if row.get('reason')=='compiler-artifact'
   and row.get('executable') and row['profile']['test'] and row['manifest_path'].replace('\\','/').endswith('/'+plan['package']+'/Cargo.toml')]
  assert len(artifacts)==1,artifacts
  artifact=artifacts[0];binaries=json.loads((e/plan['prior']).read_text())
  record=binaries['rust-binaries'][plan['identity']]
  assert record['package-id']==artifact['package_id'] and record['binary-name']==artifact['target']['name']
  record['binary-path']=artifact['executable'];binaries['rust-binaries']={plan['identity']:record}
  (out/'binaries.json').write_text(json.dumps(binaries,indent=2),encoding='utf8')
  (out/'artifact-binding.json').write_text(json.dumps(dict(artifact=artifact,descriptor=record,roots=roots),indent=2))
  reuse=['--cargo-metadata',str(out/'cargo-metadata.json'),'--binaries-metadata',str(out/'binaries.json')]
  runner.environment=environment;expression='test(='+plan['test']+')'
  rows=inventory(list_tests(reuse,out/'selection.json',runner,expression));selected=[row for row in rows if row['selected']]
  assert len(selected)==1 and selected[0]['test']==plan['test'] and not selected[0]['ignored']
  (out/'selection-audit.json').write_text(json.dumps(selected,indent=2))
  configuration(r/'.config/nextest.toml',out/'nextest.toml','ci',{},r/'scripts/test_cost/account.py')
  result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(out/'nextest.toml'),
   '--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','--success-output','immediate','-E',expression],manifest.parent,out/'run.log')
  results.append((name,'run',result.returncode))
 except Exception as error:
  print('DIAGNOSTIC_FAILURE',name,repr(error),flush=True);results.append((name,'driver',1))
 print('COMPLETED',results[-1],flush=True)
(e/'type-size-correction-results.json').write_text(json.dumps(results,indent=2))
print('CORRECTION RESULTS',results,flush=True)
raise SystemExit(int(any(row[2] for row in results)))
