from pathlib import Path
import sys,json
root=Path.cwd().resolve()
sys.path.insert(0,str(root/'scripts/test_cost'))
from run_lane import child_environment
from cargo_capture import CargoRunner
from nextest_build import build,list_tests
from coverage import inventory
from logs import json_objects
from reservations import configuration
label,expression,expected=sys.argv[1:]
out=root/'target/test-cost/store'/label
prior=root/'target/test-cost/store/funding-finish-store-refusal-and-ack'
out.mkdir(parents=True,exist_ok=True)
reservations=json.loads((prior/'reservations.json').read_text())
key='worth-store-court-application::worth_store_court_application\nprocess::application::native_components::checkpoint::selected_checkpoint_survives_observer_drop_online_history_and_mutation_before_ack_recovery'
# The original Serial caller joins one independent Serial Home upkeep worker.
reservations[key]=2
(out/'reservations.json').write_text(json.dumps(reservations))
env=child_environment()
env.update(WORTH_REPO=str(root),WORTH_TEST_COST_RESERVATIONS=str(out/'reservations.json'),WORTH_TEST_COST_ACCOUNTING_DIRECTORY=str(out/'cpu'),WORTH_TEST_COST_SETUP_DIRECTORY=str(out/'setup'))
runner=CargoRunner(root,env)
manifest=root/'workspaces/worth-store/Cargo.toml'
reuse=build(manifest,out/'build',runner,selection={'packages':['worth-store-court-application'],'targets':{'test':['worth_store_court_application']}})
selected=[r for r in inventory(list_tests(reuse,out/'build/request-selection.json',runner,expression)) if r['selected']]
assert len(selected)==int(expected) and not any(row['ignored'] for row in selected),selected
chosen={row['binary']+'\n'+row['test']:reservations[row['binary']+'\n'+row['test']] for row in selected}
configuration(prior/'nextest.toml',out/'nextest.toml','ci',chosen,root/'scripts/test_cost/account.py')
result=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(out/'nextest.toml'),'--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','-E',expression],manifest.parent,out/'run-selection.log')
counts=[0,0,0]
for event in json_objects(result.stdout):
 if event.get('type')=='test' and event.get('event') in ('ok','failed'):
  counts[0 if event['event']=='ok' else 1]+=1
for path in runner.logs:print('FULL_OWNED_LOG_PATH:',path)
print(result.stdout)
print('EVIDENCE_TEST_COUNTS: '+','.join(map(str,counts)))
sys.exit(result.returncode)
