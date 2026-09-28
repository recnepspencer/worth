# S.8 Domain Architecture Completion Plan

## Purpose

Finish S.8 against permanent database boundaries rather than treating the
milestone sequence as production architecture.

The current implementation has allowed roadmap vocabulary to become a
surrogate domain graph. `worth-store-layout-indexes/src` alone contains 334
unique `S8...` identifiers and 15 `S9...` identifiers. A broader Store scan
found milestone vocabulary across 1,144 Rust files. Some occurrences belong to
tests or historical format descriptions, but the scale proves this is systemic
rather than an isolated naming problem.

The correction is architectural, not cosmetic:

- production types must describe durable database concepts;
- operations must be organized by the authority that performs them;
- certification must observe production law rather than define it;
- readiness must represent real runtime admission rather than milestone
  completion;
- test support may prepare inputs but may never issue production outcomes;
- milestone identifiers belong in specifications, roadmaps, runner state, and
  project history, not Rust identifiers or module topology.

The current `S8OwnerIssuedCase` work is temporary scaffolding. It must not
become the permanent foundation merely because it is already partially wired.

## Permanent Boundary Map

| Boundary | Permanent owner | Authority produced |
|---|---|---|
| Artifact catalog | `worth-store-layout-indexes` | admitted durable-family declaration |
| Physical keyspace | `worth-store-layout-indexes` | canonical encoding, ordering, scope, and collision laws |
| Strategy registry | `worth-store-layout-indexes` | admitted B-tree or LSM strategy and invariant capability |
| Materialization | `worth-store-layout-indexes` | coverage, freshness, completeness, and absence authority |
| Access planning | `worth-store-layout-indexes` | selected plan and admitted budget |
| Access execution | `worth-store-layout-indexes` | lowered, ready, executed, and counter-bound outcomes |
| Maintenance | `worth-store-layout-indexes` | mutation, publication, rebuild, and parity capabilities |
| Evolution | `worth-store-layout-indexes` | migration, rollback, compatibility, and rebind outcomes |
| Integrity response | layout indexes adapting lower owners | corruption classification and readmission capabilities |
| Physical compaction | physical isolation and WAL | cutover, publication, retention, and reclaim outcomes |
| Runtime readiness | `worth-store-readiness` | real runtime admission only |
| Certification | certification crates | courtroom observations and verdicts, never production law |
| Test support | `worth-store-test-support` | inputs and drivers, never outcomes or authority |

## Target Directory Skeleton

This is the expected domain shape. Individual files may become directories
when composition requires it, but implementation may not collapse these
responsibilities back into broad milestone, proof, or handoff modules.

```text
worth-store-layout-indexes/src/
  catalog/
    declaration.rs
    authority.rs
    lifecycle.rs
    inventory/
  keyspace/
    domain.rs
    encoding.rs
    ordering.rs
    scope.rs
    admission.rs
  strategy/
    registry/
    btree/
    lsm/
    admission/
  materialization/
    state.rs
    coverage/
    completeness.rs
    absence.rs
    freshness.rs
  access/
    shape/
    planning/
    budget/
    lowering/
    readiness/
    execution/
    counters/
  maintenance/
    admission/
    publication/
    rebuild/
    parity/
  evolution/
    migration/
    rollback/
    compatibility/
    rebind/
  integrity/
    classification/
    quarantine/
    readmission/
  bootstrap/
  customization/
  compaction_projection/
  facade/
```

Production layout code must not retain `phase*.rs`, `s8_*`, `s9_*`,
`skeleton`, `layout_closeout`, `layout_certification`, milestone `handoff`, or
generic `production_transition` topology.

## Target Outcome Shape

Every operation returns an opaque, owner-issued domain outcome. The outcome's
private case and authority witness are created inside that operation's domain
module. Callers receive only the operation-specific view and the capability
needed by the next valid operation.

```rust
let selection = layout_access().plan(request, &catalog, &materialization);

let selected = match selection.view() {
    PlanSelectionView::Selected(selected) => selected,
    PlanSelectionView::BudgetDenied(denial) => return reject(denial),
    PlanSelectionView::NoEligibleStrategy(denial) => return reject(denial),
};

let readiness = layout_access().admit_execution(selected);

match readiness.view() {
    ExecutionReadinessView::Ready(ready) => execute(ready),
    ExecutionReadinessView::Deferred(reason) => defer(reason),
    ExecutionReadinessView::Stale(stale) => rebind(stale),
    ExecutionReadinessView::Denied(denial) => reject(denial),
}
```

There is no public or crate-wide generic outcome issuer. Domain views, payloads,
denials, copied transition descriptions, and prior-stage witnesses cannot mint
the authoritative outcome.

## Implementation Phases

### Phase 0: Stabilize The Interrupted Work

Keep the runner stopped. Restore a compiling test baseline while preserving
unrelated user and runner changes. Inventory every file modified by the partial
Phase 34 implementation and classify it as permanent domain work, temporary
transition scaffolding, or unrelated work. Do not preserve an abstraction only
because it has already been wired.

The phase closes when production and test code compile and there is an explicit
replacement disposition for `S8OwnerIssuedCase`, the global transition fact
types, milestone handoff types, and partially migrated outcomes.

### Phase 1: Rewrite The S.8 Closeout Contract

Revise Phases 34 through 38 of the S.8 specification. Replace the production
S.8-to-S.9 handoff with permanent owner outcomes and certification-side
milestone closeout. Name every production boundary from the boundary map and
state where its authority originates, what capability it returns, and which
operation may consume it.

The phase closes when no spec requirement asks production code to model a
milestone transition or expose a roadmap-named handoff.

### Phase 2: Enforce Permanent Vocabulary

Add a structured Rust-source guard covering identifiers, module names,
filenames, exports, and public documentation. Production and test Rust code may
not use roadmap ordering as domain vocabulary. Persisted numeric values and
wire bytes remain stable, but their Rust names must become domain terms such as
`initial_format`, `legacy_format`, or a semantic version.

The guard must reject `S<number>`, `Phase<number>`, `Milestone<number>`, and
roadmap-named Rust surfaces. It must use an explicit, reviewed exception list
only for unavoidable literal compatibility bytes, never for Rust API names.

### Phase 3: Establish The Domain Topology

Create the target directory skeleton and move modules according to permanent
responsibility. Root files and `mod.rs` files remain aggregation-only. Facades
must delegate to domain operations and may not contain transition tables,
classification engines, or business logic.

This phase is not complete merely because directories exist. Each moved module
must have one coherent responsibility, and dependencies must point from higher
operations toward lower authority rather than sideways through milestone
facades.

### Phase 4: Remove Milestone Closeout From Production

Remove `StorageFoundationS9LayoutHandoff`, `S9LayoutStateMachineInventory`,
`S8LayoutHandoffReadiness`, phase skeleton inventories, milestone hazard
handoffs, and related aliases from runtime crates.

Move genuinely useful qualification scenarios, coverage matrices, hazard
inventories, and closeout verdicts into
`worth-store-certification::courtroom::layout`. Delete readiness surfaces that
are consumed only by certification. If a readiness gate has a real runtime
consumer, rebuild it around the exact runtime capability and give it a domain
name.

### Phase 5: Remove Generic Transition Facts From Runtime Authority

Audit customization, planning alternatives, strategy admission, execution,
maintenance, and all other runtime consumers of generic transition facts.
Replace every use with the exact typed capability produced by the preceding
owner operation.

A transition description may be exposed as read-only observation for
certification or formal modeling, but it may not admit runtime work, select a
machine, prove authority, or be paired with a payload to create an outcome.

### Phase 6: Build Owner-Local Outcome Issuance

For each operation, define a private case enum, opaque public outcome,
domain-specific read-only view, owner-local constructors, and direct
`worth_proof::AuthorityWitness` binding. Use `worth_proof::TransitionOutcome`
for success, denial, deferred, stale, rebind, and failed projections where it
fits.

Delete the crate-wide `issue(...)` function, generic `OwnerIssuedCase`, generic
raw-result wrapper, and any `pub(crate)` constructor that can mint outcomes for
unrelated domains. Construction visibility must terminate at the owner module,
not the crate boundary.

### Phase 7: Migrate Catalog And Keyspace

Rename and reorganize artifact declarations, authority roles, lifecycle
postures, inventory lookup, key-domain admission, canonical encoding,
comparison, range, prefix, composite ordering, scope, and collision behavior.

Separate declaration from admission. An inventory row is not authority. Only an
admitted declaration may enter keyspace admission, and only an admitted keyspace
capability may enter strategy admission.

### Phase 8: Migrate Strategy And Materialization

Give strategy declaration, invariant verification, registry admission, B-tree
verification, and LSM verification distinct owner outcomes. Remove broad result
types that span unrelated operations.

Separate materialization state, coverage, completeness, freshness, and absence.
Exactness must come from the materialization owner and cannot be reconstructed
from state labels, counters, or a generic transition description.

### Phase 9: Migrate Access Planning And Budgeting

Separate access-shape admission, candidate classification, deterministic
selection, budget estimation, and budget admission. Candidate audits and
diagnostics are projections, not selection authority.

Every denial family must retain a distinct owner case when it represents a
different semantic transition. A generic `Denied` case may not erase budget,
strategy, hidden-scan, unsupported-shape, or degraded-path distinctions.

### Phase 10: Migrate Lowering And Execution

Make indexed and degraded selection, lowering, readiness, counter admission,
execution, stale detection, rebind, and readmission explicit operation
families. They may share implementation algorithms but may not share an
authority object that spans unrelated machines or operations.

The only route to executed evidence must be selected plan, admitted budget,
lowered operation, current readiness, owner-observed counters, and execution.

### Phase 11: Migrate Maintenance

Separate live mutation admission, publication lowering, lag handling, deferred
maintenance, rebuild, and parity verification. Rebuild and parity are different
operations and return different capabilities.

Maintenance modes such as exact, lagged, advisory, verifier-only,
migration-only, and rebuild-only must remain semantically distinct. A caller
must not obtain exact publication capability from a deferred or observational
mode.

### Phase 12: Migrate Evolution

Keep migration and rollback as separate owner operations. Separate request
resolution, compatibility admission, authority rebind, freshness admission,
interruption handling, migration planning, and rollback planning.

Compatibility declarations are not execution authority. Stale or mismatched
bindings must require explicit rebind or readmission before a plan becomes
ready.

### Phase 13: Migrate Integrity And Bootstrap

Separate corruption classification, quarantine, record-backed recovery
readmission, offline evidence readmission, import readmission, and bootstrap
catalog discovery. Preserve lower ownership from physical integrity and
recovery physics.

Bootstrap discovery may produce materialization evidence only through a real
admitted catalog read. Certification and test support may not construct
classification or readmission outcomes directly.

### Phase 14: Repair The Physical Compaction Boundary

Physical isolation and WAL remain the sole owners of compaction cutover,
tombstone retention, publication, recovery visibility, reclaim deferral,
reclaim drain, and mutation denials.

Layout indexes may project the lower vocabulary for layout modeling, but the
projection must be bijective. It cannot add, omit, merge, split, rename away, or
reinterpret physical owner cases.

### Phase 15: Clean Certification, Readiness, And Test Support

Rename certification and harness APIs by the domain behavior they verify.
Certification collects owner observations, executes scenarios, and renders
verdicts. It does not define production contracts or mint lower evidence.

Readiness keeps only real runtime admission. Test support prepares inputs,
drivers, schedules, fault injections, and hostile declarations. Neither crate
may construct production outcomes, owner cases, authority witnesses, or
transition contracts.

### Phase 16: Remove Milestone Vocabulary Across The Workspace

Perform a hard-cutover cleanup in dependency order:

1. contracts, authority, foundational vocabulary, and shared proof adapters;
2. physical format, physical backend, physical integrity, and physical
   isolation;
3. WAL, recovery physics, snapshots, branches, and compatibility;
4. blob chunks, IO scheduling, maintenance, tiering, operations, and security;
5. readiness, physical certification, certification, and test support.

Do not add deprecated aliases retaining the displaced names. Rename persisted
format APIs without changing their encoded values. Each wave must compile and
pass its focused tests before the next wave starts.

### Phase 17: Exact Production-Case Coverage

For every domain operation, compare its declared case set with cases observed
through ordinary production facades. The sets must be exactly equal. Exercise
every success, denial, deferred, stale, rebind, readmission, quarantine,
compatibility, bootstrap, and compaction case.

The observation harness may record outcomes after owners issue them. It may not
construct an outcome, invoke a hidden issuance constructor, or mark a declared
case covered without executing the production operation.

### Phase 18: Hostile Authority Proofs

Add compile-fail proofs that external callers, sibling domains, certification,
and test support cannot:

- issue an owner outcome;
- construct a private case or authority witness;
- pair copied payloads, denials, receipts, or transition descriptions;
- skip a required lifecycle stage;
- use a certification verdict as production authority;
- use a projection as source authority;
- construct physical compaction outcomes from layout code.

Every compile-fail fixture must fail for the intended privacy or type error.
Missing-crate and unrelated compiler failures are invalid evidence.

### Phase 19: Functional And Structural Closeout

Run focused crate checks and tests after every subsystem migration. Then run the
complete layout-indexes library and integration suites, physical compaction
tests, compile-fail suites, dependent crate compile checks, workspace vocabulary
guard, line-cap guard, `qa-loop`, hostile test QA, and code-quality QA.

Phase 19 does not close with warnings. Findings must be corrected and all three
QA passes repeated until they return clean.

## False-Completion Gates

S.8 remains incomplete if any of the following is true:

- any Rust identifier, module path, filename, export, or public code comment
  expresses roadmap order instead of domain meaning;
- certification, readiness, or test support contains production law;
- a generic transition fact is accepted as runtime authority;
- a crate-wide issuance function can mint outcomes for unrelated domains;
- a public enum, payload, denial, receipt, or witness can substitute for an
  owner-issued outcome;
- one outcome family spans unrelated operations or authority owners;
- a contract advertises a case no ordinary production facade emits;
- certification or test support can construct an owner outcome;
- physical compaction cases are added, merged, or reinterpreted by layout
  indexes;
- old milestone APIs survive as compatibility aliases;
- a directory tree requires milestone knowledge to locate a domain operation;
- `mod.rs`, root facades, registries, or inventory files contain business
  logic that belongs to an owner module;
- representative membership tests stand in for exact production-case
  equality;
- compile-fail tests fail for environmental reasons;
- focused checks, QA, or code-quality review report warnings.

## Required Closeout Evidence

The final closeout must demonstrate:

- a production tree organized entirely by permanent database domains;
- owner-local issuance for every authoritative operation outcome;
- typed capability flow from declaration through execution and maintenance;
- lower-owner preservation across corruption, recovery, and compaction;
- certification acting only as courtroom observer;
- test support unable to mint authority;
- exact ordinary-production coverage for every advertised case;
- zero milestone vocabulary in Rust code and module topology;
- zero architectural residue from the displaced handoff and transition
  catalogs;
- warning-free functional, test-quality, and code-quality verification.
