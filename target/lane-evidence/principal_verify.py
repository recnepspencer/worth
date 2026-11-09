from pathlib import Path
import json,re,shutil,sys
r=Path.cwd();e=r/'target/lane-evidence';o=e/'principal-final-focused';o.mkdir(exist_ok=True)
sys.path[:0]=[str(r/'scripts'),str(r/'scripts/test_cost'),str(r/'scripts/ci')]
from application_build import application_build_environment
from cargo_capture import CargoRunner
from run_lane import child_environment
from nextest_build import build,list_tests
from coverage import inventory
from reservations import configuration
from query_gate_environment import environment
base=application_build_environment(child_environment())
results=[]
def run(label,manifest,package,names,target=None,profile='dev',fixture=False,direct=False):
 free=shutil.disk_usage('C:/').free; print('C_FREE_BYTES',free,flush=True); assert free>=25*1024**3
 assert len(names)==len(set(names))
 assert all(not ({'exhaustive','scale','query_scale','scenario','scenarios'}&set(n.split('::'))) for n in names)
 env={**base,**environment(r)} if fixture else dict(base)
 if direct:
  for key in ('RUST_MIN_STACK','WORTH_BANK_ASSESSMENT_WIDTH','WORTH_BANK_HISTORY_WIDTH'):env.pop(key,None)
  env.setdefault('WORTH_REPO',str(r))
 runner=CargoRunner(r,env);d=o/label;d.mkdir(exist_ok=True)
 selection={'packages':[package],'targets':{'test':[target]} if target else {'lib':True}}
 reuse=build(r/manifest,d/'build',runner,profile=profile,selection=selection)
 expr=' + '.join('test(='+n+')' for n in names)
 rows=[row for row in inventory(list_tests(reuse,d/'selection.json',runner,expr)) if row['selected']]
 assert len(rows)==len(names) and {row['test'] for row in rows}==set(names) and all(not row['ignored'] for row in rows),rows
 (d/'selection-audit.json').write_text(json.dumps(rows,indent=2),encoding='utf-8')
 if direct:
  binaries=json.loads((d/'build/binaries.json').read_text())['rust-binaries']
  choices=[b for b in binaries.values() if b['binary-name']==target];assert len(choices)==1
  binary=choices[0]['binary-path']
  listed=runner([binary,'--list','--exact',*names],(r/manifest).parent,d/'direct-list.log');assert listed.returncode==0
  assert all(n+': test' in listed.stdout.splitlines() for n in names)
  result=runner([binary,'--exact',*names,'--nocapture'],(r/manifest).parent,d/'run.log')
 else:
  if fixture:
   config=d/'nextest.toml';configuration(r/'.config/nextest.toml',config,'ci',{},r/'scripts/test_cost/account.py')
  else:config=r/'workspaces/worth-store/.config/nextest.toml'
  result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(config),'--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','--success-output','immediate','-E',expr],(r/manifest).parent,d/'run.log')
 results.append({'lane':label,'exit':result.returncode,'names':names,'default_stack':direct,'RUST_MIN_STACK_present':'RUST_MIN_STACK' in env})
 (o/'results.json').write_text(json.dumps(results,indent=2),encoding='utf-8');print('RESULT',label,result.returncode,flush=True)
 # Complete the entire requested census even if one case fails; no source fixes mid-pass.
run('topology','workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml','worth-query-topology-entry',sorted(row['name'].split('$',1)[1] for row in json.loads((e/'pa-merge-final-topology-blocks.json').read_text())),fixture=True)
store_names=['process::application::world_durable_release::unclean::'+n for n in ['unclean_native_tail_refuses_before_world_service_and_names_the_original_effect','quiescent_kill_reopens_the_acknowledged_native_and_world_heads','graceful_close_reopens_the_checkpointed_native_and_world_heads']]
store_names+=['process::application::host_oracle::graceful::graceful_home_oracle_seed_'+n+'_preserves_relational_and_world_prefixes' for n in ['zero','one']]
run('store','workspaces/worth-store/Cargo.toml','worth-store-court-application',store_names,target='worth_store_court_application',profile='ci-test')
p=json.loads((e/'head-closure-final-bank-plan.json').read_text())
run('bank','workspaces/worth-query-bank-world/Cargo.toml',p['packages'][0],p['names'],target='worth_query_bank_court_external_rail_server',direct=True)
raise SystemExit(1 if any(row['exit'] for row in results) else 0)
