# Milestone 9.13.2: Query Authority Crate Decomposition

## Goal

Complete the one-way Query authority graph established by Milestone 9.13.1:
extract admission, execution, and publication authority one semantic surface
at a time; retarget cold certification to the resulting graph; and cut each
covered consumer through the correct audience facade without losing product
behavior. The milestone removes competing authority, not the `worth-query`
product composition root or unrelated features that still live there.

This milestone is a production decomposition, not a test-runner project. Cargo
package selection is the iteration mechanism.

## Why This Milestone Exists

Milestone 9.13.1 removes the obvious target and reconstruction waste, isolates
cold certification, dismantles the giant manually assembled library-test
binary, and extracts declaration and installation as permanent production
packages. Those cuts create a useful inner loop, but admission, execution, and
publication still share the shrinking migration package and its consumers
still have a compatibility path through `worth-query`.

The existing module tree is not itself the desired package graph. In
particular, `runtime`, `application`, `domain_capabilities`, `consumer_kit`,
and `harness` contain multiple authorities and cannot be moved wholesale into
same-named crates. Migration follows semantic ownership, even when that means
splitting an existing folder.

## Governing Summaries

- `MENTALITY.md`: solve package selection as a production-boundary problem;
  do not replace one monolith with a manifest interpreter or proof platform.
- `arch_laws.md`: phase proofs flow one way, executors consume lowered plans,
  and public construction is sealed by the authority that proves it.
- `composition_laws.md`: every package and file owns a named responsibility;
  facades aggregate and do not implement.
- `domain_structure_laws.md`: package topology preserves truth source,
  lifecycle, failure mode, dependency direction, and test ownership.
- `perf_laws.md`: ordinary paths must not inherit reconstructive,
  certification, source-scanning, or diagnostic-richness cost.
- Road 1 `NAMING.md` and `BOUNDARIES.md`: Query is a reviewed framework-family
  exception; ordinary entry and cert consumers reach it only through the
  audience facade legal for their band.
- `WORTH_query_roadmap.md`: 9.13.1 establishes the iterable upstream package
  foundation; this milestone completes and seals it before 9.14.

## Adversarial Constraint

A change confined to admission, execution, or publication must compile and
test through that authority package without building later or cold authorities.
Each slice inventories only the mixed modules and consumers it is about to move
and completes that authority cut before broad verification. The finished graph
must make reverse knowledge, alternate authority roots, deep imports, facade
behavior, shared-support buckets, and compatibility re-export cycles
unrepresentable or mechanically rejected.

The hostile false success to refuse is a set of crates underneath an aggregate
`worth-query` package while the aggregate can still independently admit,
execute, or publish the same covered meaning. That shape changes folders while
preserving competing authority. A one-way product composition root that lowers
into the sole authority package is lawful.

## Product Decision Lock

- The current `worth-query` package is the product composition root and a
  migration source for the exact authority surfaces named by this milestone.
  It remains for unrelated product behavior and may lower one way through the
  three audience facades. It may not retain an independently executable copy
  of a covered authority after that surface cuts over.
- Query's internal packages are an exact, reviewed extension of the existing
  framework-family exception. They are not ordinary Road 1 band crates and are
  not legal consumer dependencies.
- `worth-query-decl`, `worth-query-host`, and `worth-query-replay` remain the
  only public Query package dependencies. Internal packages are reachable only
  by those facades and by later internal packages in the frozen DAG.
- Audience facades contain re-exports only. They may depend on the exact
  internal authority set declared below; they may not contain wrappers,
  constructors, orchestration, policy, or compatibility aliases.
- No `common`, `shared`, `helpers`, `support`, `types`, or test-platform crate
  is created. A type used by two authorities lives with the authority that
  defines its meaning, not in the lowest package that makes a cycle disappear.
- Cross-authority cycles are repaired by moving meaning to its real upstream
  owner or introducing a narrow upstream contract. Re-exporting a downstream
  type through an upstream package is not a cycle repair.
- Certification and replay are cold leaves. No ordinary package depends on
  them, and no ordinary test imports their fixtures or setup.
- Source scans, file catalogs, documentation-source audits, fixture manifests,
  and tests that prove another test exists are deleted. A load-bearing rule is
  enforced through visibility, the type system, the package DAG, boundary-check,
  or a product-behavior test.
- Declaration, installation, and certification are inherited from 9.13.1 as
  permanent packages. This milestone may refine their contracts and test
  ownership but may not recreate them, rename them, or move their meaning back
  into the monolith.
- Every phase is a complete slice: boundary-local inventory, authority move,
  consumer parity proof, atomic cutover, exact predecessor retirement,
  owner-local tests, and one elapsed observation. There is no milestone-wide
  inventory phase and no requirement to classify untouched code before moving
  an obvious authority boundary.

The migration unit is one named semantic authority surface, never a whole
directory or crate merely because it contains that surface. Before cutover the
predecessor remains the sole production authority. At cutover every covered
consumer switches to the destination and the exact predecessor's ability to
mint or execute authority is removed in the same slice. Missing destination
behavior keeps the migration open; it is not permission to delete the feature.

## Frozen Package And Dependency Graph

The final graph contains these exact internal framework packages. Milestone
9.13.1 has already added declaration, installation, and certification to the
Query exception in `NAMING.md` and `tools/boundary-check/config/road1.toml`;
this milestone adds admission, execution, and publication in their creation
slices and verifies the complete graph:

- `worth-query-declaration`
- `worth-query-installation`
- `worth-query-admission`
- `worth-query-execution`
- `worth-query-publication`
- `worth-query-certification`

Their dependency graph is:

```text
worth-query-declaration
  -> worth-query-installation
       -> worth-query-admission
            -> worth-query-execution
                 -> worth-query-publication

worth-query-certification
  -> declaration + installation + admission + execution + publication

worth-query-decl
  -> declaration

worth-query-host
  -> declaration + installation + admission + execution + publication

worth-query-replay
  -> certification
```

The arrows above mean "is consumed by." In Cargo dependency notation the
direction is reversed: publication depends on execution, execution depends on
admission and installation, and so on. Additional transitive shortcuts are
forbidden unless a public signature genuinely carries an earlier authority
type and the direct edge is recorded in the machine constitution.

### Authority Ownership

`worth-query-declaration` owns canonical query intent, declaration identity,
authoring grammar, result-shape meaning, schema-visible validation contracts,
canonicalization, collection/view declarations, and Query-specific value
typing over Foundational authority. It does not import relational, bridge,
signal, workspace, execution, live, or replay machinery.

`worth-query-installation` owns portable domain package definitions,
installation admission inputs, runtime generation identity, installed domain
and operation identity, conflict semantics, and rebuildable installed indexes.
It does not execute work or mint basis, publication, or consumer authority.

`worth-query-admission` owns basis capabilities, tenant/policy/relationship
admission, intent decision lattices, graph access and obligation admission,
support decisions, and the proof-bearing handoff into execution. Denial
precedes planning and lower-runtime contact.

`worth-query-execution` owns planning, lowering, the installed operating-world
root, workspace and provider binding, effect/workflow progression, lower-
runtime routing, execution receipts, operational counters, and ordinary
re-execution. It cannot construct publication or replay authority.

`worth-query-publication` owns derived publication, authorized projections,
projection consumption, result settlement, live/subscription maintenance,
continuation, collection delivery, shared-consumer lifecycle, and invalidation
meaning. It consumes execution authority and never reopens source truth or
certification replay.

`worth-query-certification` owns cert-only replay/reconstruction, hostile
cross-authority certification, the retained compiler portfolio, and any
consumer kit that truly exists to certify adoption. It contains no production
source scanner and is depended on by no ordinary path.

### Lower-Runtime Direction

- declaration may depend on `worth-foundational` and sealed proof vocabulary
  only where declaration meaning genuinely carries it
- installation may additionally depend on proof authority but not physical
  runtime providers
- admission may consume lower-runtime contract types needed to decide
  eligibility, but performs no lower-runtime work
- execution may depend on relational, runtime bridge, and signal facades
- publication may depend on lower-runtime observation and delivery contracts,
  never on their internal modules
- certification may depend broadly; nothing depends back on it

## Intended Consumer DX

The split changes package ownership, not the ordinary capability grammar:

```rust
use worth_query_decl::facade::CanonicalQueryArtifact;
use worth_query_host::facade::{domain, runtime};

fn install_and_run(
    declaration: CanonicalQueryArtifact,
    workspace: &mut runtime::WorthQueryWorkspace,
) {
    // Domain installation, admission, and execution remain available through
    // the host audience facade. Internal authority crates are not imported.
    let _ = (
        declaration,
        workspace,
        domain::WorthQueryCapabilityFamily::QueryComposition,
    );
}
```

Cert-only code may separately import `worth_query_replay`; ordinary entry,
derived, app, and UI code cannot.

## Phase Plan

### Phase 1: Extract Admission Authority

Inventory only basis lifecycle, intent decision lattices,
tenant/policy/relationship admission, graph access and obligation admission,
support decisions, typed execution handoffs, and their direct consumers.
Create `worth-query-admission`, amend the machine constitution for that edge,
move its owned tests, and cut each discovered consumer before inspecting the
next mixed module.

Admission consumes declaration and installation authority and produces the
only type accepted by execution. Planning, providers, workspace mutation,
publication, and replay remain outside. If a decision currently calls runtime
code, repair the input contract instead of retaining a reverse edge.

Before admitting the first public handoff, remove the 9.13.1 doc-hidden
digest-to-canonical-identity and digest-to-schema-basis compatibility
constructors. Planning and downstream phases must retain the exact authority
minted by canonical/validated declaration artifacts; copied digests may report
identity but may not reconstruct a carrier accepted by admission.

**Adversarial tests**

- stale basis, wrong tenant, foreign installation, unsupported capability, and
  policy violation deny before planning, allocation, or lower-runtime counters
  increment
- success, advisory, and violation traces retain exact typed context across
  facade and direct-authority paths
- raw declarations, copied reporting digests, and local lookalike handoffs
  cannot be passed to execution as admitted authority

**Exit condition**

`cargo test -p worth-query-admission` builds declaration and installation but
not execution, publication, the migration monolith, or certification. The one
elapsed observation is owner-local and measured after representative admission
invalidation.

### Phase 2: Extract Installed Execution Authority

Inventory only planning, lowering, the installed operating-world root,
workspace/provider binding, lower-runtime routing, effect/workflow progression,
recovery posture, execution receipts, counters, and their direct consumers.
Create `worth-query-execution`, amend its machine edges, move its owned tests,
and cut consumers within the same slice.

Execution consumes admitted handoffs. It does not re-decide declaration
legality, installation compatibility, basis, policy, strategy, or artifact
posture. Mixed `runtime` and `application` files split at this boundary;
observation, delivery, replay, and test-only behavior do not hitchhike.

**Adversarial tests**

- raw declarations, unadmitted plans, foreign providers, alternate operating
  roots, and stale installations deny before lower-runtime contact
- serial/parallel, direct/facade, and rebuilt-index paths converge on outcomes,
  receipts, warnings, result state, and exact counters
- execution inputs remain move-only and execution cannot construct
  publication, consumption, settlement, or replay authority

**Exit condition**

`cargo test -p worth-query-execution` builds its upstream authorities but not
publication, the migration monolith, or certification. The host facade executes
the ordinary read and workflow transcripts through one installed root.

### Phase 3: Extract Publication And Consumption Authority

Inventory only derived publication, authorized projection, projection
consumption, settlement, live/subscription maintenance, continuation,
invalidation, collection delivery, shared-consumer lifecycle, and direct
consumers. Create `worth-query-publication`, amend its machine edges, move its
owned tests, and delete any miniature parallel consumer encountered in the
slice.

Publication consumes execution authority and admitted publication semantics;
it does not reopen source truth or certification. Mixed `subscription`,
`projection_consumption`, `authorized_projection`, `view_shape_live`,
`ordinary`, `runtime`, and `domain_capabilities` modules split only as reached
by this owned boundary, not through an up-front tree inventory.

**Adversarial tests**

- cross-run, cross-basis, cross-installation, cross-operation, stale, disposed,
  and digest-lookalike publications fail before consumption or maintenance
- one-shot and live delivery converge with fresh execution for identity,
  ordering, result state, warnings, facts, patches, and exact counters
- consumption cannot precede publication, settlement cannot precede
  consumption, and move-only authority cannot be reused or tuple-reassembled

**Exit condition**

`cargo test -p worth-query-publication` builds the ordinary upstream graph but
not the migration monolith or certification. Consumers reach publication and
settlement only through `worth-query-host`.

### Phase 4: Retarget Cold Certification To The Completed Graph

Inventory only the certification package's remaining dependency on the
migration monolith and the cert-only modules still trapped there. Move genuine
cross-authority replay/reconstruction and hostile behavior into the existing
cold leaf, retarget it to the five production authorities, and move any
ordinary behavioral test found during that cut back to its production owner.

Do not reopen the full test corpus. The 9.13.1 compiler selection and test-
ownership decisions stand. A reusable certification helper is permitted only
when it executes a real public journey; it may not expose a registry, manifest,
source scanner, proof bundle, or pre-solved authority constructor.

**Adversarial tests**

- removing certification from the selected graph leaves all five ordinary
  authority builds and outcomes unchanged
- cert-only replay reconstructs declared scenarios while host, entry, derived,
  app, and UI packages cannot name replay types or import replay authority
- retained compile fixtures fail first at their named boundary and no orphan
  diagnostic, tombstone, privacy mirror, or proof-of-proof fixture returns

**Exit condition**

Certification has no dependency on the migration monolith. Authority-local
commands exclude it, while the explicit cold command composes real public
authority journeys without product-owned test infrastructure.

### Phase 5: Cut Facades And Consumers, Then Retire Exact Predecessor Authority

Inventory consumers one audience at a time: declaration-only, ordinary host,
then cert/replay. Expand the three audience facades only with explicit re-
exports from their allowed packages, cut that audience completely, and run its
owned tests before proceeding to the next. No consumer receives a temporary
direct internal dependency.

For each covered admission, execution, or publication authority surface, prove
behavioral, denial, lifecycle, receipt, and relevant cost parity through every
real consumer. Cut those consumers together, then remove the exact predecessor
constructor, executor, or authority-capable alias in the same slice. Retain the
`worth-query` product composition root, its unrelated responsibilities, and
lawful one-way facade adapters. Update boundary-check and generated agent
contexts in the same slice.

**Adversarial tests**

- dependency inspection reports zero destination authority packages depending
  back on `worth-query`, and no consumer imports an internal authority outside
  the exact facade and internal DAG allowlists
- declaration-only, ordinary host, and cert/replay transcripts compile with
  exactly their intended closures; app/UI and derived replay imports fail
- each authority cutover changes no canonical declaration, installation,
  admission, execution, publication, settlement, replay, or counter outcome,
  and unrelated `worth-query` features remain green

**Exit condition**

Every covered predecessor authority and every compatibility path that can
reopen it are absent. Boundary-check and agent-context enforce the final graph,
every covered consumer uses one audience facade or lawful one-way composition
adapter, unrelated product behavior remains available, and owner-local commands
remain within the iteration budgets established by 9.13.1.

## Test Ownership Rules

- Unit tests live in the authority package whose private invariant they
  falsify.
- Integration tests live in the package whose public contract they exercise.
- A test may build upstream dependencies, but not later or sibling authorities
  irrelevant to the behavior under test.
- Cross-authority end-to-end journeys and cert-only reconstruction live in
  `worth-query-certification`.
- Compiler fixtures are selective counterexamples, not exhaustive privacy
  mirrors. One fixture may represent a structurally equivalent family.
- Test helpers follow the same dependency direction as production. No global
  test world, shared fixture package, or test-only authority constructor is
  permitted.
- No test scans another test, counts tests, asserts a fixture path exists, or
  compares a product digest to a test-manifest digest.

## Complexity And Iteration Obligations

- `cargo check -p worth-query-declaration` and its ordinary tests build no
  other Query authority package.
- Each later authority-local command builds only its transitive upstream Query
  authorities; certification is absent unless explicitly selected.
- Per-authority warm elapsed observations are recorded once at phase close.
  They are decision evidence, not timing tests or permanent thresholds.
- The declaration and installation packages contain no lower-runtime calls.
- Admission denials perform exact zero planning, allocation, provider, and
  publication work.
- Execution does no certification, replay, source scanning, or rich diagnostic
  materialization unless an admitted artifact policy requests operational
  diagnostics.
- Publication work scales with declared/affected publication and consumer
  breadth, not unrelated workspace, registry, test, or diagnostic inventory.
- Every touched Rust code or test file is at most 400 lines unless the phase
  spec names a short-lived exception with owner and deletion phase. The
  migration may not expand the existing workspace allowlist to normalize debt.

## Must Preserve

- Foundational remains value, contract, canonical wrapper, and validation
  authority.
- Relational remains truth, commit, snapshot, branch, and merge authority.
- Runtime bridge remains cross-runtime routing, preview, writeback, and bridge
  protocol authority.
- Signal remains scheduling, invalidation execution, observation, temporal,
  and async lifecycle authority.
- Query remains canonical declaration, installation, admission, execution
  orchestration, result shaping, publication/consumption, and public
  certification authority at the package named for each responsibility.
- Store-facing declarations and provider-independent semantics remain
  unchanged; physical persistence stays in the Store roadmap.
- Every retained compiler-owned authority, substitution, phase-order,
  move-only, and facade denial survives with the production owner.
- The existing Query feature set and ordinary public facade remain available
  until each exact semantic surface has proved parity and cut over.

## Allowed Debt

- Store-backed execution, durable restore, restart-stable continuation, and
  physical provider integration remain Store milestones.
- Later 9.14 operation, workflow, sharing, lease, invalidation, and patch
  semantics remain unimplemented until 9.14, but their package homes and
  dependency direction must be ready.
- No authority-capable compatibility re-export, alternate engine root, direct
  internal-package consumer edge, source-audit authority, shared test platform,
  reverse dependency, or ordinary-to-certification dependency may remain debt.
  A one-way facade adapter that reaches the sole destination authority is not a
  compatibility authority.

## Verification

At each phase, run the new authority-local package tests before any all-up
command. At milestone close run:

```text
cargo fmt --all -- --check
cargo check --workspace --tests
cargo test -p worth-query-declaration
cargo test -p worth-query-installation
cargo test -p worth-query-admission
cargo test -p worth-query-execution
cargo test -p worth-query-publication
cargo test -p worth-query-certification
cargo test -p worth-query-decl
cargo test -p worth-query-host
cargo test -p worth-query-replay
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
scripts/ci/check_workspace_rust_line_caps.sh
git diff --check
```

Also run the Worth UI-owned Query binding suite from its workspace and the
root consumers that use `worth-query-host`. Full workspace verification is a
closeout gate, not the inner development loop.

## Acceptance Evidence

- the exact package DAG and audience matrix are machine-enforced
- every authority-local package command omits unrelated later authorities
- ordinary declaration, installation, admission, execution, publication, and
  settlement journeys preserve canonical outcomes and exact counters
- cert-only replay remains inaccessible to ordinary consumers
- Worth UI and server consumers use legal audience facades with no direct
  internal or former-engine dependency
- retained compiler denials remain load-bearing and the compiler portfolio
  does not regress into per-type privacy mirrors
- source-audit catalogs, proof-of-proof tests, shared test platforms, and every
  authority-capable predecessor or compatibility alias for a completed
  migration row are absent
- the `worth-query` composition root and all unrelated Query, Worth UI, and
  server-consumer behavior remain green
- the workspace line-cap guard is green for the migrated packages
- one closeout report records per-authority and full-graph elapsed observations
  without enforcing them through a timing framework

## Sequencing Notes

This milestone follows 9.13.1 because obvious compiler, target, reconstruction,
and consumer-coupling waste must be removed and declaration, installation, and
cold certification must already provide useful package selection before the
remaining multi-week migration. It precedes 9.14 because installed operation
semantics, bound execution progression, publication, consumer support, sharing,
and settlement must land directly in their permanent authority packages.

Phases are ordered by the dependency DAG. Work inside a phase may be parallel,
but a downstream package must not be created as a dumping ground before its
upstream contracts are frozen.
