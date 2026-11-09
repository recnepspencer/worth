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
The registry is empty after P2's native index-maintenance cutover. Every measured
phase and independent axis must satisfy the growth guard.

At records=128, history=64 and installed indexes=8, doubling only installed
indexes to 16 measures **62,541 → 62,673 whole-commit work (1.002×)** and
**54,700 → 54,777 index-maintenance work (1.001×)**. The former ratchet entries
were 1.706× and 1.750× respectively. Model doubling measures 1.031× whole
commit and 1.029× index maintenance; history doubling measures 1.015× and
1.017×. These are observations, never exact-count assertions. Preparation
runs once in the native commit pipeline under its original request; phase
work comes from the owner's inclusive observation. Carried generations may
retain their creation commit while remaining selected at the successor basis.

The injected-loop fixture charges actual whole-state visits through Execution.
It checks every phase and every axis, rejects the loop through both the growth
comparison and registry admission, and accepts the corresponding fixed-change
operation. It does not pretend that this tests instrumentation inside every
production phase; owner-specific mutations still belong to the fixing lanes.

## Baseline qualification (D1308)

Baseline: clean `origin/master` at
`5f454efe0341d3527c43471ce219aa5005abb194`, using its own detached checkout
and target directory. Both were removed after preserving the results.
Candidate implementation: `392abe8c9bb8800dec765e3a22b736d61f898a01`,
including P1 `1158697177`. Landing remains ordered after P1's contract slice
certifies.

The exact full-gate command, run from each checkout root, was:

```sh
python3 scripts/ci/query_gate.py relational,query-execution,query-rest,query-process,boundary --out target/sc-qualification
```

Both runs exited **1**. The Query execution, rest and process suites have the
same source-census refusal, with zero executed tests and zero new refusals:
`worth-query-court-execution/tests/compile_fail.rs:12`, function
`foreign_schema_member_reports_the_schema_requirement`, invokes
`Command::new("cargo")`. D1308 accepts this baseline comparison for this lane;
the census and test remain unchanged. The independent owner tracks the
baseline red in [#92](https://github.com/recnepspencer/worth/issues/92).
Query runtime coverage remains unmeasured by those three suites.

The initial candidate run also exposed an obsolete private unique-index
refresh assertion retired by P1. After deleting that one test, the exact
final relational command was:

```sh
python3 scripts/ci/query_gate.py relational --out target/sc-relational-final
```

It exited **1**: 2,540 of 2,542 tests passed, with the same two historical-kind
custody/funding failures as clean master (2,532 of 2,534 passed), tracked in
[#64](https://github.com/recnepspencer/worth/issues/64). There are **zero new
failures**. Boundary suites pass on both checkouts. Root and Query workspace
builds, boundary-check, agent-context, both rustfmt checks, dirty Rust line
caps and `git diff --check` pass. The focused court's three tests pass, and an
actual injected-loop mutation fails before SHA-256-verified restoration.

The preserved machine comparison is
`../build-cache/sc-test-diff.json` relative to the checkout root; raw baseline
and candidate evidence is in the adjacent `sc-final-baseline-evidence` and
`sc-final-current-evidence` directories.

## Isolated runtime (D1308)

Each court test ran **once**, in its own process, on 2026-11-06 at 18:23
America/Denver. The runner waited for the one-minute load average to fall
below 15 and recorded `uptime` immediately before each sample. N, H and K
were unchanged. All three tests passed and remained below 20 seconds.

| Test | Starting load (1 / 5 / 15 min) | Wall seconds | User / system seconds |
|---|---|---|---|
| Public document edit/read/page/undo | 11.69 / 45.23 / 47.96 | 8.695 | 8.366 / 0.100 |
| Native fixed edit and independent doublings | 10.60 / 43.89 / 47.46 | 1.087 | 1.052 / 0.022 |
| Injected whole-state loop sensitivity | 10.60 / 43.89 / 47.46 | 0.008 | 0.005 / 0.001 |

These measurements exclude compilation and execute the already-built
canonical court binary. Each invocation used
`RUST_MIN_STACK=67108864 <binary> --exact <full-test-name> --nocapture --test-threads=1`
from the checkout root and exited **0**. Exact commands, uptime output, CPU
times and per-test logs are preserved in
`../build-cache/sc-final-current-evidence/d1308-isolated/results.json`.
Elapsed times remain telemetry; only deterministic work growth decides the
court result.
