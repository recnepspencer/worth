# State-bounded funding lane evidence ? IN PROGRESS

Logs record exact command, HEAD, dirty state, times, exit and counts. All source
is dirty at3be5798075; the next documentation-only commit does not land code.
Snapshot: funding-state-source-snapshot.json. Prior accepted Host-partial lanes
remain separately indexed in checkpoint-funding-lanes.md.

| Log under target/lane-evidence | Exit | Pass/fail/ignored | Evidence limit |
|---|---:|---:|---|
| [funding-state-attribution-head-forty-one-accounted.log](funding-state-attribution-head-forty-one-accounted.log) | 0 | 1/0/0 |  |
| [funding-state-attribution-head-forty-one-corrected.log](funding-state-attribution-head-forty-one-corrected.log) | 0 | 1/0/0 | test passed, accounting unavailable; superseded |
| [funding-state-attribution-head-forty-one.log](funding-state-attribution-head-forty-one.log) | 1 | 0/0/0 |  |
| [funding-state-close-fresh-and-explicit.log](funding-state-close-fresh-and-explicit.log) | 100 | 0/2/0 |  |
| [funding-state-courts-format-query.log](funding-state-courts-format-query.log) | 0 | 0/0/0 |  |
| [funding-state-courts-format-root.log](funding-state-courts-format-root.log) | 0 | 0/0/0 |  |
| [funding-state-courts-format-store.log](funding-state-courts-format-store.log) | 0 | 0/0/0 |  |
| [funding-state-fresh-and-close-corrected.log](funding-state-fresh-and-close-corrected.log) | 0 | 2/0/0 |  |
| [funding-state-fresh-refusal-diagnostic.log](funding-state-fresh-refusal-diagnostic.log) | 100 | 0/1/0 |  |
| [funding-state-fresh-target-format.log](funding-state-fresh-target-format.log) | 0 | 0/0/0 |  |
| [funding-state-marker-free-rebuild.log](funding-state-marker-free-rebuild.log) | 0 | 0/0/0 |  |
| [funding-state-native-check.log](funding-state-native-check.log) | 101 | 0/0/0 |  |
| [funding-state-native-court-check.log](funding-state-native-court-check.log) | 0 | 0/0/0 |  |
| [funding-state-native-format.log](funding-state-native-format.log) | 0 | 0/0/0 |  |
| [funding-state-native-prefix-and-missing-base.log](funding-state-native-prefix-and-missing-base.log) | 0 | 2/0/0 |  |
| [funding-state-native-proof-check.log](funding-state-native-proof-check.log) | 0 | 0/0/0 |  |
| [funding-state-native-proof-format.log](funding-state-native-proof-format.log) | 0 | 0/0/0 |  |
| [funding-state-native-query-format.log](funding-state-native-query-format.log) | 0 | 0/0/0 |  |
| [funding-state-prefix-court-check.log](funding-state-prefix-court-check.log) | 0 | 0/0/0 |  |
| [funding-state-prefix-format.log](funding-state-prefix-format.log) | 0 | 0/0/0 |  |
| [funding-state-query-check.log](funding-state-query-check.log) | 0 | 0/0/0 |  |
| [funding-state-query-court-check.log](funding-state-query-court-check.log) | 0 | 0/0/0 |  |
| [funding-state-query-format.log](funding-state-query-format.log) | 0 | 0/0/0 |  |
| [funding-state-root-format.log](funding-state-root-format.log) | 0 | 0/0/0 |  |
| [funding-state-store-check.log](funding-state-store-check.log) | 0 | 0/0/0 |  |
| [funding-state-store-format.log](funding-state-store-format.log) | 0 | 0/0/0 |  |

Final functional controls: Host2/0/0 and Native2/0/0. Final attribution1/0/0
with complete accounting, distinct from qualification. Marker-free rebuild0/0/0.
All four functional CPU observations are complete/no errors (Host2 slots each,
Native1 each). Historical failures remain visible; see failure map.

UNRUN: selected live/cold recapture, discarded-history growth, automatic head41
success, close-exhaustion recovery, unchanged bounded-tail and affected D12
refusal/kill/ACK/CDC/recovery qualification; phase-completion gates and queue.
No ignored/exhaustive/scenario/query-scale tests executed. No current funding
approval or source landing.
