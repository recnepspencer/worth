# Cost follows the change

This leaf court implements the D1289/D1295 scale guard for public WORTH (#89,
umbrella #87). Its name follows boundary-check's product-frontier rule:
`decl-host-replay`. The Query workspace's existing court wildcard registers the
package and its one canonical test target. The gate discovers its `cost::`
tests in `query-cost`; no parallel target, feature or gate runner is needed.
Agent-context also discovers the workspace automatically.

Run the focused court with:

```sh
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host-replay
```

Cargo supplies the repository's configured test-thread stack. Direct binary
execution must supply the same `RUST_MIN_STACK=67108864` environment. The
deterministic work assertions are independent of elapsed time. Runtime samples
are telemetry retained in the qualification logs.

## Specimens

Both specimens measure `(N,H,K)`, `(2N,H,K)`, `(N,2H,K)` and `(N,H,2K)`.
N starts at 128; H starts at 64. Setup and cold oracles run outside measured
ordinary intervals. Work on the fixed operations must grow by less than 1.5x
on each independent axis. Commit counts and touched-record counts must agree
between samples. There are no exact-work-count assertions.

- **Native:** two populated entity partitions, a distinct bystander kind,
  relation history, field indexes and a populated relation join. The edit
  changes one scalar on the last entity and retargets one relation. K is the
  number of unrelated field/index definitions. H prior commits edit an
  unrelated entity and retarget another relation, maintaining real index
  generations on every commit. One genuine old snapshot remains open to keep
  the history horizon honest; the builder does not silently reclaim H away.
  The fixed read performs one indexed lookup and one record
  read. Maintained index contents are compared with independent cold builds;
  the indexed lookup is checked against an authoritative storage query.
- **Application:** the existing public document-retention app, genuine
  authentication, installed invariant and conditional/producer contracts,
  N documents and K unrelated program revisions. H recorded edits include
  periodic undo/redo, plus unrelated-record writes on the main branch. The
  measured operation records one fixed edit, reads the selected document,
  reads one history page near the tail through a genuine positioned cursor,
  and performs one undo. The receipt supplies touched
  records and World supplies actual commit growth.

Native generation initialization is an explicit cold fixture operation. Every
measured incremental index update requires a real predecessor and forbids cold
reconstruction. The ordinary fixture does not checkpoint per edit.

## Evidence boundaries

| Phase | Existing owner observation |
|---|---|
| Whole commit | Original Execution report around prepare, index maintenance and publication |
| Commit preparation | Execution report; Query provider preparation count |
| Publication | Execution report and commit complexity delta; Query prepared-commit count |
| Root admission | P1 `observe_maintenance_work`, `root_admission` |
| Kind commitment | P1 `observe_maintenance_work`, `kind_commitment` |
| Index maintenance | Execution report and native entry-edit/record-read counts |
| Invariant passes | Native scan/phase events and Query state-load/execution counts |
| Rebase | Query retained-resolution count; this does not measure all rebase work |
| Read admission | Execution reports for the indexed, point, document and history-page reads; examined index entries; Query readmission count |
| Snapshot open | Execution report around the actual snapshot door |
| History record | Execution report around the public edit/advance/finish lifecycle |
| Navigation | Execution report around a real public undo |

P1's root and kind observations are **inclusive**. They overlap preparation and
must never be summed into the whole-commit total. Missing, unaccounted or
truncated phase observations fail; they are never replaced by zero.
Publication, indexed lookup and snapshot open currently report zero charged
work on the Native specimen. A zero is absence of charged evidence, not proof
that the operation did no physical work. Operation counts also cannot reveal
uncharged work inside an operation. This guard makes no claim about invisible
work, checkpoint/restore growth, or fixture shapes it does not exercise.

`observation::PhaseWorkSource` is the court's `{ phase, charged_work }`
consumption interface. The P1 adapter consumes owner observations without
minting authority or assigning synthetic prices. Newly observed owner phase
names are retained and compared automatically. Future owner scopes can
provide the same interface.

## Ratchet registry

The executable registry is `tests/worth_query_court_decl_host_replay/cost/ratchet.rs`.
It admits only these measured failing coordinates:

| Specimen | Phase / metric | Axis | Owner | Issue |
|---|---|---|---|---|
| Native | Whole commit / charged work | Unrelated installed indexes | P2 `pub-indexes` | #86 |
| Native | Index maintenance / charged work | Unrelated installed indexes | P2 `pub-indexes` | #86 |

Entries were created 2026-11-06 UTC and expire 2026-11-20T00:00Z. They also
expire when a subsequent `Fixes #86` closeout reaches fetched `origin/master`.
Every new failing metric or axis fails the gate. A healed coordinate fails
until its owning lane removes the stale exception in its landing change.
Kind commitment and root admission have no exception.

The injected-loop fixture charges actual whole-state visits through Execution.
It checks every phase and every axis, rejects the loop through both the growth
comparison and registry admission, and accepts the corresponding fixed-change
operation. It does not pretend that this tests instrumentation inside every
production phase; owner-specific mutations still belong to the fixing lanes.

## Qualification limitation

The ordinary Query suites currently refuse on clean `origin/master` as well
as this branch: `worth-query-court-execution/tests/compile_fail.rs` launches
Cargo inside a test, contrary to the governed compiler-prebuild boundary.
There is no registered Query prebuild path for that fixture. Consequently the
required full Query runtime test diff remains unmeasured. The focused scale
court, workspace builds and static checks are separate evidence, not a
replacement for that missing qualification.
