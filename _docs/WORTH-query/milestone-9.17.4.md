# Milestone 9.17.4: Application API Hardening

> **Status:** Phase 1 was certified on 2026-09-11. Phases 2-6 remain open.
> The focused Pre-M0 application foundation was certified on 2026-09-12. It implements entry-local
> value bindings, same-schema contribution configuration, complete in-memory
> installation, borrowed typed requests and handlers, bounded actual candidate
> validation, descriptive discovery, output correspondence, and committed
> change observations. Its public consumer publishes cyclic planar groups and
> replaces a vertex with preserve/create/retire roles and same-commit lineage.
> Destination examples below also include wider APIs whose phase remains open.
>
> **Product posture:** One typed application experience over the certified
> in-memory Query/Relational/Bridge/Signal/World runtime.

The Pre-M0 public consumer uses a Query-free value package, two independently
compiled contribution packages, and one application root. It exercises bounded
cyclic allocation, actual candidate and untouched-neighbor validation, atomic
publication/readback, preserve/create/retire output roles, native committed
entity changes and same-commit lineage, exact denials, and idempotent recovery.
The external wrong-schema build fails at its schema-binding trait boundary;
owner proofs retain installation, stale-source, capacity, and authority checks.
The affected Bank public create/read/recovery journey passes with ordinary and
maximum-length names. Persistent Astra review cleared the final source and tests;
normal boundary/context checks, scoped formatting and dirty line caps pass.

Warm incremental builds measured on 2026-09-12 with the consumer workspace's
shared Query target took 9.90s after touching the declaration crate, 3.22s after
touching the topology entry, and 2.89s after touching the consumer root. Each
measurement changed only the source timestamp; these measure invalidation and
relink cost with compiled dependencies, not arbitrary semantic API changes.
The entry touch rebuilt that entry and the root; the consumer touch rebuilt only
the root. No Query dependency or generic bound enters the pure value package.

## Goal And Roadmap Placement

Make application intent the ordinary Query interface. An application declares
its meaning and explicitly registers its contributions; Query supplies the
binding, installation, request execution, publication, and managed lifecycle.
Bank, the existing UI/server consumers, and a cross-crate geometry-shaped
consumer must use that interface without sequencing Query's admission pipeline
in application glue. Query already owns admission truth; this removes repeated
consumer orchestration rather than inventing its authority for the first time.

[9.17.3](./milestone-9.17.3.md) remains certified: it establishes exact product
selection, carried authority, coordinated publication, conditional execution,
retained reads, independent branch progress, and cleanup. This milestone is
the post-closure application hardening prerequisite to
[9.18](./milestone-9.18.md); it does not reopen the historical certification of
the 9.17 branching umbrella. 9.18 adds semantic correction through this API.

The central claim is falsified if a new query or operation still needs an
application-owned select/resolve/admit/execute/publish pipeline; a pure value
crate must import Query; a convenient call loses occurrence, security, cost,
or recovery semantics; or Bank passes only through a parallel compatibility
path. Shorter names and forwarding wrappers alone do not close the milestone.

### Scope decision

Ship declaration bindings, coherent installation, request-bound reads and
mutations, history/paging/live use, existing workflow and conditional binding,
typed recovery, discovery, and complete Bank, `worth-ui-query-binding`, and
`crates/worth-server` adoption. Also ship the primary
relation-integrity, candidate-invariant, and runtime-cardinality contracts
needed to prove that a bounded geometry-shaped operation is a real consumer.
Those are necessary cross-crate foundations, not application prechecks.

Do not build the proprietary geometry kernel, engineering methods, renderer,
or AI agent here. Ship bounded Query-managed demand for the small synchronous
output group required by house M0. House M3 extends that same contract to
transitive regeneration and deferred numerical completion; native project
Save/Open remains the house M7E Query/Store handoff. Unsupported producer
strategies deny explicitly; no placeholder completion methods ship. Advanced
access/footprints and correlated execution remain 9.19/9.20.

## Inspected Boundary

Evidence baseline: public `73c6019677b959fc6fe9299abf5d23d7b9719a3d`.
The private house plans were inspected at
`c93b18e8277546d434d334abb1b1cb5406f69b2c`; their platform dependency still pins
`dcf0fd70162f489c2e6380525fe40fa25e63b451`. A future private adoption must select
the certified public revision explicitly; it cannot mix Git and path copies of
authority crates. These revisions are provenance, not compatibility promises.

The material current boundaries are:

| Evidence | Current responsibility or defect |
| --- | --- |
| `worth-query-bank-world/crates/bank-server/src/ordinary/` | Bank already offers `query/mutate -> as_principal -> controls -> execute`; its application glue implements that experience |
| `bank-server/src/application_query/execution.rs` and `ordinary/read/query/application_execution.rs` | Generic invocation plumbing and repeated per-query scope, identity-field, parameter, installation, execution, and publication binding |
| `bank-server/src/operation_commit/` and `estate_progression/` | Domain computation is interleaved with repeated Query progression and terminal adaptation |
| `worth-query-declaration/src/application_schema/values.rs` | `TypedApplicationValue: WorthQueryPortableType` attaches Query integration directly to domain values; a separate entry crate cannot implement these foreign traits for a foreign pure value |
| Declaration schema/query/operation macros and Bank manifests | Useful typed references already exist, but binding and membership/program information are repeatedly authored |
| `worth-query-execution/src/domain_computation/primary_graph/product_operation/` | Real selected-product admission and transaction progression; `transaction()` belongs to product entry, not the consumed selected read object |
| Primary `schema_layout/registry_lowering.rs` | Relation lowering installs default integrity, cross-context prohibition, and retained dangling audit posture; it does not install a domain's closed-loop integrity contract |
| Installation `application_operation/contracts/compilation.rs` and primary `provider/resource_support.rs` | Fixed primary invariant slot and declaration-width-derived resources, including a fixed retained-byte envelope; repeated creates alone do not prove admitted variable-cardinality construction |
| `worth-query/docs/capabilities/declarative-query-experience.md` versus `foundations/ordinary-application-front-door.md` | Competing ordinary-workflow explanations; the workspace string declaration language is not a substitute for the authenticated application product |

Paths in this table are relative to the named workspace's `crates` directory
unless a workspace is explicit. The recorded Bank compile failure is dominated
by portable-type bound cascades. It is evidence of a broken consumer, not a
count of independent architectural defects or proof that later server paths
compile. `compare_and_commit_application` currently exists; this milestone
internalizes its ordinary orchestration rather than claiming it was removed.

Bank has no green compile baseline at this revision. Phase 1 repairs its value
bindings directly into the destination contract; Phases 2/3 repair and migrate
the dependent read/operation paths. Do not first restore the superseded API.
Use certified 9.17.3 owner/public oracles and Bank's accepted product contracts
as independent expectations. Record existing diagnostics separately from new
failures; a cleared declaration target does not imply the server is green.

## Adversarial Constraint And Decisive Proof

The plausible false implementation is a friendly facade which caches authority
or latest state, moves the old pipeline into a product helper, generates
unstable identities, validates only an algorithm's buffer, or flattens partial
effects into an error that invites retry. The following worlds must convict it.
All actions enter through declaration and host audience facades and the actual
primary application composition root. Certification sidecars observe lower
owners; they do not create the action's authority.

### Bank: one consumer across time, permission, and effect boundaries

Install the existing Bank schema and real principal/capability bindings.
Use ordinary account creation/funding and immutable journal postings to give A
100 currency units and B zero. A transfer of 30 yields 70 and 30 under the
independent accounting oracle; balance is derived from authoritative postings.
Use the actual approved business-payment and estate journeys as additional
operation families, including distinct-approver and elevation restrictions.

1. Retain an account-activity read, open a paged read and live subscription,
   and prepare two mutations against the same selected source. Publish one.
   The retained read remains old, a new current request sees the successor,
   the competing attempt is stale before effects, and pages remain bound to
   their original ordering and occurrence.
2. Revoke the observer's access before delivering its queued live update and
   before resuming its continuation. Fresh delivery/readmission denies access
   with no protected payload. Retaining old data does not retain permission.
3. Substitute another account, branch, installation, request attempt, cursor,
   or bound query at the next applicable boundary. Equal visible ordinals do
   not make genuine foreign artifacts interchangeable. Include valid twins
   reaching the disputed provider or publication boundary.
4. Duplicate the transfer request with its stable idempotency key. No second
   journal or external transport contact occurs. Reuse that key with different
   intent: preserve the existing typed conflict, never silently replay it.
5. Exercise cancellation before effects, retained owner effects before product
   publication, and deferred settlement after performed publication. Recovery
   resumes the exact obligation and never executes the transfer again.
6. Preserve the 9.17.3 losing-outbox scenario: the owner row really exists,
   its composite publication lost, and neither initial dispatch nor redispatch
   becomes eligible, including after a later descendant inherits the row.
7. Hold A at a deterministic owner rendezvous while B completes unrelated
   branch work. The API introduces no global session lock. The shared Query
   Signal court below supplies the conditional proof; do not pretend Bank
   already installs that producer graph.
8. Exhaust installed subscription/retention capacity, close clients, retire the
   branch, settle retained obligations, and close the application. Counts
   plateau; cleanup neither evicts protected reads nor loses recovery custody.

The real Bank HTTP/user-node boundary must also run current read, mutation,
continuation, live revocation, and recovery adaptation. An in-process imitation
does not certify wire adaptation. Authentik and external-rail observations use
their existing honest lanes; a unit fixture's principal is not process proof.
Phase 3 explicitly adds Bank lifecycle forwarding to the public owner close/
retirement routes and installs the real external-rail transport in the process
composition used to certify rail recovery. Existing unit-only rail installation
is insufficient. Exercise response loss and owner recovery with independently
observed rail contacts; preserve the process court's real authentication lane.

### Shared Query Signal court

Reuse the certified primary composition with actual installed Bridge/Signal
bindings. Through the new public entry, select/read a branch, perform a World
publication, deliver its actual patch, and run the matching conditional under
fresh admission. Nonmatching patches invoke no producer; duplicate delivery
does not duplicate effects. Hold one branch at the owner rendezvous while its
sibling advances, preserve a retained predecessor read, then close all resources.
Observe real producer contacts and owner retention. Bank's prohibition on raw
`SignalGraph`/local scheduling is preserved; it does not prohibit Query-managed
Signal. No new Bank business rule is invented to host this shared proof.

### Geometry-shaped consumer: independent values and actual graph effects

Use a small public certification domain, not a dependency on proprietary code.
A separate pure value crate defines finite lengths and local numeric values
without Query. Two entry contributions bind modeled topology and a parameter
operation to one application schema. Register them explicitly at one root.

Construct 1, 10, and 100 independently placed box-shaped topology groups in one
bounded operation per requested group batch. Use explicit vertices, edge uses,
ordered cyclic incidence, face ownership, and model identity. The domain's
scope is the fixture's declared planar topology contract, not a certified B-rep
kernel or a building-safety claim. A simple independent combinatorial oracle
predicts counts, incidence, and coordinates from dimensions and placement.

- Reserve requested counts and bytes before allocating candidate identities.
  Allocate handles first, then populate fields and cyclic relations. One
  successful operation produces one World publication with a complete group;
  no per-vertex commit, subscription, or provider session is permitted.
- Inject zero required links, two links in a single-successor relation, a
  broken cycle, foreign-model endpoint, wrong unit/frame binding, and deletion
  with an untouched inbound neighbor. The
  primary owner's actual candidate validator rejects before publication.
  A valid algorithm result followed by a mislowered candidate must still fail.
- Change a tracked source after preparation and cancel a second attempt after
  allocation but before publication. Both leave the source graph unchanged,
  publish nothing, and release reservations; valid twins reach publication.
- Replace or delete a group while holding an old read. A fresh read sees the
  complete successor; the retained read sees the complete predecessor. No
  dangling live incidence is legalized by an audit tombstone.
- Admit a batch at the declared maximum and deny maximum-plus-one before
  allocation. Exhaust candidate bytes and validator work independently.
  There is no fake declaration widening to obtain more resources.
- Edit a Rust module or local marker name while retaining the declared semantic
  identity. Portable meaning remains identical. Change a representation/unit
  contract without the required identity/revision change and reject binding.
- Feed a contribution from another schema or installation into the root's
  binding. Compiler denial or installation rejection occurs before execution;
  no public generic marker supplies authority.

For demand, install a real producer deriving a bounded body summary from the
published group. A committed group alone cannot satisfy its demand. Delay its
patch, change the source, exhaust demand capacity, and cancel one of two shared
interests. Require exact-source `Superseded`, bounded custody, surviving sibling
interest, and settled output only after the actual producer publication and
required closure evidence. Missing required output is never vacuous success.

Retain 9.17.3's independent cost/occurrence observations. Instrument actual
sessions, selections, allocations, candidate touches, validator visits, World
publications, matching producer visits, and retained bytes. An oracle must not
call production lowering to compute its expected answer. Repeated create calls
passing isolated owner tests are supporting evidence, not this journey.

### Existing UI and server consumers

`workspaces/worth-ui/crates/worth-ui-query-binding` must derive and refresh its
real projections from the application entry, retain snapshot isolation, deliver
bounded granular invalidation, and release live resources on view retirement.
`crates/worth-server/src/worth_native/direct` must register its actual product
adapters and execute reads/projections through the same entry. Run affected
direct and HTTP adapter integration journeys, including stale projection and
fresh authorization denial, against their real roots. These are mandatory
consumer migrations, not examples covered by Bank's success. Source/export
deletion checks support these runtime proofs; they cannot replace them.

## Product Decision Lock

1. One application runtime and one admitted execution spine. Product adapters
   supply vocabulary, domain handlers, and external transport; they do not
   implement admission, selection, disclosure, publication, or recovery policy.
2. Explicit contribution registration owns membership. A declaration owns
   meaning once; code generation derives wiring. No filesystem scan, linker
   registry, reflection, or global registration order is semantic authority.
3. Pure meaning and numeric crates remain Query-agnostic. Entry-owned binding
   markers associate pure Rust values with Query contracts without requiring
   foreign-trait implementations on those values or wrapper values in solvers.
4. Portable identity is explicit, stable, and versioned. Rust spelling, process
   addresses, type names, numeric registration order, and ad hoc hashes do not
   define portable meaning. Existing aspect identity remains domain-owned.
5. App lifetime, request context, selected attempt, and retained resource are
   distinct. A long-lived app or request builder is not cached admission.
6. Each execution selects exactly once before admission and carries the
   occurrence. Explicit retained reads can reuse exact data; they readmit the
   request's current security. No later phase resolves latest.
7. Query internally owns the ordinary phase pipeline. Installed handlers see
   only the exact bounded read/candidate capabilities needed for their role.
   Typestate remains at meaningful read/effect and authority boundaries.
8. Declarations state the permitted effect ceiling. Actual effects are produced
   by the executed handler and owner candidate. Neither substitutes for the
   other. Generate duplicate wiring, never invent the effects of an algorithm.
9. Primary graph integrity is mandatory and declaration-owned. Engineering
   findings can describe a lawful but unsatisfactory design; they cannot waive
   hard graph validity or manufacture a complete-house assessment.
10. Every variable-cardinality mutation has a pre-allocation resource envelope.
    Complete atomic groups never become a series of visible scalar commits.
11. Local mutation, performed composite publication, durability settlement,
    external completion, and derived-output completion remain different facts.
    No ordinary `Result<(), String>` or success boolean erases their posture.
12. Ordinary current reads, pages, retained reads, and live views use one typed
    query binding. Their resource and security lifecycles remain distinct.
13. The default is bounded current-branch execution under installed policy.
    Branch choice, pinning, wider scope, special disclosure, external effects,
    retries, and changed guarantees require typed explicit choices.
14. Existing `worth-proof` carriers and Foundational vocabulary are reused.
    A new legality witness belongs in `worth-proof`; portable cross-runtime
    meaning in Foundational; counters, leases, tables, and Drop in the owner.
15. All migrated ordinary consumers cut over. No deprecated aliases, forwarding
    legacy methods, dual authority lanes, or Bank-only compatibility helpers.
    Historical specifications remain historical records, not current guides.

## Required API Experience

The namespaces and call shapes below are normative targets. Implementation
planning may refine private names and layout within the destination boundaries;
changing the caller semantics or responsibility split requires a spec revision.
Examples become executable public-facade examples when their phase ships.

### Audiences and discoverability

`worth-query-decl::facade` exposes declaration families: value bindings,
application contributions, queries, operations, invariants, capabilities,
effects, and existing conditional/workflow meaning. It executes no request.

`worth-query-host::facade` reexports owner-defined `application`, `installation`,
`handlers`, `resources`, and `inspection` entry namespaces. The host crate adds
no implementation, history, dispatch policy, or wrapper runtime. `application`
exposes the real hardened primary runtime and its request entry. Handler
contracts are a deliberate integration audience, not an alternate application
executor. `worth-query-replay` remains certification-only.

Only domain entry/composition code needs the host binding types. HTTP handlers,
UI actions, and agent tools consume generated typed intents and application
results. Numeric algorithms receive their specialized packed values, not
`HouseSchema`, field-reference generic tuples, runtime handles, or Query plans.

### Value binding and explicit composition

This destination declaration creates a local marker, not an implementation on
the `AccountId` value. The same contract works when the value is owned by a
separate crate, as the pure-value certification consumer must demonstrate:

```rust,ignore
worth_query_value_binding! {
    pub AccountIdBinding for crate::model::AccountId {
        identity: "worth.bank.account-id.v1",
        scalar: UInt64,
        encode: crate::model::AccountId::get,
        decode: crate::model::AccountId::new,
    }
}
```

Bindings carry representation, exact conversion, unit/frame identity where
applicable, validation, and portable contract identity. Decode failure is typed;
it is never truncation or a permissive default. Generic `Money<Currency>` binds
currency meaning explicitly; distinct currencies cannot share an accidental
identity. Composite inputs/results declare structured bindings; a field or
empty role marker obtains its identity from the owning declared contract instead
of requiring an unrelated manual portable-type census.

Existing `PORTABLE_TYPE_NAME` strings, explicit slot-role identities, native
aspect identities, and package archive encodings remain byte-stable where the
semantic contract is unchanged. Moving an implementation to a local marker
does not rename its portable protocol. Changed meaning needs an explicit
revision; do not rewrite archive goldens to bless incidental codegen churn.
Optional-field bindings preserve clearing as distinct from zero or an empty
value. A unit/frame binding describes representation; frame relationships and
cross-model transform validity still require graph-carried context evidence.

The value binding authorizes nothing. Installation checks it against the sealed
native aspect/field contract. Field/query/operation binding internals change
together so consumers do not manually bridge old value traits to new bindings.
Foundational-native codecs remain in their semantic owner; Query-specific
binding identity remains in declaration. Do not move Query contracts into
Foundational merely to make an orphan implementation legal.

```rust,ignore
worth_query_query_binding! {
    pub AccountSummary in BankSchema {
        identity: "worth.bank.account-summary.v1",
        input: AccountSummaryInput via AccountSummaryInputBinding,
        scope: AccountIdentity <- input.account,
        result: AccountSummaryResult via AccountSummaryResultBinding,
        definition: account_summary_definition,
    }
}

worth_query_application! {
    pub BankSchema {
        owner: "worth.bank",
        version: (1, 0),
        contributions: [BankAccounts, BankPayments, BankEstate],
    }
}
```

`definition` supplies the existing typed graph/query semantics, including
projection and disclosure. Binding associates input fields, scope resolution,
and result shape once. Generated constructors such as
`queries::account_summary(account)` return that typed intent; handwritten
product names may delegate to the same constructor without reimplementing it.

The value-marker example shows identity syntax, not a proposed rename for an
existing Bank wire type. Phase 1 uses the type's established identity where one
exists; currently unbound values receive explicit domain-owned identities.

Each contribution explicitly registers its definitions once; the root registers
contributions once. It does not repeat every child member. Cross-domain entry
contributions target one schema through an entry-owned typed binding contract
implemented by the root. Required entity/field/relation references are concrete
schema-derived types; installation verifies exact ownership and generation.
Do not introduce a universal runtime schema-fragment merger or a registry of
untyped references. Bank estate and the geometry-shaped fixture prove modular
composition before the proprietary house adds CAD/building/MEP contributors.

### Current foundation reference

The current host constructor is
`application_installation::in_memory(Schema::declaration()?, configuration,
limits, initial_state)`. Its configuration is the tuple derived from the root's
`ApplicationSchemaComposition::Contributions`; each entry configures only its
installed members through `WorthQueryApplicationContributionSetup`. It validates
contribution inventory before callbacks and handler/invariant completeness
before initial state. Current limits compose World, candidate, query, and
conditional resource profiles. The broader resource families in the destination
contract below retain their phase obligations.

The [host reference](../../workspaces/worth-query/crates/worth-query-host/README.md#contribution-composed-applications)
and [two-entry public consumer](../../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/main.rs)
show the implemented call shape. That consumer covers the focused foundation;
it does not establish completion of Bank, UI/server, history/paging/live,
workflow, recovery, or every remaining 9.17.4 obligation.

### Installation

```rust,ignore
let app = installation::in_memory(
    BankSchema::definition()?,
    bank_bindings,
    bank_limits,
    initial_state,
)?;
```

`bank_bindings` explicitly supplies authentication/principal mapping, installed
domain handlers, required invariant providers, clocks, and declared external
adapters. `initial_state` uses the existing governed bootstrap boundary and is
unavailable after publication. The constructor publishes a usable runtime only
after complete validation; it does not return a half-installed application.

Named typed resource profiles bind operation, read, page, live, retained-read,
history, recovery, and candidate limits. Different resource families retain
their dimensions; there is no giant options bag or universal scalar budget.
Absent required providers, incompatible schema contributions, identity clashes,
unsupported modes, and contradictory bounds return precise installation denials.
Installation derives canonical contracts once. Requests use generation-bound
installed references; they do not enumerate the schema or reinstall handlers.
Package signature verification and host trust selection remain explicit.

### Current requests and selected attempts

```rust,ignore
let request = app.request(&principal, &request_scope);

let summary = request
    .query(queries::account_summary(account))
    .execute()?;

let outcome = request
    .mutate(mutations::send_money(input))
    .idempotency(key)
    .execute();
```

`request` borrows `WorthQueryAuthenticatedExternalPrincipal<Schema>` and
`WorthQueryRequestScope` from the existing admission facade. Its construction
neither admits an operation nor pins a world. Principal input is proof from the
existing authentication/binding route,
never a principal ID, role string, or caller-defined authority marker. The
installed binding resolves and checks the request principal for each execution.

`execute`, `page`, and `subscribe` select one exact product observation when
they begin admission. Reusing the request builder starts a fresh attempt;
two calls are not implicitly a snapshot transaction. Queries return governed
published results with occurrence, ordering, disclosure, and cost evidence.
Idempotency is required by the mutation builder before `execute` is available.
Operation-specific preconditions remain typed refinements on that builder.

The application has one installed default branch. `request.on_branch(branch)`
chooses another Query-issued branch occurrence before execution; it performs no
component selection. `query.controls(read_limits)` and
`mutate.controls(mutation_limits)` narrow installed bounds. Defaults are visible
through inspection and retain the strongest ordinary guarantees. Retry is not
implicit; a stale candidate is never replayed under fresh authority unnoticed.

### Capability, delegation, and elevation admission

Capability definitions bind scope/action, all/any composition, purpose, and
disclosure to the intent once. Ordinary intent execution derives the exact
installed capability route; consumers do not repeat registry lookup or call
`admit_capability_access` themselves. If a caller chooses a narrower capability
or delegation, `.using_capability(&grant)` accepts the existing concrete,
owner-issued grant only. Current activation, revocation, scope, expiry, and
installation affinity are checked during the same single selected attempt.

```rust,ignore
let requested = request
    .request_elevation(elevations::estate_emergency_access(action))
    .idempotency(request_key).execute();
let preview = request.query(queries::estate_preview(estate))
    .with_elevation(&approved_elevation).execute()?;
let outcome = request.mutate(mutations::estate_action(input))
    .with_elevation(&approved_elevation).idempotency(action_key).execute();
```

`request_elevation` is a distinct declared intent binding to the existing
elevation-request admission/program/commit owner. It requests authority; it
does not grant elevated execution. Approval, revocation, and mandatory review
are separately bound workflow intents preserving distinct-approver rules.
`with_elevation` consumes the existing approved, scope-bound artifact as fresh
admission evidence; a request receipt, transport token, or boolean cannot fill
that role. Only bindings declaring elevation support expose this refinement.
Retained preview, page resume, and live delivery readmit the same requirements
under current authority. Query retains typed expiry, revoked, foreign-scope,
wrong-purpose, and pending-approval denials without a generic authority bag.

### Retention, paging, history, and live reads

```rust,ignore
let pinned = request.retain_read()?;
let old = request.at(&pinned)
    .query(queries::account_summary(account)).execute()?;

let page = request.query(queries::account_activity(account))
    .page(page_limits)?;
let next = fresh_request.resume(page.continuation(), page_limits)?;

let live = request.query(queries::account_activity(account))
    .subscribe(live_limits)?;
```

`retain_read` selects and retains an exact owner-issued occurrence under its
installed resource bound. `at` is read-only and does not cache authorization.
The retained read is not an operation permit. Joining several reads at one
occurrence uses this explicit read context. No mutation method exists on it.

`request.history(branch).page(history_limits)` uses World-owned bounded ancestry.
A protected entry's `retain_read()` asks the owner to issue its exact retained
observation. It does not construct authority from a commit identifier.
Mutation publication retention is explicit and reserved before owner effects:

```rust,ignore
let outcome = request.mutate(mutations::send_money(input))
    .idempotency(key).retain_publication(retention_limits).execute();
// Only a performed terminal carrying the admitted lease exposes this read.
let old = fresh_request.read_after(&retained_publication)
    .query(queries::account_summary(account)).execute()?;
```

`retained_publication` is the move-only lease-bearing projection of the
performed terminal, with its canonical receipt; it is not another commit fact.
The ordinary descriptive receipt cannot type-check as `read_after` input.
The default mutation adds zero client publication pins. Opt-in reserves one
bounded lease before effects, transfers custody on publication, and releases
unused capacity on pre-effect failure or `NoEffect`. Post-effect failures preserve owner
recovery and any still-required retention together. Cloning descriptive metadata
never clones a lease. Dropping/closing the read interest releases its pin, not
an unsettled recovery obligation. Foreign or retired owner affinity denies.

Idempotent replay cannot recover a lease from an ID: opt-in replay asks World
for bounded, exact retained-publication readmission. If unavailable, preserve
`AlreadyCommitted` and report typed retention unavailability separately; never
rerun the operation or relabel its effect as failure. History uses its existing
bounded protection route. No per-commit hidden pin or ancestry reconstruction
is permitted. Prove default zero/opt-in one client pin and capacity denial
before effects, independently of World's existing internal history retention.

Continuation state binds the installed query, input, scope, order, occurrence,
and disclosure. Resume takes a fresh request; authority never comes from the
cursor. A live resource accepts fresh delivery request evidence through
`live.next(&fresh_request)` and exposes bounded progress, overflow, cancellation,
and close. Transport adapters may retain an opaque cursor/lease token in their
bounded registry; it cannot authorize a later request.
Entity, region, and collection subscription selections remain declaration-bound
query modes. Preserve existing mixed-cause and granular invalidation semantics;
do not replace them with an application-wide refresh. Saved views bind the same
query/input/order contract and are readmitted rather than storing permission.

Dropping ordinary read resources releases their leases. Explicit `close()`
returns completed or pending cleanup with the existing typed custody. Neither
Drop nor a convenience method discards unsettled owner effects. Product close
fences new admission, releases disposable resources, and preserves the exact
recovery actions needed before owner retirement can finish.

### Operations and domain handlers

Each operation binds typed input/scope, the decision-read contract, effect
ceiling, invariant requirements, resource profile, and its domain handler once.
External-effect and aftermath choices stay explicit at declaration time:
`no_external_effect()` is a decision, not omission; so is `no_aftermath()`.
A macro may generate the existing typestate completion from explicit clauses.

Query drives principal/capability admission, operation projection, tracked
reads, completed dependencies, candidate construction, invariant execution,
owner preparation, World publication, settlement, and terminal publication.
An installed handler supplies domain calculations and candidate effects, not
an executor callback which receives the whole runtime.

The handler contract has two meaningful capabilities:

- **Decision reader:** exact projected source and bounded field/relation/
  aggregate/absence reads. Completion consumes the reader and seals actual
  dependencies. Domain data retained beyond it is value-only.
- **Candidate writer:** available only after completed reads and admitted
  candidate reservation. It exposes operation-permitted create, initialize,
  write, link, unlink, delete, and emit actions with program-affine handles.
  It cannot commit, select another source, execute an external effect, or
  disable required validators.

The binding-selected `OperationHandler<Binding>` contract has these required
roles; associated types carry the operation's input, decision, and domain denial
so an author does not repeat schema/scope/field generic tuples:

```rust,ignore
fn decide(
    &self,
    input: &Binding::Input,
    reader: &mut OperationReader<Binding>,
) -> HandlerResult<Self::Decision, Self::Denial>;

fn candidate_requirements(
    &self,
    input: &Binding::Input,
    decision: &Self::Decision,
) -> CandidateRequirements;

fn build_candidate(
    &self,
    input: &Binding::Input,
    decision: Self::Decision,
    candidate: &mut OperationCandidate<Binding>,
) -> HandlerResult<(), Self::Denial>;
```

These are trait-signature fragments, not standalone functions. Query calls
`decide`, ends the borrow, completes the reader's actual dependencies, checks
and reserves `candidate_requirements` under the installed ceiling, and only
then calls `build_candidate`. The requirement report is a request, never proof
of adequate resources. Candidates cannot exceed it. Decisions contain domain
values and checked local references; neither handler role receives publication
authority. Domain denial before owner effects discards the local candidate;
failure after owner effects retains the real terminal. Fixed-shape operations
generate the requirement method from their explicit contract. `decide` runs
under admitted decision/read/scratch bounds; it cannot allocate an unreserved
variable-sized output and ask for permission afterward. Numerical output
construction runs during `build_candidate` under the reservation. Numerical
helpers receive only values and their bounded work/scratch interface, whichever
handler phase invokes them.

`&self` is the explicitly installed handler instance; immutable domain
configuration belongs there, never in a global registry. Binding lookup is
once per attempt. After fresh admission and key/intent conflict checks, a
known idempotent result short-circuits before `decide`, handler decision projection,
candidate allocation, or external contact. Replay retention is a separate
bounded owner action as specified above.

Readers and writers expose an admitted work context with cancellation/deadline
checkpoints and scratch/work limits. `HandlerResult` preserves interruption
separately from domain denial. Query checks before each owner contact, between
declared work granules, and before publication; loops and numeric helpers check
the same context. The requirements method is bounded metadata arithmetic, not
another solve. Profiles bound records/bytes and maximum non-yielding work per
granule; unsupported uninterruptible algorithms deny installation/admission.
House defaults are 1,024 gather records and 4 MiB per contact, with a 4 ms
cooperative target to measure, not a preemption guarantee. M0 proves bounded
synchronous checkpoints; M3 adds owner-managed yielding/deferred completion.
Cancellation after effects retains the actual outcome and cleanup custody.

Fixed-shape declared programs may generate candidate-writing code. Dynamic
algorithms produce value-only candidate deltas which this same writer checks.
The effect ceiling and the executed effect set remain distinct inspectable
contracts. Bank's double-entry, sufficient-funds, and distinct-approver logic
remains domain code. Query owns complete tracked posting aggregates and rejects
incomplete membership evidence; the host cannot substitute a cached balance.

An expert may request an admitted preparation through
`request.mutate(intent).idempotency(key).prepare()` and execute that returned
attempt. It carries one selection and exposes explanation/cost, cancellation,
and execution only. It does not expose constructors for intermediate authority
or widen the operation. Ordinary `execute()` calls the same preparation path.

### Actual candidate integrity and bulk construction

Relations declare endpoint types, context/model scope, cardinality, live
reference requirements, and deletion posture. Required domain validators
declare identity/revision, applicability, complete affected-neighborhood reads,
enforcement, resource bounds, and typed diagnostics. Installation lowers these
contracts into the real primary provider and Relational registry.

Validation consumes the owner-sealed proposed graph combined with the selected
source and untouched affected neighbors. An algorithm's success return or
mutable candidate buffer is insufficient evidence. Carried proof and gathered
data may be reused within the same validated boundary; do not double an
expensive gather solely for ceremony. An operator cannot opt out of a required
rule or authorize a smaller closure by reporting fewer touched objects.

After tracked reads, a handler requests its actual entity/relation/field counts,
candidate bytes, validator work, and retained-output pressure under installed
ceilings. Admission reserves before allocation; allocation produces local
program-affine handles; population can then form cycles. Publication rechecks
actual consumption. Exhaustion never buys capacity through fake field width,
duplicate operation declarations, or a hidden unbounded candidate buffer.

This contract includes a bounded already-known affected neighborhood; it does
not claim 9.19 verified arbitrary search completeness or 9.20 correlated
set-oriented execution. Those add strategies through the same reader/writer
boundary. Existing graph/index/copy-on-write owners are extended where this
journey exposes a defect; the application may not build a parallel graph.

Producer-supplied output roles name correspondence, not identities. The writer
binds preserve/create/retire actions to actual source identities and allocated
handles, with explicit ambiguous/unmatched outcomes. Identity correspondence
and retirement co-commit with the candidate under platform lineage ownership;
no coordinate hash or UI side table defines identity. Phase 4 proves simple
replace/delete correspondence. House M2 supplies feature-specific split/merge
policies and ambiguity resolution through that contract. Sparse promotion
still requires the existing carried publication authority.

### Bounded output demand and settlement

```rust,ignore
let demand = request.demand(outputs::body_group(body))
    .controls(demand_limits).start()?;
let progress = demand.advance(&fresh_request)?;
```

`start` admits a bounded managed interest, fixing the selected authored source,
required output inventory/applicability, installed producer versions, disclosure,
deadline, cancellation, work, memory, and retention. `advance` drives an admitted
owner contact, never application traversal or producer polling. `Pending` owns
the continuation and owner wake/progress subscription; consumers wait on that
notification before another contact. No busy-loop `Continue` adapter ships.
The notification stream is `demand.notifications()`; it emits payload-free wake
signals. Protected progress/results require fresh delivery admission through
`advance`, including for a completed handle; reading its terminal resumes no
work, and `close()` releases interest through the existing managed owner.

Phase 4 supports a declared bounded output group produced by installed
synchronous producers from complete tracked authored inputs. It reuses real
Signal eligibility, patch delivery, managed-run capacity, and World publication.
All required members must have owner-validated completion evidence for the fixed
source. An unsupported dependency strategy denies; excluding an applicable
ancestor to fit this scope is forbidden. M3 extends these same handles/terminals
to sketch/profile/body transitive regeneration and deferred workers.

`Settled` carries one retained, selectable output occurrence and required-closure
evidence. It is a descendant publication for those inputs, not a mutation of
the selected historical world. Source drift before settlement is `Superseded`,
never automatic latest selection. Required missing output is `Incomplete`;
pending, dependency failure, denial, cancellation, timeout, exhaustion, and
owner recovery remain distinct. A completed failing domain finding can be
settled; completion does not mean a passing design. An ordinary committed
receipt or locally Fresh child cannot manufacture this terminal.

Demand coalesces only identical installed closure/source requirements and
compatible policy/budgets. Each interest retains fresh disclosure and independent
release; closing one must not cancel another's needed producer. Reservation,
queued progress, retained roots, and cleanup are bounded by actual owners.
Historical recomputation requires an explicit branch and fresh admission;
reading an already retained result does not schedule new work. M0 must use this
route before claiming body-derived output settled. No off-thread numerical
completion is implied by this first synchronous producer strategy.

### Outcomes, recovery, conditionals, and inspection

Mutation outcomes preserve the existing concrete variants and custody:
`Committed`, `AlreadyCommitted`, `Stale`, `ProductStale`, `NoEffect`, `Denied`,
`Cancelled`, `TimedOut`, `Deferred`, `Aborted`, `ProductUnpublished`,
`SettlementDeferred`, and `Indeterminate`, wherever the operation supports
them. Product/domain denial detail remains typed. An outer adapter cannot
translate a performed or unpublished effect into ordinary pre-effect failure.

`Committed` means composite publication, not external completion or completed
derived computation. Recovery enters from the exact carrier, e.g.
`app.recovery().resume_settlement(deferred, &fresh_request)`. Unpublished-owner
cleanup, indeterminate provider recovery, and external-dispatch inspection
retain separate methods and legal successors. `Indeterminate` provider state
must not be assumed to mean an external effect happened; external dispatch also
has its own unresolved posture. Readmission rules remain those of the existing recovery owner;
no new permission check may strand mandatory cleanup after capability loss.
Generic `retry()` is not a replacement for these distinctions.

Temporal and conditional definitions bind predicates, clocks, tracked source
projections, and typed operation intents once. Signal remains the eligibility/
scheduling owner. Query turns eligible evidence into fresh operation admission.
Committed patch delivery uses installed Bridge correspondence, not per-handler
dirty calls. Existing workflow continuations resume through fresh requests with
their own typed artifacts; this milestone adds no workflow language or scheduler.

`app.inspect()` supplies installation/support and resource information;
`request.inspect()` supplies governed query/operation discovery. Installed
declarations generate input schemas, units, scopes, effects, and availability
for UI/AI/transport bindings. Discovery is descriptive, not an executable
authority token. A decoded intent still validates its values and goes through
the ordinary fresh request. This is not an arbitrary string-query endpoint.
Result explanations retain domain-level causes and lazy, policy-controlled
details; diagnostics and replay do not become ordinary execution dependencies.

Aftermath discovery preserves reversible, compensatable, reconcilable, and
irreversible classifications, plus explicit absence of an aftermath contract.
It preserves the installed mechanism/authority distinction and legal actions; it
does not advertise unsupported correction. Consumer-kit compatibility is
validated at installation by schema/binding versions and required capabilities,
including the exact producer/consumer pair. Projection sharing also requires
matching source occurrence, input/order and disclosure contracts with separately
admitted consumer interest; sharing bytes never shares permission. Existing readiness tokens remain bound to
the exact installed query/source and causal evidence; inspection exposes
bounded explanations. Readiness cannot stand in for the demand closure above.
9.19/9.20 strengthen access/explanation strategies behind these same contracts.

### Bank provisional correction transition

9.16 explicitly treats linear undo/redo as experiments, excluded from Bank's
accepted closure. Phase 3 retires their Bank server commands, HTTP routes,
user-node actions, token variants, and provisional-only tests with the ordinary
`provisional_aftermath` export. No compatibility adapter, disabled command stub,
or success-shaped substitute survives. Preserve accepted aftermath inspection,
settlement, provider/external recovery, and mandatory estate review tests; split
mixed tests so removing an experiment cannot delete independent obligations.
Current product guides must state only available capabilities. Historical 9.16
evidence remains historical. 9.18 introduces accepted tree correction through
new typed intents, HTTP/user-node journeys, and independent correction proofs;
it cannot count the retired experiment as its implementation.

## Replacement And Abstraction Contract

| Existing surface or repeated responsibility | Destination | Treatment |
| --- | --- | --- |
| Value-owned Query conversion/portable bounds in application binding | Entry-local binding with associated pure value and explicit identity | Replace application binding requirements coherently; remove superseded public trait requirements after all affected consumers migrate |
| Empty role-marker identity declarations and redundant parameter plumbing | Owning query/operation declaration generates its binding | Delete duplicate declarations; keep distinct semantic roles typed |
| Root manifest re-listing child members | Explicit registered contributions owning their members | Replace repeated membership, not explicit registration |
| Eight-parameter field refs/nine-parameter invocation glue in handlers | Associated binding types inferred through typed intent | Internalize generic wiring; retain exact static affinity |
| Bank `execute_one_shot`/preview helpers and per-query execute clones | Query request/query execution | Delete application-owned progression; keep product intent constructors |
| Per-operation authorize/prepare/commit orchestration | Installed domain handler under Query execution | Move framework orchestration to its owner; preserve domain rules and real effects |
| Capability-scoped admission and operation authorization | Intent-bound capability plus optional concrete grant refinement | Internalize lookup/admission; preserve scope, all/any composition, delegation and fresh revocation |
| `authorize_elevation_request` / `compare_and_commit_elevation_request` | `request_elevation` typed workflow intent | Preserve request-only authority, idempotency and the existing specialized owner program |
| Elevated preview and approved-elevation mutation | Binding-gated `with_elevation` refinement | Freshly validate the approved artifact on execution and retained/live delivery |
| Caller select/resolve/access-context/admit/execute/publish chain | Request-bound execution and explicit retained reads | Remove from ordinary exports; handler-facing capability access only where needed |
| Direct ordinary `compare_and_commit_application` or manual `AdmittedChange` assembly | Mutation execution / admitted prepared attempt | Internal implementation consumes these mechanisms; no second ordinary commit route |
| Separate historical/preview query grammars and controls selecting another basis | Same query intent on explicit read context | Replace; current and retained semantics remain distinct |
| Receipt-to-history reconstruction and ad hoc ancestry search | Opt-in `retain_publication`, lease-only `read_after`, bounded history | Replace; descriptive IDs never mint observations or hide per-commit retention |
| Per-query page/live wrappers and transport-owned permission decisions | Query-owned page/live resources, thin bounded transport adapters | Replace lifecycle/security duplication; preserve wire types and actual fresh admission |
| Numeric controls recreated in every handler | Installed typed profiles with caller narrowing | Abstract repetitive defaults; preserve every actual bound |
| Manual conditional patch routing and domain wake orchestration | Installed Bridge/Signal binding to typed operations | Abstract framework wiring; preserve actual cause and owner service |
| Application terminal flattening and generic retry helpers | Exact outcome carriers and action-specific recovery | Delete lossy adaptation; domain presentation may classify without consuming custody |
| Public workspace string API taught as the ordinary application route | Typed application declarations and request execution | Remove competing ordinary exports/docs; required provider/cert mechanisms remain internal or cert-only |
| `provisional_aftermath` and Bank's experimental undo/redo product routes | Accepted aftermath/recovery now; explicit retirement, then 9.18 tree correction | Remove exports, commands and provisional-only consumers together; no replacement stub |
| Hardcoded primary invariant/default relation policy for all domains | Installed candidate integrity and domain validators | Extend actual primary/Relational owners; delete bypassing substitutes |
| Declaration-width capacity masquerading as runtime batch size | Actual admitted cardinality/resource reservation | Replace the insufficient contract; retain independently valid fixed-shape bounds |

The mandatory cutover includes Bank, Query examples/courts,
`workspaces/worth-ui/crates/worth-ui-query-binding`, and `crates/worth-server`,
including direct runtime/projection and affected HTTP adapter consumers.
Follow their actual dependencies to close every affected ordinary call site.
It is ordinary implementation planning, not a new persistent tracking system.
Do not delete unrelated lower-level provider capability solely because a name
resembles an old application API. Anything retained must have a named internal,
handler, or certification audience and no competing ordinary route.

## Authority And Lifecycle Ownership

The application root is shareable (`Send + Sync`) with thread-safe installed
owner adapters/handler configuration, as required by Bank's HTTP application
boundary. It adds no global request lock. Request contexts are lexical borrows;
selected attempts, readers/writers, and managed read/page/live/demand handles
are owner-thread confined and do not implement `Send` or `Sync`. Create those
handles on the servicing thread; sharing the root does not transfer a lease.
Transport tokens and immutable result DTOs may cross threads but authorize
nothing. Compile-pass root sharing and compile-fail resource transfer enforce
the destination contract; do not infer it merely from current Arc/Mutex fields
or add unsafe auto-trait implementations. House's dedicated model thread is a
product custody choice, compatible with this shareable root. Deferred workers
receive bounded values only; M3 returns completion to the authority owner.

| Product | Constructor / truth owner | Permits and consumer | Cannot establish |
| --- | --- | --- | --- |
| Value/query/operation binding | Domain entry declaration; installation validates | Describes portable meaning and associated Rust types | Authentication, graph truth, or execution authority |
| Installed contribution and handler binding | Query installation, exact schema/provider generation | Request executor binds supported declared work | Membership in another schema or generation |
| Request context | Query runtime borrowing concrete principal proof and request scope | Starts fresh attempts or readmits managed delivery | Cached capability, selected world, or transport-token authority |
| Selected/admitted attempt | Query execution consuming owner-issued observation and concrete proof carriers | Exact read or operation progression | Reselection, branch substitution, or new effect permissions |
| Read snapshot/history protection | World retention composed through Query | Exact read-only observation under fresh request security | Mutation or permission preservation |
| Opt-in publication read | World lease reserved through Query before effects | Retained exact published occurrence, fresh `read_after` admission | Retention from a descriptive receipt or a second commit fact |
| Demand interest / settled output | Query managed custody, Signal closure/readiness, World publication and retention | Bounded fixed-source progress and exact output read | Completion from a numerical return, local Fresh child, or absent required output |
| Completed dependencies / candidate reservation | Query/provider progression with concrete `worth-proof` carriers | Permitted candidate construction under exact bounds | Commit, broader footprint, or unchecked allocation |
| Candidate and invariant evidence | Actual provider/Relational proposed state and installed validators | Owner preparation and World publication | Performed publication from numerical success |
| Published result / committed carrier | Query consuming real performed publication/disclosure | Governed consumption and exact legal next actions | External completion, current engineering pass, or new admission |
| Live/page/recovery resource | Existing owning runtime lifecycle | Bounded continuation, delivery, recovery, close | Unbounded retention, reconstructed authority, or silent lost effects |

Place genuinely new legality witnesses in `worth-proof`, not a generic
`AuthorityMarker` contract. Keep live custody in execution and immutable
cross-runtime vocabulary in Foundational where it has that meaning. Verify
manifest dependencies before duplicating existing substrate vocabulary.
No new application/session wrapper owns another World or Signal runtime.

## Destination Directory And Module Skeleton

Paths below are under `workspaces/worth-query/crates/` unless stated otherwise.
`E` existing, `N` new, `R` replaced/refactored responsibility, `D` removed
ordinary surface, `S` committed successor destination, not an empty placeholder.
Every code/test file remains at most 400 lines absent an explicit exemption;
this specification grants none. Facades are exports only.

```text
worth-query-declaration/src/
  application_schema/                         E/R: existing schema meaning
    values.rs                                 R: binding-associated pure values
    value_binding/{identity,scalar,unit,decode}.rs N: distinct binding contracts
    contribution/{definition,membership,schema_binding}.rs N
    relation_integrity/{endpoints,cardinality,deletion}.rs N
  application_query/
    binding/{definition,input,scope,result}.rs N: one query's typed association
  application_operation/
    binding/{definition,input,handler}.rs       N: operation association
    candidate/{cardinality,resources}.rs        N: declared ceilings
    invariant/{definition,neighborhood,enforcement}.rs N
  application_schema_macro.rs                 R: existing typed declaration lowering
  application_query_macro.rs                  R: binding generation
  application_operation_macro.rs              R: binding/effect-contract generation
  value_binding_macro.rs                     N: local binding, never foreign impl
  application_contribution_macro.rs           N: explicit contribution generation
  portable_identity/                         E: canonical declared identities
worth-query-admission/src/                    E/R: authenticated/admitted read contracts
  authenticated_principal/                   E: principal and request-scope owner
  graph_read_access/                          E/R: capability-bound admitted read contract
worth-query-package-archive/                  E/R: binding codecs; unchanged-meaning goldens
worth-query-installation/src/
  application_schema/
    contribution/{closure,ownership}.rs         N: root composition validation
    value_binding/{validation,native_contract}.rs N
    relation_integrity/lowering.rs             N: exact registry contract
  application_query/binding/{compilation,lookup}.rs N
  application_operation/
    binding/{compilation,handler_installation}.rs N
    contracts/compilation.rs                  R: preserve one compiled contract
    invariant/{installation,closure}.rs         N
    candidate/resource_contract.rs            N
  application_resources/profiles.rs           N: typed installed profile meanings
worth-query-execution/src/domain_computation/primary_graph/
  application_entry/
    request/{context,branch,controls}.rs        N: context, no cached admission
    request/{capability,elevation}.rs           N/R: typed admission refinements
    query/{execution,retained,continuation,live}.rs N/R: ordinary query entry
    mutation/{execution,preparation,outcome}.rs N/R: one operation executor
    mutation/publication_retention.rs          N: explicit pre-effect lease reservation
    history/{page,publication_read}.rs          N/R: bounded owner retention
    recovery/{settlement,unpublished,provider,external}.rs N/R: separate custody
    lifecycle/{close,inventory}.rs             R: delegate to existing owners
    inspection/{catalog,support}.rs            N/R: governed derived discovery
    demand/{request,closure,settlement,interest}.rs N: M0 bounded synchronous group
    demand/{transitive_closure,deferred_progress}.rs S: house M3 extensions
  handler/
    decision/{reader,completion}.rs             N/R: projected reads and proof
    candidate/{reservation,allocation,population}.rs N/R
    candidate/identity_correspondence.rs        N/R: co-committed output roles
    work/{budget,checkpoint}.rs                N/R: bounded cooperative work
    invariant/{candidate_view,execution}.rs     N/R
    compute/{prepare,worker_completion}.rs      S: house M3; owner-thread return
  product_operation/                          E/R: internal exact World progression
  application_query/                         E/R: existing planner/read machinery
  schema_layout/registry_lowering.rs          R: installed relation integrity
  provider/                                  E/R: actual candidate and resource owners
  live_delivery/                             E/R: real publication/revocation path
  conditional_operation/                     E/R: installed causal reentry, no new scheduler
worth-query-execution/src/domain_computation/authorization/
  capability_registry/{delegation,elevation,elevation_lifecycle}.rs E/R
  {operation_admission,elevation_progression}.rs E/R: existing authorization owners
worth-query-publication/src/                  E/R: existing result/terminal owners
worth-query-decl/src/facade.rs                 R: declaration reexports
worth-query-host/src/facade.rs                 R: semantic audience reexports
  [ordinary phase-constructor and provisional exports] D
worth-query-replay/                           E: certification-only audience
worth-query-certification/
  examples/{ordinary_application,retained_application,application_recovery}.rs N
  examples/ordinary_product_workflow.rs        R: migrated public experience
  examples/advanced_product_branching.rs       R: same owner path, advanced choices
  tests/application_api.rs                     N: one integration target
  tests/application_api/{bank_affinity,candidate_integrity,resource_cost,demand}.rs N
  fixtures/consumer_values/                   N: one tiny Query-free value crate
  fixtures/consumer_entry/                    N: separately compiled contribution
  [existing cumulative World/Signal courts]   E/R: reuse valid evidence
```

The dominant axes are declaration meaning, installation validation, runtime
custody, and domain-specific handler capabilities. Their direction is
declaration -> installation -> admitted execution -> owner-backed publication.
Query application entry does not own a graph registry; candidate validation
does not own numerical geometry; inspection does not own authority. Refining a
large listed responsibility must retain these axes, not create `common.rs`,
`helpers.rs`, a catch-all `session.rs`, or a cross-owner `manager`.

The integration root includes its case modules in one binary. The two fixture
crates establish the required foreign-value boundary, not one crate per case.
9.19/9.20 extend these same values and entry contributions; their
`reference_domains/{geometry,bank_compliance}` modules own advanced court
orchestration only, never a second schema/value definition for the same fixture.

The lower-owner extension destinations, relative to the repository root, are:

```text
crates/worth-relational/src/
  schema/                                    E/R: relation registration/meaning
  mvcc/validation/{proposal_invariants,invariant_plan}.rs E/R
  validation/invariant_access/{metadata,execution}.rs E/R: actual candidate access
  validation/invariant_authority/             E/R: owner evidence/diagnostics
crates/worth-runtime-world/src/
  history/{catalog,retention,publication}.rs   E/R: exact publication read retention
  retention/registry/owner/                   E/R: actual bounded retention custody
```

These owners receive required integrity or publication-read extensions through
their existing facades. Query may not emulate them. The existing Signal/Bridge
declaration binding and execution owners remain responsible for conditional
and temporal mechanisms; this milestone changes their Query binding, not their
truth or scheduling ownership.

Bank placement, relative to `workspaces/worth-query-bank-world/crates/`:

```text
bank-domain/src/
  schema/{values,manifest,program_manifest}.rs R: binding + contribution meaning
  queries/                                   R: typed input/query/result bindings
  proposals/                                 E/R: domain computation only
bank-server/src/
  identity_runtime/installation.rs            R: explicit root/adapters/profiles
  ordinary/{read,mutation}/                   R: thin product naming/presentation
  application_query/                         D/R: remove Query pipeline copies
  operation_commit/                          D/R: move domain effects to handlers
  estate_progression/                        R: rules + installed handlers/continuations
  estate_progression/{undo,redo,undo_admission,redo_admission}.rs D
  query_binding/{handlers,profiles,transport_outcomes}/ N/R: Bank integration only
bank-http-adapter/                            R: fresh request + bounded wire adaptation
bank-user-node/                               R: real client contracts
bank-estate-certification/                    R: real product journeys
bank-external-rail/                           E/R: actual external-effect ownership
bank-courtroom/                               R: process evidence through the new API
```

`D/R` requires inspecting each file's real contents: delete framework glue,
move still-needed domain rules, and remove the emptied surface. It is not
permission to delete domain behavior. No product-local facade may keep the old
pipeline alive. The pure fixture crate and separate contribution prove the
cross-crate boundary without creating production CAD packages or imposing
Query dependencies on existing numerical crates.

Existing ordinary consumer destinations, relative to the repository root:

```text
workspaces/worth-ui/crates/worth-ui-query-binding/src/
  operation_live/{resource,retention,retirement}.rs R: public managed resources
  projection_invalidation/                     R: governed granular delivery
  [existing snapshot projection owners]        R: request-bound reads and isolation
crates/worth-server/src/worth_native/direct/
  {state,read}.rs                               R: real app installation/request ownership
  projection/request.rs                         R: typed query/projection intent
crates/worth-server/tests/                      R: affected direct/HTTP adapter courts
```

Preserve UI presentation, server protocol, and product adapter registration
semantics while replacing their ordinary Query calls. Query owns selection,
readmission and managed resources; UI owns projections/view interest, and
server owns authentication and transport adaptation. No replacement facade in
either consumer may reproduce the phase pipeline. Committed successors add
new view or adapter families under these existing semantic owners.

## Ordered Phase Plan

Each phase begins with `plan-implementation`, including the current owner
boundary, concrete DX, and directory population. Use `implementation-batch`
for coherent slices. Reapply planning at a repeatedly failing architectural
boundary; replace its design completely instead of patching symptoms.

The persistent reviewer applies `qa-loop`, `qa-tests`, and `code-quality-qa`
to substantial completed batches. Distinguish certification blockers from
optional suggestions. Preserve cleared findings and unaffected test evidence.
Certify the current phase, commit and push it, then begin the next phase.
Phase plans are inline decisions; do not create another proof ledger.

### Phase 1: Declaration Binding And Explicit Composition

> **Certified 2026-09-11.** Pure-value bindings, generic field/unit affinity,
> explicit contribution composition, root installation, canonical identity,
> archive round trips, historical relation semantics, hostile foreign-schema
> denial, and Bank declaration adoption pass through their production owners.
> Bank read, live, mutation, proposal, continuation, capability, and product-
> currentness regressions are green after the declaration cutover.

Make pure value binding and one-root contribution composition real. Replace
application value/query/operation binding contracts across declaration,
canonicalization, and installation as one coherent cutover. Generate mechanical
identity/parameter wiring from explicit contracts; preserve aspect truth.
Include admission and package-archive binding dependencies in the causal scope;
unchanged portable-meaning goldens must pass without regeneration.

Evidence is public compilation of the pure-value/separate-entry consumer,
Bank declaration adoption, exact portable contract round trips, and hostile
foreign-schema/unit/identity cases. No hidden Query dependency or wrapper-value
escape passes. Root-installed references, not a generic marker, open subsequent
work. The next phase may trust installed binding identity and closure.
Migrate mechanically affected declarations in UI/server and public courts in
this phase if their bindings change. Later consumer phases own their execution
cutover, not permission to leave new compile failures behind. Scope the known
Bank baseline explicitly; do not excuse a new dependency break as existing debt.

### Phase 2: Requests And All Existing Read Lifecycles

Ship installation entry, request context, fresh selection, typed query
execution, retained reads, bounded history/publication reads, paging, live
delivery, profiles, and relevant inspection. Migrate Bank ordinary and estate
read paths together, including special-purpose/elevation admission, HTTP
continuation, and live revocation. Remove their copied Query read pipelines.
Certify capability composition/delegation revocation and approved-elevation
preview versus pending request artifacts; preserve granular region/collection
delivery and saved-view readmission. Prove root sharing and handle confinement.

The read courtroom proves old/new separation, fresh security over old data,
foreign-artifact denial, bounded lifecycle, and exact publication retention.
Here publication reads use World history protection; mutation-side opt-in
retention and its terminal carrier are certified with Phase 3's effect boundary.
No long-lived session freezes current state or permissions. The next phase
inherits a usable public read experience, not just owner tests.

### Phase 3: Operation Execution And Complete Bank Adoption

Bind installed domain handlers and drive the complete mutation spine through
request execution. Install relation integrity and domain invariants against the
actual primary candidate, including complete affected neighborhoods, before
claiming Bank's exclusive posting relations or domain rules are enforced.
Fixed-shape handlers reserve their declared candidate requirements before
allocation. Move Bank's domain rules/effects behind these handlers;
delete its authorize/prepare/commit framework duplication. Migrate every
existing ordinary mutation family, estate transitions, workflows, conditional/
temporal invocations, external rail, and exact recovery adaptation.
Retire the provisional correction consumers described above in this same
cutover. Prove request/approve/use/revoke elevation, required review, and
capability-scoped estate actions through the typed bindings. Install the rail
in the real process court and add public lifecycle forwarding. Prove opt-in
publication retention, including replay and post-effect custody, and that
idempotent replay never invokes a domain handler.

Prove the accounting, approval, stale-attempt, cancellation, unpublished-outbox,
settlement, process-transport, and unrelated-branch journeys. No half-performed
state is flattened and no recovery reruns the operation. Bank must actually
pass through its public product and process paths before moving on; CAD demand
or off-thread workers are not prerequisites for this acceptance checkpoint.

### Phase 4: Bounded Graph Construction And Output Settlement

Extend the certified primary candidate/invariant boundary with actual runtime-
cardinality reservation/allocation/population and cyclic geometry-shaped
construction. Domain-specific topology rules use the installed invariant route
from Phase 3. Extend required lower-owner seams directly.
Use the already certified handler/runtime experience for the geometry-shaped
consumer; do not invent a special geometry executor.

Certify valid cyclic construction, hostile mislowering, foreign model, inbound
reference deletion, stale source, independent retained reads, and independent
resource exhaustion. Report structural amplification for 1/10/100 groups.
Prove output-role correspondence and retirement in the same publication.
Ship the bounded synchronous group-demand contract and its real producer court:
source drift, missing output, delayed patch, capacity, cancellation, shared
interest release, and exact selectable settled output. Reuse the shared Query
Signal court for conditional/sibling/retention evidence through the new entry.
This establishes the public foundation for house M0, not a geometry-kernel or
engineering certification. Bank affected integrity cases run as regression
evidence where the provider changes its shared boundary.

No phase may defer a prerequisite it actually needs to a later phase because
of these labels. Phase 3 must pull forward any candidate-resource machinery
needed for its honest fixed-shape admission; Phase 4 owns the additional
variable-cardinality and geometry consumer proof, not permission to leave Bank
integrity incomplete. Phase 2 may use the existing certified 9.17.3 mutation
entry to establish real read/revocation fixtures until Phase 3 replaces that
entry; this is internal staged implementation, not a compatibility export.

### Phase 5: UI And Server Consumer Cutover

Migrate `worth-ui-query-binding` and `crates/worth-server`, their actual product
adapter registrations, direct read/projection roots, live resource ownership,
and affected HTTP integration paths. Consume the certified request/handler/
resource contracts without product-local orchestration. UI source isolation,
granular invalidation and view retirement, plus server fresh authorization and
projection behavior, must pass through the real roots before this phase closes.
Dependency and compiler checks deny their superseded ordinary imports. This is
an implementation phase with its own runtime evidence, not final cleanup debt.

### Phase 6: Complete Audience Closure And Certification

Migrate every remaining affected ordinary export, in-repo entry consumer,
example, and guide. Close the compiled facade so raw phase construction,
workspace-string application execution, and provisional correction cannot be
selected as ordinary alternatives. Preserve internal/certification mechanisms
only with their proper audiences. Finalize discovery and domain-level errors.

The final public journeys exercise one sealed graph, real patch delivery and
conditional execution, sibling progress, retained data, honest outcomes,
recovery, and cleanup through the new experience. Reuse predecessor oracles
and binaries when valid; rerun affected boundaries after the final changes.
9.18 may then depend on this application contract rather than another facade
redesign. No phase claims milestone closure while an affected consumer is
broken or a replacement remains a forwarding compatibility shim.

## Documentation Deliverables

Documentation ships with each public capability, not only during final cleanup.

| Continuing audience | Authoritative document | Required change and evidence |
| --- | --- | --- |
| Query application integrators | `workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md` | Define contributions, installation, requests, handlers, controls, result semantics, read lifetimes, and recovery using executable examples |
| All Query readers | `workspaces/worth-query/crates/worth-query/docs/AI_README.md` | Normal current reference for owners and actual public workflows; no migration narrative or statements such as "legacy surfaces are removed" |
| Declaration authors | `worth-query-decl/README.md` and existing schema/declaration guides | Cross-crate binding, explicit identity/registration, generated intent, units, handler and invariant contracts |
| Host authors | `worth-query-host/README.md` | Semantic namespace map, installation adapters, controls, advanced preparation, lifecycle, typed outcomes |
| Query users choosing an API | `worth-query/docs/capabilities/declarative-query-experience.md` | Rewrite around the accepted typed application path; remove competing workspace-based ordinary examples |
| History/live/recovery consumers | Existing `foundations/branches-and-previews.md`, `capabilities/historical-diff-and-basis.md`, and domain/aftermath guides | Same query binding, exact retention, fresh security, overflow, performed versus settled, owner-bound legal next actions |
| Bank product/transport authors | `workspaces/worth-query-bank-world/docs/public-consumer-contract.md`, `banking-product-contract.md`, `process-transport.md` | Real current APIs, all operation families, continuation/live/recovery wire behavior, honest process evidence |
| UI adapter authors | `workspaces/worth-ui/docs/query-binding.md`, `application-lifecycle.md`, and `workspaces/worth-ui/AI_README.md` | Current root/request ownership, projection isolation and live disposal, checked with real UI binding tests; AI reference stays free of migration narrative |
| Server adapter authors | `crates/worth-server/docs/read-data.md`, `write-data.md`, `stream-results.md`, and `connect-another-backend.md` | Current typed product registration and transport adaptation, checked with affected direct/HTTP integration targets |
| House platform consumers | Private `docs/house/query-platform.md`, `milestones.md`, `rust-ui-architecture.md` | M0 bounded demand/checkpoints, M3 transitive/deferred extension, explicit root/handle thread custody; no claim the dependency pin already provides the specified API |
| Milestone implementers | This specification and `WORTH_query_roadmap.md`; 9.18 handoff | Replacement obligations and ordering; no claim that proposed APIs are already implemented |

Guide snippets are included from or compiled alongside owning examples/tests.
Documentation-only edits do not trigger a full runtime rebuild. Keep the
historical requirements and valuable operator catalog; they are not obsolete
API reference. Do not create a second feature guide, migration tracker, or
closeout ledger merely to summarize this milestone.

## Acceptance, Cost, And Verification Discipline

Acceptance requires the full consumer journeys, public compiler boundaries,
exact outcomes, real candidate rules, and absence of competing ordinary lanes.
Compilation alone and isolated owner success do not close a phase.

The API adds no schema-wide lookup, history scan, graph reconstruction, or
canonical re-encoding to each ordinary request. Installed binding lookup is
direct or bounded indexed lookup; principal/security checks retain their real
cost. Query execution remains O(admitted graph work + output); candidate work
is O(materialized effects + required invariant closure + actual owner index/
copy-on-write work). Do not claim constant time for a shell-wide invariant.
Continuation/history work is charged for examined entries, including vacancies,
not only returned rows. Closure, not superficial line count, determines the
needed graph gather. The framework does not do a second numerical solve.

Structural observations separate selections, authority checks, provider reads,
candidate allocation, validation, publication, delivery, retention, and
reconstructive work. Each accepted mutation batch has one selected attempt/provider
session and one successful product publication; subsequent derived producer
publications are counted separately, never hidden inside that claim. A denied pre-allocation batch
has zero candidate allocations and zero publication contacts. Unrelated
branches and producer routes retain the predecessor's independent progress and
bounded matching-work guarantees.

Measure cold/warm compile and focused rerun costs for the relevant public
examples and Bank target. Reuse one compiled fixture crate per responsibility
and shared existing test support; do not make a new crate/test binary for each
negative case. Pure algorithms and ordinary UI code must not instantiate the
application schema's generic graph. Public compiler tests and manifest
boundaries, not source-text approximations, enforce that separation.

Keep execution ownership in `worth-query-execution`; this milestone does not
create a speculative wrapper/application runtime crate. Generate small binding
adapters and hand off to existing runtime owners, limiting schema-wide generic
instantiation to entry code. Measure the execution/host/consumer rebuild chain
explicitly; locating entry code inside execution does not isolate its rebuild.
If measured iteration cost requires a physical crate split, reapply planning to
that demonstrated boundary and revise this topology before implementation;
never split authority merely to improve a stopwatch. Bank's first green target
is the new-binding baseline; its current failed build is not a valid before
timing for functional or runtime performance comparisons.

Finish verification of a frozen batch before expanding it. Build the affected
targets once, reuse their binaries, and rerun only checks invalidated by later
changes. Reuse existing independent oracles and cleared review findings.
Expensive process, scale, and scheduled lanes run when their actual boundary
changes or final certification requires them, not after every local edit.
Use deterministic synchronization for races; time spent compiling is not proof.

For boundary-relevant implementation, run the governing repository tools:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
scripts/ci/check_workspace_rust_line_caps.sh dirty
```

Also run affected owner/public integration tests, formatting, specification-
required lint and process proofs. Preserve the currently requested quick
line-check CI; this milestone does not restore the old CI pipeline. Required
local certification still applies. Generated `AGENT_CONTEXT.md` files are tool
outputs, never hand-edited. Report unrelated untouched debt separately; scoped
constitutional failures block completion. Follow
[test-requirements.md](./test-requirements.md) rather than creating evidence
inventories or tests which only certify other tests.

## Complete Feature Adoption And Successor Handoff

This is the consumer-facing disposition of the house Query adoption matrix.
It is not a second implementation tracker or a promise that every future
capability ships in 9.17.4.

| Requirement family | 9.17.4 destination | Remaining governed handoff |
| --- | --- | --- |
| Entities, aspects, fields, units, relations, optional values, typed identities | Entry-owned bindings, explicit composition, complete native contracts | Domain schema additions remain domain work |
| Queries, projections, operations, effects, aggregates | Typed intent, installed handlers, real tracked reads and candidates | 9.19/9.20 add advanced strategies/correlated execution |
| Packages, archives, signatures, adapters, policy, catalog, support | Coherent installation and descriptive discovery; current trust required | Store persistence does not come from portable definitions |
| Principal, capability, purpose, disclosure, delegation, elevation | Fresh request/attempt/delivery admission under installed contracts | Domain rules and actual identity-provider evidence remain mandatory |
| Traversal, ordering, paging, collections, saved views, scopes | Same installed query binding with explicit mode/resource lifetime | Search completeness and verified footprints remain 9.19 |
| Invariants, relation integrity, atomic output groups, cardinality | Actual primary candidate validation and reservation | Private kernel supplies geometry/body-class predicates; no manifold-only platform default |
| World occurrence, publication, branches, retained reads, ancestry, cleanup | Query request/resources over the existing owners | Semantic multi-parent merge remains a later cross-runtime contract |
| Idempotency, external effects, outbox, recovery, aftermath | Exact terminals and custody through ordinary execution | 9.18 tree correction; house M3B source correction consumes accepted semantics |
| Signal dependencies, producers, conditional/temporal/workflow work | Installed bindings, causal delivery and bounded synchronous group demand; existing scheduling owner | House M3 extends transitive settlement and deferred-compute integration |
| Output freshness, demand, off-thread numeric work | M0 fixed-source bounded group handle, exact settled occurrence, Superseded and independent shared-interest custody | M3 extends required-output closure and value-only workers with owner-thread revalidation/publication; no placeholder deferred strategy |
| UI, AI, regions, inspection, explanation, consumer kits, lineage | Governed typed discovery/results/resources; one entry experience | House UI/AI consumes the facade; durable identity lineage remains platform truth |
| Native Save/Open, durable restart, replay | Accepted package APIs remain definition-only; replay cert-only | House M7E Query/Store snapshot/restore with fresh authority; broader durability M10 |

The M3 handoff extends M0's distinction between immutable selected source and a
settled descendant output occurrence. It must preserve required inventory/applicability,
method versions, complete dependencies, freshness, cancellation, and consumer
release without introducing another scheduler. Settled does not mean passing.
It extends `application_entry/demand` and `handler/compute`; it cannot move
ordinary declarations into solvers or replace the application entry again.

9.18 consumes typed intents, exact retained source selection, installed
operation/aftermath bindings, and fresh mutation/recovery entry. An accepted
correction creates a new World occurrence; it does not re-export the provisional
API. 9.19-9.22 attach their stronger access, footprint, evidence, and reuse
products behind the same binding and handler boundaries, with their own real
proofs. They must not introduce another ordinary consumer language.
