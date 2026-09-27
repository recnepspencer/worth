# Milestone 9.16.1.1: Installed Graph Contract Integrity Repair

> **Status:** Closed on 2026-08-23 — required predecessor for Milestone 9.16.2
>
> **Historical posture:** Milestone 9.16.1 remains historical. This corrective
> sub-milestone repairs the current installed application-operation contract;
> it does not reopen, amend, validate, or depend on the 9.16.1 closure record.

## Goal

Make the installed application schema and operation contract the single typed
authority for every declared graph read, declared graph touch, native aspect
contract, and inspectable aftermath fact.

For one exact installed application operation, Query must expose and carry:

- the exact schema-bound entity, aspect, field, and relation loci it may read;
- the exact Foundational native aspect contract and projection mask for every
  declared field or whole-aspect projection;
- the exact typed graph mutation scopes it may perform;
- graph mutation, application-effect emission, and escaping external-effect
  dispatch as distinct contract families;
- the exact installed reconciliation procedure and typed external-effect
  correlation family when those facts are declared; and
- one canonical installed contract consumed unchanged by graph-obligation
  selection, admission, execution lowering, terminal comparison, public host
  inspection, and successor package export.

The repaired progression is:

```text
typed application schema and operation declaration
    -> declaration-owned stable aspect identity and revision
    -> installed native application-schema contract catalog
    -> typed installed operation read and touch scopes
    -> sealed installed graph-obligation set
    -> admitted graph-work plan and provider session
    -> Relational registration from the installed native contracts
    -> actual read facts and touched-scope evidence
    -> terminal and descriptive public inspection
```

No runtime component may recreate, parse, widen, or guess the installed
meaning. No public inspection value grants read, mutation, correction,
dispatch, or recovery authority.

## Roadmap Placement

Milestone 9.16.1.1 consumes:

- the closed canonical graph-obligation and provider-session progression from
  Milestone 9.16.1;
- the application-schema, operation, aftermath, and external-effect meaning
  already delivered by Milestone 9.16;
- Foundational native aspect contracts, identities, revisions, field paths,
  projection masks, canonical basis, and admissibility;
- Relational ownership of authoritative graph truth, exact performed touches,
  and native aspect registration; and
- the existing `worth-query-declaration`, `worth-query-installation`,
  `worth-query-execution`, and `worth-query-host` package direction.

Milestone 9.16.1 remains historical. Current tests and review protect the
repaired contract without reopening or certifying its prior closure artifacts.
The finding corrects the narrower claim that installed application-operation
read and touch meaning was completely inspectable and typed.

Milestone 9.16.2 remains proposed and may not begin production implementation
until this milestone closes. Its stable-identity and package-export phases
consume the aspect identities, revisions, correlation-family identity, and
complete installed operation contracts established here. It may extend those
families with other package identities; it may not replace or reinterpret
them. The NCR vertical court in 9.16.2 must use the repaired public facade and
may not carry a local compatibility representation.

This milestone does not reopen Milestone 9.16 Runtime work or authorize a
general Query schema redesign. A newly discovered defect with an independent
telos receives its own append-only milestone.

## Central Claim

For an exact installed application operation, the public installed contract,
the sealed graph-obligation selection basis, and the runtime lowering all
refer to the same typed schema-bound read and mutation meaning.

The claim is false if any of the following is possible:

- an operation declares a decision read while its installed graph-read scopes
  are empty;
- an entity-existence or relation read is silently represented as a native
  aspect projection;
- a one-field read is widened to a whole-aspect projection;
- a same-named field, aspect, relation, or effect from another entity, schema,
  package, or runtime generation can substitute;
- execution assigns a second application aspect identity or rebuilds an
  application `AspectContract` from erased field declarations;
- a graph touch must be parsed from a string such as
  `write:Entity/Aspect/Field` or `link:Relation:From->To`;
- an external-effect emission is published as a graph mutation touch;
- declared touch scope is accepted as evidence that a touch actually occurred;
- canonical identity and public inspection are derived from different
  representations of the operation;
- reconciliation affects canonical identity but cannot be recovered from the
  installed contract as a typed value;
- the aftermath external-effect posture discards or stringly reinterprets the
  operation's correlation family; or
- a facade consumer must import declaration, execution, Relational, or private
  installation modules to inspect the contract.

## Current Boundary

### Preserved foundations

The following existing boundaries are correct and remain authoritative:

- `ApplicationOperationDecisionReadTarget` and
  `ApplicationOperationProgramTarget` retain the operation's typed declaration
  closure through compilation.
- `WorthQueryOperationNativeProjectionContract` already retains a real
  Foundational `AspectContract`, an admitted `AspectMask<ProjectionMask>`, and
  their canonical material.
- every installed application operation already owns one sealed graph-
  obligation set and one managed provider-session progression.
- Relational owns authoritative graph state, actual projection and traversal,
  committed mutation interpretation, and exact performed touched-scope
  evidence.
- installation already validates application schema closure, operation target
  closure, aftermath axis compatibility, pre-image coverage, and external-
  effect posture.
- `WorthQueryInstalledAftermathContract` already exposes correction authority,
  correction mechanism, recovery posture, canonical artifact, next actions,
  and published posture.
- `InstalledExternalEffectContract` already retains the operation's external-
  effect correlation family and protocol.
- `worth_query_host::facade::domain` is the stable inspection audience and
  re-exports installation-owned domain meaning without owning it.

### Defects repaired here

The current application-operation compiler creates the primary graph-read role
with an unconditional empty `semantic_reads` vector. It therefore loses the
typed aspect contract and projection-mask evidence already promised by the
operation's decision reads.

The current touch contract lowers `ApplicationOperationProgramTarget` into
structured `String` values. That representation combines operation kind and
semantic components in a private grammar and includes `Emit` beside actual
graph mutation targets.

The native application `AspectContract`, `AspectIdentity`, and revision are
currently produced later by primary-graph execution lowering. Installation
cannot honestly construct the promised native projection contract without
duplicating that lowering or depending backward on execution.

The installed aftermath canonical basis includes the declared reconciliation
procedure, but `WorthQueryInstalledAftermathContract` does not retain and
expose the installed procedure. Its derived external-effect posture retains a
correlation-family `String` but exposes only `is_declared()`.

### Corrected interpretation of the incoming report

The public aftermath surface is incomplete, not absent. Authority, mechanism,
recovery, next actions, canonical artifact, and published posture already have
typed installed accessors. The operation-level external-effect contract
already exposes its correlation family. This milestone adds only the missing
typed retained meaning and makes the operation and aftermath views share one
identity type; it does not create duplicate getters or a second aftermath
contract.

## Ownership And Truth Lock

| Meaning or product | Authoritative owner | Constructed at | Consumed by | Cannot authorize |
|---|---|---|---|---|
| Application aspect semantic identity and contract revision | Query declaration | Typed aspect declaration | Installation schema compilation and 9.16.2 package export | A graph read, write, or runtime registration by itself |
| Installed native application aspect contract catalog | Query installation | Exact schema installation | Operation compilation, graph obligations, execution schema lowering, Host inspection | Relational mutation or runtime execution by inspection alone |
| Installed operation graph-read scopes | Query installation | Operation compilation against the installed schema catalog | Obligation identity/selection, admission, execution comparison, Host inspection | Additional reads beyond the admitted operation and session |
| Installed operation declared touch scopes | Query installation | Operation compilation against the same catalog | Obligation identity/selection, mutation admission, performed-touch comparison, Host inspection | Proof that a mutation happened |
| Actual reads and performed touched scope | Relational, carried through the Query session | Runtime owner execution | Decision read-set, invalidation, invariant and terminal evidence | Widening the installed operation ceiling |
| External-effect contract and correlation family | Query declaration and installation | Operation contract installation | Outbox/dispatch admission, aftermath projection, Host inspection | External completion or retry authority |
| Installed reconciliation procedure | Query installation from declared aftermath meaning | Aftermath installation | Correction admission, package export, Host inspection | Execution of reconciliation by inspection alone |
| Canonical installed artifacts | Query installation using Foundational canonical basis | Cold installation | Identity comparison, package export, inspection | Reconstruction from digest or rendered text |
| Execution schema layout | Query execution, derived from installed contracts and Relational registration | Runtime installation | Primary-graph execution | A competing application aspect contract |

Foundational continues to own the native contract, mask, field-path,
admissibility, canonical-basis, and digest mechanics. Query must carry those
types directly and must not create mirror representations.

Application aspect identities and revisions are stable declaration meaning.
Provider idempotency, dispatch-outbox, and aftermath-causality aspects remain
execution-owned runtime schema because they describe provider storage and have
a different lifecycle, authority, and replacement fate. They do not move into
the application schema catalog.

## Verification Scenario

### Production boundary

The decisive court is a normal external `worth-query-host` consumer. It
declares and installs an application schema and operations through
`worth_query_decl::facade`, inspects them through
`worth_query_host::facade::domain`, executes through the ordinary host runtime,
and observes authoritative Relational results only through the existing Query
terminal and publication surfaces.

The consumer has no dependency on the `worth-query` monolith,
`worth-query-execution`, `worth-relational`, `worth-runtime-bridge`,
`worth-signal`, or private installation modules.

### Hostile schema and operation world

The schema contains:

- two entity kinds with the same local aspect name and same local field name;
- one aspect containing at least three fields, including two fields with the
  same scalar family;
- a second aspect on the same entity;
- two relations with the same endpoint entity kinds;
- one mutation operation that reads entity existence, exactly one field path,
  and one relation, then writes a sibling field and links the other relation;
- one read-only operation that reads exactly one field;
- one operation that emits an external effect but performs no graph mutation;
  and
- one external-owner aftermath contract with a reconciliation procedure and
  correlation family.

The test world includes enough same-named and sibling loci to catch accidental
string matching or projection widening without constructing an exhaustive
mutant matrix.

### Required observations

The public Host inspection must show:

- one typed entity read scope;
- one schema-bound native projection whose `AspectContract` is the catalog's
  exact contract and whose mask contains exactly the declared field path;
- one typed relation read scope with exact endpoint kinds;
- one typed sibling-field write scope and one typed link scope;
- no graph touch for either in-process or external emit-only operations;
- an installed application-effect emission contract for every emit target;
- an external-effect contract only when the emitted effect is also declared to
  escape the process;
- the exact installed reconciliation procedure; and
- the exact typed correlation family shared by the operation external-effect
  and aftermath views.

### QA considerations

Focused tests should exercise the repaired product contracts directly:

- installation retains the declared aspect identity, revision, contract, and
  exact partial-versus-whole projection mask;
- an installed operation exposes typed entity, projection, and relation reads
  plus typed create, delete, write, link, and unlink scopes;
- an emit-only operation exposes its installed emission without claiming a
  graph mutation, while only an escaping emission exposes external-effect
  authority;
- declared touch scope remains distinct from actual performed-touch evidence;
- reconciliation and external-effect correlation family are inspectable through
  `worth_query_host::facade::domain`; and
- the ordinary execution integration uses the installed native contract rather
  than reconstructing it or parsing a private string grammar.

Compiler tests are appropriate for the few public construction boundaries that
must be mechanically closed. Tests need not exhaust every constructor, encode
one mutant per field, prove pointer identity, or maintain a second ledger about
the accepted test suite. Repository checks and review decide whether the final
set is adequate for the changed boundary.

## Product Decision Lock

### Stable application aspect identity

Application aspect identity and contract revision move to the declaration
surface because they are stable semantic meaning that crosses runtimes.
`ApplicationAspectMarkerIdentity` gains the exact Foundational
`AspectIdentity` and `AspectContractRevision` associated with the aspect.

The ordinary macro makes those values explicit:

```rust
worth_query_aspect!(
    pub AccountFacts in BankingSchema, Account;
    identity = AspectIdentity(0x9161_1001),
    revision = AspectContractRevision(1),
);
```

The macro remains the only ordinary constructor of a typed application aspect
reference. Installation denies duplicate aspect identities within one schema,
an identity reused for incompatible aspect meaning, revision zero, missing
field closure, or an aspect contract whose declared shape cannot be admitted
by Foundational.

The identity namespace is one exact application-schema binding. Two different
owner/schema bindings may use the same `AspectIdentity`. Within one erased
schema, however, an identity may belong to only one `(entity, aspect)` locus;
reuse at any second locus is incompatible regardless of revision or shape.
Across generations, full schema declaration equality and canonical identity
govern compatibility, so changing identity, revision, field shape, presence,
or scalar family changes installed schema meaning.

The old macro form without identity and revision is retired when covered
consumers migrate. Installation may not synthesize a fallback identity,
derive one from `type_name`, truncate a digest, or allocate one by declaration
or sort order. Source compatibility does not outrank stable semantic identity.

### Installed native schema catalog

Installation creates exactly one
`WorthQueryInstalledApplicationSchemaContractCatalog` for each installed
schema binding. It is immutable, canonical-order independent, and contains one
`WorthQueryInstalledApplicationAspectContract` per declared `(entity, aspect)`
pair.

The installed package index compiles and stores that catalog exactly once per
unique `(owner, schema)` record during each index build, rebuild, or successor
generation. The stored record owns the erased declaration and an immutable
shared catalog. `bind_application_schema` compares the supplied typed
declaration and shares the stored catalog with the typed handle; binding,
lookup, and inspection never compile or canonicalize it again.

Each entry retains:

- the exact application schema binding identity and compatibility generation;
- validated entity and aspect semantic keys;
- the Foundational `AspectContract` built from the declared identity,
  revision, field shape, presence, scalar family, absence, and evolution law;
- exact typed field-key membership;
- canonical contract material prepared once at installation; and
- the Relational semantic binding required to register the contract.

Catalog lookup is bounded indexed lookup by schema-bound entity/aspect locus.
The catalog has no public constructor or mutation method. A canonical digest
is derived compression and cannot substitute for the retained contract or
schema binding.

Catalog compilation is cold installed-index work with explicit catalog,
contract, field, and prepared-basis counters. Aggregate canonical preparation
is admitted under a checked constant-factor entry/byte budget derived from the
already admitted schema canonical-work bound. Overflow or budget exhaustion
denies index construction before graph work.

### Typed graph-read scopes

`WorthQueryOperationGraphReadRole` replaces its native-only
`semantic_reads` collection with a closed collection of
`WorthQueryOperationGraphReadScope` values:

```rust
pub enum WorthQueryOperationGraphReadScope {
    Entity(WorthQueryOperationEntityReadScope),
    NativeProjection(WorthQueryOperationApplicationProjectionScope),
    Relation(WorthQueryOperationRelationReadScope),
}
```

An application projection scope contains the schema-bound entity/aspect locus
and one `WorthQueryOperationNativeProjectionContract` built from the exact
catalog contract. Field decision reads for the same entity/aspect are grouped
into one projection mask containing their exact Foundational field paths.

Entity decision reads remain entity-existence scopes. Relation decision reads
remain relation scopes with exact declared endpoint kinds. Neither is encoded
as an aspect projection. A whole-aspect projection exists only when the typed
declaration explicitly names that posture; an entity read or empty mask does
not imply it. This milestone does not add a new whole-aspect application read
feature: current field-derived masks are explicitly partial and report
`is_whole_aspect() == false`. A later whole-aspect authoring feature must add a
named typed declaration variant before installation may emit that posture.

Read-scope fields are private. Public accessors support exhaustive typed
matching and return borrowed Foundational contracts and masks. Public callers
cannot construct an installed scope or add it to an installed contract.

### Typed declared touch scopes

`WorthQueryOperationTouchContract::Declared` carries
`Vec<WorthQueryOperationTouchScope>` rather than `Vec<String>`:

```rust
pub enum WorthQueryOperationTouchScope {
    CreateEntity(WorthQueryOperationEntityTouchScope),
    DeleteEntity(WorthQueryOperationEntityTouchScope),
    WriteField(WorthQueryOperationFieldTouchScope),
    LinkRelation(WorthQueryOperationRelationTouchScope),
    UnlinkRelation(WorthQueryOperationRelationTouchScope),
    DeclaredDomain(WorthQueryDeclaredDomainTouchScopeIdentity),
}
```

Application operations compile only to the first five structured variants.
The `DeclaredDomain` variant preserves generic installed domain-operation
scope as one validated atomic semantic identity. Its representation contains
no separators or parseable substructure and cannot impersonate an application
entity, field, or relation locus.

`ApplicationOperationProgramTarget::Emit` compiles into the sealed installed
application-effect emission contract, never into a graph touch. When the
operation definition also declares that effect external, the same effect
additionally carries the external correlation and protocol contract.
`NotRequired` is used when an operation has no graph mutation targets even if
it emits an effect. Mutation effect posture is derived from graph mutation
targets; external-effect authority is derived independently from the installed
external-effect contract.

Declared touch scope is the installed legal ceiling. Actual Relational touched
scope is performed evidence. Query compares them through typed schema binding,
entity/aspect identity, field paths, relation identity, and operation/session
affinity. It never promotes the declaration to performed evidence.

### Read/write duality and invalidation

The installed contract index computes overlap without strings:

- a native projection intersects a field write only when schema binding,
  entity locus, aspect contract identity/revision, and field path overlap;
- a whole-aspect projection intersects any field touch within that exact
  bound aspect;
- entity-existence scope intersects the exact entity structural mutation
  family defined by the installed operation contract;
- relation scope intersects link or unlink only for the exact schema-bound
  relation and endpoint contract; and
- external-effect emission has no graph-read intersection.

Installation computes and retains the comparison-ready typed index once.
Admission and execution consume it. They do not rebuild masks, walk schema
members, or parse canonical material.

### Canonical contract basis

Canonical encoding writes a closed variant discriminant followed by each
typed component's canonical basis. Ordering and deduplication operate on typed
values before encoding. Rendered strings are diagnostics only and are never
canonical input when a retained typed value exists.

The installed schema catalog, operation contracts, and graph-obligation set
must agree on the same canonical read and touch material. Equivalent order
produces equal artifacts. Any semantic field, mask posture, identity, revision,
scope variant, endpoint, correlation family, or reconciliation change changes
the appropriate canonical basis.

Canonical work is cold installation/package work. Ordinary selection,
admission, execution, publication, and inspection carry retained fixed-width
identity and typed contract references and perform exact-zero canonical/SHA
work.

### Complete installed aftermath inspection

Declaration introduces a validated
`WorthQueryExternalEffectCorrelationFamily` atomic identity. It may render as
text but exposes no private grammar. The operation external-effect contract,
aftermath external-effect posture, outbox binding, dispatch admission, and
package record carry this exact type rather than independent `String` copies.

Installation creates a
`WorthQueryInstalledReconciliationProcedure` from the declared procedure and
retains it in `WorthQueryInstalledAftermathContract`. The canonical aftermath
basis consumes that retained installed value; it does not hash the portable
declaration through a separate path.

The stable inspection surface includes:

```rust
aftermath.authority();
aftermath.mechanism();
aftermath.recovery();
aftermath.reconciliation();
aftermath.external_effect().correlation_family();
aftermath.canonical();
```

The accessors return borrowed or copyable typed values and grant no correction,
dispatch, recovery, or execution authority. The types are re-exported through
`worth_query_host::facade::domain`; Host contains no duplicate representation
or translation logic.

## Compiler-Visible Progression

Installation progression is compiler-visible:

```text
validated application schema declaration
    -> WorthQueryInstalledApplicationSchemaContractCatalog
    -> operation compilation against &catalog
    -> WorthQueryCompiledApplicationOperationContracts
    -> WorthQueryInstalledGraphObligationSet
```

Execution progression consumes the same authority one way:

```text
&WorthQueryInstalledApplicationSchemaContractCatalog
    -> Relational application-contract registration
    -> derived WorthQueryPrimaryGraphLayout
    -> admitted operation + exact provider session
    -> performed read/touch evidence
    -> typed subset/intersection comparison
    -> terminal
```

No API accepts an unbound scope vector, raw `AspectContract`, caller-selected
mask, scope string, correlation-family string, reconciliation slot, or digest
as a substitute for an installed operation.

The derived primary-graph layout may cache Relational kind IDs, field locators,
and index IDs. Those runtime values are not portable meaning and cannot mint or
modify the installed catalog. Rebuilding the layout from the same installed
catalog must retain exact application aspect contracts.

## Placement

The dominant axes are:

- declaration owns stable application semantic identity and authoring;
- installation schema owns cold compilation of authoritative installed native
  contracts;
- installation domain-operation owns reusable typed read/touch vocabulary;
- installation application-operation owns lowering from application targets
  into that vocabulary;
- graph-obligation owns sealed selection and canonical binding, not scope
  invention;
- aftermath owns installed correction/effect inspection;
- execution schema layout owns volatile Relational mechanics derived from the
  installed catalog; and
- Host owns audience re-export only.

Forbidden placements include adding the catalog to execution, putting
Relational types in declaration or installation, adding application-specific
scope parsing to graph obligation, retaining new logic in a Host facade,
placing effect emission in graph touch, or combining read, touch, aftermath,
and canonical encoding back into one `contracts.rs` bag.

This repair extracts the new read and touch families from the legacy semantic
contract module but does not require decomposition of unrelated pre-existing
families. Destination-only placeholder files are not created.

## Ordered Phase Plan

### Phase 1: Stable Aspect Meaning And Installed Native Catalog

Add declaration-owned `AspectIdentity` and `AspectContractRevision`, migrate
every covered application aspect declaration, and compile the exact installed
native application-schema catalog. Validate identity/revision uniqueness,
shape closure, field membership, and Foundational mask/contract admission.

Move only application aspect-contract construction out of execution. Keep
provider-internal schema aspects with their execution owner. Installation
validates one exact native contract per installed entity/aspect locus before
operation contracts consume it.

This phase also performs the minimal atomic execution cutover required to make
that move honest: primary-graph bootstrap passes the sealed catalog into
layout lowering, Relational registers its exact application contracts, and
execution deletes application identity allocation, revision choice, field
shape reconstruction, and application member scans. Phase 3 retains
performed-read/touch comparison and the remaining operation/capability
consumer cutover; it does not defer this registration handoff.

Provider aspects allocate three deterministic runtime-local identities above
the maximum installed application identity. An empty catalog uses maximum
zero and therefore provider identities `1..=3`. Checked exhaustion denies
before Relational installation; `u64::MAX - 3` is the last successful
application maximum and `u64::MAX - 2` or greater denies.

This phase mechanically forbids synthesized application aspect identities,
hard-coded application revisions, `type_name` identity, digest truncation, and
execution-owned application contract construction.

### Phase 2: Typed Read And Touch Contract Compilation

Compile decision reads against the Phase 1 catalog into typed entity, native
projection, and relation scopes. Compile graph mutation program targets into
typed create/delete/write/link/unlink scopes. Split effect emission from graph
touch and derive mutation/effect posture from the correct target families.

Bind the typed values into graph-obligation identity and selection. Add the
comparison-ready overlap index and remove the structured-string scope
function. The sealed obligation exposes the same exact typed meaning as the
installed operation without a parallel string or summary representation.

This phase mechanically forbids empty installed read scope for a non-empty
decision-read declaration, string parsing, inferred whole-aspect posture, and
declared-scope promotion to performed evidence.

### Phase 3: Execution Consumption And Performed Touches

Consume the Phase 1 registered application contracts and compare runtime
read/touch evidence against the Phase 2 typed contract. Cut over every
remaining ordinary operation and capability-operation consumer atomically.

Runtime registration, read admission, and performed-touch comparison consume
the installed typed contracts through their existing owner boundaries.

Application `AspectIdentity` allocation and application `AspectContract`
construction were already removed from execution in Phase 1. This phase may
not restore either through a private compatibility cache or parallel old
comparison lane.

### Phase 4: Complete Aftermath And Host Inspection Surface

Introduce the typed correlation-family identity, retain the exact installed
reconciliation procedure, derive aftermath external-effect posture from the
operation's installed external-effect contract, and expose the complete typed
surface through `worth_query_host::facade::domain`.

Package and NCR consumers need no private imports or string interpretation.

This phase does not add correction execution, durable recovery, or external-
effect completion authority.

### Phase 5: Integration, Documentation, And Residue

Run the focused owner and integration tests for the changed boundaries plus the
required repository checks. Remove the old aspect macro form, structured scope
rendering as authority, application contract reconstruction, public untyped
scope exposure, and any compatibility re-export that can preserve the defective
contract.

Revise the durable feature documents. Once the implementation, focused tests,
and required repository checks pass review, Milestone 9.16.2 may trust the
repaired installed contract for stable package records and NCR adoption.

## Caller DX Target

Application authors declare stable aspect meaning and continue to declare
operation reads and writes through typed schema references:

```rust
worth_query_aspect!(
    pub AccountFacts in BankingSchema, Account;
    identity = AspectIdentity(0x9161_1001),
    revision = AspectContractRevision(1),
);

schema
    .operation_read(operation, AccountStatus::reference())
    .operation_write(operation, AccountBalance::reference());
```

A Host consumer inspects the installed contract without importing owners or
reconstructing meaning:

```rust
use worth_query_host::facade::domain::{
    WorthQueryOperationGraphReadScope,
    WorthQueryOperationTouchScope,
};

let contracts = installed_operation.contracts();

let projection = contracts.graph_reads().roles()[0]
    .read_scopes()
    .iter()
    .find_map(|scope| match scope {
        WorthQueryOperationGraphReadScope::NativeProjection(scope) => Some(scope),
        _ => None,
    })
    .expect("installed native projection");

assert_eq!(
    projection.entity().semantic_key(),
    Account::reference().semantic_key(),
);
assert!(
    projection
        .projection()
        .mask()
        .selects(AccountStatus::reference().field_path()),
);
assert!(!projection.projection().mask().is_whole_aspect());

assert!(contracts
    .touches()
    .scopes()
    .iter()
    .any(|scope| matches!(scope, WorthQueryOperationTouchScope::WriteField(_))));

let aftermath = installed_operation.aftermath().expect("declared aftermath");
let procedure = aftermath.reconciliation().expect("external-owner procedure");
let family = aftermath
    .external_effect()
    .correlation_family()
    .expect("declared external lane");
```

Exact method names may align with the destination types during planning, but
the semantic operations, typed matching, and absence of caller-authored strings
are locked. Convenience predicates may supplement, not replace, access to the
real Foundational contract, mask, and typed scope variants.

## Work And Performance Contract

- Native schema-contract compilation, mask preparation, canonical encoding,
  and obligation identity are bounded cold installation work.
- Catalog construction is proportional to the installed schema's aspects and
  fields and is performed once per installation attempt.
- Operation contract compilation is proportional to that operation's declared
  decision reads and program targets, with indexed schema-locus lookup.
- Equivalent read fields for one aspect are grouped without scanning unrelated
  operations.
- Warm obligation selection is bounded by the selected operation's fixed index
  and selected scope counts, not total installed operations or schema members.
- Execution registration iterates the installed schema catalog once during
  runtime installation; ordinary execution performs no contract reconstruction.
- Typed read/touch intersection is proportional to the selected operation's
  scopes and actual performed evidence, not unrelated graph population.
- Host inspection borrows retained contracts and performs no canonicalization,
  SHA, registry scan, or digest-text materialization.
- Package export in 9.16.2 may encode retained canonical artifacts as cold work
  but may not rebuild them from declaration strings.

## Documentation Deliverables

Implementation closure must revise:

- `workspaces/worth-query/crates/worth-query/docs/domain-capabilities/canonical-graph-obligation-progression.md`
  for the exact installed read/touch contract, declared-versus-performed touch
  distinction, and installation-to-execution catalog flow;
- `workspaces/worth-query/crates/worth-query/docs/authoring/graph-touch-obligation-authority.md`
  for typed scope variants, effect separation, and the prohibition on string
  grammar;
- `workspaces/worth-query/crates/worth-query/docs/modeling/aspects-and-authority-lanes.md`
  for declaration-owned application aspect identity/revision and execution-
  owned provider schema separation;
- `workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md`
  for complete installed reconciliation and correlation-family inspection;
- `workspaces/worth-query/crates/worth-query/docs/AI_README.md` for the installed
  schema catalog and public Host inspection path;
- the declaration and Host crate-level documentation for the new aspect macro
  form and read-only inspection examples; and
- Milestone 9.16.2 package and NCR language so it consumes these retained typed
  contracts and does not create package-local mirrors.

All public examples compile or run through their ordinary documentation gates.
No new guide duplicates the canonical graph-obligation document.

## Must Ship

- declaration-owned stable application aspect identity and revision;
- one sealed installed native application-schema contract catalog;
- exact typed entity, native projection, and relation read scopes;
- exact typed create, delete, field-write, relation-link, and relation-unlink
  declared touch scopes;
- exact Foundational projection masks with explicit whole-aspect posture;
- graph mutation, application-effect emission, and external dispatch separated
  into their own installed contracts;
- one typed read/touch overlap path and one canonical contract basis;
- execution registration from the exact installed application contracts;
- actual Relational touched evidence kept distinct from declared scope;
- typed retained reconciliation and correlation-family inspection;
- complete `worth_query_host::facade::domain` re-export;
- focused owner, Host-consumer, compile-boundary, and execution integration
  tests; and
- durable documentation, roadmap, and 9.16.2 handoff updates.

## Must Preserve

- Milestone 9.16.1's single obligation, admission, provider-session, owner-
  execution, terminal, publication, and lifecycle progression;
- all recorded historical milestone and phase statuses;
- Relational ownership of graph truth, exact performed touches, and commit;
- Foundational ownership of aspect contract, identity, revision, mask,
  admissibility, canonical basis, and digest mechanics;
- Query declaration ownership of portable application meaning;
- Query installation ownership of installed schema, operation, obligation, and
  aftermath meaning;
- Runtime Bridge and Signal ownership boundaries and Query-independent package
  direction;
- Query execution ownership of runtime sessions, provider-internal schema,
  actual work, and volatile layouts;
- read-only, mutation, capability, conditional, aftermath, external-effect,
  recovery, publication, and cert-only replay behavior not contradicted here;
- typed branch, schema binding, runtime generation, plan, session, and basis
  affinity;
- ordinary public declaration and Host composition roots; and
- exact lifecycle release and existing warm-path bounds.

## Explicit Non-Goals

- PostgreSQL, durability, restart recovery, release archives, or package
  persistence;
- NCR workflow rules, UI, notifications, or acceptance behavior;
- multiple branches, branch-local MVCC, merge, rebase, or durable composite
  history;
- correction, compensation, or reconciliation execution;
- a durable recovery handle or external-effect completion decision;
- changing Relational performed-touch semantics or Signal invalidation
  authority;
- moving provider idempotency, dispatch-outbox, or aftermath-causality schema
  into application declaration or installation;
- replacing Foundational aspect, mask, path, canonical, or denial types with
  Query mirrors;
- adding new whole-aspect application read behavior beyond exposing the exact
  partial-versus-whole posture of the installed Foundational mask;
- generic crate decomposition beyond the files whose responsibilities must
  separate for this repair; or
- preserving source compatibility for the defective aspect macro or structured
  string touch surface after covered consumers migrate.

## Acceptance

Milestone 9.16.1.1 is complete when:

1. installed application aspects retain declaration-owned identity, revision,
   and exact Foundational contracts;
2. installed operations expose exact typed entity, projection, and relation
   reads plus typed graph touches, while emit-only operations declare no graph
   mutation;
3. declared touch scope remains distinct from actual performed-touch evidence;
4. execution and graph obligations consume the installed contracts without
   reconstructing application aspect meaning or parsing private strings;
5. reconciliation and typed correlation family are inspectable through
   `worth_query_host::facade::domain` without granting operational authority;
6. covered consumers use the typed public contracts with no compatibility
   representation, parser, old macro form, or private facade import; and
7. focused owner and integration tests plus formatting, dirty line-cap,
   boundary, generated-context, dependency, Clippy, and proportionate workspace
   checks pass review.

## Successor Handoff

Milestone 9.16.2 may trust:

- stable declaration-owned application aspect identity and revision;
- one complete installed native schema catalog;
- complete typed application-operation read, touch, external-effect, and
  aftermath meaning;
- canonical installed artifacts that package export may carry but not
  reconstruct;
- public Host inspection sufficient for the NCR consumer; and
- execution that consumes the same contracts package reconstruction will
  readmit.

Milestone 9.16.2 Phase 1 extends stable identity to every remaining package-
relevant Rust type axis. Phase 2 exports the retained contracts as typed
records. Phase 3 reconstructs candidates and asks Query to rebuild and compare
the same installed catalog and operation contracts under fresh validation.
Neither phase may introduce package-local aspect identities, string touch
scopes, correlation-family text authority, reconciliation summaries, or an NCR
adapter contract.

Milestones 9.17 and 9.18 may add durable owner bases, composite publication,
and reconciliation execution around these contracts. They inherit the typed
installed meaning and must not move its facade, make SQL authoritative, or
reinterpret descriptive aftermath inspection as correction authority.
