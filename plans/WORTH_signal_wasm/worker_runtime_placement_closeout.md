# Worker Runtime Placement Closeout Acceptance Map

> **Status:** Completed
>
> **Spec:** [worker_runtime_placement_plan.md](./worker_runtime_placement_plan.md)
>
> **Test requirements:** [worker_runtime_test_requirements.md](./worker_runtime_test_requirements.md)
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Prerequisite closeouts:**
>
> - [host_capability_closeout.md](./host_capability_closeout.md)
> - [api_surface_closeout.md](./api_surface_closeout.md)
> - [plans/worth_signal/milestone-d-closeout.md](../WORTH_signal/milestone-d-closeout.md)
> - [plans/worth_signal/milestone-11-closeout.md](../WORTH_signal/milestone-11-closeout.md)

## Purpose

This document maps the worker-runtime placement milestone to concrete
implementation, certification, hostile-test, and closeout evidence.

It is the closeout ledger for the hostile question:

> Can `worth-signals-wasm` recommend worker-first execution as the canonical
> product posture for runtime-owned graph work without moving semantic truth
> back to the main thread, hiding fallback, or pretending browser capabilities
> and live JavaScript closures are portable worker data?

## Closeout Summary

Milestone 9 is implemented as a worker-first runtime placement substrate for
`worth-signals-wasm`.

The implementation now includes:

- a worker runtime shell around the existing runtime truth, not a second graph
  engine
- typed worker graph publication for portable source/recipe definitions
- explicit worker-executable, main-thread-hosted, denied, and unavailable
  placement categories
- host-capability ingress, browser-history ingress, and host-effect egress as
  typed boundary lanes
- main-thread-hosted callback execution with closed request/result readmission
- callback-unavailability export/import evidence and explicit callback
  reattachment lanes
- committed observation and output delivery packets
- diagnostics summary/history reads with cold-work honesty
- lifecycle attach/detach control for worker-owned public delivery
- replay, restore, checkpoint, retained-history, import/export, and
  worker-unavailable capability certificates
- Phase 5, Phase 6, and final Phase 7 closeout packages
- named performance counters, complexity contracts, prohibited failure modes,
  and bridge allocation posture
- product guidance that recommends worker-first runtime-owned graphs while
  preserving explicit compatibility lanes
- final Suite 0 closeout with zero pending proof families and
  `milestoneClosed = true`

The direct closeout gate is:

- `SignalWorkerRuntime.certifyWorkerPhase7Closeout`

That package emits `workerPhase7CloseoutCertification` and top-level binds:

- Phase 5 delivery/diagnostics/lifecycle closeout
- Phase 6 replay/restore/import/export/worker-unavailable closeout
- Phase 7 proof-family certification
- performance counter catalogue
- complexity-contract catalogue
- prohibited failure-mode catalogue
- bridge allocation posture
- product guidance
- acceptance artifacts
- worker-first committed truth
- capability parity
- boundary performance

## Primary Implementation Surfaces

Worker runtime shell and bridge authority:

- [worker_runtime_shell.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_runtime_shell.rs)
- [worker_runtime_shell_branches.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_runtime_shell_branches.rs)
- [worker_host_boundary_causality.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_host_boundary_causality.rs)
- [worker_host_boundary_performance.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_host_boundary_performance.rs)
- [worker_graph_publication.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_graph_publication.rs)

Typed host and browser boundary lanes:

- [worker_host_capability_ingress.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_host_capability_ingress.rs)
- [worker_browser_history_ingress.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_browser_history_ingress.rs)
- [worker_host_effect_boundary.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_host_effect_boundary.rs)
- [main_thread_host_bridge_certification.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/main_thread_host_bridge_certification.rs)

Callback placement, hosted execution, and transport honesty:

- [worker_main_thread_hosted_callback_boundary.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_main_thread_hosted_callback_boundary.rs)
- [worker_main_thread_hosted_callback_validation.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_main_thread_hosted_callback_validation.rs)
- [worker_main_thread_hosted_callback_certification.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_main_thread_hosted_callback_certification.rs)
- [worker_callback_capability_transport.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_callback_capability_transport.rs)
- [worker_callback_definition_publication.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_callback_definition_publication.rs)
- [worker_callback_phase4_closeout_certification.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_callback_phase4_closeout_certification.rs)

Committed delivery and diagnostics:

- [worker_observation_delivery.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_observation_delivery.rs)
- [worker_output_delivery.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_output_delivery.rs)
- [worker_diagnostics_history_read.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_diagnostics_history_read.rs)
- [worker_lifecycle_control.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_lifecycle_control.rs)
- [worker_phase5_closeout_certification.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_phase5_closeout_certification.rs)

Replay, restore, import/export, and unavailable-worker capability evidence:

- [worker_replay_restore_capability.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_replay_restore_capability.rs)
- [worker_replay_checkpoint_retained_history.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_replay_checkpoint_retained_history.rs)
- [worker_import_export_callback_unavailability.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_import_export_callback_unavailability.rs)
- [worker_unavailable_compatibility_artifact.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_unavailable_compatibility_artifact.rs)
- [worker_phase6_closeout_certification.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_phase6_closeout_certification.rs)

Final Phase 7 certification:

- [worker_phase7_performance_contracts.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_phase7_performance_contracts.rs)
- [worker_phase7_performance_catalog.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_phase7_performance_catalog.rs)
- [worker_phase7_product_guidance.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_phase7_product_guidance.rs)
- [worker_phase7_test_requirements.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_phase7_test_requirements.rs)
- [worker_phase7_closeout_certification.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_phase7_closeout_certification.rs)

Wasm boundary surface:

- [worker.rs](../../crates/worth-signal-wasm/src/boundary/worker.rs)
- [worker_callback_reattachments.rs](../../crates/worth-signal-wasm/src/boundary/worker_callback_reattachments.rs)
- [worker_diagnostics_history_read.rs](../../crates/worth-signal-wasm/src/boundary/worker_diagnostics_history_read.rs)
- [worker_lifecycle_control.rs](../../crates/worth-signal-wasm/src/boundary/worker_lifecycle_control.rs)
- [worker_phase5_closeout.rs](../../crates/worth-signal-wasm/src/boundary/worker_phase5_closeout.rs)
- [worker_phase6_closeout.rs](../../crates/worth-signal-wasm/src/boundary/worker_phase6_closeout.rs)
- [worker_phase7_closeout.rs](../../crates/worth-signal-wasm/src/boundary/worker_phase7_closeout.rs)
- [worker_replay_restore_capability.rs](../../crates/worth-signal-wasm/src/boundary/worker_replay_restore_capability.rs)
- [diagnostics.rs](../../crates/worth-signal-wasm/src/boundary/diagnostics.rs)

## Must-Ship Acceptance Map

| Spec requirement | Implementation evidence | Certification / test evidence |
| --- | --- | --- |
| Worker-owned runtime posture without a second truth engine | `WorkerRuntimeShell`, compatibility truth probes, worker committed-truth digests | compatibility runtime tests and `SignalWorkerRuntime.certifyWorkerPhase7Closeout` |
| Typed placement taxonomy | placement classification and callback eligibility reports | placement tests, hosted-callback execution tests, compile-fail type-surface tests |
| Typed host bridge lanes | host-capability ingress, browser-history ingress, host-effect egress | worker-host-boundary runtime and boundary suites plus main-thread bridge certification |
| Callback portability honesty | callback unavailability export, portable import denial, explicit reattachment | callback capability transport, definition publication, and Phase 4 closeout tests |
| Committed public delivery | observation and output delivery packets with lifecycle ownership | Phase 5 delivery tests and `certifyWorkerPhase5Closeout` |
| Diagnostics cost honesty | summary reads remain summary-only; rich history is separately attributed | diagnostics history read tests and Phase 5 closeout |
| Replay/restore/import/export honesty | same-runtime restore, checkpoint retained history, import/export callback posture | replay restore, checkpoint retained-history, import/export, and Phase 6 closeout tests |
| Worker-unavailable compatibility honesty | no-worker compatibility artifact with explicit main-thread posture | worker-unavailable compatibility boundary/runtime tests and Phase 6 closeout |
| Named performance contracts | required counters, complexity contracts, failure modes, allocation posture | Phase 7 performance-contract tests |
| Product guidance | worker-first recommended posture and explicit compatibility lanes | Phase 7 product-guidance tests |
| Final test requirement closure | all 13 proof families closed with zero pending count | Phase 7 test-requirements tests |
| Final Suite 0 closure | one package binds Phase 5, Phase 6, and Phase 7 evidence | Phase 7 closeout runtime and boundary tests |

## Required Proof Families

All 13 required proof families from Section 15 of the spec are closed as
`ClosedByCanonicalCertification` in
`workerPhase7TestRequirementsCertification`:

- `The Worker Compatibility Truth Equivalence Test`
- `The Mixed Placement Graph Isolation Test`
- `The Host Capability Worker Bridge Parity Test`
- `The Browser History Worker Admission Parity Test`
- `The Main-Thread Host Effect Boundary Test`
- `The Callback Placement Eligibility And Denial Test`
- `The Worker Ineligible Node Does Not Collapse Graph Breadth Test`
- `The Observation And Output Delivery Boundary Test`
- `The Diagnostics Summary Cost Honesty Test`
- `The Worker Replay Restore Capability Honesty Test`
- `The Import Export Callback Unavailability Test`
- `The Worker Bridge Boundedness Test`
- `The UI Freeze Surface Denial Test`

The proof-family matrix lives in:

- [worker_phase7_test_requirements.rs](../../crates/worth-signal-wasm/src/runtime/worker_host/worker_phase7_test_requirements.rs)
- [phase7/test_requirements.rs](../../crates/worth-signal-wasm/src/runtime/tests/worker_runtime/phase7/test_requirements.rs)
- [phase7/test_requirements.rs](../../crates/worth-signal-wasm/src/boundary/tests/phase7/test_requirements.rs)

## Final Closeout Gate

The final closeout gate is:

- [phase7/closeout.rs](../../crates/worth-signal-wasm/src/runtime/tests/worker_runtime/phase7/closeout.rs)
- [phase7/closeout.rs](../../crates/worth-signal-wasm/src/boundary/tests/phase7/closeout.rs)

Those tests prove:

- Suite 0 reports `Suite0FinalCloseoutCertified`
- `milestoneClosed` is `true`
- all 13 proof families are covered
- `finalCloseoutPendingCount` is `0`
- Phase 5 evidence must be current
- Phase 6 evidence must be present
- hidden bridge allocation is rejected
- stale pending test-requirement rows are rejected
- the wasm boundary exposes the top-level Section 16 digests rather than
  dropping proof artifacts at serialization time

## Certification And QA

The milestone was closed only after:

- phase-by-phase implementation from placement through Phase 7
- QA loops over production closeout semantics
- a dedicated `$qa-tests` pass over the Phase 7 test surface
- runtime and boundary test topology cleanup to keep touched directories under
  the 10-direct-file cap
- adapter splitting to keep touched Rust files under the 400-line cap
- full crate verification after the final closeout and test-QA pass

The final Phase 7 test topology is:

- [runtime/tests/worker_runtime/phase7](../../crates/worth-signal-wasm/src/runtime/tests/worker_runtime/phase7)
- [boundary/tests/phase7](../../crates/worth-signal-wasm/src/boundary/tests/phase7)

## Verification At Closeout

Final verification commands:

```powershell
cargo fmt -p worth-signals-wasm --check
cargo test -p worth-signals-wasm phase7 -- --nocapture
cargo test -p worth-signals-wasm
git diff --check
```

Final result at closeout:

- `cargo fmt -p worth-signals-wasm --check` passed
- `cargo test -p worth-signals-wasm phase7 -- --nocapture` passed with `32`
  tests
- `cargo test -p worth-signals-wasm` passed with `208` unit tests, the
  compile-fail suite, and doc-tests
- `git diff --check` passed, with only existing CRLF normalization warnings

## Explicit Deferrals

The following are intentionally not part of this milestone closeout:

- general closure-source serialization and restoration across hosts
- arbitrary compiler transforms that infer worker-executable representations
  from normal callback source
- service-worker and shared-worker product surfaces
- cross-tab distributed runtime authority
- general DOM-capability virtualization for worker code
- making every callback-authored computed automatically worker-executable
  without an admitted lowering path

Those deferrals do not block this milestone. The closed scope is worker-first
deployment for runtime-owned graph work with explicit host, callback, fallback,
and unavailable-worker lanes.

## Residual Risk

No open milestone blocker remains at closeout.

The most sensitive future regression classes are:

- newly added host capability families weakening the explicit bridge taxonomy
- convenience APIs trying to treat live callbacks as portable worker data
- broad public delivery or serialization work reintroducing UI freeze risk
- compatibility mode becoming a hidden semantic authority instead of an
  explicit parity lane
- future replay/import/export surfaces preserving values while losing
  capability posture

Those classes are guarded by the final closeout package, the Phase 7
test-requirements matrix, the performance contract catalogue, and the hostile
runtime/boundary suites listed above.

## Closeout Decision

Milestone 9 is complete and can be treated as closed.

Worker-first runtime placement is now the honest default execution posture for
serious `worth-signals-wasm` applications: runtime-owned work lives off the UI
thread, host authority remains explicit, callback portability limits are
machine-visible, and compatibility mode is a parity lane rather than a hidden
fallback engine.
