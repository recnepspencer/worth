# Storage Foundation S.5: Physical Isolation, Latches, Epochs, And Stable Reads

## Goal

Make online physical reads stable while roots, manifests, segments, extents,
pages, chunks, checkpoints, compaction outputs, and reclaim state change around
them. S.5 owns physical isolation only; semantic MVCC visibility remains with
the semantic storage owners.

## Product Decisions

- Physical read stability is explicit and typed. It is not inferred from a
  semantic snapshot or from a successful decode.
- Root, manifest, segment, extent, page, and chunk lifetimes use named epochs
  and reachability rules.
- Latch classes and acquisition order are compiler-visible vocabulary.
- A stable read plan is admitted before execution and carries every authority,
  epoch, scope, capability, and budget decision needed by the executor.
- Executors follow the plan. They do not reacquire policy, widen scope, choose
  a different root, or rediscover topology.
- Copy-on-write publication makes new physical state visible atomically while
  old readers retain valid reachability.
- Compaction and checkpoint publication interlock with reads through explicit
  epochs and publication boundaries.
- Reclaim is legal only after reachability and hazard barriers prove the old
  object is no longer observable.
- Tier movement and blob migration expose stability hooks here without
  claiming the later S.7 policy.

## Authority And Dependency Boundaries

- Concrete Store authority types come from `worth-proof`; generic marker
  bounds open no governed operation.
- Cross-runtime identities and immutable boundary facts belong in
  `worth-foundational` only when they retain the same meaning elsewhere.
- Live epoch tables, latch state, hazard tracking, counters, and `Drop`-based
  guards belong in the owning runtime.
- Ordinary read paths do not import replay or reconstruction crates.
- Certification code observes public or dedicated test-facing boundaries; it
  does not mutate private runtime state.

## S.4.5 Harness Consumption

S.5 consumes the simulation harness directly:

- define S.5-owned scenario families for stable reads, copy-on-write root
  swaps, read-during-compaction, checkpoint publication, reclaim barriers, and
  security-scope failures;
- lower each scenario through the ordinary physical simulation planner;
- execute the returned plan and schedule through real S.5 boundaries;
- assert the transcript, oracle verdicts, and execution counters directly;
- compare repeated or deliberately changed schedules directly.

No readiness object, registration protocol, coverage registry, or second
certification facade sits between S.5 and the harness.

## Core Contracts

### Physical epochs

Every physically replaceable object has an owner, generation, and epoch. Epoch
comparison is meaningful only within the bound Store/runtime identity. Raw
integers and cross-Store epochs are rejected.

### Latches

Latch acquisition follows a declared partial order. Blocking, retry, and
deadlock behavior are explicit. A latch guard authorizes only the protected
physical operation and cannot become semantic or recovery authority.

### Stable read plans

A stable plan binds the selected root, reachable objects, security scope,
epoch guards, backend capabilities, and resource limits. Plan admission fails
before I/O when any binding is stale, ambiguous, unsupported, or out of scope.

### Publication and reclaim

Publication installs a fully prepared new root. Existing readers continue on
their admitted root. Reclaim consumes an object-specific eligibility proof;
age, filename, batch membership, or a generic success signal is insufficient.

## Phase Plan

1. Admit the typed recovery-to-physical-isolation entry boundary.
2. Separate physical stability from semantic visibility.
3. Define object epochs and generation bindings.
4. Define latch classes, acquisition order, and deadlock policy.
5. Admit stable physical read plans.
6. Execute stable reads without policy re-decision.
7. Publish copy-on-write physical updates.
8. Interlock reads with compaction.
9. Interlock reads with checkpoint publication.
10. Enforce reachability and hazard barriers before reclaim.
11. Reserve tier-movement and blob-migration stability hooks.
12. Exercise S.5 interleavings through the S.4.5 harness.
13. Preserve cross-boundary identities and proof progression where required.
14. Expose the typed S.6 I/O/QoS handoff.
15. Run direct hostile scenarios and owner tests for the completed surface.

Phase numbering is architectural sequence, not a live status database. Git
records the reviewed historical revision. Current validity is computed by the
current build, tests, and boundary checks.

## Required Direct Tests

- old readers remain on the old root during a copy-on-write swap;
- new readers bind only to the published root;
- compaction cannot reclaim an object reachable by an admitted reader;
- checkpoint publication cannot expose a partial root or mixed generation;
- stale, foreign, wrong-scope, or forged epoch and authority inputs are denied;
- latch-order violations are detected without relying on timing luck;
- plan identity changes when any authority-relevant binding changes;
- execution counters reflect actual reads, retries, waits, publications, and
  reclaim decisions;
- deterministic interleavings replay identically, while mutated schedules
  exercise distinct behavior;
- fresh process or real storage boundaries are used where process memory would
  otherwise make the claim dishonest.

## Prohibited Machinery

Do not add requirement ledgers, generated coverage rows, mutation catalogs,
readiness receipts, lane-registration protocols, materialized certification
packages, reviewer-result types, or tests that only prove a harness report was
assembled. Do not preserve obsolete facades as aliases.

Behavioral gaps receive direct owner or integration tests. Structural gaps
receive compiler-visible types, compile-fail tests, or boundary-check rules.

## Completion Rule

At the reviewed revision, focused epoch/latch/read/publication/reclaim tests,
S.5 simulation scenarios, affected integration tests, boundary checks,
formatting, and line-cap checks must pass. Later work evaluates current
validity from current code; historical phase prose is never manually reopened.
