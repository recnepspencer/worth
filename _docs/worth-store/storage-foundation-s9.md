# Storage Foundation S.9: Checked Physical Protocols

## Goal

Define finite executable models for the highest-risk physical protocols and
check them directly with a pinned formal toolchain. Connect selected concrete
owner outcomes to model actions with focused exhaustive Rust mappings.

Formal results are diagnostics about protocol law. They never authorize Store
operations.

## Ownership

- Production owner crates define legal runtime operations, outcomes, denials,
  and durable facts.
- `worth-store-formal-models` owns finite model state, actions, invariants,
  checker invocation, typed verdicts, and parsed counterexamples.
- Focused owner tests invoke real owner behavior and map the returned typed
  outcome to one model action.
- Certification and test-support crates do not mediate the mapping or checker
  verdict.

## Required Protocol Families

- WAL, checkpoint, page-flush, and publication ordering;
- recovery source precedence;
- LSM membership, maintenance, compaction cutover, and visibility;
- hazard leases, reclaim barriers, and generation reuse;
- quarantine, verification, and current-scope readmission;
- import admission and durable publication;
- operational recovery, bootstrap, promotion, and rejoin;
- replication admission when concrete production outcomes exist.

Missing production behavior is a typed capability gap and blocks that model
family. The model must not invent fictional production states.

## Model Contract

Each family declares:

- a finite collapsed state space;
- legal actions and invariants;
- backend, durability, atomicity, I/O, and clock assumptions;
- explicit safety claims and explicit liveness non-claims;
- bounded checker configuration;
- a typed protocol-family identity used only to select the checked artifact.

The checker returns:

- `Checked` when all reachable states within the declared bounds satisfy the
  invariants;
- `Counterexample` with ordered states, actions, and valuations;
- `BoundExhausted` or another typed inconclusive outcome when the configured
  proof bound is insufficient;
- `RunnerFailure` when invocation, timeout, output, or tool execution fails.

Only `Checked` is a successful bounded check. No verdict becomes runtime
authority.

## Owner Mapping Contract

Mappings are family-local exhaustive functions over concrete owner outcomes.
They return the corresponding model action directly. They do not return a
case wrapper, receipt, digest, manifest row, or coverage declaration.

Use an owner mapping only when it compares meaningfully distinct
implementations. Model actions without a concrete owner are not covered by a
registry; they remain visibly unimplemented in the relevant phase or test.

## Phase Plan

1. Pin the formal toolchain and implement bounded direct invocation.
2. Define typed checker verdicts and parsed transient counterexamples.
3. Model durability and recovery-source precedence.
4. Model compaction membership, cutover, and visibility.
5. Model reclaim and generation safety.
6. Model quarantine and readmission.
7. Model import and replication only where production outcomes exist.
8. Model operational recovery workflows.
9. Add focused exhaustive owner-to-action tests.
10. Run every retained model directly in CI.

## Required Direct Tests

- direct checker execution for every retained family;
- legal traces reach the expected final model state;
- illegal transitions return the exact counterexample;
- nonzero, timed-out, truncated, malformed, or bound-exhausted checker runs
  cannot claim success;
- backend assumptions are derived from real admitted backend profiles;
- concrete compaction, membership, maintenance, recovery, quarantine, and
  publication outcomes map to exact model actions;
- mapping tests compare exact action sets where exhaustiveness is claimed;
- fresh process or physical media tests remain responsible for facts that a
  finite model cannot establish.

## Prohibited Machinery

Do not add binding manifests, owner-gap registries, generated action coverage,
refinement receipts, transition-count digests, checker artifact fingerprints,
counterexample localization databases, controlled-defect catalogs, mutation
summaries, or certification adjudication. Do not hash model/config/source files
to create a second version-control system.

The formal tool download may verify its published checksum for supply-chain
integrity. That checksum is not part of a protocol verdict.

## Completion Rule

At the reviewed revision, every retained model runs through the direct pinned
checker, focused owner mappings pass, backend assumptions are explicit,
counterexamples remain actionable, and boundary checks prove formal code opens
no runtime authority. Current validity is the current checker and test result,
not a stored phase status.
