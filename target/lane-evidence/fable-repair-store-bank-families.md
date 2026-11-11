# Store and Bank selected-failure reruns after the repair commit `54b4fa230d`

Lanes: `fable-repair-store-focused.log` (exit 1, END 2026-11-10T18:38Z) and `fable-repair-bank-focused.log`
(exit 0, END 19:13Z), run by `fable_repair_lanes_b.ps1` on the repair batch (same bytes as `54b4fa230d`).
Out dirs `fable-repair-store-focused/`, `fable-repair-bank-focused/`. Selection: prior failures plus changed bodies.

## Bank: GREEN

bank-http-adapter 44/44, bank-server 97/97, external-rail-server 54/54 and 125/125, bank court server 1/1,
user-node 3/3. 324 passed, 0 failed.

## Store: 52 failed

Totals: application court 87 + 15 passed / 34 + 10 failed; phase-eight process 18 passed, 4 failed, 4 timed out;
recovery-runtime production x3, store-physical, source, store, physical-record journeys, worth-store,
physical-backend, worth-store-application all green (369 passed).

| # | Family (cause as printed) | Tests | Track call | Next step |
|---|---|---|---|---|
| S1 | Query Home idempotency after reopen: `WorthQueryApplicationIdempotencyResolutionDenial { kind: ProviderUnavailable }`, directly or inside the Host oracle child (`host_observation/requests.rs:55`) | ~13 | likely HAPPY, same as Root F6 | Rerun after the merge batch (H8 stored-text `2d6822abbe`, P-A reopen). Then root-cause as F6. |
| S2 | Host oracle child refused at opening: `WorthQueryApplicationHomeOpenRefused("Query(OriginalBindingUnavailable)")` (`host_observation/opening.rs:34`). Includes `partial_released_key_admission_retries_same_home_without_duplicate_retention` | 1+ | likely HAPPY, Query reopen | The earlier "recovered authored index writes disagree" cause is GONE for this court (fixed in `54b4fa230d`); this is the next layer. Rerun after the merge batch. |
| S3 | `IdentityIndexUnavailable` on principal resolution after reopen | 2 | HAPPY, Q1 | Expected cured by P-C `2dce9c178c`. |
| S4 | World readmission `UncleanRecoveryUnsupported { owner: Relational }`: `no_pending_owners/transition_*` (6) and `world_durable_release/*` (4) | 10 | the 4 `world_durable_release` courts are the integrate_pending seam "classify against D18 gate `d27064d826`"; the 6 transition courts need the same call | Classify after the merge batch, against D18. |
| S5 | `WorldSectionRequired` from the Home checkpoint (`support/durable_run/declarations.rs:111` x5, `residency_capture`, `world_occurrence_readmission.rs:349`) | 7 | UNCLASSIFIED (courts that drive a relational-only checkpoint through a World Home) | Read after the merge batch; HAPPY if a graceful World Home closes without its section. |
| S6 | Exact cost / residency counters and memory ceilings (`residency_agreement`, `residency_reopen` x2, `home_close.rs:178`, `request_controls.rs:45`, C8 recovery memory budget x2, `size_independence`) | 8 | C2 by Ruling 2 where the court asserts exact equality | Per-court read at the C2 move-out. |
| S7 | `ColdReconstructionRequired` publication after a failed checkpoint (`native_components/index_catalog.rs:267`) | 1 | HAPPY, Root F3 | With F3. |
| S8 | Version / retired-format refusal text (`retired_format.rs:103`, `native_artifacts.rs:147`, `idempotency.rs:144`) | 3 | CLEANUP (C1/C4) unless a refusal order regressed | With Root F5. |
| S9 | Phase-eight process courts: 4 TIMEOUT at 600 s and the C8 writer exiting with code 2 (`harness/declarations.rs:87`), killed-child anchor replay that did not park | 6 | C1 (crash track) | Recorded; not run again in track H. |
| S10 | Other single causes: `concurrent_mutation.rs:29` (`Any`), `workload/declarations.rs:280` (`StorageUnavailable SourceUnavailable`) | 2 | UNCLASSIFIED | Read after the merge batch. |

Counts are from the panic groups (`groups.py`); "~" marks families split across direct and child panics.
None of these is claimed fixed by `54b4fa230d`.
