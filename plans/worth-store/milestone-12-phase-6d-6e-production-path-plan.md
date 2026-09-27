# Worth Store Milestone 12 Phase 6D/6E Production Path Plan

## Purpose

Define the next implementation batch for Milestone 12 as a production-facing
store path, not a test seam.

This batch exists because two things are now true:

1. restore publication execution is real
2. compatibility-triggered derived rebuild can already reach the Milestone 11
   runtime through a focused internal execution helper

What is still missing is the actual shipped path:

- rolling compatibility does not yet constrain real writer publication
- compatibility-triggered derived rebuild is not yet reachable from a
  production store surface

This plan closes that gap while staying inside the domain laws.

## Domain-Law Constraints

The next batch must obey `docs/coding-guidelines/domain_laws.md`.

That means:

- do not grow `compatibility_runtime.rs` into a catch-all production workflow
  file
- keep rolling admission, rolling operational state, rebuild trigger lowering,
  and rebuild execution routing as separate responsibilities
- keep public access through the existing store facade instead of exposing deep
  compatibility internals
- add new modules only where they name real domain responsibilities, not
  generic mechanics

In practice, the production path should be decomposed into:

- compatibility production-path lowering
- rolling operational state and publication gates
- derived rebuild trigger lowering
- facade entry points and diagnostics

## Batch Goal

Ship the first real production-facing compatibility execution path for the
remaining M12 runtime gaps.

At the end of this batch:

- a real store write path can opt into rolling publication admission and fail
  typed when the declared rolling window does not allow publication
- a real store compatibility path can trigger derived rebuild through Milestone
  11 without going through a test-only helper
- the certification runner can honestly stop claiming derived rebuild execution
  is deferred only if the production path is wired and verified

This batch does not have to close all of rolling posture persistence yet, but
it must create the real publication gate and real production rebuild trigger.

## Scope

### In scope

- production-facing derived rebuild trigger path
- rolling writer publication gate on a real store surface
- typed diagnostics and counters from those paths
- focused runtime tests proving the facade path, not only direct helper calls

### Out of scope

- adapter execution
- full Milestone 14 replica consumption
- broad new artifact-family rebuild implementations beyond first-ship M12
  families already represented in compatibility
- speculative operator tooling

## Sequencing Decision

Do the next batch in this order:

1. promote derived rebuild from test helper to production-facing store path
2. add rolling writer publication gate on a production path
3. add restart-visible rolling posture only if needed to honestly satisfy the
   first production slice

Reason:

- derived rebuild already has the strongest execution footing because Milestone
  11 is real
- rolling publication still needs operational state decisions and is the riskier
  seam
- getting one production rebuild path into the facade removes the test-only
  shortcut immediately and gives us a clean pattern for rolling

## Phase 6E.1: Production-Facing Derived Rebuild Trigger

### Goal

Replace the current test-only compatibility rebuild helper with a real store
surface that lowers compatibility drift into Milestone 11 maintenance.

### Production path

Add a facade-owned operation that means:

`compatibility says this derived family is stale; lower and execute the required rebuild path through Milestone 11`

This should not expose raw compatibility planning internals.

The preferred shape is:

- facade method on `WorthStore`
- backend facade dispatch
- backend engine lowering into compatibility-owned rebuild requirement
- handoff into Milestone 11 maintenance admission/execution

### Required work

- extract the current rebuild lowering logic from
  `backend/engine/compatibility_runtime.rs` into a narrower compatibility
  production-path module or submodule
- define a production-facing request/result pair for compatibility-triggered
  rebuild execution
- route that request through `WorthStore` and `StoreBackend`
- keep compatibility responsible for:
  - manifest-backed artifact identity
  - read admission
  - derived lane planning
  - retained-authority requirement
  - maintenance-lane requirement
- keep Milestone 11 responsible for:
  - admission
  - queueing / status
  - execution container
  - restart readmission

### Shape constraints

- do not expose `DerivedRebuildRequirement` creation to external callers
- do not expose maintenance declaration fabrication to callers
- do not let external callers jump directly into compatibility engine modules
- the facade method should return a domain result, not internal witness objects

### First shipped family set

The first production path should support the derived families where M12 already
has meaningful rebuild semantics and M11 can honestly contain the work:

- `Milestone11MaintenanceRecord`
- `Milestone10RetentionRebuildRecord`

If one of those is not executable yet without fake semantics, narrow the batch
to the single family that can be shipped honestly and say so in tests/docs.

### Tests

- store-level production test: compatibility-triggered rebuild for
  `Milestone11MaintenanceRecord` executes through `WorthStore`
- store-level production test: resulting maintenance status is persisted and
  queryable
- hostile test: family without a rebuild requirement fails typed before
  maintenance admission
- hostile test: missing manifest or incompatible family still fails through the
  production path, not through raw helper calls

### Exit condition

- there is no need for a test-only facade helper to prove compatibility rebuild
  execution for the shipped family set

## Phase 6D.1: Rolling Writer Publication Gate

### Goal

Make a real writer-facing publication path consume rolling admission before the
store publishes inside a mixed-version window.

### Production path

Use a dedicated facade surface for rolling publication instead of smuggling
rolling behavior into ordinary append semantics.

Preferred first-ship surface:

- a store facade method that accepts:
  - family identity
  - rolling window
  - reader capability set(s)
  - writer capability set
  - publication operation intent
- the method performs rolling admission and then delegates to the existing write
  path only on admitted windows

This keeps the domain explicit and avoids pretending ordinary append magically
knows deployment posture.

### Required work

- create a rolling production-path module responsible only for:
  - rolling admission execution
  - operational posture materialization
  - publication gate result shaping
- add a production-facing facade API for rolling writer publication
- preserve relation and posture from the admitted rolling window through the
  operational result
- reject:
  - multi-writer windows
  - missing-edge windows
  - skew outside the declared window
  - adapter-required and rebuild-required rolling relations

### State decision

The first batch must decide whether rolling posture needs persistence now.

Use this rule:

- if the production path only proves admission-at-publication-time, persistence
  can stay for the next slice
- if the path claims restart-visible mixed-version store posture, persistence is
  required in this batch

Default:

- ship real rolling publication gating first
- defer restart-visible posture persistence to the follow-on slice unless the
  spec wording being claimed would otherwise be dishonest

### Tests

- production rolling publication test: admitted two-capability window succeeds
- production rolling publication test: missing-edge window fails typed
- production rolling publication test: multi-writer window fails typed
- production rolling publication test: selected relation is preserved in the
  result
- certification-oriented test: the production rolling path and current runner
  lane agree on admitted/rejected status for the same window

### Exit condition

- a real public store surface enforces rolling publication admission before the
  underlying write proceeds

## Module Plan

To obey the domain laws, prefer this decomposition instead of enlarging
`compatibility_runtime.rs`:

```text
crates/worth-store/src/backend/engine/
  compatibility_runtime.rs                 existing runtime helpers retained small
  compatibility_production/
    mod.rs
    derived_rebuild.rs                     production lowering for rebuild trigger
    rolling_publication.rs                 production rolling gate and result
    request_types.rs                       production-facing request/result shells
```

If `request_types.rs` is too small, fold it into the two responsibility files
above. Do not create a generic `helpers.rs`.

Facade wiring should stay in the normal facade files:

```text
crates/worth-store/src/backend/facade/
  support.rs or maintenance.rs             derived rebuild production entry
  publication.rs                           rolling publication production entry
crates/worth-store/src/facade/
  support.rs or maintenance.rs             user-facing store method
```

Choose the facade file by responsibility:

- derived rebuild path belongs with maintenance/support semantics
- rolling publication path belongs with publication semantics

## Counter And Diagnostics Requirements

This batch must not hide execution behind opaque booleans.

Add result/report fields or counters for:

- production derived rebuild trigger count
- production derived rebuild typed rejection count
- production rolling publication admission count
- production rolling publication rejection count
- maintenance declaration id / lane id in production rebuild result
- rolling relation and posture in production rolling result

If existing compatibility counters already cover a field exactly, reuse them.
Do not create duplicate near-synonyms.

## Certification Impact

Do not update the runner gap labels purely because a helper exists.

Only remove `derived_rebuild_execution_deferred` when:

- the production facade path exists
- focused production tests pass
- the runner or closeout evidence points at that shipped path honestly

Only remove `rolling_writer_publication_deferred` when:

- the rolling publication gate exists on a public store path
- focused production tests pass
- the relation/posture claimed by the runner is backed by the real path

## Verification

Minimum focused verification for this batch:

```text
cargo test -p worth-store compatibility_rebuild_execution --lib
cargo test -p worth-store compatibility_rolling_execution --lib
cargo test -p worth-store artifact_format_evolution --lib
cargo test -p worth-store --test phase_boundaries_compile_fail -- --test-threads=1
```

If the rolling test file does not exist yet, create it in this batch.

## Honest Completion Standard

This batch is complete when we can say all of the following without hedging:

- compatibility-triggered derived rebuild is reachable from a real store facade
  path
- that path lowers into Milestone 11 maintenance rather than a bespoke side
  loop
- rolling writer publication is blocked or admitted by a real public store path
- the implementation respects the domain laws by splitting rolling publication,
  rebuild lowering, and facade wiring into distinct responsibilities

If any of those remain test-only, helper-only, or collapsed into one giant
runtime file, the batch is not honestly done.
