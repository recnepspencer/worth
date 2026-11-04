"""Focused diagnostic using owned Cargo/nextest build and selection components."""
from pathlib import Path
import sys
root=Path.cwd().resolve()
sys.path.insert(0,str(root/'scripts/test_cost'))
from run_lane import child_environment
from cargo_capture import CargoRunner
from nextest_build import build,list_tests
from coverage import inventory
from reservations import configuration
from logs import json_objects
label=sys.argv[1]
out=root/'target/test-cost/query'/label
prior=root/'target/test-cost/query/d12-single-emission-bounded-host-court'
env=child_environment()
import json
out.mkdir(parents=True,exist_ok=True)
reservations=json.loads((prior/'reservations.json').read_text())
# The ordinary Serial caller joins one independently admitted Serial Host worker (two live workers).
reservations['worth-query-court-query-decl-host::worth_query_court_query_decl_host\nhost::application_home::reopening::checkpoint_funding::automatic_checkpoint_exhaustion_keeps_business_results_and_explicit_capture_uses_its_own_allowance']=2
reservations['worth-query-court-query-decl-host::worth_query_court_query_decl_host\nhost::application_home::reopening::checkpoint_funding::automatic_checkpoint_progresses_after_caller_cancellation_without_replacing_the_body_value']=2
for name in ('live_checkpoint_recapture_keeps_all_covered_canonical_bodies_cold', 'automatic_capture_at_head_forty_one_preserves_ordinary_request_results', 'close_checkpoint_uses_host_allowance_with_an_ordinary_close_caller', 'close_checkpoint_exhaustion_reports_failure_and_preserves_journal_reopen'):
 reservations['worth-query-court-query-decl-host::worth_query_court_query_decl_host\nhost::application_home::reopening::checkpoint_state::'+name]=2
for name in ('journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing', 'selected_complete_home_reopens_and_selects_another_complete_checkpoint', 'killed_home_reopens_normalized_image_plus_nonempty_tail', 'cold_image_plus_tail_recapture_reads_only_the_uncovered_original_frames', 'interrupted_owner_ack_resumes_original_attempt_and_reopens_unacknowledged_selection', 'automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom'):
 reservations['worth-query-court-query-decl-host::worth_query_court_query_decl_host\nhost::application_home::reopening::checkpoint_restart::'+name]=2
for name in ('host::application_home::reopening::same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key', 'host::application_home::reopening::flushed_and_pending_multibatch_homes_readmit_their_completed_publication', 'host::application_home::reopening::separate_process::separate_processes_readmit_old_keys_and_restore_the_actual_outcome_frontier'):
 reservations['worth-query-court-query-decl-host::worth_query_court_query_decl_host\n'+name]=2
(out/'reservations.json').write_text(json.dumps(reservations))
env.update(WORTH_REPO=str(root),WORTH_TEST_COST_RESERVATIONS=str(out/'reservations.json'),
           WORTH_TEST_COST_ACCOUNTING_DIRECTORY=str(out/'cpu'),WORTH_TEST_COST_SETUP_DIRECTORY=str(out/'setup'))
runner=CargoRunner(root,env)
manifest=root/'workspaces/worth-query/Cargo.toml'
reuse=build(manifest,out/'build',runner,selection={'packages':['worth-query-court-query-decl-host'],
            'targets':{'test':['worth_query_court_query_decl_host']}})
# Rebuilt source must not leave diagnostic instrumentation in the executed artifact.
import mmap
binaries=json.loads((out/'build/binaries.json').read_text())
for row in binaries['rust-binaries'].values():
 path=Path(row['binary-path'])
 with path.open('rb') as stream, mmap.mmap(stream.fileno(),0,access=mmap.ACCESS_READ) as data:
  for marker in (b'WORTH-UI-TEMPORARY-INSTRUMENTATION','WORTH-UI-TEMPORARY-INSTRUMENTATION'.encode('utf-16le')):
   assert data.find(marker)==-1, f'diagnostic residue: {path}'
 print('MARKER_FREE_EXECUTABLE:',path,flush=True)
expression=sys.argv[2] if len(sys.argv)>2 else 'test(automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom)'
expected=int(sys.argv[3]) if len(sys.argv)>3 else 1
selected=[r for r in inventory(list_tests(reuse,out/'build/request-selection.json',runner,expression)) if r['selected']]
assert len(selected)==expected and not any(row['ignored'] for row in selected),selected
chosen={row['binary']+'\n'+row['test']:reservations[row['binary']+'\n'+row['test']] for row in selected}
configuration(prior/'nextest.toml',out/'nextest.toml','ci',chosen,root/'scripts/test_cost/account.py')
observed=runner(['cargo','nextest','run',*reuse,'--profile','ci','--config-file',str(out/'nextest.toml'),
                 '--success-output','immediate','--ignore-default-filter','--no-fail-fast','--no-tests=fail','--message-format','libtest-json','-E',expression],
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
