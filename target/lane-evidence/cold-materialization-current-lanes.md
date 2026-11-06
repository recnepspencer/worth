# Current cold-materialization lane evidence

All six logs identify base HEAD146a4f5e13 plus a dirty worktree. They preserve
exact command, status-empty flag, UTC start/end, exit code and counts.
`cold-materialization-current-source.json` records the final ten-file source
SHA256 inventory. The last check was compiled from that source; subsequent
changes are evidence/docs only. No source implementation commit is claimed.

| Log | Command | Exit | Passed/failed/ignored |
| --- | --- | ---: | ---: |
| [funded-cold-owner-check.log](funded-cold-owner-check.log) | cargo check -p worth-relational --tests | 101 | 0/0/0 |
| [funded-cold-owner-format.log](funded-cold-owner-format.log) | cargo fmt -p worth-relational | 0 | 0/0/0 |
| [funded-cold-owner-check-corrected.log](funded-cold-owner-check-corrected.log) | cargo check -p worth-relational --tests | 101 | 0/0/0 |
| [funded-cold-review-corrections-format.log](funded-cold-review-corrections-format.log) | cargo fmt -p worth-relational | 0 | 0/0/0 |
| [funded-cold-reviewed-preflight-format.log](funded-cold-reviewed-preflight-format.log) | cargo fmt -p worth-relational | 0 | 0/0/0 |
| [funded-cold-reviewed-preflight-check.log](funded-cold-reviewed-preflight-check.log) | cargo check -p worth-relational --tests | 0 | 0/0/0 |

First check: five implementation errors in cause alias, two borrowed variant
closures' classified-error conversion, unsupported SharedColumn<u128>, and the
Full contract error conversion. All corrected at their source. Second check:
the new funded collection omitted its length observation used by an unchanged
private refusal court. Added the collection's length method and used it in
production fold construction. No assertion changed. Final check passed, with
35 lib warnings and27 lib-test warnings (22 duplicates). These are check
warnings; full Root/Store Clippy has not run this resume. No tests executed.

Earlier accepted qualification is indexed in cold-funding-resume-lanes.md;
prior cold failures in funding-finish-lanes.md remain unresolved evidence.
The accepted partial-Home regression court was not changed or rerun here.
The growth/control evidence from earlier commits is not claimed to certify
the new cold loader accounting.

## Unrun, pending actual-owner repair

- Native cold selection: complete_source_recovery_restores_original_inputs_for_another_complete_capture;
  cold_complete_recapture_preserves_original_index_generations_and_live_tail;
  cloned_complete_plan_retries_after_member_read_cancellation_without_directory_contamination.
- Host cold selection: killed_home_reopens_normalized_image_plus_nonempty_tail;
  automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom.
- Store: selected_checkpoint_survives_observer_drop_online_history_and_mutation_before_ack_recovery.
- Filesystem zero-covered-read: cold_image_plus_tail_recapture_reads_only_the_uncovered_original_frames.
- Final CDC/recovery/ACK/refusal selection; new runtime memory/work/refund and
  lifetime controls; new instrumented attribution; full Root and Store Clippy.
- Planned completion gates: dirty Rust line guard, boundary, agent-context.
  Ten scoped source files were counted in the snapshot (all <=400); that is
  not a claim that the mandatory dirty guard ran. Last recorded boundary lane
  is red at the three queued BC9002 sites; it was not rerun here.

No ignored tests, query-scale, exhaustive oracle, Store scenario suite, push,
other-worktree mutation, build-profile override or cleanup ran. One build at a
time. Latest observed C: free343,617,777,664 bytes. No build remains running.
