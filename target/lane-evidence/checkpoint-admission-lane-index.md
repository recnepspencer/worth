# Checkpoint admission implementation lane evidence

Counts are selected tests passed/failed/ignored; compile/format commands execute zero tests. Historical green logs certify their recorded source state, not unimplemented admission. Each log has exact command, HEAD/dirty flag, start/end and exit. No ignored test body was executed; nextest may include metadata events for unselected ignored cases. Full owned build/run child logs are embedded in the Native focused runtime logs.

| Lane log | Result | P/F/I | Recorded HEAD | Initially clean |
|---|---|---|---|---|
| [admission-start-root-check-tests.log](admission-start-root-check-tests.log) | FAIL | 0/0/0 | 284bd3215a | true |
| [admission-start-query-check-tests.log](admission-start-query-check-tests.log) | FAIL | 0/0/0 | 284bd3215a | true |
| [admission-start-store-check-tests.log](admission-start-store-check-tests.log) | PASS | 0/0/0 | 284bd3215a | true |
| [admission-query-check-tests-issued.log](admission-query-check-tests-issued.log) | PASS | 0/0/0 | 2fb3d638e1 | true |
| [admission-root-check-tests-issued.log](admission-root-check-tests-issued.log) | PASS | 0/0/0 | 2fb3d638e1 | true |
| [held-native-courts-check.log](held-native-courts-check.log) | FAIL | 0/0/0 | 2fb3d638e1 | false |
| [held-native-courts-check-corrected.log](held-native-courts-check-corrected.log) | PASS | 0/0/0 | 503e31d904 | true |
| [held-native-courts-check-final.log](held-native-courts-check-final.log) | PASS | 0/0/0 | 766cf768e7 | true |
| [held-native-courts-first.log](held-native-courts-first.log) | FAIL | 3/3/0 | 503e31d904 | true |
| [held-native-courts-three-rerun.log](held-native-courts-three-rerun.log) | FAIL | 2/1/0 | 766cf768e7 | true |
| [public-observation-check-tests.log](public-observation-check-tests.log) | PASS | 0/0/0 | 503e31d904 | false |
| [admission-public-observation-fmt.log](admission-public-observation-fmt.log) | PASS | 0/0/0 | 503e31d904 | false |
| [native-image-durable-reference-check.log](native-image-durable-reference-check.log) | PASS | 0/0/0 | 766cf768e7 | false |
| [native-image-logical-reference-courts.log](native-image-logical-reference-courts.log) | PASS | 7/0/0 | 4215bd9518 | true |
| [native-image-output-projection-check.log](native-image-output-projection-check.log) | PASS | 0/0/0 | 4215bd9518 | false |
| [native-image-output-projection-format.log](native-image-output-projection-format.log) | PASS | 0/0/0 | 4215bd9518 | false |
| [native-image-output-public-courts.log](native-image-output-public-courts.log) | PASS | 2/0/0 | e9a74cafd5 | true |
| [native-image-producer-check-corrected.log](native-image-producer-check-corrected.log) | PASS | 0/0/0 | 766cf768e7 | false |
| [native-image-producer-check-tests.log](native-image-producer-check-tests.log) | FAIL | 0/0/0 | 766cf768e7 | false |
| [native-image-producer-durable-reference-format.log](native-image-producer-durable-reference-format.log) | PASS | 0/0/0 | 766cf768e7 | false |
| [native-image-producer-focused-check.log](native-image-producer-focused-check.log) | PASS | 0/0/0 | 766cf768e7 | false |
| [native-image-producer-format.log](native-image-producer-format.log) | PASS | 0/0/0 | 766cf768e7 | false |
| [native-image-producer-public-courts.log](native-image-producer-public-courts.log) | PASS | 8/0/0 | 766cf768e7 | false |
| [native-image-proof-format.log](native-image-proof-format.log) | PASS | 0/0/0 | 766cf768e7 | false |
| [native-image-proof-prelude-format.log](native-image-proof-prelude-format.log) | PASS | 0/0/0 | 766cf768e7 | false |
| [native-image-pure-codec-units.log](native-image-pure-codec-units.log) | PASS | 2/0/0 | e9a74cafd5 | true |
| [native-image-query-callers-check.log](native-image-query-callers-check.log) | FAIL | 0/0/0 | e9a74cafd5 | true |
| [native-image-query-callers-check-corrected.log](native-image-query-callers-check-corrected.log) | PASS | 0/0/0 | 8e06ce9e12 | true |
| [native-image-query-denial-format.log](native-image-query-denial-format.log) | PASS | 0/0/0 | e9a74cafd5 | false |
| [native-image-query-format.log](native-image-query-format.log) | PASS | 0/0/0 | 766cf768e7 | false |
| [native-image-store-callers-check.log](native-image-store-callers-check.log) | PASS | 0/0/0 | 8e06ce9e12 | true |
| [native-image-store-format.log](native-image-store-format.log) | PASS | 0/0/0 | 8e06ce9e12 | true |

Failures and fixes: checkpoint-admission-failure-ledger.md.

The five held Store passes are the union of three first-run passes and two corrected rerun passes. Interrupted ACK remains red: unchanged Native-only reopen observes five member reads instead of zero. Image admission is not consumed by reopen yet. P-A owns this remainder. Current producer/caller source is reviewed at8e06ce9e12; the following metadata-only evidence commit changes no Rust source.

Unrun for this unfinished implementation phase: bounded-tail filesystem acceptance; current-HEAD Host reopen/kill/refusal/CDC/recovery qualification and direct unset-stack controls; genuine Home codec-retirement image admission; selected missing/predecessor-image and byte-only interim-proof refusals; phase-completion boundary/context/line-cap gates; whole-task independent approval. Remaining grant12 provider/court migrations and baseline shrink are incomplete. Native missing-image guard is unchanged.
