# WORTH Query Orientation For AI Agents

This is the canonical implementation reference for `worth-query`. Read it
before changing Query or integrating a consumer with it.

The purpose of this document is to give an AI a complete mental model of the
runtime: what each layer owns, how authority moves, which public facade is
lawful, and which apparently convenient substitutions are false. Detailed
feature guides provide API-level depth, but they do not redefine this model.

Every claim here describes the runtime that exists. If a capability is not
described here or admitted by the public surface, do not infer it from a type
name, an internal module, a digest, a report, or a neighboring runtime.

## Query In One Sentence

WORTH Query turns typed application intent and proof from the runtimes that own
truth into an admitted, bounded operation whose execution, aftermath, recovery,
and publication retain the exact authority, basis, and causality that made it
lawful.

Query is not a database, policy engine, identity provider, or storage system.
It is the application-facing composition authority over those owners.

## The Runtime Stack

The ordinary application path shares one admitted session spine and then
branches by operation kind:

```text
application schema and declarations
    -> installed application meaning
    -> authenticated request and resolved principal
    -> capability, purpose, disclosure, and conflict admission
    -> graph obligation and access-plan admission
    -> provider-session execution
       |-> read result -> governed disclosure and publication
       `-> validated mutation candidate
            -> opaque Relational prepared candidate
            -> branch-local compare-and-publish
            -> performed publication
            -> durability settlement and Query index publication
            -> idempotency and optional co-committed dispatch outbox
            -> external-effect observation and typed commit outcome
            -> aftermath publication
            -> optional settlement recovery or receipt-bound aftermath recovery
```

Each arrow is a proof transition. A later product may retain evidence from an
earlier product, but no caller may reconstruct the later product from fields,
identifiers, or equivalent-looking reports.

### Authority owners

| Owner | Owns | Does not own |
|---|---|---|
| Application domain | Business vocabulary, schema meaning, operations, invariants, capability intent, and disclosure classifications | Runtime proof or lower-runtime truth |
| Proof substrate (`worth-proof`) | Generic proof-bearing progression, freshness, readmission, composition, and capability carriers | Query permission, live runtime state, or owner-specific authority |
| Foundational | Exact canonical values, keys, paths, portable bases, provenance, receipts, and shared boundary vocabulary | Proof progression, application permission, or relational truth |
| Relational | Entities, relations, aspects, immutable branch roots, mutable branch-reference cells, exact branch observations, detached transactions, opaque prepared candidates, branch-local linearization, commit history, and durable publication settlement | Product authorization, application operation meaning, Query index publication, or external completion |
| Runtime Bridge | Installed correspondence and lawful lowering between Query and lower runtimes | Relational facts, Signal decisions, or application policy |
| Signal | Policy evaluation, producer-local scoped invalidation, readiness and scheduling, performed execution receipts, component branch graph and bases, per-branch execution cells, weak owner services, local evaluation slots, and condition outcomes | Application capability admission, Query maintenance authority, composite product currentness, or relational mutation |
| Runtime World | Memory-resident product branch references, immutable single-parent composite history, exact component-basis composition, coordinated publication, and bounded retained owner effects | Application authorization, component truth or settlement authority, durable restart, or Query public completion |
| Query | Installed application meaning, authority composition, admission, typed progression, execution products, idempotency/outbox meaning, Query index publication, typed settlement recovery, runtime-local aftermath recovery, and consumer publication | Authentication truth, graph truth, policy truth, external completion, or Relational durability authority |
| Store | Durable persistence, journals, restart checkpoints, and reconstructive state | Ordinary Query admission, live recovery authority, or external completion |
| External effect owner | Whether an escaping consequence was accepted or completed | Query commit, application authorization, or recovery authority |

The distinction between **truth** and **authority** is fundamental. A lower
runtime can truthfully report that it can perform an action without proving
that a particular application principal may request that action for a
particular purpose and scope.

## Core Laws

### Meaning is declared; authority is admitted

Application declarations describe what a query or operation means. Installation
validates and canonicalizes that meaning. Admission combines installed meaning
with current request evidence. Execution consumes the admitted product.

Skipping any of those steps creates a parallel authority lane.

### Proof is carried, not rediscovered

An admitted object carries the identities and evidence needed by its legal
successor. Downstream code must pass the typed product forward rather than
re-querying state and trying to rebuild authority.

`worth-proof` supplies reusable progression law, but a generic proof or a
caller-defined `AuthorityMarker` cannot open a Query operation. Authority-
bearing Query methods accept the exact Query-owned types returned by their
owning workflows.

### Narrowing cannot widen

Purpose, tenant, relationship proof, capability, disclosure, branch, basis,
and lifecycle constraints may narrow a request. No projection, helper, adapter,
or lower-runtime result may expand it again.

### Reporting is not authority

Digests, counters, inspection reports, explanations, support rows, serialized
documents, and public projections are useful evidence for humans and tools.
They do not authorize execution unless a public typed contract explicitly says
that they do.

### State progression is explicit

Requested, admitted, prepared, executing, performed, settled, completed,
stopped, published, and released objects are different states. In particular,
`performed` means the branch reference already moved; `settled` means the
owning runtime also acknowledged durability and any required Query publication.
Methods appear only on the states that may legally perform them. Do not
simulate progression with booleans or status strings.

### Currentness is part of authority

Authentication, principal mapping, graph observations, Signal decisions,
capability grants, lifecycle state, branch, snapshot, and version may change.
Query binds them to a request and revalidates the relevant dependencies before
governed work or commit.

### Commit and external completion are separate facts

A committed mutation or dispatch-outbox row proves local state. It does not
prove that an external consequence completed. Acknowledgement, silence,
timeout, disconnect, and lost response retain their exact typed posture. Query
never guesses completion from transport behavior.

### Product support is explicit

An exported type may be accepted, provisional, deferred, or vocabulary-only.
The owning facade documentation and support/admission contract decide which.
In particular, `provisional_aftermath` is a compiled undo/redo experiment, not
an accepted Phase 8 product contract.

## Public Audience Facades

Application code consumes Query through audience crates. It does not import
the internal authority packages that implement the progression.

### Declaration audience

Use `worth-query-decl` for application schema and declaration code:

```rust
use worth_query_decl::facade::{
    application_aftermath,
    application_query,
    application_schema,
};
```

This facade re-exports declaration types and macros without adding another
type identity or behavior layer.

Pure schema crates remain Query-agnostic. Query declaration integration belongs
in the application entry band, not in reusable schema-meaning crates.

### Host audience

Use `worth-query-host` for installation, admission, execution, and publication:

```rust
use worth_query_host::facade::{admission, domain, primary_graph, publication, runtime};
```

The host facade exposes the production authority graph. It intentionally does
not expose raw primary-graph handles that would let a consumer bypass Query.
Stable application aftermath and recovery enter through `primary_graph` and
`publication::application_aftermath`. Do not teach
`facade::provisional_aftermath` as stable undo/redo support.

The same `primary_graph` audience exposes opaque application settlement
recovery through `WorthQueryApplicationSettlementDeferred` and
`WorthQueryPrimaryGraphApplicationRuntime::recover_deferred_application_settlement`.
The host never receives Relational's raw settlement capability. This recovery
finishes an already-performed commit and refreshes Query-owned publication; it
does not rerun the application operation.

Installed application meaning is inspectable without importing an owner crate.
Through `facade::domain`, use `installed_schema.native_contracts()` for the
sealed native aspect catalog, and use an installed operation's
`contracts().graph_reads()`, `contracts().touches()`,
`contracts().emissions()`, `contracts().external_effect()`, and
`contracts().aftermath()` for exact typed graph scopes, application-effect
emissions, escaping-effect meaning, and aftermath meaning. The aftermath view
includes correction authority, correction mechanism, published and recovery
posture, legal next actions, exact reconciliation procedure, canonical
evidence, and the typed external-effect correlation family. These are borrowed
inspection values, not operational authority.

The facade route is part of the contract, not just the final list of names.
Boundary enforcement verifies that `worth-query-host` re-exports the exact
installed owner namespace and snapshots that namespace recursively. Retargeting
an alias to a broader implementation namespace is a contract change even when
some existing imports still compile. Do not preserve an obsolete path by
re-exporting the same types from a second authority lane.

### Certification audience

Use `worth-query-replay` only from certification code:

```rust
use worth_query_replay::facade::ScopedReplayBasis;
```

Replay reconstructs and compares prior semantic execution. It is not an
ordinary application operation and must not enter application or host code.

### Facade rule

If an example requires an application consumer to import
`worth_query_installation`, `worth_query_admission`, `worth_query_execution`,
`worth_query_publication`, Relational, Runtime Bridge, Signal, or Runtime World
directly, the example is crossing an authority boundary. The audience facade
must expose the needed lawful product instead.

## Declaration And Installation

### Application schema

An application schema gives Query typed references for:

- entities;
- relations;
- aspects and fields;
- queries and result slots;
- operations and their graph effects;
- operation external-effect and aftermath slots;
- capabilities and purposes;
- policies and principal bindings;
- context slots used by capability rules.

Schema-derived references prevent application code from using strings as
authority. A name may identify a declaration for diagnostics, but only the
typed reference participates in installed meaning.

### Declarations

A declaration states portable intent. Depending on the family, it can describe:

- fields and aspects to read;
- predicates, ordering, traversal, grouping, and aggregation;
- result shape and disclosure requirements;
- operation inputs, reads, writes, links, unlinks, creates, and deletes;
- one explicit external-effect choice and one explicit aftermath choice;
- graph obligations and access requirements;
- capability requirements and composition rules;
- workflow or continuation meaning;
- conditional evaluation and effects.

A declaration does not authorize a request. It is input to installation.

### Installation

Installation validates the complete application package and derives canonical
runtime contracts. It binds related declarations together so execution cannot
mix pieces from different schemas, generations, operations, policies, or
lower-runtime layouts.

Installed products include the exact identities and contracts needed by
admission:

- application schema and operation identity;
- one sealed native application-aspect catalog retaining each declaration-owned
  `AspectIdentity`, `AspectContractRevision`, contract, and field closure;
- query and result-shape identity;
- capability and purpose requirements;
- principal binding;
- typed entity, native-projection, and relation read scopes;
- typed create, delete, field-write, relation-link, and relation-unlink
  declared touch scopes;
- typed application-effect emissions kept separate from graph touches;
- graph obligations derived from those same exact scopes;
- graph-read access requirements;
- effect and invariant contracts;
- external-effect protocol, correlation, payload-bound, and outbox contracts;
- aftermath correction authority, correction mechanism, pre-image demand,
  published posture, recovery, legal next actions, exact reconciliation, and
  canonical evidence;
- publication and consumer-support posture.

Installation is the point where domain meaning becomes executable runtime
meaning. Hosts may supply adapters and resources, but they may not add or alter
application semantics after installation.

### Portable package export

After validation, hosts may call `export_typed_records()` through
`worth_query_host::facade::domain`. The result is a versioned, bounded,
authority-free record set containing every package family plus the exact
retained native and application-operation contracts. The package-archive
surface deterministically frames those records, release metadata, provenance,
requirements, and opaque external signature bytes. A host repository or future
Worth Store binding may retain the resulting exact envelope without owning
Query meaning. This retains package definitions, not application state,
workflow instances, answers, live handles, or runtime authority.

Every decoded signing payload, signed envelope, repository load, and
reconstructed candidate remains untrusted. Signature presence is not signer
trust. A consuming host must independently select the expected semantic
identity, apply its current trust and cryptographic policy, and obtain fresh
Query validation before installation. The protected GitHub workflow publishes
a human release artifact but grants no discovery, `latest`, activation, or
runtime authority. See [Portable Query Packages](./portable-packages.md) for
the end-to-end API, limits, signing workflow, and authority boundaries.

Operation definitions use typestate to make both static choices explicit. A
builder must call either `external_effect(...)` or `no_external_effect()`, and
either `aftermath(...)` or `no_aftermath()`, before `finish()` is available.
Installation derives the accepted aftermath posture as `Reversible`,
`Compensatable`, `Reconcilable`, or `Irreversible`; an escaping external effect
cannot be reversible.

Do not recover any of this meaning by parsing rendered scope strings or by
rebuilding application aspect contracts in execution. Declared touches are the
legal ceiling. Before commit, Relational exposes validated candidate touches so
Query can prove that the proposed mutation remains inside that ceiling;
candidate validation is not performed evidence. Only commit-sealed touched
records say what the committed attempt actually changed. An ordinary typed
`Emit` target is not a graph touch, and the one escaping external-effect lane
remains a separate contract.

## Request, Authentication, And Principal Resolution

Authentication answers who an external caller claims to be. It does not answer
what that caller may do.

The ordinary request path is:

```text
external identity proof
    -> authenticated external principal
    -> installed principal-binding resolution
    -> Query principal bound to request scope
```

The request scope carries cancellation and deadline state. The resolved
principal remains bound to the authentication and mapping evidence used to
construct it. A role string, subject string, or copied principal identifier
cannot replace that proof.

Principal currentness is checked where governed work requires it. Cancellation
or mapping drift can therefore deny a later transition even when an earlier
admission succeeded.

## Capability Authorization

### Four different concepts

Keep these concepts separate:

1. A **lower-runtime ability** says that infrastructure can perform or observe
   something.
2. An **application capability** says that a principal may request a declared
   application operation under exact constraints.
3. A **lifecycle command** says which state transition the principal may
   perform now.
4. A **governed upper bound** states the maximum resource, operation, purpose,
   field, and provenance authority that progression may activate.

They can participate in one decision without sharing a target or meaning.

### Graph authorization

Capability admission evaluates installed subject-relation-object paths against
current Relational truth through the installed Bridge lowering and Signal
decision boundary.

Conceptually:

```text
principal --relation path--> command or governed resource
```

The path answers **who may perform the requested command**. Query also verifies
the installed operation, input scope, purpose, exact grant, prohibitions, and
composition rules.

### Exact-grant binding

When several grants could satisfy a capability family, Query selects and
retains the exact installed grant witness that authorized the request. Later
revalidation uses that witness. An equivalent-looking replacement grant does
not silently become the original authority.

### Composition

An operation may require several capabilities or distinct actors. Query
evaluates the installed composition law; callers cannot collect independent
booleans and claim that the combination is lawful.

Composition can require:

- all named capabilities;
- one lawful alternative;
- distinct principals for distinct duties;
- conflict prohibitions;
- exact relationship or tenant scope;
- purpose and field constraints.

### Delegation

Delegation derives a narrower capability from existing capability authority.
It retains lineage to its source and enforces depth, scope, purpose, resource,
operation, field, and validity bounds.

When one installed operation composes several capabilities, a selected
delegation-activation program may be a proper subset of the operation's full
installed program union. The selected targets must be duplicate-free and every
target must be contained in that installed union; requiring equality with the
whole union would reject a lawful narrower activation.

Delegation cannot:

- widen the source capability;
- discard provenance;
- outlive its source;
- cross a foreign runtime, branch, or installation generation;
- turn a reporting artifact into a grant.

Revocation is a separately authorized command over the delegated grant. It is
not proof that the revoker may perform the governed application operation.

See [Application Authorization And Emergency Elevation](./capabilities/application-authorization-and-emergency-elevation.md)
and [Policy, Tenant, And Relationship-Proof Narrowing](./foundations/policy-tenant-and-relationship-proof-narrowing.md).

## Emergency Elevation

Emergency elevation is a governed state machine over a request for a bounded
application capability. It is not a superuser switch.

The request retains:

- requester;
- governed resource;
- application operation;
- purpose;
- exact field or disclosure bound;
- grant and provenance constraints;
- validity window;
- installed lifecycle identity.

Lifecycle commands authorize transitions against the lifecycle object. The
carried upper bound remains the maximum authority the approved elevation may
activate. Approving a request is therefore not the same operation as using the
requested capability.

```text
request -> approve -> active use -> close -> required review -> reviewed
                    \-> expire
```

Revocation can cut off active use. Expiry is evaluated from trusted runtime
time. Approval, use, close, revocation, and review each require their own
installed command authorization and consume the state appropriate to that
transition.

Important consequences:

- requesters cannot approve their own elevation when separation of duty
  forbids it;
- an approver relationship that conflicts with installed rules blocks
  approval;
- ordinary operation admission cannot publish lifecycle-transition authority;
- lifecycle drift before commit produces a stale outcome rather than a false
  success;
- approved elevation cannot exceed the original governed upper bound;
- publication stops when revocation or expiry invalidates delivery authority;
- completion does not erase the required review.

## Purpose And Disclosure

Permission to use a protected fact inside governed computation is distinct from
permission to disclose that fact to a consumer.

### Internal computation

An admitted operation may use a protected field to determine membership,
ordering, conflict, invariant outcome, or another internal result when its
capability and purpose allow that use.

### Consumer disclosure

Publication evaluates the result shape and field-level disclosure requirements
for the same request authority. Protected values can be omitted even when they
lawfully influenced internal computation.

### Noninterference

Omission must cover indirect channels as well as visible cells. A protected
field cannot leak through:

- result membership;
- ordering or rank;
- counts and aggregates;
- cursors;
- summaries;
- explanations;
- patches or invalidation metadata;
- live-delivery timing or shape.

Masking a value after materialization is not sufficient. Query must shape the
published result from governed disclosure authority.

## Graph Obligations And Access Planning

Application meaning describes graph work through sealed obligation rows:

- graph reads;
- authorization observations;
- mutation touches;
- effect application;
- invariant execution.

Each obligation names its owner, selection basis, resource posture, and required
terminal evidence. A support row or obligation kind is not execution proof.

Graph-read access planning answers a different question: how the declared graph
read can execute without hidden N+1 traversal, unbounded expansion, or
consumer-local materialization.

The access plan binds:

- required adjacency, predicate, ordering, traversal, deduplication, proof,
  and buffering support;
- cost and capacity bounds;
- selected access strategy;
- plan consumption;
- receipt counters.

Do not collapse graph obligation meaning into access strategy. The first says
what work must be proved; the second says how the read may lawfully and
efficiently obtain it.

See [Canonical Graph Obligation Progression](./domain-capabilities/canonical-graph-obligation-progression.md)
and [Graph Read Access Planning](./authoring/graph-read-access-planning.md).

## Provider Sessions And Execution

Execution occurs inside a managed provider session bound to the admitted
application, branch, basis, request, and installed graph obligations.

The session coordinates lower-runtime observations without transferring their
ownership to Query. It retains:

- session and installation identity;
- branch-qualified snapshot and version basis;
- principal and authorization dependencies;
- graph-read products;
- Bridge correspondence;
- Signal decision facts;
- proposed mutation state;
- invariant receipts;
- commit serialization and terminal evidence.

### Read execution

```text
admitted application query
    -> selected graph obligations and access plan
    -> session-bound lower-runtime reads
    -> complete decision read-set
    -> typed result shape
    -> governed disclosure
    -> publication
```

The result is not merely a vector of values. It retains query identity, basis,
ordering, cursor, disclosure, and execution evidence needed by its lawful
consumers.

### Mutation execution

```text
admitted application operation
    -> complete decision read-set
    -> proposed state and effect program
    -> invariant execution
    -> authorization revalidation
    -> provider prepares an opaque branch-bound candidate
    -> branch-local compare-and-publish
    -> performed publication
    -> durability settlement and Query index publication
    -> idempotency resolution
    -> committed mutation and optional dispatch-outbox fact
    -> external dispatch observation
    -> typed commit outcome and published aftermath
    -> optional receipt-bound recovery
```

A proposed state is not committed truth. A selected invariant is not an
executed invariant. A successful local effect program is not a commit receipt.
The prepared candidate is opaque, runtime-affine, branch-bound, and single-use;
it has no method that can publish itself. Only the Relational owner can consume
it through compare-and-publish or explicit discard.

Optional application fields participate in this same decision and currentness
path. An installed operation must declare the field as both a decision read and
a write. Its invariant projection observes either the exact value or lawful
absence, and `write_optional_field` authors presence with `Some(value)` or
absence with `None`:

```rust
let note = reader.decision_field(projected, DraftNote::reference())?;
let quantity = reader.decision_field(projected, DraftQuantity::reference())?;

// After projected dependencies are completed and the effect target is bound:
set_effects.write_optional_field(&draft, DraftNote::reference(), Some(String::new()))?;
set_effects.write_optional_field(&draft, DraftQuantity::reference(), Some(0_u64))?;

// A later admitted operation clears the optional note without a sentinel:
clear_effects.write_optional_field(&draft, DraftNote::reference(), None)?;
```

Empty text, zero, and absence remain distinct authoritative states. Absence is
also a retained decision fact: a competing absent-to-present change makes the
older attempt stale. Required fields cannot call `write_optional_field`; the
field marker enforces that boundary at compile time. Do not encode absence as
an empty string, zero, `AspectValue::Null`, or an application-side sentinel,
and do not bypass the operation projection with a direct Relational patch.
Query lowers the admitted optional-field effect to Relational's native,
contract-validated field patch.

The commit transition revalidates the dependencies whose drift could make the
operation unlawful. Commit authority remains bound to its originating
admission and serialization proof; it cannot be paired with another admitted
operation.

When an external effect is declared, the local mutation and dispatch intent
share one Relational commit. Query dispatches only from that committed fact.
`Committed` and `AlreadyCommitted` preserve idempotency meaning;
`PartialEffect` and `Indeterminate` preserve uncertainty rather than flattening
it. Even an operation with no domain mutation must commit its outbox and
idempotency fact before an external consequence may escape.

`SettlementDeferred` is different from all of those outcomes. It means the
authoritative branch movement already happened, but durability acknowledgement
or Query's derived publication did not finish. The typed deferred carrier is
the only legal retry input. Recovery repairs the exact performed publication,
refreshes Query indexes, observes the current branch basis, proves the original
commit remains in current ancestry, binds the current Bridge head, and
re-admits idempotency when required. It runs under the application commit
serialization boundary and is idempotent. Never rerun the mutation or obtain a
raw Relational settlement token.

External dispatch has its own published posture: `NotDeclared`,
`PendingDispatch`, `Acknowledged`, `Completed`, or `Unresolved`. The external
owner decides completion. Query records what it observed.

### Provisional discard is not committed aftermath

Relational savepoints and rollback discard provisional transaction work. They
do not create application authority or alter committed history. They are not
recorded inverse, compensation, reconciliation, or recovery.

### Application aftermath and recovery

Publication-settlement recovery happens before ordinary application aftermath.
It completes an already-performed Relational publication and Query's derived
index/head publication. `WorthQueryApplicationSettlementDeferred` directs the
host to
`WorthQueryApplicationSettlementNextAction::RecoverDeferredApplicationSettlement`,
and the installed application owner accepts it through
`recover_deferred_application_settlement(...)`.

After commit, the installed aftermath contract determines whether the result
is reversible, compensatable, reconcilable, or irreversible. Runtime-local
recovery opens from the exact sealed commit receipt and remains bound to the
originating runtime, operation, principal, action, scope, idempotency record,
outbox observation, and currentness evidence. A wire identity or published
recovery report is not the live handle.

The accepted recovery surface supports inspection, resolution, safe retry,
disposal, and expiry through exact typed authority. Reconciliation and
compensation currently stop at owner-bound admission products; Query does not
yet execute those corrective effects. Undo and redo remain under
`provisional_aftermath`; they are not accepted product contracts.

See [Provider Sessions And Decision Read-Sets](./domain-capabilities/provider-sessions-and-decision-read-sets.md),
[Provisional State And Invariant Execution](./domain-capabilities/provisional-state-and-invariant-execution.md),
[Authoritative Mutation Evidence](./capabilities/authoritative-mutation-evidence.md),
and [Application Aftermath, External Effects, And Recovery](./execution/application-aftermath-and-recovery.md).

## Basis, Branch, And Currentness

A **basis** identifies the exact truth context against which work was admitted
or executed. Depending on the operation, it includes:

- runtime and installation generation;
- branch identity;
- snapshot and version;
- schema and query identity;
- policy, tenant, relationship, and purpose context;
- principal mapping;
- continuation, cursor, or live-delivery identity.

Equal version ordinals on different branches are not equal bases. Matching
digests from different owners are not interchangeable authority. A cursor is
meaningful only with the query, ordering, branch, and basis that produced it.

Relational branch truth has two distinct parts: an immutable root selected by
an exact basis and a mutable reference cell that may later select another root.
An owner-issued reference observation records the exact target and branch-local
truth version seen together. Resolution of a serialized descriptor proves only
its shape; the owning runtime must readmit it before it becomes operational.

An admitted basis pins its immutable root. Reads through that basis are
repeatable even if the live branch reference advances. A branch-bound
Relational transaction is detached from the runtime after it opens and keeps
that same basis for reads and staged writes. Publication later compares the
prepared candidate's expected observation with the one current branch cell;
unrelated branches do not share that linearization point.

Signal likewise retains one canonical component branch graph. Its owner-issued
`SignalOwnerServicePorts` expose weak basis, mutation, and lifecycle ports;
same-branch work serializes in one execution cell while unrelated branches can
progress independently. These are lower-owner contracts, not public Query
composite branches or product-currentness authority.

`worth-runtime-world` now owns the lower-runtime composition boundary. Its
`ProductBranchObservation` binds the exact product reference and admitted
Relational, Signal, and Bridge bases. Only its final compare-and-publish step
can install a performed composite publication. Component movement without that
installation remains `ProductUnpublished`, with settlement or cleanup obligations;
it is neither rollback nor permission to run a missing sibling or adopt a successor.

This owner is implemented in milestone 9.17.2. Query carriage, dispatch-outbox
gating on performed composite publication, and the public product-branch facade
remain milestone 9.17.3 work. The application path above still describes the
current Query runtime; importing World directly does not provide that integration.
See the [Runtime World contract](../../../../../crates/worth-runtime-world/README.md)
for construction, outcomes, history, retention, and recovery.

Currentness checks compare retained dependencies with the owning runtime. They
do not rebuild authority from a fresh report. Relevant drift returns a typed
stale or denied outcome before governed work proceeds.

See [Basis Capability Lifecycle](./capabilities/basis-capability-lifecycle.md),
[Branches And Previews](./foundations/branches-and-previews.md), and
[Historical Diff And Basis](./capabilities/historical-diff-and-basis.md).

## Query Authoring And Result Shapes

Query authoring is typed application intent, not a string query language.

The declaration surface supports:

- field and aspect selection;
- predicates and expression validation;
- graph traversal and composition;
- ordering and stable cursor construction;
- collections, grouping, and aggregation;
- named scopes and templates;
- saved queries and view shapes;
- detail, table, inspector, and grouped result families.

Canonicalization resolves equivalent authoring forms into one portable query
artifact. Validation rejects ill-typed fields, incompatible predicates,
unsupported graph shapes, invalid result bindings, and ambiguous ordering
before runtime work begins.

Result shapes participate in disclosure and downstream identity. A caller may
not add an undeclared field to a published row or reinterpret one result family
as another because their storage representations happen to match.

See [Query Expressions And Result Shapes](./authoring/query-expressions-and-result-shapes.md),
[Collections, Cursors, Ordering, And Aggregations](./authoring/collections-cursors-ordering-and-aggregations.md),
and [Scopes, Templates, Saved Queries, And View Shapes](./authoring/scopes-templates-saved-queries-and-view-shapes.md).

## Installed Domain Computation

Domains contribute portable operation meaning while Query owns installation,
admission, execution state, and typed outcomes.

Installed computation can include:

- operation inputs and declared graph effects;
- graph participation and touched scope;
- required lower-runtime observations;
- effect programs;
- invariant programs;
- external-effect protocol and correlation contracts;
- aftermath correction, pre-image, and next-action contracts;
- conditional nodes;
- workflow stages;
- publication contracts;
- consumer-support requirements.

Domain hooks provide domain semantics at the installed seam. They do not gain
raw authority to mutate Relational state or mint Query receipts.

Managed artifacts remain owned by the runtime that produced them. Query may
provide native typed access or a bound projection, but copying their fields
into a domain struct does not transfer ownership or proof strength.

See [Runtime-Installed Domains And Operations](./domain-capabilities/runtime-installed-domains.md),
[Installed Computation Artifact Contracts](./domain-capabilities/installed-computation-artifact-contracts.md),
and [Managed Artifact Ownership And Native Access](./domain-capabilities/managed-artifact-ownership-and-native-access.md).

## Conditional Operations And Signal

Conditional operation declarations describe when installed nodes may evaluate
and which effects may follow. Query installs the application meaning; Bridge
lowers the exact correspondence; Signal evaluates the installed condition and
mints decision evidence.

Node evaluation and effect execution are distinct:

- evaluated true can make an effect eligible;
- evaluated false can skip it;
- evaluation that cannot yet finish retains a typed continuation posture;
- denial performs no governed effect;
- an effect still requires its own admitted execution path.

A Signal boolean, slot value, or diagnostic explanation cannot authorize an
application effect by itself.

Primary-graph temporal operations add one crucial ownership rule: durable
temporal intent remains authoritative Relational/domain truth, while Signal's
wake table is volatile derived state. The host supplies a typed predicate, a
named clock source, a bounded reconstruction projection, and an ordinary
application-operation invoker through `worth-query-host`. A clock reading is
time evidence only. Signal decides eligibility, and Query then performs fresh
principal, capability, purpose, invariant, idempotency, and branch-local
compare-and-publish progression. The effect and the intent's completed posture
commit atomically.

Bridge decision evidence enters Query as one of five public postures:
eligible, dependency-unchanged, reverted-clean, suppressed, or deferred. Only
eligible evidence can reach fresh application-operation admission.
Dependency-unchanged, suppressed, and deferred wakes remain non-invoking;
reverted-clean retains the completed compute cost but creates no new
application consequence. Query classifies the real Bridge evidence rather than
re-running the host predicate or copying Signal's decision into a local
boolean.

Temporal identity follows the same canonical seam as the rest of Query. The
portable binding identity covers the installed node authority, clock, source,
timeline, reconstruction query and projector, principal source, and invoker.
Publication derives a second runtime-qualified identity that adds the exact
runtime, installation generation, provider, and branch. Both use Foundational
canonical-basis preparation and typed canonical digests; Query does not own a
private byte grammar or direct hashing lane. The binding and runtime identities
are derived at installation and carried forward.

A due wake derives its idempotency key and intent identity once during fresh
application admission from the carried runtime binding plus the authoritative
intent identity, revision, input, and host idempotency value. Compare-and-commit
consumes that prepared binding. No later phase of that attempt regenerates it.
If a later re-entry lawfully performs another fresh admission, its derivation
is reported again as admission work, never as retry, recovery, provider,
projection, live-delivery, or publication work.

Commit publication refreshes the derived temporal-intent index before it
returns. Cancellation, completion, or an active successor revision is therefore
reconciled before predicate and operation contact; ordinary clock observation
uses that derived index and never performs reconstruction. Relevant changes are
retained on route-local exact-record journals; unrelated global commits neither
consume the route's retention nor create false overrun. Dependency observations
expose authoritative snapshot absence and only the declared projection fields.
Absence is an explicit `Option` posture throughout snapshot materialization;
there is no present-only accessor that can panic on a lawful missing record or
aspect. Same-installation conditional-runtime reinstallation
discards Bridge/Signal state and reconstructs active work from current
authoritative intent records; completed or cancelled work does not return, and
an already committed effect cannot be repeated. A successor installation must
either be rebound through fresh typed host bindings or fails closed with a typed
rebind requirement.

Each accepted clock receipt also exposes descriptive `execution_provenance()`:
the stable intent and revision, derived wake ordinals, Signal decision,
application-attempt presence, and terminal posture. This is inspection
evidence, not replay data or a reusable authority token.

Clock receipts report relevant authoritative-commit work separately from due
wake fan-out. Reinstallation receipts separately report reconstructed binding
and intent counts together with examined candidates, projected records/fields,
and total query work, so ordinary and reconstructive costs cannot be conflated.
Canonical work is equally phase-exact: the clock handle exposes base binding
work, runtime inspection exposes complete installation work, and each
execution-provenance row exposes the fresh admission work in its admission
slot. Later execution, retry, recovery, and publication slots remain zero.
These counters describe where canonical work occurred; they do not reveal a
digest basis or authorize another attempt.

`conditional_runtime_lifecycle_probe()` captures weak liveness observations of
the actual Query binding, lease, wake, intent, and attempt owners plus the
Bridge provider, managed-clock, and owned-Signal-graph owners. Retain it outside
the application runtime and call `live_inventory()` after ordinary Rust `Drop`;
zero means those concrete owners were released, not that a Drop hook published
an expected answer. The probe carries no close or execution authority.

See [Conditional Installed Operations](./domain-capabilities/conditional-installed-operations.md)
and [Signal Compatibility Orchestration](./domain-capabilities/signal-compatibility-orchestration.md).

## Workflows And Continuations

A workflow is an installed directed graph of stages. Query owns stage
progression and run identity; domains own the meaning of each stage.

The current stage product carries the only lawful next-stage authority. A
caller cannot jump to a stage by naming it or by reconstructing a prior stage
receipt.

A continuation retains unfinished work together with the basis, workspace,
runtime, query, request, and execution posture needed to resume it. Resumption
is a new checked transition, not a callback that inherits ambient authority.

Continuation execution can complete, remain pending, stop, or deny. Suggested
next actions derived from a stop are descriptive guidance; application code
must still enter the ordinary public boundary with whatever authority the next
command requires.

See [Continuation Pipeline](./domain-capabilities/continuation-pipeline.md),
[Execution Resource Admission And Managed Runs](./domain-capabilities/execution-resource-admission-and-managed-runs.md),
and [Typed Stops And Remediation Guidance](./domain-capabilities/typed-stops-and-remediation-guidance.md).

## Live Views, Subscriptions, And Async State

Live execution promotes an admitted query result into a managed subscription
bound to the same query, basis, branch, disclosure, and support contracts.

Changes reach a live consumer through Query-owned invalidation and patch
meaning. A lower-runtime notification is evidence that something changed; it
is not itself a lawful application patch.

Live delivery must preserve:

- subscription selection;
- current authorization and disclosure;
- result ordering and cursor meaning;
- region or collection scope;
- mixed-cause change classification;
- backpressure and resource lifecycle;
- terminal release.

Permission, purpose, relationship, tenant, or elevation drift can narrow or
terminate delivery before protected data is projected.

Async result state describes pending, completed, stopped, or denied managed
work. It does not prove that an unrelated command is safe to execute.

See [Live Views](./runtime-surfaces/live-views.md),
[Granular Live Invalidation](./runtime-surfaces/granular-live-invalidation.md),
[Region-Scoped Live Invalidation And Stream Contracts](./runtime-surfaces/region-scoped-live-invalidation-and-stream-contracts.md),
[Subscription Selection And Diagnostics](./capabilities/subscription-selection-and-diagnostics.md),
and [Async Resources And Result State](./capabilities/async-resources-and-result-state.md).

## Publication And Downstream Consumption

Publication is an authority boundary, not serialization convenience.

The publication layer takes a completed or recovered Query-owned terminal and
derives the consumer-facing product allowed by its disclosure, purpose, basis,
and publication contract. It preserves omission evidence and enough identity
for a downstream consumer to verify what it received.

For application mutations, publication can describe the commit, accepted
aftermath posture, external dispatch posture, and disclosure-admitted recovery
support. Those values are intentionally weaker than execution authority. They
cannot mint a recovery handle, redispatch an effect, compensate a commit, or
resolve an indeterminate result.

A performed-but-unsettled mutation returns opaque typed recovery authority,
not a successful publication receipt and not a denial. Generic effect paths use
`EffectExecutionSettlementDeferred` or `EffectBatchSettlementDeferred` and
repair with fresh owning `EffectExecutionAuthority`. Application and branch
merge paths wrap the same lower-runtime fact in their own public carriers so
callers cannot reach the raw `DeferredPublicationSettlement`.

Downstream runtimes consume bound projections or publication receipts. They do
not reach behind the facade to recover raw Query or Relational state.

Transport adapts a published product to HTTP, messaging, UI, or another process.
Transport headers, routes, and user-node state do not become policy or Query
authority.

See [Projection Consumption](./capabilities/projection-consumption.md),
[Downstream Runtime Integration](./foundations/downstream-runtime-integration.md),
[Application Aftermath, External Effects, And Recovery](./execution/application-aftermath-and-recovery.md),
and [Bound Projection Sharing And Invalidation](./domain-capabilities/bound-projection-sharing-and-invalidation.md).

## Outcomes, Stops, And Managed Resources

Public operations return typed outcomes that preserve why execution did or did
not advance.

Important distinctions include:

- admitted versus denied;
- completed versus stopped;
- stale versus invalid;
- cancelled versus timed out;
- skipped versus suppressed;
- pending versus terminal;
- published versus internally completed;
- committed versus already committed;
- prepared versus performed versus settled publication;
- execution denied versus performed-but-settlement-deferred;
- partial effect versus indeterminate;
- external dispatch pending, acknowledged, completed, or unresolved;
- live recovery authority versus published recovery support.

Do not flatten these into `bool`, `Option`, or a generic error string when the
distinction changes legal next actions, effects, inspection, or resource
release.

Managed resources include provider sessions, runs, subscriptions, continuations,
leases, checkpoints, recovery handles, and admitted capacity. Every terminal
path must release or transfer them explicitly. Dropping a report or serializing
an opaque recovery identity does not prove that the underlying resource was
released or transferred.

See [Ordinary Outcomes](./domain-capabilities/ordinary-outcomes.md),
[State](./foundations/state.md), and
[Inspection](./capabilities/inspection.md).

## Support And Admission

Public vocabulary and executable support are different facts. A type or method
can exist without being admitted by a particular runtime profile.

The Query support matrix is the runtime-owned source of support posture.
Admission is the executable check. Callers may inspect support, but they cannot
promote a report, matching digest, or provider presence into support.

Installed operations also carry consumer-support requirements. Compatibility
admission binds one operation's requirements to one runtime support profile and
returns either a pair-bound witness or a typed denial.

See [Support Matrix And Admission](./foundations/support-matrix-and-admission.md)
and [Consumer Kit](./foundations/consumer-kit.md).

## Inspection, Explanation, And Certification

Inspection explains retained runtime state without creating operational
authority. It is appropriate for debugging, tooling, audits, support reports,
and certification.

Explanation preserves typed causes across boundaries. A scope mismatch,
authorization denial, stale basis, unsupported access strategy, or invariant
failure should remain distinguishable rather than collapsing into a generic
failure.

Certification uses independent evidence and hostile cases to prove the public
contract. Replay is confined to this audience because reconstruction and
comparison must not become an ordinary execution shortcut.

See [Cross-Runtime Causal Inspection](./capabilities/cross-runtime-causal-inspection.md),
[Operational Identity Authority](./foundations/operational-identity-authority.md),
and [Certification Surface And Closeout Bundle](./domain-capabilities/certification/certification-surface-and-closeout-bundle.md).

## Representative Journeys

### Governed application read

```text
typed application-query reference
    + parameters
    + authenticated request scope
    -> installed query lookup
    -> principal resolution
    -> capability and purpose admission
    -> graph obligations and access-plan admission
    -> provider-session reads
    -> result-shape construction
    -> disclosure shaping
    -> published result
```

At no point may the handler replace a typed reference with a query name, read
Relational directly, or append a field after disclosure.

### Governed mutation

```text
typed operation reference
    + typed input
    + authenticated request scope
    -> installed operation lookup
    -> capability admission
    -> complete decision read-set
    -> proposed state
    -> effect and invariant execution
    -> authorization revalidation
    -> opaque candidate preparation
    -> branch-local compare-and-publish
    -> durability and Query publication settlement
    -> typed terminal outcome
    -> governed publication
```

The commit receipt comes from actual provider terminal evidence. A proposed
mutation, invariant selection, or effect summary cannot manufacture it.

### Emergency access

```text
requester authorization
    -> elevation request carrying exact upper bound
approver command authorization
    -> approved elevation
approved-use admission
    -> governed operation and disclosure
close command authorization
    -> required review
reviewer command authorization
    -> completed review
```

Each command targets its lifecycle object. The exact resource, operation,
purpose, and field bound remains carried through the progression.

### Live delivery

```text
published query result
    -> live promotion
    -> lower-runtime change evidence
    -> Query invalidation and patch admission
    -> authorization and disclosure revalidation
    -> governed patch or typed termination
```

A notification never bypasses revalidation or result-shape semantics.

### Granular live invalidation

The supported production path is committed Relational truth, installed Runtime
Bridge correspondence, optional performed Signal work, Query impact admission,
Query-owned maintenance, and current consumer publication. Direct truth and
performed Signal evidence are deliberately separate. Bind a live owner through
`bind_primary_runtime_granular_invalidations` (or the shared equivalent), then
consume the runtime-owned observation or batch through the matching `maintain_*`
entry point. Never reconstruct this authority from raw CDC, copied aspect/scope
fields, or a prior installation identity.

See [Granular Live Invalidation](./runtime-surfaces/granular-live-invalidation.md)
for the entry points, examples, stale/rebind behavior, owner counters, and the
future semantic-hierarchy and physical-placement boundary.

## Lower-Runtime Routing

Use this table when deciding where a change belongs.

| Question | Owner |
|---|---|
| What entities, relations, fields, or versions exist? | Relational |
| What transaction committed and at which version? | Relational |
| Which immutable root and exact observation does a branch reference select? | Relational |
| Which branch cell linearizes a prepared candidate? | Relational, independently per branch |
| Which exact Signal branch basis and execution cell govern component work? | Signal owner services, independently per branch |
| Did canonical branch movement occur even though durability acknowledgement failed? | Relational performed-publication evidence |
| May an already-performed application or merge publication be repaired through this facade? | Query typed settlement recovery over the owning Relational runtime |
| How does installed Query meaning correspond to lower-runtime structures? | Runtime Bridge |
| Which installed semantic dependencies match one committed change? | Runtime Bridge candidate selection followed by Query admission |
| Which scoped recomputation did the lower runtime actually perform? | Signal performed execution receipt |
| Which projection, membership, ordering, group, or window consequence is required? | Query impact admission and maintenance |
| What did an installed policy condition evaluate to? | Signal |
| Which Relational and Signal bases form the current product? | Runtime World composition authority; Query integration remains milestone 9.17.3 |
| What generic proof progression or readmission law applies? | `worth-proof` |
| What exact canonical value, provenance, receipt, or portable basis represents this meaning? | Foundational |
| What application operation or query was declared? | Application domain |
| Is this principal authorized for this operation, purpose, and scope? | Query composition over owner evidence |
| May this field be disclosed to this consumer? | Query publication over installed disclosure meaning |
| Which lifecycle transition is legal now? | Query typed progression over current owner evidence |
| What idempotency and dispatch-outbox meaning belongs to this operation? | Query over committed Relational facts |
| Did an escaping consequence complete? | External effect owner |
| Is a receipt-bound recovery action legal now? | Query runtime |
| Can a live Query aftermath recovery handle survive restart or cross a process boundary? | Store-backed capability; currently deferred |
| How is committed graph state persisted and versioned? | Relational |
| How are durable reconstructive artifacts retained? | Store |
| How is a published product encoded for another process? | Transport adapter |

Put pure meaning in the domain schema. Put lower-runtime truth and mechanics in
their owning runtime. Put cross-owner application authority in Query. Put
presentation outside all of them.

## Prohibited Shortcuts

Do not:

- treat authentication as authorization;
- treat a role as an unconstrained capability;
- treat entity visibility as field disclosure;
- treat a lower-runtime ability as application permission;
- treat a lifecycle command as the governed application operation;
- use the governed upper bound as the authority target of every lifecycle
  command;
- reconstruct proof from a digest, ID, report, serialized document, or copied
  fields;
- use a generic proof marker where a Query operation requires its concrete
  owner-issued proof, authority, or capability type;
- combine independently valid proofs when no installed composition contract
  authorizes the combination;
- expose internal Query authority packages to application consumers;
- import Query into pure schema crates;
- import replay into ordinary code;
- read Relational directly to bypass graph obligations or access planning;
- accept a Signal decision as effect authority;
- treat direct Bridge truth as proof that Signal executed, or require a Signal
  receipt for a direct Query consequence that performs no Signal work;
- copy a producer-local aspect or semantic scope through transitive Signal
  descendants instead of deriving each immediate dependency cause;
- treat a reverse-index lookup key such as `ProducerAspectKey`, a semantic
  scope path, or a shard, region, or worker identifier as authority;
- reuse a granular invalidation binding, delivery batch, source-read basis, or
  consumer lease after runtime restore, reinstallation, or rebind;
- construct Query patches directly from raw CDC or copied Bridge/Signal fields;
- dispatch an external effect without its co-committed local outbox and
  idempotency fact;
- treat acknowledgement, silence, timeout, disconnect, or lost response as
  external completion;
- serialize a recovery handle or reuse its opaque wire identity as live
  authority;
- use `provisional_aftermath` as accepted undo/redo support;
- treat proposed state as committed truth;
- treat selected invariants as executed invariants;
- publish protected fields and mask them afterward;
- use a cursor outside its query, ordering, branch, and basis;
- reuse consumed lifecycle or commit authority;
- infer support from method presence;
- add production glob imports or glob reexports to an authority-governed
  surface; keep those bindings explicit and named;
- hide typed denial, stale, cancellation, or resource state inside a generic
  success/failure flag;
- treat Relational rollback as an application-level authority transition;
- retry an application mutation or merge after a settlement-deferred outcome;
- expose, serialize, or repair from a raw `DeferredPublicationSettlement`
  instead of the audience facade's opaque typed carrier;
- let an opaque prepared Relational candidate publish itself or be reused after
  publication or discard.

## Documentation Map

Start with the guide that owns the concept you are changing:

- [Ordinary Application Front Door](./foundations/ordinary-application-front-door.md)
- [Documentation Index](./README.md)
- [Application Authorization And Emergency Elevation](./capabilities/application-authorization-and-emergency-elevation.md)
- [Query Operating Modes](./foundations/query-operating-modes.md)
- [Workspace Overview](./foundations/workspace-overview.md)
- [Declarative Query Experience](./capabilities/declarative-query-experience.md)
- [Runtime-Installed Domains And Operations](./domain-capabilities/runtime-installed-domains.md)
- [Canonical Graph Obligation Progression](./domain-capabilities/canonical-graph-obligation-progression.md)
- [Graph Touch Obligation Authority](./authoring/graph-touch-obligation-authority.md)
- [Graph Read Access Planning](./authoring/graph-read-access-planning.md)
- [Provider Sessions And Decision Read-Sets](./domain-capabilities/provider-sessions-and-decision-read-sets.md)
- [Provisional State And Invariant Execution](./domain-capabilities/provisional-state-and-invariant-execution.md)
- [Authority-Scoped Effect Execution](./execution/authority-scoped-effect-execution.md)
- [Application Aftermath, External Effects, And Recovery](./execution/application-aftermath-and-recovery.md)
- [Branches And Previews](./foundations/branches-and-previews.md)
- [Lower-Runtime Capability Routing](./domain-capabilities/lower-runtime-capability-routing.md)
- [Projection Consumption](./capabilities/projection-consumption.md)
- [Granular Live Invalidation](./runtime-surfaces/granular-live-invalidation.md)
- [Inspection](./capabilities/inspection.md)
- [Hard Prohibitions](./foundations/hard-prohibitions.md)
- [Operational Identity Authority](./foundations/operational-identity-authority.md)
- [worth-proof Authority And Workflow Contracts](../../../../../crates/worth-proof/docs/features/authority-and-workflow-contracts.md)

Feature guides explain usage. Generated `AGENT_CONTEXT.md` files explain local
crate dependencies and enforcement. Specifications and engineering ledgers are
not substitutes for the public runtime model.

## AI Checklist Before Editing

Before changing Query, answer these questions:

1. Which runtime owns the underlying truth?
2. Which application declaration owns the meaning?
3. What exact typed authority enters this path?
4. What does the next product prove that the input did not?
5. Is the proof carried forward or being reconstructed from representation?
6. Which basis and currentness dependencies remain bound?
7. Does the change preserve capability, purpose, disclosure, branch, and
   lifecycle bounds?
8. Is the code entering through the correct audience facade and repository
   band?
9. Are denial, stale, cancellation, and resource-release outcomes still typed?
10. Could a forged, copied, foreign, stale, or equivalent-looking product open
    the path?
11. Does a lower-runtime observation remain evidence rather than application
    authority?
12. If an external effect exists, what local fact was co-committed before it
    escaped, and which owner decides completion?
13. Does an uncertain result remain acknowledged, unresolved, partial, or
    indeterminate instead of being guessed into success or failure?
14. Is the API accepted, provisional, deferred, or vocabulary-only?
15. Is the authority an exact owner-issued type rather than a generic proof
    substrate value?
16. Do the focused tests fail if the disputed authority check is bypassed?

If any answer is unclear, stop and identify the semantic owner before editing.
