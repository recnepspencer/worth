# Funding/integration merge lane evidence: IN PROGRESS

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
| [funding-merge-agent-context.log](funding-merge-agent-context.log) | 0 | 0/0/0 |
| [funding-merge-boundary-owner-audit.log](funding-merge-boundary-owner-audit.log) | 0 | 0/0/0 |
| [funding-merge-boundary.log](funding-merge-boundary.log) | 1 | 0/0/0 |
| [funding-merge-catalogs.log](funding-merge-catalogs.log) | 0 | 0/0/0 |
| [funding-merge-commit-identity.log](funding-merge-commit-identity.log) | 0 | 0/0/0 |
| [funding-merge-final-format.log](funding-merge-final-format.log) | 0 | 0/0/0 |
| [funding-merge-final-line-caps.log](funding-merge-final-line-caps.log) | 0 | 0/0/0 |
| [funding-merge-final-source-identity.log](funding-merge-final-source-identity.log) | 0 | 0/0/0 |
| [funding-merge-format-02.log](funding-merge-format-02.log) | 0 | 0/0/0 |
| [funding-merge-format.log](funding-merge-format.log) | 0 | 0/0/0 |
| [funding-merge-line-caps.log](funding-merge-line-caps.log) | 0 | 0/0/0 |
| [funding-merge-query-check.log](funding-merge-query-check.log) | 0 | 0/0/0 |
| [funding-merge-query-extra.log](funding-merge-query-extra.log) | 0 | 5/0/0 |
| [funding-merge-query.log](funding-merge-query.log) | 0 | 9/0/0 |
| [funding-merge-review-format.log](funding-merge-review-format.log) | 0 | 0/0/0 |
| [funding-merge-root-clippy-02.log](funding-merge-root-clippy-02.log) | 101 | 0/0/0 |
| [funding-merge-root-clippy-03.log](funding-merge-root-clippy-03.log) | 0 | 0/0/0 |
| [funding-merge-root-clippy-04.log](funding-merge-root-clippy-04.log) | 0 | 0/0/0 |
| [funding-merge-root-clippy.log](funding-merge-root-clippy.log) | 101 | 0/0/0 |
| [funding-merge-root-courts.log](funding-merge-root-courts.log) | 0 | 14/0/0 |
| [funding-merge-root-review.log](funding-merge-root-review.log) | 0 | 1/0/0 |
| [funding-merge-root-units.log](funding-merge-root-units.log) | 0 | 4/0/0 |
| [funding-merge-source-identity.log](funding-merge-source-identity.log) | 0 | 0/0/0 |
| [funding-merge-source-whitespace.log](funding-merge-source-whitespace.log) | 0 | 0/0/0 |
| [funding-merge-store-clippy.log](funding-merge-store-clippy.log) | 0 | 0/0/0 |
| [funding-merge-store.log](funding-merge-store.log) | 0 | 1/0/0 |
| [funding-merge-ui-lock-audit.log](funding-merge-ui-lock-audit.log) | 1 | 0/0/0 |
| [funding-merge-ui-lock-normalized-audit.log](funding-merge-ui-lock-normalized-audit.log) | 0 | 0/0/0 |
| [funding-merge-whitespace.log](funding-merge-whitespace.log) | 2 | 0/0/0 |

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
and BC9002 consumer migration are unrun/unimplemented in this merge slice.

## Exact merge seams and compile-only caller cutovers

- Transferred checkpoint_complete/branches.rs was resolved only to retain the
  actual current/external roots and incoming content ownership. No named digest
  edit was made there or in original_image.rs. roots.rs/regions/emission.rs use
  that same held-root custody, without restoring historical replay.
- The two new private kind_projection/reconstruction leaves separate request
  installation from the incoming controlled-publication custody driver; neither
  introduces another projection validator or aggregate funding constructor.
- Mechanical obsolete reexports removed from these exact Native files:
  authority/mutation/mod.rs; runtime/mod.rs; runtime/state/mod.rs;
  runtime/state/subsystems/mod.rs; runtime/state/subsystems/record_identity/mod.rs;
  runtime/state/subsystems/services/mod.rs;
  runtime/state/subsystems/services/symbol_table.rs. The private definitions and
  their real remaining uses/tests are retained.
- Caller vocabulary/outcome migrations: Foundation values/aspect_value.rs
  (retired Bytes quote arm); Native durability/checkpoints/aspect_state_images/
  readmission_funding.rs (retired Content shape), checkpoints/images.rs,
  storage/residency/region_load.rs and region_load/materialization_funding.rs
  (typed Content stops), region_load/materialization_funding/aspect_state.rs
  (retired Bytes scalar spelling), residency/history_fold.rs and overlay/
  partition_content/digest_index/append_frontier.rs (named digest access).
  All are small merged-type/caller changes; no new feature or shim is added.
- Exact retained boundary rows and the five unapplied caller proposals are
  listed in funding-merge-boundary-handoff.md. The UI lock propagation is only
  generated resolution for the committed Host/Store dependency graph.

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
