# Milestone 5 Closeout: Structural Delta Storage And Branch Delta Layering

Status: Completed on 2026-04-15

Parent spec: [milestone-5.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-5.md)

Roadmap: [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)

## Summary

Milestone 5 is closed.

`worth-store` now supports shared-base branch creation, persisted branch-delta
layers, replay-parity branch-delta reads, deterministic rewrite and
auto-compaction, rebuild-from-authority for derived delta artifacts, explicit
authority replay control reads, milestone-grade delta storage evidence, and a
named certification suite proving proportionality and parity across backend and
rewrite variation.

The core closure claim is:

- branch creation is proportional to shared-base metadata, not copied full state
- branch-delta reads remain subordinate to canonical authority and are parity
  checked against an explicit control lane
- rewritten and rebuilt delta artifacts preserve the same branch-visible truth
- milestone 7-facing references are admitted through branch/frontier authority
  vocabulary rather than delta-layer internals

## What Shipped

- shared-base branch creation with explicit branch-base identity
- persisted branch-delta layer records with explicit basis and replacement
  lineage
- direct delta-layer reads with replay-parity verification
- explicit `AuthorityReplayControl` read surface
- deterministic rewrite planning, execution, and auto-compaction
- rebuild of delta layers from canonical authoritative history
- budgeted planning and typed fallback / reject outcomes
- compile-time witnesses for shared-base creation, same-branch descendant
  targeting, rewrite eligibility, and milestone-7-independent references
- machine-checkable `Milestone5CertificationBundle`
- machine-checkable `Milestone5DeltaStorageReport`
- named milestone certification suite:
  `Branch Delta Proportionality And Replay Parity Test`

## Acceptance Evidence

The closeout bundle now emits:

- `truth_digest`
- `history_digest`
- `delta_storage_report`
- `counter_snapshot`

The `delta_storage_report` explicitly carries:

- shared-base source branch and frontier
- live layer count
- live layer commit count
- replacement layer count
- direct-path strategy and cost surface
- control-path strategy and cost surface
- milestone 7 control-reference surface

## Certification Result

The milestone 5 named suite now exists in
[crates/worth-store/src/tests/milestone_5_certification.rs](/Users/Esther/Documents/Programming/worth_workspace/worth/crates/worth-store/src/tests/milestone_5_certification.rs)
and covers:

- backend variation parity
- delta growth tracking semantic delta instead of copied baseline size
- rewritten-stack parity against the control lane
- no-edit shared-base branch admission as a near-free lane

## Verification

The final verification run used:

- `cargo test -p worth-store milestone_5_certification -- --nocapture`
- `cargo test -p worth-store --lib`
- `cargo test -p worth-store --test phase_boundaries_compile_fail`

All passed.

## Concurrency Boundary With Milestone 7

Milestone 5 closed without absorbing milestone 7 authority.

The maintained boundary is:

- milestone 5 owns branch-base sharing, branch-delta layering, rewrite, rebuild,
  and control-lane parity
- milestone 7 owns schema, lineage, cursor, and checkpoint durability meaning
- milestone 7-facing adapters use `Milestone7IndependentReference`, not
  delta-layer handles
- support-family rows are still resolved from current authority during control
  and direct read envelopes rather than being allowed to become shadow truth in
  delta payloads

## Residual Notes

No in-scope milestone 5 debt remains in the delta lane.

Future work still exists, but it belongs to later milestones:

- aspect-aware physical layout
- structural block deduplication
- retention / compaction / reclaim
- replication and capsule programs
- later rewrite-policy sophistication beyond milestone 5 boundedness
