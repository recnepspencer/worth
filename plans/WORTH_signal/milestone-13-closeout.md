# Milestone 13 Closeout: Locality-First Frontier Execution

> **Status:** Completed
>
> **Plan:** [milestone-13-plan.md](./milestone-13-plan.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Acceptance map:** [s9_16_acceptance_map.md](./s9_16_acceptance_map.md)
>
> **Successor:** [milestone-13.1-plan.md](./milestone-13.1-plan.md)

## Outcome

Milestone 13 is complete.

`worth-signal` now derives invalidation work one immediate producer hop at a
time from committed Milestone 12 output authority. The ordinary runtime no
longer walks or pre-marks a reachable descendant closure. A graph-owned,
non-authoritative reverse subscription index narrows direct candidates by
producer, aspect, and correlated partition/detail scope; authoritative edges,
snapshots, committed versions, causes, dependency revisions, readiness epochs,
and graph identity still decide admission.

Prepared, committed, admitted, resolved, lowered, ready, and executed work are
compiler-distinct. Only current ready work can execute, and public predicted
planning estimates remain separate from performed execution receipts and
realized counters.

This closes the locality portion of `S9.16.3` and the invalidation portion of
`S9.16.6`. Milestone 13.1 carries the sealed canonical work stream and measured
resource envelopes through Runtime Bridge and Query before Milestone 14 changes
execution placement. Neither successor inherits authority to replace semantic
work identity with an implementation-specific queue or shard.

## Phase Closure Ledger

| Phase | Closed authority and evidence |
| --- | --- |
| 1 - Boundary Inventory, Red Slopes, And Proof Contract Freeze | Froze the current routing, constructor, counter, restore, planner, and publication boundaries; established independent `Q/C/K/U/E/S/P` manifests, 24 realized counter rows, red locality slopes, and an actual-source compile-failure progression mutation. |
| 2 - Owner-Specific Proof Progression And Atomic Promotion | Added private owner-specific admitted, prepared, committed, resolved, lowered, ready, and executed forms; bound executable work to graph, target, dependency revision, readiness epoch, causal origin, and stage; kept atomic output publication as the only direct-invalidation promotion. |
| 3 - Direct-Hop Routing Cutover | Removed ordinary transitive closure walking and pre-marking; added the producer-local aspect/scope subscriber index; validated indexed candidates against authoritative edges; proved unknown scopes, destroy/rebuild, drift, disjoint fanout, and unchanged-output behavior. |
| 4 - Topology Lowering, Readiness, And Causal Deduplication | Lowered direct settlements into canonical stage-bound work; made pending predecessors, conditions, async state, and current bindings precede execution; deduplicated same-target/same-epoch work without losing causal origins. |
| 5 - Rewire, Restore, And Trust-Boundary Reconstitution | Rejected stale same-shaped work across rewires, topology epochs, readiness epochs, causal-origin changes, graph instances, and restores; reconstructed work from persistent M12 authority rather than restoring ready queues. |
| 6 - Realized Counters And Foundational Evidence Cutover | Separated predicted and realized types and consumers; attached Signal performed receipts only after execution; split hot execution from recovery/support disclosure; used `worth-foundational` for canonical case/report identity without moving Signal vocabulary into Foundational. |
| 7 - Scale Courtroom, Strategy Decision, And Closeout | Sealed all six financial locality families, exact work/cause/counter evidence, cost slopes, deterministic/optimized semantic equivalence, ordinary and scheduled scale lanes, documentation, WASM, and constitutional gates. |

## Financial Locality Courtroom

The sealed `FinancialFrontierLocalityCertificationRun` owns these six scenario
families:

- `sparse_book_fanout`
- `partitioned_curve_universe`
- `convergent_factor_batch`
- `dense_market_close`
- `portfolio_dependency_churn`
- `branch_restore_locality_replay`

Each case binds an immutable financial definition and exact mutation trace to
independent semantic necessity, query candidates, admitted causes, canonical
work, evaluator contacts, scope behavior, publication, and all 24 performed
counter rows. The oracle does not import runtime routing or scheduling
authority. Canonical identities additionally bind scale, policy, diagnostics
tier, execution posture, semantic-work identity, and performed receipt.

The scheduled courtroom exercised the declared `10^3`, `10^4`, and `10^5`
scale contracts, including irrelevant-fanout and semantic-density axes. The
retained `10^5` restore case is scheduled evidence rather than an ordinary
merge-gate cost.

The final performance audit removed four accidental superlinear or
graph-sized costs discovered by the scheduled courtroom. The independent
oracle now indexes declarations once per immutable topology snapshot instead
of rebuilding the whole declaration set for every produced delta; canonical
cause storage maintains occupied slots and output-commit references
incrementally instead of rescanning every cause set per mutation; sparse cause
compaction now remaps only allocated cause slots instead of scanning the whole
node arena; and batch topology cycle preflight shares one traversal over the
proposed topology.
Multi-target transactional reads also stage the union of their rollback
candidates once, closing the correctness hole exposed during that audit.

## Public And Successor Contract

Public facades expose `InvalidationPlanningEstimate` for predicted planning and
`SignalInvalidationExecutionReceipt` with
`SignalInvalidationRealizedCounters` for performed execution. Reachability-
shaped frontier constructors are not a public compatibility lane.

The strategy decision compares deterministic and optimized executions only
after each has independently matched its exact graph-bound canonical work. Its
cross-run comparison normalizes runtime-local graph identity while preserving
semantic target, revision, readiness, stage, and causal-origin content.

Milestone 13.1 may carry the sealed current ready-work stream across installed
Runtime Bridge and Query boundaries. Milestone 14 may then schedule or
parallelize it. Both must preserve canonical work identity, deterministic
publication, direct-hop admission, and the performed counter contract.

## Final Verification

The closeout source passed:

```text
cargo test -q -p worth-signal --lib
1220 passed; 0 failed; 26 ignored

cargo test -q -p worth-signal --features parallel --lib
1252 passed; 0 failed; 28 ignored

cargo test -q -p worth-signal --doc
3 passed; 0 failed

cargo test -q -p worth-signal --test milestone_13_compile_time
4 passed; 0 failed

cargo test --release -q -p worth-signal scheduled_run_seals_all_declared_scale_contracts --lib -- --ignored
1 passed; 0 failed; finished in 48.64s

cargo check -q -p worth-signal --target wasm32-unknown-unknown
cargo fmt --all -- --check
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
git diff --check
```

The explicitly scheduled scale courtroom
`scheduled_run_seals_all_declared_scale_contracts` passed its complete
`10^3/10^4/10^5` matrix in the release test profile in 48.64 seconds. The
isolated 100,000-output quarter-density case completed in 5.02 seconds in the
same profile. The robust tracked, staged, and untracked `worth-signal` source
packet contained 249 files, of which 241 were Rust; no Rust file exceeded the
constitutional 400-line limit. Composition scrutiny examined all 241 dirty
Rust files and reported zero scan errors; nonfatal
function-size and parameter-count advisories were reviewed rather than
converted into mechanical blockers.

The final frozen packet fingerprint is
`7ec5f7a291bd68bb8c236f8c142ec62dfecaf65394ea2b0aa5c05766bf106280`.
The digest uses the sorted union of tracked changes and
untracked files; for each existing path it hashes the path length, UTF-8 path,
raw-content length, and raw content. This freezes contents rather than only
the status-row names.

## Final Independent Review

Fresh Sol reviewer `m13_final_frozen_sol` returned **ACCEPT** against frozen
fingerprint
`7ec5f7a291bd68bb8c236f8c142ec62dfecaf65394ea2b0aa5c05766bf106280`.
The review independently verified that sparse cause compaction is bounded by
allocated cause slots rather than total graph nodes, dirty-consumer retirement
releases cause and output-commit authority before vacating the node, and
checkpoint capture/restore remains valid. It found no concrete current-tree
runtime defect or false Milestone 13 claim.

## Closeout Decision

Milestone 13 is complete. Its required execution, scale, portability,
documentation, constitutional evidence, and final independent review are
green. The frozen implementation is accepted for merge.
