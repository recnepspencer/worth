"""Focused diagnostic using owned Cargo/nextest build and selection components."""
from pathlib import Path
import sys
root=Path.cwd().resolve()
sys.path.insert(0,str(root/'scripts/test_cost'))
from run_lane import child_environment
from cargo_capture import CargoRunner
from nextest_build import build,list_tests
from coverage import inventory
from logs import json_objects
label=sys.argv[1]
out=root/'target/test-cost/root'/label
prior=root/'target/test-cost/root/d12-retained-image-and-sparse-courts'
env=child_environment()
import json
out.mkdir(parents=True,exist_ok=True)
reservations=json.loads((prior/'reservations.json').read_text())
for name in ('selected_checkpoint_without_captured_original_base_refuses_without_body_demand','checkpoint_capture_restores_many_branches_with_long_common_names'):
 reservations['worth-court-relational-certification::worth_court_relational_certification\nrelational::durable_publication::source_initial_inputs::checkpoint_admission::capture_cost::'+name]=1
for name in ('selected_embedded_capture_refuses_before_canonical_body_demand',):
 reservations['worth-court-relational-certification::worth_court_relational_certification\nrelational::durable_publication::source_initial_inputs::checkpoint_admission::capture_cost::'+name]=1
reservations['worth-court-relational-certification::worth_court_relational_certification\ncost::relational::source_initial_inputs::checkpoint_admission::selected_state_recapture_work_does_not_follow_discarded_covered_update_bytes']=1
# Each added materialization court has one Serial caller and no joined worker.
for name in ('head_materialization_budget_survives_edit_and_last_basis_drop', 'full_materialization_budget_survives_mirror_edits_until_runtime_drop'):
 reservations['worth-court-relational-certification::worth_court_relational_certification\nrelational::durable_publication::source_initial_inputs::checkpoint_admission::region_memory::'+name]=1
reservations['worth-court-relational-certification::worth_court_relational_certification\nrelational::durable_publication::source_initial_inputs::checkpoint_admission::contract_work::cold_head_work_exhaustion_keeps_a_wide_contract_region_retryable']=1
# One serial caller per public cold-materialization control; no joined worker.
for path in ('byte_payloads::cold_inline_bytes_head_refuses_after_member_read_then_retains_exact_payload_through_edit',
             'byte_payloads::cold_inline_bytes_history_refuses_after_read_then_retries_full_with_exact_old_values',
             'shared_custody::shared_history_custody_refuses_unfunded_admission_then_survives_until_the_last_basis_drops'):
 reservations['worth-court-relational-certification::worth_court_relational_certification\nrelational::durable_publication::source_initial_inputs::checkpoint_admission::cold_materialization::'+path]=1
reservations['worth-court-relational-certification::worth_court_relational_certification\nrelational::durable_publication::source_initial_inputs::checkpoint_admission::cost::retired_peer_history_keeps_its_funding_through_edits_and_returns_real_historical_neighbors']=1
for path in ('byte_payloads::cold_inline_bytes_head_refuses_after_member_read_then_retains_exact_payload_through_edit', 'byte_payloads::cold_inline_bytes_history_refuses_after_read_then_retries_full_with_exact_old_values', 'peer_history::retired_peer_history_keeps_its_funding_through_edits_and_returns_real_historical_neighbors'):
 reservations['worth-court-relational-certification::worth_court_relational_certification\ncost::relational::source_initial_inputs::checkpoint_admission::cold_materialization::'+path]=1
(out/'reservations.json').write_text(json.dumps(reservations))
env.update(WORTH_REPO=str(root),WORTH_TEST_COST_RESERVATIONS=str(out/'reservations.json'),
           WORTH_TEST_COST_ACCOUNTING_DIRECTORY=str(out/'cpu'),WORTH_TEST_COST_SETUP_DIRECTORY=str(out/'setup'))
runner=CargoRunner(root,env)
manifest=root/'Cargo.toml'
reuse=build(manifest,out/'build',runner,selection={'packages':['worth-court-relational-certification'],
            'targets':{'test':['worth_court_relational_certification']}})
expression=sys.argv[2] if len(sys.argv)>2 else 'test(automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom)'
expected=int(sys.argv[3]) if len(sys.argv)>3 else 1
selected=[r for r in inventory(list_tests(reuse,out/'build/request-selection.json',runner,expression)) if r['selected']]
assert len(selected)==expected and not any(row['ignored'] for row in selected),selected
observed=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(prior/'nextest.toml'),
                 '--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','-E',expression],
                 manifest.parent,out/'run-selection.log')
counts=[0,0,0]
for event in json_objects(observed.stdout):
 if event.get('type')=='test' and event.get('event') in ('ok','failed'):
  counts[0 if event['event']=='ok' else 1]+=1
for path in runner.logs: print('FULL_OWNED_LOG_PATH:',path)
for event in json_objects(observed.stdout):
 if event.get('type')=='test' and event.get('event')=='failed': print(event.get('stdout',''))
print(observed.stdout)
print('EVIDENCE_TEST_COUNTS: '+','.join(map(str,counts)))
sys.exit(observed.returncode)
