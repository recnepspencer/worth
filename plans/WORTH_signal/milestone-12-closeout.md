# Milestone 12 Closeout: Aspect-Causal Invalidation

> **Status:** Completed
>
> **Plan:** [milestone-12-plan.md](./milestone-12-plan.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Acceptance map:** [s9_16_acceptance_map.md](./s9_16_acceptance_map.md)
>
> **Successor:** [milestone-13-plan.md](./milestone-13-plan.md)

## Outcome

Milestone 12 is complete.

`worth-signal` now treats a root mutation as unresolved recompute work. A
descendant receives aspect, scope, version, and dependency-revision authority
only from the atomically committed output delta of its immediate producer.
Reachability, a root-local aspect slot, a dirty cache, a diagnostic trace, or a
consumer comparator cannot mint that authority.

This closes the correctness portion of `S9.16.3`. Milestone 13 owns the
separate locality question: bounding invalidation breadth by realized semantic
reach rather than by reachable subscriber closure.

## Inherited Red Control

The inherited implementation at baseline revision `6b51e9c77` copied one root
aspect through the transitive frontier. The decisive control used a source that
changed `PRICE`, an intermediate computation that consumed `PRICE` and
produced `RISK`, and matched and unmatched `RISK` consumers. The inherited
leaf received the source's `PRICE` slot and could be incorrectly deferred by a
`RISK` filter even after the intermediate producer had changed `RISK`.

The exact historical test patch, checkout command, test command, and captured
failure are retained in
[milestone-12-phase1-red-control.md](./evidence/milestone-12-phase1-red-control.md).
The control was rerun against a detached checkout of `6b51e9c77`: its matched
leaf evaluator ran once instead of twice, while the current named financial
translation scenario passes.

The repaired runtime removes aspect and scope claims from structural
transitive summaries. Only an immediate committed producer delta can create a
canonical downstream dependency cause.

## Phase Closure Ledger

| Phase | Closed authority and evidence |
| --- | --- |
| 1 - Authentic Financial Courtroom And Red Control | Replaced parallel financial meaning owners with immutable `FinancialWorldDefinition` inputs and world-owned projections; added causally complete baselines, `FreshFinancialRecompute`, `FinancialNecessityManifest`, reproduction metadata, and the reproducible inherited `PRICE -> RISK` red control. |
| 2 - Output Contract And Owner-Specific Proof Forms | Separated producer output equivalence from consumer dependency comparison; introduced owner-specific source, commit, dependency-cause, scope, revision, and performed-publication proof forms; added drift and forgery denials for every binding axis. |
| 3 - Canonical Cause Storage And Recovery Basis | Established one graph-owned canonical pending-cause lifecycle with exact admission, release, checkpoint quarantine and readmission, rollback, compaction, direct-source basis, correlated aspect/scope identity, and dependency-revision validation. |
| 4 - Atomic Output Commit And Direct Cause Admission | Made output identity, version, snapshots, committed delta, artifact publication, and downstream causes one atomic packet; proved failure seams, exact and tolerance policies, heterogeneous consumers, and serial/parallel publication equivalence. |
| 5 - Planner, Condition, Scope, And Rewire Cutover | Cut ordinary, installed, temporal, custom, on-demand, async, partition, and topology paths over to pending-first immediate-dependency causality; proved correlated scope unions and same-shaped rewires under a new revision. |
| 6 - Branch Composition, Financial Certification, And Documentation | Proved exact causes and financial truth across branch, checkpoint, restore, replay, diagnostic-tier, async-capability, rollback, and WASM lanes; added canonical Foundational case/report identities and sealed the eight-scenario certification run. |

## Financial Courtroom

The sealed `FinancialAspectCausalityCertificationRun` owns these eight named
scenario verdicts:

- `quote_to_risk_aspect_translation`
- `heterogeneous_consumer_comparators`
- `tolerance_suppressed_repricing`
- `producer_local_factor_slot_collision`
- `partitioned_curve_bucket_bump`
- `gated_repricing_release`
- `instrument_dependency_rewire`
- `branch_shock_restore_replay`

Each verdict binds the executed compiled world to independent fresh financial
truth and mutation-sensitive necessity evidence. The sealed run rejects
missing or duplicate scenarios, mixed seeds, stale or wrong nonzero dependency
revisions, wrong scenario or policy identity, producer-policy drift, wrong
diagnostics tier, reproduction drift, oracle/work disagreement, and incomplete
lifecycle evidence. Canonical case and report identities use
`worth-foundational`; financial meaning and runtime authority remain owned by
their existing crates.

## Documentation And Public Contract

The final developer-facing contract is recorded in:

- [worth-signal README](../../crates/worth-signal/README.md)
- [conditions and comparators](../../crates/worth-signal/docs/reference/conditions-and-comparators.md)
- [fintech test world](../../crates/worth-signal/src/tests/domains/fintech/README.md)
- [signal architecture](./signal_architecture2.md)
- [acceptance map](./s9_16_acceptance_map.md)

The README example is executable documentation and passes the crate doctest
lane.

## Final Verification

The closeout tree passed:

```text
cargo test -q -p worth-signal --lib
1108 passed; 0 failed; 23 ignored

cargo test -q -p worth-signal --features parallel --lib
1138 passed; 0 failed; 25 ignored

cargo test -q -p worth-signal --doc
3 passed; 0 failed

cargo check -q -p worth-signal --target wasm32-unknown-unknown
cargo fmt --all -- --check
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
git diff --check
```

The robust tracked, staged, and untracked dirty-Rust inventory contained 231
files and no file above the constitutional 400-line limit. Composition
scrutiny reported no hard error. Nonfatal function-size and parameter-count
advisories were reviewed rather than treated as mechanical blockers.

The frozen `crates/worth-signal` source packet contains 236 dirty source files
and has path-and-content SHA-256 fingerprint
`3E70C65C14874FE0A24ECA01C738FBC9472A07E85C05B853F85CD0116A9D7E9B`.

Fresh final-source review was performed by the independent GPT-5.6 Sol high
critic `topology_closure_sol` against that exact fingerprint. Its scope covered
the single fintech truth owner, direct-source versus dependency-cause
invariants, structural revalidation, all six phase authorities, the
eight-scenario sealed run, the historical red control, parallel execution, and
the Milestone 13 boundary. It independently reproduced 236 source files, 231
Rust files, and zero files over 400 lines; ran the fintech invalidation suite
(`9/9`), certification suite (`7/7`), and parallel certification suite (`7/7`);
and returned `ACCEPT` with no supported P1, P2, or closure defect.

## Successor Boundary

Milestone 12 does not claim that the runtime already avoids walking every
semantically irrelevant reachable descendant. It proves that structural
reachability cannot fabricate descendant aspect or scope truth and that every
actual downstream cause is immediate, committed, exact, and recoverable.

Milestone 13 must reuse those causes while adding canonical work items, ready
batches, realized breadth counters, sparse and dense financial workloads, and
scale-sensitive locality certification. It must not reopen a copied-aspect or
copied-scope compatibility lane.

## Closeout Decision

Milestone 12 is implemented, documented, independently reviewed, and closed.
