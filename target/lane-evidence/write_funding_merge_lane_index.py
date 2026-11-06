"""Index every current-turn lane without hiding failed attempts."""
from pathlib import Path
import re

directory = Path('target/lane-evidence')
rows = []
for path in sorted(directory.glob('funding-merge*.log')):
    content = path.read_text(encoding='utf-8')
    if 'HEAD: 25619ac5d1ae6092ab5a91bc4b7123be6042475c' not in content:
        continue
    fields = {}
    for key in ['exit_code', 'tests_passed', 'tests_failed', 'tests_ignored']:
        matches = re.findall(r'^' + key + r': (\d+)$', content, re.M)
        fields[key] = int(matches[-1]) if matches else None
    assert all(value is not None for value in fields.values()), (path, fields)
    rows.append(f'| [{path.name}]({path.name}) | {fields["exit_code"]} | '
                f'{fields["tests_passed"]}/{fields["tests_failed"]}/{fields["tests_ignored"]} |')
header = '''# Funding/integration merge lane evidence: IN PROGRESS

All paths below are under `target/lane-evidence/`. Each raw log carries the
exact command, pre-merge HEAD, dirty-state flag, UTC start/end and exit code.
Counts are selected passed/failed/ignored; nextest also prints unselected
ignored inventory events, which were not run. No ignored test was selected.
Compiler/runner subsidiary logs are linked by each packet's FULL_OWNED_LOG_PATH.

Source provenance: initial compilation tree c7e6413c053ef82c98dbccf8bfb3bdad930c6bde;
the only later Rust changes are the exhaustive Complete refusal mapper and its
pure regression. `funding-merge-final-source-identity.log` and the JSON blob
inventory bind those changes and the mechanical UI lock propagation. Host-extra
uses the existing Host binary from the initial tree; its scalar-only subjects do
not reach the added Content refusal arm. Final Root Clippy and the new pure
regression qualify that arm without rebuilding unchanged dependent workspaces.

| Raw log | Exit | P/F/I |
| --- | ---: | ---: |
'''
footer = '''
## Failures, corrections and limits

- Root Clippy attempt1: an auto-merged Foundation quote still named the retired
  Bytes variant. Product merge error; remove that arm and retain InlineBytes's
  actual length quote. Attempt2: a retired Content aspect-shape arm and four
  missing typed Content outcomes; two obsolete import chains also remained.
  Product/caller merge errors; remove retired vocabulary, preserve each actual
  Content outcome, remove only unused reexports. Attempts3 and4 pass. The33
  Native warnings have no diagnostic additions versus accepted funding evidence.
- Initial independent source review found a real Complete tail decoder error:
  content admission, including request exhaustion, became corruption. Both
  frame and initial-input mappings now use the existing content conversion.
  The new pure regression passes; structural framing failures stay corruption.
- Source review also caught the test-only reset call and stale folded-root
  field access before the first Native compile. The production reset and the
  actual captured-root field are used. No test/assertion was weakened or removed.
- Optional whitespace audit initially treated historical patch context spaces
  as source whitespace. The corrected CRLF-aware source check preserves the
  literal patch artifacts and passes. It changes no enforcement configuration.
- UI lock audit initially compared dependency spellings (windows versus windows
  0.62.2). The corrected audit resolves exact package tuples before comparison;
  all41 additions match qualified workspace resolutions, with zero removals and
  no changed external version/checksum. Four existing local dependency rows
  reflect the committed Host/Native/World/Bridge manifest graph. No UI build ran.
- Boundary remains exit1: five caller seals and12 BC9002 findings. Six Native
  topology sites are P-A's explicit queued work; the remaining paths reach
  Lifecycle and Bank. Exact findings/quiet-diff probes are recorded in
  funding-merge-boundary-handoff.json. No baseline growth, suppression, broad
  caller rule or ungranted owner edit was made. Final independent review02 is
  REJECT for constitutional closure, with no supported funding semantic defect.

## Validation budget and scope

This turn: five Clippy invocations (four Root, one Store), one Query check;
five focused build invocations; seven top-level selected test-run commands,
34 passing parent tests, zero failed or selected ignored tests. Their filesystem
writer subprocesses are parts of those parent courts. Two additional Cargo run
commands built/invoked the mandatory boundary and agent-context tools. The
multi-workspace/cold-cache budget exception is recorded in merge-14b16-plan.md.
There was no broad qualification, query-scale, exhaustive Home oracle, Store
scenario run, or tool self-test suite. D13 receipts/restore, named-digest cutover
and BC9002 consumer migration are unrun/unimplemented after this pending merge.

## Requirement trace

| Requirement | Evidence |
| --- | --- |
| D12 ordinary8M/default upkeep bounded tails | automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom; unchanged <=40 assertion |
| D12 cold/live no covered-body recapture | cold_image_plus_tail_recapture_reads_only_the_uncovered_original_frames; live_checkpoint_recapture_keeps_all_covered_canonical_bodies_cold |
| D12 maintenance/caller funding | the two checkpoint_funding controls, automatic_capture_at_head_forty_one_preserves_ordinary_request_results, close_checkpoint_uses_host_allowance_with_an_ordinary_close_caller |
| D11 failed close retains tails | close_checkpoint_exhaustion_reports_failure_and_preserves_journal_reopen |
| D12/D17 typed original-source refusal; Architecture5 | complete_original_frame_content_denials_preserve_their_semantic_and_request_causes |
| D12 original/receiver custody and growth |14 named Native court selections; three CDC/recovery units; Store observer/history/mutation/ACK selection |
| Happy same-directory default stack | direct same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key, RUST_MIN_STACK absent |

No format/version change is introduced by the P-A merge resolutions. The accepted
selected-Home missing/predecessor admission-image cutoff remains. No new class
of development Home is retired by these resolutions. The D13 format bump and
capture-proof registry restoration remain pending; format13 is unchanged here.
'''
(directory / 'funding-merge-lanes.md').write_text(header + '\n'.join(rows) + '\n' + footer, encoding='utf-8')
print('Indexed lanes:', len(rows))
