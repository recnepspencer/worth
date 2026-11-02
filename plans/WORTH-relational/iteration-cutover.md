# Relational iteration cutover

Status: planned; no subsystem or court extraction is claimed complete.
This is the first production-owner migration under the
[cross-runtime iteration plan](../cross-runtime/iteration-speed.md).
Complete its global setup G1-G5 before mechanical extraction begins.

## Completion boundary

Migrate the whole `worth-relational` responsibility family: production
subsystems, runtime composition, courts, fixtures, serialization, generic
expansion, configuration, and direct callers. Moving only `src/tests`, adding a
new test binary, or making one scenario faster does not close this cutover.

Preserve the public `worth_relational::facade` namespace as the application
entry. It becomes the contractual facade over the one extracted implementation,
not an old implementation forwarding to a parallel new runtime. Internal
subsystem packages expose narrow contractual facades to their actual peers;
ordinary Query/Bridge/World consumers continue through the Relational facade.

The adversarial constraint is concurrent edits to independent Relational
subsystems and courts without shared compiler invalidation, while the runtime
still has one exact authority for branch movement, candidate custody,
settlement, retention, and recovery. No package split may turn private
construction into forgeable public authority or introduce a global runtime lock.

## Protected production journey

Retain the real progression demonstrated by
[`branch_local_mvcc.rs`](../../crates/worth-relational/examples/branch_local_mvcc.rs)
and governed by [Branch-local MVCC](../../crates/worth-relational/BRANCH_LOCAL_MVCC.md)
and the [Owner Component Port](../../crates/worth-relational/OWNER_COMPONENT_PORT.md):

1. Build a runtime with its actual installed schema and caller execution policy.
2. Admit an exact selected branch basis, stage typed entity/relation/aspect
   mutations, and prepare a validated single-use successor.
3. Compare and publish against that branch's exact current root. A stale or
   foreign basis cannot move it; independent branches can progress independently.
4. Settle the performed publication through the owner registry. A receipt or
   commit ID cannot recreate performed authority or settle twice.
5. Observe through owner-issued exact snapshots; fork shares immutable backing
   without sharing mutable heads. Derived indexes remain disposable projections.
6. Release snapshots and retention leases, discard abandoned candidates, and
   close the owner. Ports retain weak owner affinity and cannot resurrect it.
7. Capture/reopen/replay supported native artifacts through validation and
   readmission, preserving bytes, identity, typed failure, and cleanup contracts.

Supply Chain, Fintech, and existing semantic oracles remain independent of the
runtime adapter they judge. Current supported checkpoint/replay formats and
identity/digest definitions remain unchanged unless a separately reviewed
semantic defect makes a change necessary. This is not a durability feature or
a new branch/merge design.

## Source boundary findings

The inspected source is `bd68c7b5c6` on 2026-10-28. Reconcile concurrent accepted
changes before implementation; do not reset active work to this snapshot.

| Current owner | Physical Rust lines beneath directory | Material coupling |
|---|---:|---|
| `src/tests` | 106,662 | Runtime worlds and shared fixtures inside the production crate |
| `validation` | 31,401 | Invariant evaluation mixed with runtime preparation and embedded courts |
| `authority` | 20,169 | Preparation, mutation, publication, and runtime access |
| `visibility` | 19,999 | Root reads, pins, materialization, and runtime owner selection |
| `durability` | 20,357 | Wire, local-store effects, recovery orchestration, and courts |
| `runtime` | 17,099 | Composition plus autonomous state currently in one compilation unit |
| `merge` | 17,642 | Planning, proof interpretation, and effect orchestration |
| `mvcc` | 14,832 | Branch preparation/publication and lifecycle |
| `history` | 14,414 | Canonical data, catalog state, retention, and runtime access |
| `indexes` | 11,700 | Derived generation state and exact-view execution |

These counts include embedded tests, comments, and module files. They locate
work; G1's parsed production counts determine ceiling compliance. Lexical
cross-directory references are evidence of coupling, not a valid package DAG.

The three currently declared integration targets are `relational_contracts`,
`keyed_kept_allocation`, and `leased_preparation`. `relational_contracts` mounts
the certification/world portfolio. The other two preserve process-global
allocation/Execution state. `branch_local_mvcc` is separately executable.
Do not lose those isolation or public-DX properties during consolidation.

`runtime/state/subsystems` already names storage, visibility, history, indexes,
lineage, identity allocation, publication, durability, schema contracts, and
services. Those state owners are real starting points. Their current
`pub(crate)` visibility and upward `RelationalRuntime` references cannot simply
be replaced by public fields when packages are separated.

## Production destination

All packages below are populated by current responsibilities. They are not
one-package-per-old-folder moves. R1 must resolve the concrete exported contracts
and production counts before R2; refine a package along its next named domain
boundary if it would remain over the global ceiling.

| Destination package | Existing responsibility to move | Facade and exclusions |
|---|---|---|
| `worth-relational-identity` | `identity`, `identity_authority` vocabulary, symbol/text identity, graph-member identities | Descriptive/generational identity and admitted text vocabulary; no branch catalog, runtime clocks, or authority reconstruction |
| `worth-relational-schema` | `schema` declarations/lowering, source catalogs, declarative aggregate/derived relation and merge-policy meaning | Schema definitions and admitted immutable contracts; no live runtime or installed mutation owner |
| `worth-relational-storage` | `storage`, persistent partition editions, root-backed record representation | Persistent truth storage and immutable read material; no current-head selection or publication permission |
| `worth-relational-mutation` | `transactions/data`, mutation intents/effects, concrete application to working storage | Typed proposed changes and mutation kernel; no head movement, settlement, or caller authorization bypass |
| `worth-relational-indexes` | Index definitions, generation construction, prepared lookups, discard state | Exact-basis derived access through borrowed storage views; no truth ownership or full-runtime dependency |
| `worth-relational-validation` | Invariant kernels, declared rule compilation, proposed-state evaluation, structural traversal and aggregate checks | Validated results from exact borrowed proposals; no root selection, publication, or generic runtime service locator |
| `worth-relational-history` | Canonical commit/envelope data, immutable catalog, lineage, history retention | Canonical history custody and queries; no reference-head authority or interpretation of an ID as permission |
| `worth-relational-branches` | Branch cells/catalog, owner lifecycle, basis/retention admission, atomic head comparison | Sole mutable-reference authority and weak service bindings; no durability I/O or global commit lock |
| `worth-relational-visibility` | Snapshot/pin lifecycle, exact-root reads, materialization, native query read kernels | Observations and retained read custody, separate from mutable reference decisions |
| `worth-relational-merge` | Causal/identity/conflict classification, schema reconciliation, lowered merge plans | Plans from explicit immutable history/read evidence; no root movement or complete runtime handle |
| `worth-relational-strategies` | `commit_strategies` registry, lowering, strategy execution over explicit read contracts | Frozen strategy meaning and lowered mutation intents; no runtime construction or alternate write route |
| `worth-relational-durability` | Concrete checkpoint/segment encoding, local-store port/adapter, recovery decoding and plan preparation | Wire and external storage effects with readmission inputs; no independent runtime constructor or public forged recovered authority |
| `worth-relational-preparation` | Transaction admission/planning, candidate construction, preparation portions of `authority`/`mvcc` | Phase-scoped preparation using exact owner bindings and borrowed subsystem ports; no compare-and-publish or settlement |
| `worth-relational-publication` | Performed-publication settlement, patch/CDC publication, diagnostic projection | Effects derived from the one performed canonical commit; no second head, canonical envelope, or independently minted movement witness |
| `worth-relational` | Builder, nested configuration, public facade, cross-subsystem workflows and owner-service assembly | Thin composition and explicit operation orchestration; no leftover storage/invariant/index/codec implementation or court portfolio |

Configuration types, errors, diagnostics, performance observations, and codecs
move with the meaning they describe. Runtime configuration aggregates their
sections; they do not collect in a new `contracts`, `common`, or `support`
package. `aspect_wire` moves to the concrete wire boundary for its encoded type.
`simulation` and compiled-artifact access follow their actual owner; preparation
cannot grow a second simulator/runtime. `presentation` retains public entry
composition; its test harness belongs in a testkit. Every remaining old source
directory must receive an explicit assignment in R1 before deletion.

### Dependency direction and cycle resolution

The destination has these permitted dependency layers; arrows point from
consumer to dependency:

```text
Relational public composition
  -> preparation, publication, strategies, merge, visibility, durability
preparation -> branches, validation, mutation, indexes, history, visibility
publication -> branches, history, durability, mutation
strategies -> merge, visibility, mutation, schema
merge -> visibility, history, validation, mutation
durability -> history, storage, schema
visibility -> branches, indexes, storage
branches -> history, validation, storage
history -> mutation, storage, schema
validation -> mutation, indexes, storage, schema
indexes -> storage, schema
mutation -> storage, schema
storage -> schema, identity
schema -> identity
all -> their existing applicable Foundational/Proof/Execution contracts
```

This is a maximum permitted graph, not a requirement to add unused edges.
No lower package imports `worth-relational`, preparation/publication composition,
Query, Signal, World, a court, or a testkit. Keep existing legal-authority
vocabulary in Proof and portable meaning in Foundational; extraction is not
permission to duplicate them in Relational identity/schema.

Resolve current cycles through these concrete ownership changes:

- Schema owns declarative merge-policy meaning; merge owns runtime interpretation.
  Schema does not depend on merge execution.
- Mutation intent/effect meaning moves below validation and history. Those owners
  no longer import transaction orchestration to describe canonical changes.
- Indexes consume explicit immutable storage views and exact basis bindings;
  they cannot call upward to discover a runtime or visibility authority.
- Storage accepts concrete, already selected read/write material and local policy.
  Current-root selection stays with branches/visibility.
- Histories carry descriptive branch/commit identities from their meaning owner.
  Their append/retention kernels do not call branch services or settlement.
- Validation consumes explicit proposed-state views, schema contracts, admitted
  index access, execution policy, and live controls. Runtime callbacks are
  replaced by the narrow read contract relevant to that invariant.
- Preparation produces a sealed validated successor with exact branch/basis
  custody. Branch publication consumes that custody and issues the performed
  witness; settlement consumes the witness. Constructors remain owner-sealed.
  Any legal-grant witness crosses through the existing Proof authority pattern,
  not a publicly implementable trait or a numeric/digest claim.
- Durability decoding returns untrusted/readmitted wire and prepared recovery
  material. Final recovered installation and identity activation are orchestrated
  by the existing owning composition; durability never imports it to construct it.
- A strategy receives its declared read contract and returns typed intents.
  It never receives the complete runtime merely for convenient access.

Keep branch-generation/owner-affinity and linear custody checks at their real
owners. A new public subsystem method requiring only a descriptive ID is not a
replacement for a private constructor. Extend existing sealed controls only for
a demonstrated boundary test, with closed callers and receiver-scoped arming.
Do not publish private runtime state to make moved courts compile.

### Physical skeleton

```text
crates/worth-relational-<subsystem>/
  Cargo.toml
  src/lib.rs                         private module assembly, local unit tests
  src/facade.rs                      narrow contractual exports
  src/<owned responsibility>/        domain/lifecycle children
crates/worth-relational/
  src/facade/                        stable existing audience namespaces
  src/runtime/                       construction and owner-service composition
  src/operations/                    prepare/publish/settle/recover orchestration
  src/config/                        nested assembly of owned policy sections
  README.md, BRANCH_LOCAL_MVCC.md, OWNER_COMPONENT_PORT.md
courts/relational/<surface>/          leaf court packages listed below
testkits/relational/schema/           declaration-only fixtures
testkits/relational/runtime/          production setup and observation bindings
testkits/relational/worlds/           pure semantic worlds and independent oracles
```

Do not create empty packages as scaffolding. Add each with its real implementation
and caller migration. New 400-line files must still own meaningful responsibilities.

## Court destination and evidence ownership

Every court package has `autotests = false`, one explicit `[[test]]`, and a
module tree under that target. Production never depends on courts/testkits.
Court coverage and path prefixes live in the global registry. Similar test
names and old milestone membership do not determine placement.

| Court package | Narrow dependency surface | Evidence |
|---|---|---|
| `worth-relational-identity-court` | identity plus genuinely required low-level dependencies | Identity/text boundary behavior beyond permitted private local logic |
| `worth-relational-schema-court` | schema/identity, schema testkit | Declaration, lowering, schema continuity and invalid contract cases |
| `worth-relational-storage-court` | storage/mutation and their lower dependencies | Persistent editions, field presence/revisions, adjacency and COW invariants |
| `worth-relational-validation-court` | validation/storage/indexes and schema testkit | Real proposed-state invariant and structural rule boundaries |
| `worth-relational-indexes-court` | indexes/storage and schema testkit | Exact-basis build/read/discard and authoritative-scan parity |
| `worth-relational-merge-court` | merge/history/visibility and lower contracts | Lowered planning, causal and schema-policy behavior without runtime composition |
| `worth-relational-runtime-court` | public Relational/Proof/Execution, runtime testkit, pure worlds | Installed runtime, branch/MVCC, strategies, history, publication, retention and service lifecycle |
| `worth-relational-recovery-court` | public runtime plus actual durability/IO adapters and runtime testkit | Checkpoint, restart, replay, migration and retained-format integrity |
| `worth-relational-cost-court` | actual measured runtime/subsystems, allocator probe, runtime testkit only where needed | Growth, allocation, large-world and timing workloads under `cost` segments |
| `worth-relational-compile-court` | public boundary and consolidated compiler harness | Valuable public minting/phase/visibility prohibitions and valid counterparts |

The table establishes distinct dependency surfaces, not a mandate to move every
unit test outside production. Keep truly private local logic tests beside their
owner when permitted by Iteration Law 2. Register those targets/families too.
If an intended lower court needs the full runtime, classify that scenario under
the runtime/recovery court; do not quietly add the runtime testkit dependency.
Refine a dependency surface only for an actual required dependency difference.

Existing `keyed_kept_allocation` exercises Execution's keyed partitioner without
using Relational. Move that evidence to an Execution-owned cost court as a
causally required correction; do not refactor Execution production in this batch.
`leased_preparation` spans actual Relational admission/index behavior and belongs
under Relational cost/runtime areas according to its claim. Preserve allocator
and one-Execution-authority process isolation. Prefer one nextest process per
case; where correctness must hold under raw libtest too, retain explicit
same-binary child isolation or safe immutable initialization. Do not create a
new linked target per case or globally serialize unrelated ordinary courts.

### Fixtures and test identity

Schema testkit depends only on declaration owners and constructs no runtime.
Runtime testkit depends on the real public facade and schema testkit; it performs
actual installation, intent staging, commit and observation, preserving separate
setup/action/observation failure stages. Pure worlds own semantic definitions and
independent expected behavior; they do not import production DTOs, canonicalizers,
digests, runtime handles, or the adapter that compiles them.

Move `src/tests/support` and presentation harness responsibilities into those
owners, deleting unused or duplicate builders. Sharing location requires sharing
meaning and lifecycle, not just both needing a runtime. No testkit re-exports a
new product API or mints authority. Runtime controls remain compiled production
controls with sealed access, not feature-selected test semantics.

Use module areas such as `branch::basis`, `publication::settlement`,
`validation::proposed_state`, `indexes::lookup`, and `recovery::checkpoint`.
Place cost/GPU/external-oracle/determinism categories as their exact named path
segments and give tests behavior sentences without historical phase names.
Supply Chain's pure in-process oracle is independent evidence but does not need
the external-process `oracle` category merely because it is called an oracle.

Delete source-text assertions that only police old placement when the new graph
or compiler fence protects that boundary. Preserve distinct behavioral cases,
valid compile counterparts, independently computed expected values, and meaningful
fault/denial/cleanup distinctions. Remove tests only when their unique protection
is demonstrably retained elsewhere; counts alone justify neither retention nor
deletion. Move executable public example bodies into the runtime court or merged
doctests without adding duplicate source copies or per-example link targets.

## Implementation sequence

### R1. Resolve exact ownership and freeze the extraction graph

After G1-G5, record parsed production sizes, target graph, direct callers, and
the largest generic families for this source revision. Classify the whole old
tree, including embedded tests, controls and unmounted test files. Distinguish
live code from genuinely unreachable source before deleting anything. Check
the planned DAG against concrete type/signature uses and resolve the cycles
listed above. Refine over-ceiling owners along their next semantic boundary.

Freeze the source-to-owner mapping, narrow contracts, authority minting homes,
court dependency sets, and old targets to remove as the implementation boundary
review. This is a finite design input, not a persistent run journal. R1 is not
closed by a crate-name diagram while upward runtime references remain unresolved.

### R2. Extract meaning and storage owners with their courts

Move identity, schema, storage, mutation meaning/kernel, and their concrete wire
entrypoints in dependency order. Move lower courts/testkit fixtures with them;
keep existing runtime behavior compiling through the same facade. Retire old
implementations in each reviewed slice. Lift policy to its semantic owner rather
than adding a cross-package public internal-data bag.

Prove exact identity/generation, field/aspect/adjacency behavior, persistent
sharing, mutation semantics and schema validation. Confirm lower tests omit the
full runtime. A facade re-export is the same canonical type, not a compatibility
copy with conversions and a second source of truth.

### R3. Extract independent evaluation and history owners

Move indexes, validation, canonical history and lineage, then their callers.
Replace broad runtime lookups with already-selected views and owner bindings.
Keep unique-index correctness on its existing synchronous path while allowing
derived cache generations to be discarded/rebuilt. Carry immutable prepared
results through phases rather than revalidating after package crossings.

Prove the same proposed-state denials, exact-root index parity, canonical record
and history behavior, retention, interruption, and allocation posture. Update
their narrow courts and verify no back edge to the runtime facade exists.

### R4. Extract live lifecycle and operation owners

Move branches, visibility, merge, strategies, durability, preparation, and
publication in the declared dependency order. Separate kernels from upper
operation orchestration before moving a source file that takes the full runtime.
Keep weak lifecycle bindings and per-subsystem/per-branch state; move their
existing cells rather than copying them. Runtime becomes their composition root.

Prove prepare/discard, exact compare/publish, one-winner races, independent
branches, single settlement, partial/deferred failure, retained snapshots,
archive/delete, owner close, native capture/restore and replay. Move runtime and
recovery courts and retire old `src/tests` mounts. Package extraction cannot
claim completion with a large leftover runtime containing the same kernels.

### R5. Finish generic and court topology cleanup

Do generic-body extraction alongside R2-R4 where it enables the boundary; R5
closes the remaining whole-family accounting. Inspect large mode/strategy/type
families, serialize each wire through one concrete owning call site, remove
unused derives, and move branch selectors out of deep generic paths when they
do not change semantics. Preserve type-dependent proof and numerical kernels.

Compare the aggregate IR/instantiation surface across production and all court
binaries to the R1 baseline under the same compiler/target/settings. Splitting
one monomorphized body into ten copies is not improvement. No arbitrary numeric
reduction target replaces inspecting the identified families. Establish the
resulting per-owner ratchets and remove resolved PDB/size exceptions.

Close remaining target/category/name migration; delete the old three integration
targets, test mounts, redundant harnesses and unused dependencies after their
evidence has moved. Each leaf court has exactly one registered test binary;
private unit tests remain only where permitted. Doctest executable examples
retain their public facade contracts.

### R6. Qualify the entire owner and cut over consumers

Run `loop` for the final family and registered direct callers: Execution-facing
controls, Bridge, Runtime World, Query admission/installation/execution and
publication consumers, and affected Store boundaries. Select actual dependency
closures rather than editing consumers unnecessarily. A private consumer adopts
a reviewed immutable public source and qualifies its own lockfile/profile/SDK
role; a successful public build does not establish private acceptance.

Update root membership/lockfiles, generated profiles, boundary rules, sealed
control callers, agent contexts through their generator, court/nextest/CI
selection, crate docs, and the Relational roadmap. Remove stale raw-command
briefs and obsolete helper scripts after `loop` covers their obligations.
Do not retain production code solely for historical test imports.

## Closure proof

Use owned disposable verification copies/targets for edit experiments, never
touch or temporarily instrument another active lane. In each supported case,
compare reported build units rather than only elapsed time:

- A runtime-court-only edit rebuilds that court/testkit closure and no production
  unit; a schema-court edit does not build runtime/recovery courts.
- An index implementation edit rebuilds indexes and actual dependents without
  selecting identity/schema-only courts or recompiling their implementation.
- A preparation edit does not recompile lower storage/validation implementations.
- A second target/worktree retrieves eligible same-input library artifacts
  through sccache; final links and metadata-only checks have honest eligibility.
- Registry/manifest/tool/testkit edits expand the selected closure correctly;
  missing coverage, zero selected cases, ambient flags, stale tools and duplicate
  writers refuse with the appropriate narrow remediation.

Required semantic evidence includes the real branch journey and public owner
port suite, independent Supply Chain/Fintech observations, invariant/index
boundaries, generation/foreign-owner/stale cases, meaningful interruption and
partial effects, retention/disposal, native recovery/replay and valuable compiler
fences. Retain existing scale obligations in `cost`; this cleanup earns no new
scale claim merely by moving them.

Run the final registered gate once, including formatting, file/source ceilings,
boundary/context checks, public docs/examples and direct-consumer contracts.
Review architecture/authority, composition, and evidence adequacy against this
destination. Commit reviewed slices; mark Relational migrated only after all
R1-R6 obligations close and replaced machinery is gone. Report residual
unmigrated repository debt separately without calling the whole repository
iteration-compliant.
