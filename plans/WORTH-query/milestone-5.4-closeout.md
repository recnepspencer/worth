# Milestone 5.4 Closeout: Structural Correspondence And Historical Evaluation Contracts

## Status

Milestone 5.4 is closed as of 2026-04-17 for the runtime-backed structural
correspondence and historical materialization-path honesty scope.

`worth-query` now treats lineage-backed continuity, advisory structural
correspondence, explicit ambiguity, explicit lineage-versus-structural
disagreement, historical path admission, and historical materialization-path
identity as crate-owned query artifacts rather than bridge folklore, executor
reconstruction, or host-side interpretation. Requested, admitted, and resolved
historical path classes, parity bundles, failure digests, counter snapshots,
compile-fail boundaries, and milestone-native certification are now explicit,
typed, replay-safe, and closeout-proven surfaces.

The semantic center shipped in this milestone is:

the same canonical query meaning can now report whether sameness was proved by
lineage or only observed structurally, and can now report whether historical
truth was served by retained snapshot, delta replay, or full reconstruction,
without silently collapsing ambiguity, disagreement, or path substitution into
one generic success story.

## Shipped Scope

Milestone 5.4 delivered:

- explicit correspondence outcome families, historical result envelopes, and
  composition boundaries in
  [crates/worth-query/src/correspondence_history](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence_history)
- replay-safe correspondence/history parity bundles, digest lowering, and
  denial carriers in
  [crates/worth-query/src/correspondence_history_parity](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence_history_parity)
- structural and lineage-backed correspondence surfaces plus historical-path
  admission/resolution counters in
  [crates/worth-query/src/correspondence](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence)
  and
  [crates/worth-query/src/historical](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/historical)
- milestone-native certification artifacts, row catalogs, fixture lanes,
  rejection builders, and closeout mapping in
  [crates/worth-query/src/harness/correspondence_history_certification](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/correspondence_history_certification)
- compile-fail proof boundaries for structural-authority WORTHry, raw ambiguity
  collapse, naked historical payload access, and related privacy constraints in
  [crates/worth-query/tests/ui](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/tests/ui)

## Acceptance Mapping

Milestone 5.4 is considered closed against
[milestone-5.4.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/milestone-5.4.md),
[worth_query_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/worth_query_roadmap.md),
and
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-query/test-requirements.md)
because the required correspondence and historical-path proof surfaces now
exist directly.

### `Structural Correspondence And Historical Materialization Path Test`

Covered by:

- [mod.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/correspondence_history_certification/mod.rs)
- [model.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/correspondence_history_certification/model.rs)
- [tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/harness/correspondence_history_certification/tests.rs)

What is proven:

- the named certification artifact exists as a first-class Milestone 5.4
  closeout surface
- required canonical rows are present:
  - `lineage-correspondence-authoritative`
  - `structural-correspondence-advisory`
  - `lineage-structural-disagreement-explicit`
  - `structural-ambiguity-explicit`
  - `historical-retained-snapshot-path`
  - `historical-delta-replay-path`
  - `historical-full-reconstruction-path`
  - `historical-path-no-substitution`
  - `correspondence-cost-posture-parity`
  - `historical-cost-posture-parity`
  - `prediction-drift-explicit`
  - `work-avoided-counter-parity`
- required rejection rows are present:
  - `structural-as-authoritative-forbidden`
  - `ambiguous-correspondence-not-collapsed`
  - `unsupported-correspondence-family`
  - `unsupported-historical-materialization-path`
  - `hidden-materialization-path-substitution-forbidden`
  - `broad-candidate-scan-success-forbidden`
  - `no-executor-path-mutation-after-planning`
  - `host-cache-history-authority-forbidden`
  - `raw-ambiguity-bool-forbidden`
  - `naked-historical-payload-forbidden`
- the certification matrix now asserts exact row coverage, uniqueness, hostile
  boundary distinctions, exact compile-fail fixture bindings, determinism, and
  tamper-sensitive artifact digests rather than only proving row presence
- admitted lanes emit production-owned parity bundles with zero rediscovery
  instead of harness-local synthetic artifacts

### `Advisory-versus-authoritative and path-identity honesty`

Covered by:

- [crates/worth-query/src/correspondence_history/success.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence_history/success.rs)
- [crates/worth-query/src/correspondence_history/denied.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence_history/denied.rs)
- [crates/worth-query/src/correspondence_history/view.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence_history/view.rs)
- [crates/worth-query/src/correspondence_history/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence_history/tests.rs)

What is proven:

- lineage continuity, structural ambiguity, and typed denials stay distinct
  envelope families
- admitted results preserve payload and historical materialization metadata
  together rather than exposing one naked payload lane
- denied historical lanes do not expose a successful result view
- compatibility outcome remains typed and explicit on denial paths

### `Replay-safe parity, denial shape, and counter truth`

Covered by:

- [crates/worth-query/src/correspondence_history_parity/lowering.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence_history_parity/lowering.rs)
- [crates/worth-query/src/correspondence_history_parity/digests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence_history_parity/digests.rs)
- [crates/worth-query/src/correspondence_history_parity/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/correspondence_history_parity/tests.rs)
- [crates/worth-query/src/historical/tests.rs](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-query/src/historical/tests.rs)

What is proven:

- ambiguity and disagreement parity bundles remain semantically distinct
- retained-snapshot and delta-replay lanes bind different path and cost-posture
  digests
- historical denial bundles carry exact failure digests and compatibility
  outcomes
- correspondence-denied bundles cannot accidentally expose historical admitted
  or resolved path digests
- `history_work_avoided_by_retained_path_count` is production-owned and only
  increments when retained-path reuse is actually proved by capability

## Hardening Beyond Minimum Scope

Milestone 5.4 closeout also includes hardening work beyond the bare spec:

- the original mega-modules for correspondence-history envelopes, parity, and
  certification were decomposed by responsibility before closeout so later
  milestones do not inherit one giant cross-domain editing surface
- certification fixture code was split again into lane assembly, scenario
  construction, compile-fail synthesis, correspondence denials, and historical
  denials so domain-law boundaries are preserved inside the harness too
- the certification tests were tightened from shape-only checks into
  adversarial checks that prove exact row coverage, exact compile-fail file
  mapping, and digest sensitivity under tampering

These changes were made because the closeout bar was not "correspondence and
history work on fixtures." The closeout bar was production-grade honesty about
why two things correspond and how historical truth was materialized.

## Explicit Deferred Scope

Milestone 5.4 is closed for structural correspondence and historical
materialization-path honesty only.

The following remain later-milestone work, not implied completeness:

- broader historical query semantics, diff semantics, and richer basis replay,
  which remain Milestone 6 work
- richer lineage traversal and broader correspondence exploration, which remain
  Milestone 7 work
- mutation, merge, workflow, and writeback declarations, which remain
  Milestone 5.5 work
- unified application facade and runtime-configuration closure, which remain
  Milestone 5.6 work
- durable historical restore, restart-stable historical replay, and
  store-backed historical parity
- broader unsupported structural families and unsupported historical path
  families beyond the currently admitted matrix

The current 5.4 surface is intentionally narrow in admitted family count and
strict in honesty requirements.

## What Later Milestones May Now Assume

Milestones 5.5, 5.6, 6, and 7 may safely assume:

- lineage continuity and advisory structural correspondence are already
  separate proof-bearing result concepts
- ambiguity and lineage-versus-structural disagreement are already typed
  product surfaces rather than diagnostics-only side channels
- requested, admitted, and resolved historical path classes already exist as
  separate proof states
- historical result envelopes already preserve materialization-path metadata
  end to end
- correspondence/history parity bundles and certification artifacts already
  exist as replay-safe closeout surfaces
- retained-path reuse now has a production-owned work-avoided counter

Those milestones must not assume:

- structural correspondence can be treated as authoritative continuity by
  default
- host caches can satisfy historical authority
- hidden path substitution is available as an implementation escape hatch
- unsupported historical or structural families are implicitly beta-supported

## Verification Baseline

Milestone 5.4 closeout was verified with:

- `cargo test -p worth-query correspondence_history -- --nocapture`
- `cargo test -p worth-query correspondence_history_certification -- --nocapture`
- `cargo test -p worth-query -- --nocapture`

This passes cleanly and includes:

- correspondence-history envelope tests
- parity-bundle and denial-shape tests
- milestone-native certification and closeout artifact tests
- historical counter and retained-path reuse tests
- trybuild compile-fail tests for privacy and proof-boundary enforcement

## Operational Conclusion

Milestone 5.4 is now closed at the structural-correspondence and historical
materialization-path contract layer.

`worth-query` no longer depends on host-side best-match storytelling, hidden
ambiguity collapse, hidden historical path substitution, host-cache historical
authority, or harness-only replay explanations to describe correspondence and
historical truth. It now has typed advisory-versus-authoritative boundaries,
typed ambiguity and disagreement outcomes, explicit historical path identity,
production-owned parity and counters, adversarial milestone certification, and
a closeout proof surface that later history, lineage, and workflow milestones
can build on safely.
