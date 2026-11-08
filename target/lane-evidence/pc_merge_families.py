"""Classify the completed census before any merge fallout repair."""
from pathlib import Path
import collections
import json
import re
import subprocess
import sys

r = Path.cwd()
e = r / 'target/lane-evidence'
subprocess.run([sys.executable, str(e / 'pc_merge_result_census.py')], check=True,
               stdout=subprocess.DEVNULL)
data = json.loads((e / 'pc-merge-raw-result-census.json').read_text(encoding='utf8'))
overrides = {
    'genuine_complete_section_retains_capture_and_resumes_same_selection_ack':
        ('HAPPY-HELD-complete-ack-contract', 'P-A',
         'Acknowledgement(CaptureNotBound): the coordinator explicitly holds the Complete/ACK contract for P-A.'),
    'a_nonempty_declared_source_reads_and_mutates_after_directory_home_reopen':
        ('HAPPY-content-data-before-codecs', 'Native original-source schema admission',
         'Authenticated source DATA is incorrectly bound to absent receiver codecs before installation.'),
    'inherited_directory_key_refuses_a_sibling_product_affinity':
        ('HAPPY-HELD-H6-durable-sibling', 'P-C H6',
         'D16 track H does not include durable sibling publication; H6 is explicitly held.'),
    'recovery_entry_authority_is_compiler_sealed':
        ('HAPPY-recovery-construction-contract', 'Store recovery compiler court',
         'Two literals name retired fields; three rendered snapshots must become semantic private-construction denials.'),
    'late_history_refusal_retains_actual_partial_root_caps_and_home_lease_until_cleanup':
        ('CLEANUP-C1-injected-recovery-custody', 'Host recovery cleanup',
         'Injected late-history refusal exercises partial opening cleanup, not a graceful service journey.'),
    'a_completed_transition_occurrence_restores_target_contracts_and_continues_only_its_missing_suffix':
        ('CLEANUP-C2-zero-budget-transition-boundary', 'Store transition court',
         'Explicit zero-memory admission now refuses earlier at authenticated schema image; normal B reopen is green.'),
    'cancelled_recovery_process_reports_a_blocked_post_effect_outcome':
        ('CLEANUP-C1-C2-recovery-interruption-fixture', 'Store physical recovery court',
         'Cancellation campaign fixture asserts its footprint exceeds admission before the interruption.'),
    'killed_recovery_process_reopens_after_each_named_recovery_seam':
        ('CLEANUP-C1-C2-recovery-interruption-fixture', 'Store physical recovery court',
         'Kill-at-recovery-seams fixture asserts its footprint exceeds admission before the interruption.'),
    'recovery_work_is_independent_of_unrelated_persisted_store_size':
        ('CLEANUP-C2-recovery-growth', 'Store physical recovery court',
         'Larger persisted-size twin cannot be built within the bounded residue fixture.'),
}
observer_cases = {
    'independent_observer_reads_closed_media_without_runtime_authority',
    'independent_observer_rejects_every_identity_grammar_corruption_class',
    'mutation_contention_confinement_and_readmission',
    'namespace_creation_and_fresh_process_discovery',
    'partial_effects_and_barrier_honesty',
}
for case in observer_cases:
    overrides[case] = ('HAPPY-observer-prebuild', 'Store executable prebuild owner',
                       'The selected gate omits the genuine physical_media_os_observer executable prerequisite.')
for word in ('zero', 'one'):
    overrides['bounded_host_oracle_seed_' + word + '_continues_across_real_process_reopens'] = (
        'CLEANUP-C1-host-abandonment-convergence', 'Host oracle / Native-Query recovery',
        'Reference-prefix entity meaning diverges at step25 reopen after abandonment; comparator remains unchanged.')

previous = {row['name']: row for row in
            json.loads((e / 'pa-merge-final-families.json').read_text(encoding='utf8'))['outcomes']}
unknown = []
for row in data['failures']:
    case = row['name'].split('$')[-1].rsplit('::', 1)[-1]
    prior = previous.get(row['name'], {})
    old_class = row.get('prior_class')
    if case in overrides:
        classification, owner, reason = overrides[case]
    elif old_class:
        classification = old_class
        owner = prior.get('owner', row['workspace'] + ' owning court / recorded cleanup')
        reason = prior.get('reason', 'Exact previously reviewed case; preserve final classification.')
    else:
        classification, owner, reason = 'UNCLASSIFIED', 'unattributed', 'Read exact assertion and owner before editing.'
        unknown.append(row['name'])
    row.update(family=classification, owner=owner, reason=reason,
               state='incomplete' if row.get('outcome') == 'incomplete' else 'red')

counts = collections.Counter(row['family'] for row in data['failures'])
data.update(head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
            totals=counts, unknown=unknown,
            note='Full failure census before repair; final outcomes are appended after focused verification.')
name = 'pc-merge-families' if '--finalize' in sys.argv else 'pc-merge-families-preview'
if '--finalize' in sys.argv:
    assert len(json.loads((e / 'pc-merge-tier2-results.json').read_text(encoding='utf8'))) == 11
    assert not unknown, unknown
(e / (name + '.json')).write_text(json.dumps(data, indent=2), encoding='utf8')
lines = ['# P-C merge failure families', '', 'Source: `' + data['head'] + '`.', '',
         '| Family | Cases | Owner |', '|---|---:|---|']
for family, count in sorted(counts.items()):
    owners = sorted({row['owner'] for row in data['failures'] if row['family'] == family})
    lines.append('| ' + family + ' | ' + str(count) + ' | ' + '; '.join(owners) + ' |')
lines += ['', '## Every non-success', '']
for row in data['failures']:
    lines += ['### `' + row['name'] + '`', '',
              row['family'] + ' — ' + row['owner'] + '.', '', row['reason'], '',
              'Evidence: `' + row['log'] + '`.', '', '```text',
              row['failure'][:10000] if row['failure'] else 'Audited incomplete campaign; see interruption_audit in JSON.', '```', '']
(e / (name + '.md')).write_text('\n'.join(lines), encoding='utf8')
print(json.dumps(counts, indent=2))
print('UNCLASSIFIED', len(unknown))
for name in unknown:
    print(name)
