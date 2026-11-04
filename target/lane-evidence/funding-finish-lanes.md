# Funding finish lane evidence - IN PROGRESS

All reported lanes below have full logs with command, HEAD, dirty state, start/end, exit and counts.
Logs are dirty at03eb43385c; source snapshots distinguish the actual edit batches.
No source landing or final phase certification is claimed.

| Full log under target/lane-evidence | Exit | Pass/fail/ignored |
|---|---:|---:|
| [funding-close-owner-host-controls.log](funding-close-owner-host-controls.log) | 0 | 9/0/0 |
| [funding-close-owner-host-format.log](funding-close-owner-host-format.log) | 0 | 0/0/0 |
| [funding-close-owner-query-check.log](funding-close-owner-query-check.log) | 0 | 0/0/0 |
| [funding-close-owner-store-actual-result-corrected.log](funding-close-owner-store-actual-result-corrected.log) | 100 | 0/1/0 |
| [funding-close-owner-store-actual-result.log](funding-close-owner-store-actual-result.log) | 1 | 0/0/0 |
| [funding-close-owner-store-check.log](funding-close-owner-store-check.log) | 0 | 0/0/0 |
| [funding-close-owner-store-court.log](funding-close-owner-store-court.log) | 100 | 0/1/0 |
| [funding-close-owner-store-format.log](funding-close-owner-store-format.log) | 0 | 0/0/0 |
| [funding-finish-cold-recapture-corrected.log](funding-finish-cold-recapture-corrected.log) | 100 | 1/4/0 |
| [funding-finish-cost-category-check.log](funding-finish-cost-category-check.log) | 0 | 0/0/0 |
| [funding-finish-cost-category-format.log](funding-finish-cost-category-format.log) | 0 | 0/0/0 |
| [funding-finish-cost-category-growth.log](funding-finish-cost-category-growth.log) | 0 | 1/0/0 |
| [funding-finish-covered-region-check.log](funding-finish-covered-region-check.log) | 0 | 0/0/0 |
| [funding-finish-covered-region-format.log](funding-finish-covered-region-format.log) | 0 | 0/0/0 |
| [funding-finish-deleted-branch-check.log](funding-finish-deleted-branch-check.log) | 0 | 0/0/0 |
| [funding-finish-deleted-branch-format.log](funding-finish-deleted-branch-format.log) | 0 | 0/0/0 |
| [funding-finish-discarded-branch-growth.log](funding-finish-discarded-branch-growth.log) | 0 | 1/0/0 |
| [funding-finish-equal-state-growth.log](funding-finish-equal-state-growth.log) | 100 | 0/1/0 |
| [funding-finish-growth-and-operational.log](funding-finish-growth-and-operational.log) | 100 | 1/1/0 |
| [funding-finish-growth-observation-format.log](funding-finish-growth-observation-format.log) | 0 | 0/0/0 |
| [funding-finish-host-bounded-reopen-kill-ack.log](funding-finish-host-bounded-reopen-kill-ack.log) | 100 | 6/3/0 |
| [funding-finish-host-live-auto-close.log](funding-finish-host-live-auto-close.log) | 0 | 3/0/0 |
| [funding-finish-native-court-check.log](funding-finish-native-court-check.log) | 0 | 0/0/0 |
| [funding-finish-native-selected-courts.log](funding-finish-native-selected-courts.log) | 100 | 3/5/0 |
| [funding-finish-query-court-check.log](funding-finish-query-court-check.log) | 0 | 0/0/0 |
| [funding-finish-root-format.log](funding-finish-root-format.log) | 0 | 0/0/0 |
| [funding-finish-store-court-check-owned.log](funding-finish-store-court-check-owned.log) | 0 | 0/0/0 |
| [funding-finish-store-court-check.log](funding-finish-store-court-check.log) | 101 | 0/0/0 |
| [funding-finish-store-refusal-and-ack.log](funding-finish-store-refusal-and-ack.log) | 1 | 3/1/0 |
| [funding-finish-vocabulary-format.log](funding-finish-vocabulary-format.log) | 0 | 0/0/0 |

Current qualified controls: funding-close-owner-host-controls.log9/0/0, with actual
generated2-slot nextest overrides; funding-finish-cost-category-growth.log1/0/0.
Store/Query owner checks both pass. Prior positive cases and all failed attempts
are retained above; failures are not relabeled. Native/Host changed cold capture
and the initializer-refusal route remain red.

Store diagnostic: first live close now passes unchanged. Later checkpoint Result
is Native(Capture(...Region(0):SourceUnavailable)), preceding its anchor-gate
timeout. Diagnostic-only test edits were restored byte-for-byte; SHA evidence in
funding-close-owner-diagnostic-source.json. The failed initial probe was attached
to the wrong inner receiver and did not run a test; corrected probe exposes the
actual product failure. No wait, assertion, policy or workload changed.

Earlier custom Host runners changed accounting rows without generating matching
nextest overrides. Those earlier outcomes are functional evidence only. The final
nine-case run fixes both via the owned configuration generator. Its generated
config and raw CPU observations are retained. Root growth requires one slot.

Current full-leaf source snapshot: funding-home-owner-final-source-leaves.json.
Earlier snapshots and both independent reviews remain available for chronology.
Fresh final verdict: local Home maintenance owner APPROVE; full funding REJECT.

UNRUN: cold zero-covered-read filesystem court, final affected CDC/recovery and
Clippy/dirty line-cap/boundary/agent-context gates. Queue unrun: decoder default
stack qualification, D13, named digests, BC9002. No ignored, query-scale, exhaustive
Home oracle or Store scenario suite executed.

Accepted attribution remains in funding-state-attribution.md/json and accounted
log:16,240,012 versus16,236,875 at head41/covered0 before this carry correction.
No current exact unit quote or fresh-capture speedup is claimed. Earlier accepted
funding lanes are indexed in funding-state-lanes.md.
