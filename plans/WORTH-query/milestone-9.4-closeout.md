# Milestone 9.4 Closeout: Runtime-Backed Temporal, Async, Mixed-Cause, And Downstream Delivery Surface

## Status

Milestone 9.4 is closed as of 2026-06-08 for the runtime-backed temporal,
async/resource, mixed-cause, remask, continuation, and downstream-delivery
surface in `worth-query`.

This closeout covers:

- runtime-backed temporal query basis and time-only delivery on ordinary live
  Query surfaces
- runtime-backed async/resource declaration meaning and retained async
  result-state on the same live/state/inspection world
- runtime-backed mixed truth/time/async delivery ordering and downstream
  projection
- runtime-backed remask posture for policy, tenant, relationship-proof, and
  schema-context drift before public projection
- runtime-backed continuation/recovery posture for temporal basis drift, async
  request drift, replay drift, remask drift, stale completion, and preview
  crossed residue
- one runtime-backed downstream delivery contract for `worth-server` and other
  transport consumers, including admitted runtime resume and explicit durable
  resume debt
- runtime-backed hostile certification and reference workload closure for the
  merged temporal/async/mixed-cause surface
- crate-doc coverage for the shipped runtime-backed surface, including
  `AI_README.md`, support/admission guidance, downstream integration guidance,
  async result-state guidance, and workspace/state surface updates

This closeout does not claim:

- store-backed temporal replay
- durable inflight async restore
- restart-stable persisted continuation beyond the runtime-backed posture
- durable resume/replay for downstream delivery
- a sibling public runtime root such as `workspace.temporal(...)`,
  `workspace.async_resource(...)`, or `workspace.mixed_cause_delivery(...)`

Those remain later-milestone scope exactly as the Milestone 9.4 spec allowed.

## Governing Source Summary

- `MENTALITY.md`: the milestone had to close one hostile truth-preserving
  product surface rather than a pile of host-local timers, loading enums, and
  callback folklore.
- `arch_laws.md`: Query had to project lower authority honestly without
  reopening bridge-owned basis, causality, ordering, or resume questions.
- `composition_laws.md`: temporal declaration, async declaration, legality,
  readiness, runtime posture, remask, continuation, delivery, and downstream
  projection had to stay distinct responsibilities.
- `domain_structure_laws.md`: shipped work needed real homes for async
  result-state, mixed-cause delivery, runtime basis/state projection,
  downstream delivery, and continuation/recovery posture.
- `perf_laws.md`: runtime-backed time-only wakes, async completions, and mixed
  deliveries had to stay canonical and bounded rather than rescanning broad
  truth or degrading into host recomputation.
- [milestone-9.4.md](./milestone-9.4.md): the shipped runtime-backed surface
  now satisfies the merged milestone's public runtime, declaration, legality,
  runtime delivery, remask, continuation, downstream contract, and hostile
  certification closure bar, while leaving durable/store-backed work explicit
  debt.

## Adversarial Constraint Closed

Milestone 9.4 had to survive the hostile case where Query would:

- treat time-only wakes as fake relational patches
- collapse async lifecycle into UI-local `loading` folklore
- let host event arrival order redefine public mixed-cause meaning
- materialize then remask instead of remasking before public delivery/state
- make downstream consumers rediscover basis negotiation or resume posture at
  the transport boundary
- reopen bridge-owned temporal basis, async identity, completion causality,
  mixed-cause ordering, or runtime-backed resume questions

The closed runtime-backed surface now guarantees that:

- temporal and async declaration meaning lower through one canonical Query
  declaration path
- time-only delivery remains query-shaped and survives without a fabricated
  truth patch
- retained async result-state remains Query-owned and typed as `pending`,
  `current`, `failed`, `stale`, `cancelled`, `retried`, `revalidating`,
  `superseded`, or `denied`
- mixed truth/time/async public delivery meaning is projected from bridge-owned
  ordering rather than host callback order
- remask posture narrows or denies retained runtime meaning before public
  state, inspection, or downstream delivery projection
- continuation and recovery localize temporal drift, async drift, replay drift,
  remask drift, stale completion, and preview-crossed residue through typed
  Query surfaces
- downstream consumers inherit one Query-owned runtime-backed delivery contract
  with admitted runtime resume and explicit durable-resume debt

## Closure Summary

Milestone 9.4 closes as one runtime-backed product surface rather than four
separate roadmap leaves.

What now ships:

- runtime-backed temporal declaration support, legality, readiness, and
  time-only delivery semantics
- runtime-backed async/resource declaration support, legality, retained async
  result-state, completion-causality projection, and drift localization
- runtime-backed mixed-cause delivery projection and retained delivery-cause
  posture
- runtime-backed remask posture and downstream delivery support rows
- runtime-backed continuation/recovery classification for temporal and async
  drift lanes
- runtime-backed downstream delivery projection and resume negotiation for
  server/transport consumers
- runtime-backed hostile reference workload and public-doc coverage for the
  merged surface

What intentionally does not ship as part of this closeout:

- sibling facade-family runtime roots for `Temporal`, `AsyncResource`, or
  `MixedCauseDelivery`
- durable/store-backed replay or restart completion

That distinction matters. The separate facade-family rows remain visible and
support-gated because Query still refuses to create a second runtime world next
to ordinary live handles. The actual runtime-backed temporal/async/mixed-cause
product surface ships through the ordinary `workspace` / live / state /
inspection / downstream-delivery path.

## Public Surface Closed

The public runtime-backed closeout is now visible through:

- `workspace.state(...)` for compact runtime posture and support posture
- `workspace.inspect(...)` for retained delivery, async result-state, and
  remask evidence
- `workspace.downstream_delivery(...)` for one transport-safe delivery
  projection
- `workspace.public_downstream_delivery_contract()` for admitted runtime resume
  and explicit durable-resume debt
- runtime-backed docs:
  - [AI_README.md](../../crates/worth-query/docs/AI_README.md)
  - [workspace-overview.md](../../crates/worth-query/docs/foundations/workspace-overview.md)
  - [state.md](../../crates/worth-query/docs/foundations/state.md)
  - [support-matrix-and-admission.md](../../crates/worth-query/docs/foundations/support-matrix-and-admission.md)
  - [downstream-runtime-integration.md](../../crates/worth-query/docs/foundations/downstream-runtime-integration.md)
  - [async-resources-and-result-state.md](../../crates/worth-query/docs/capabilities/async-resources-and-result-state.md)

The separate facade-family support rows for `Temporal`, `AsyncResource`, and
`MixedCauseDelivery` remain intentionally deferred. That is not a contradiction
in the shipped surface; it is the protection against parallel sibling APIs that
the milestone spec required.

## Verification Summary

The closeout state is grounded in the runtime-backed support, delivery, and
closure suites, including:

- `cargo test -p worth-query runtime_public_downstream_delivery_contract_freezes_runtime_backed_and_durable_resume_posture`
- `cargo test -p worth-query runtime_public_support_matrix_exposes_downstream_delivery_contract_row`
- `cargo test -p worth-query runtime_backed_reference_workload_exercises_temporal_async_preview_causal_and_follow_on_lanes`
- `cargo test -p worth-query runtime_backed_closure_matrix_preserves_equivalent_and_distinct_public_meaning`
- `cargo test -p worth-query runtime_support_profiles_expose_facade_family_posture`
- `cargo test -p worth-query runtime_public_support_matrix_freezes_stable_deferred_and_unsupported_rows`

These tests prove the closeout distinction directly:

- runtime-backed temporal/async/mixed-cause meaning is shipped on ordinary
  runtime-backed surfaces
- downstream delivery and remask support rows are supported now
- durable/store-backed resume remains deferred debt
- separate facade-family roots remain support-gated instead of becoming
  parallel runtime entry points

## Residual Deferred Scope

The following are not part of Milestone 9.4 closeout:

- store-backed temporal replay and store-backed mixed-cause replay
- durable async inflight restore and persisted retry state
- durable continuation or replay resurrection after restart
- durable downstream resume/replay beyond runtime-backed basis negotiation
- any new sibling public runtime API root for temporal/async/mixed-cause work

Milestones 10 and 11 should inherit the runtime-backed semantic surface as
closed and extend it through store-backed and durable guarantees rather than
rediscovering temporal/async meaning there.

## Handoff To WORTH Server And Later Roadmap Work

`worth-server` now inherits:

- one Query-owned downstream delivery contract instead of raw retained-batch
  folklore
- one runtime-backed delivery-class vocabulary covering truth-patch,
  time-only, async-backed, and mixed-cause deliveries
- one admitted runtime-resume negotiation surface with explicit durable-resume
  debt
- one remask-aware public delivery contract that preserves denial and drift
  posture at the Query boundary
- one runtime-backed reference workload that already exercises the merged
  temporal/async/mixed-cause surface under preview, recovery, and follow-on
  pressure

Later roadmap work must not reopen these closed questions:

- whether runtime-backed temporal and async meaning belong in the ordinary
  Query world
- whether mixed-cause delivery is a Query-owned public projection
- whether remask must happen before public delivery/state/inspection
- whether downstream transport consumers get one typed Query-owned contract
  instead of rediscovering semantics at the network edge

That is the dependency handoff the WORTH Server roadmap can now consume.
