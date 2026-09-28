# Milestone 13.1 Closeout: Cross-Runtime Granular Invalidation

> Status: complete; primary verification and fresh frozen-source Sol review
> accepted
>
> Governing spec: [milestone-13.1-plan.md](./milestone-13.1-plan.md)
>
> Prerequisite: [Milestone 13 closeout](./milestone-13-closeout.md)
>
> Successor: [Milestone 14](./milestone-14-plan.md)

## Outcome

Milestone 13.1 carries authoritative Relational aspect and locality changes
through installed Runtime Bridge correspondence, optional performed Signal
execution, Query-owned impact admission and maintenance, and governed
query-shaped publication.

The completed path preserves four separate authorities:

- Relational owns committed truth and changed semantic surfaces.
- Runtime Bridge owns installed correspondence, declared widening, and the
  direct-truth versus performed-Signal distinction.
- Signal owns runtime-local aspect slots, scoped invalidation, ready work, and
  performed execution evidence.
- Query owns dependency roles, result maintenance, sharing, disclosure,
  retained collection state, and consumer publication.

No raw aspect tuple, Bridge candidate, Signal receipt, or diagnostic count is a
substitute for the authority owned by the next runtime.

## Phase Ledger

### Phase 1 - Boundary Freeze And End-To-End Red Courtroom

The initial court froze the Relational, Bridge, Signal, and Query handoffs and
proved that direct Bridge truth and performed Signal consequences are distinct.
The six immutable world definitions and the independent necessity manifest were
established before production execution could certify them.

### Phase 2 - Signal Installed Scoped Invalidation Contract

Signal accepts installed scoped changes without importing application meaning.
`ProducerAspectKey` is a private, non-authoritative, immediate-producer-local
index key. Scope buckets keep unscoped, whole-partition, and exact
partition-plus-detail membership separate. Unknown partitions still reach
unscoped consumers without widening scoped siblings.

### Phase 3 - Runtime Bridge Exact Lowering And Performed Delivery

Runtime Bridge lowers authoritative Relational changes through the installed
correspondence registry. Exact targets, allocated Signal slots, declared
widening, direct truth deliveries, and performed Signal deliveries retain
separate typed identities and counters. Destroyed derived indexes rebuild from
authoritative registrations, including mixed Exact and Allocate targets.

### Phase 4 - Query Installed Invalidation Manifest And Admission

Query compiles installed dependency roles and selects impacts from atomically
correlated semantic change plus record locality. Whole-aspect, field, widening,
structural, and lifecycle changes use their effective breadth. Cross-record and
aspect/scope Cartesian products are denied.

### Phase 5 - Production Maintenance, Sharing, And Facade Cutover

The ordinary public path performs Query-owned projection, membership, grouping,
ordering, and bounded-window maintenance. Retained per-owner state advances
only after successful publication, including projection-only off-window facts.
Primary and shared owners revalidate current source basis, lease, purpose, and
disclosure before effects. The coarse descriptive compatibility lane is not an
operational authority path.

### Phase 6 - Rebind, Restore, Branch, And Lifecycle Closure

Restored Signal graphs receive fresh runtime identity. Runtime Bridge rebuilds
semantic, aspect, and target-aware allocation indexes before reinstallation.
Query rebinds its primary source and projection owners. Delayed predecessor
batches, stale bindings, foreign snapshots, revoked leases, and old
installations cannot progress.

### Phase 7 - Structural Slopes, Certification, Documentation, And Handoff

Six production worlds now emit evidence observed from their actual runtime
owners. An independent manifest derives exact `R/B/S/I/M/D/X` identity sets.
The Signal set binds both the performed lower-runtime dependency and the exact
Query-installed target mapping and partition; substituting the opaque Query
installation into the curve world is rejected even when the surrounding
execution remains valid. The Relational set is built from the record identity,
aspect, and field retained by the delivered Bridge change rather than copying
the declared partition or detail; a forged expected record identity is rejected
at the owner-evidence comparison. `worth-proof` gates the verified transition
into a claim; `worth-foundational` canonicalizes every case and the sorted
six-case report. Input order is irrelevant, while any changed scenario, seed,
policy, tier, runtime generation, direct/derived count, or owner identity is
rejected.

The application-facing guide is
[Granular Live Invalidation](../../workspaces/worth-query/crates/worth-query/docs/runtime-surfaces/granular-live-invalidation.md).

## Production Certification Worlds

The sealed run executes these real composition-root scenarios:

1. `curve_detail_to_live_risk`
2. `suppressed_quote_no_query_patch`
3. `ordered_portfolio_membership`
4. `shared_lease_disclosure_noninterference`
5. `correspondence_rebind_restore`
6. `opaque_region_platform_twin`

The opaque twin uses non-financial Signal target and partition vocabulary; the
WASM scoped-invalidation court separately proves exact opaque
partition-plus-detail behavior without host-only interpretation.

## Structural Slopes

Seven production-owner slopes prove that unrelated growth does not widen the
wrong boundary:

1. unrelated Runtime Bridge mappings
2. unrelated retained result rows
3. unrelated Signal subscribers
4. unrelated installed Query dependencies
5. returned but rejected Bridge candidates
6. additional shared consumers
7. genuine Query frontier expansion from value-only to ordering/window work

Each slope asserts the expected owner counter changes and the counters that
must remain constant or zero.

## Verification Evidence

Primary verification on the final implementation candidate includes:

```text
cargo test -q -p worth-signal --lib
  1224 passed; 26 ignored

cargo test -q -p worth-signal --features parallel --lib
  1256 passed; 28 ignored

cargo test -q -p worth-runtime-bridge --lib
  965 passed

cargo test --manifest-path workspaces/worth-query/Cargo.toml \
  -p worth-query-certification --test granular_invalidation -- --test-threads=1
  20 passed

cargo test -q --manifest-path workspaces/worth-query/Cargo.toml --workspace
  passed; largest lanes: 2593, 727, 323, 193, and 148 tests

cargo test -q -p worth-signal-wasm --test installed_scoped_invalidation
  1 passed

cargo check -q -p worth-signal-wasm --target wasm32-unknown-unknown
  passed

cargo test -q -p worth-signal --test milestone_12_compile_time \
  --test milestone_13_compile_time
  passed

cargo test -q -p worth-signal --doc
  3 passed

cargo test -q --manifest-path workspaces/worth-query/Cargo.toml --workspace --doc
  passed

cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
  Road 1 Cargo topology is valid

cargo run --manifest-path tools/agent-context/Cargo.toml -- check
  passed

python scripts/ci/check_composition_advisories.py dirty --json
  295 Rust files; 0 hard violations

python scripts/quality/scrutinize_rust_functions.py --dirty .
  295 Rust files; 124 advisory candidates reviewed; 0 scan errors

cargo fmt --all -- --check
git diff --check
  passed
```

The full Query workspace found and corrected one stale pre-M13.1 assertion: a
live projection refresh after inserting a real row now returns that row rather
than retaining the synthetic bootstrap anchor. The corrected full workspace run
is green.

## Frozen Review Gate

The primary candidate freeze is:

```text
HEAD: 06c610fd2afb49091ada67a178075d611a86021c
status rows: 348
content-union paths: 340
existing dirty Rust files: 295
dirty Rust files over 400 lines: 0
reviewed content paths: 339
content fingerprint: eb733318a992fabd4eb1a17976098f450ee54a34a966e3c1dcd3065153aac5c7
```

The fingerprint hashes the LF-joined, path-sorted rows
`path<TAB>SHA256(raw file bytes)` with no trailing newline. It covers every
content-union path except this self-referential closeout record. A fresh Sol
reviewer must independently reproduce the freeze and append identity, scope,
and verdict before this document claims independent final-source acceptance.

### Fresh Final Critic

- Reviewer: `m131_phase7_mutation_sensitive_final_sol` (Sol)
- Scope: full frozen M13.1 source and evidence, with focused adversarial review
  of owner-derived `R/B/S/I/M/D/X`, the curve-to-opaque Signal substitution,
  the forged Relational-record execution, all six production worlds, seven
  slopes, lifecycle/currentness, Foundational/Proof sealing, facades,
  composition, and the Milestone 14 handoff
- Verdict: `ACCEPT`
- Freeze: 339 reviewed paths at
  `eb733318a992fabd4eb1a17976098f450ee54a34a966e3c1dcd3065153aac5c7`

## Residual Boundary And Milestone 14 Handoff

Milestone 13.1 establishes semantic granularity and owner-local performed work;
it does not assign physical shards, regions, workers, queues, or leases.
Milestone 14 inherits:

- Signal's canonical ready-work stream and performed receipt
- Runtime Bridge's installed scoped lowering and direct/performed delivery
- Query's exact impact and retained maintenance state
- owner-separated counters and seven cross-runtime slopes
- the unscoped, whole-partition, and exact-detail platform base case

Milestone 14 may derive non-authoritative placement and execute independent
ready work in parallel. It may not move semantic authority into shard keys,
copy aspects transitively, weaken consumer disclosure checks, or reinterpret
Runtime Bridge correspondence.
