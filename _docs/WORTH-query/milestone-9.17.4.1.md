# Milestone 9.17.4.1: Unified Semantic Application Authoring

> **Status:** Planned post-M0 milestone. Begin after the CAD M0 demo closes,
> using certified 9.17.4 Phase 1, the Pre-M0 application foundation, and M0's
> real geometry consumer evidence before resuming the remaining 9.17.4
> migrations.
>
> **Product posture:** Authors declare domain truth, actions, lifecycle,
> authority, derived truth, and guarantees. Query compiles those semantics into
> explicit contracts consumed by its existing governed runtime.

## Goal And Roadmap Placement

Give Worth one high-level language for application meaning and one core
application IR for facts that cannot be derived. Both compile into the same
validated, inspectable, bounded runtime contracts.

> Authors declare semantics. Query derives obligations.

This is a planned authoring correction inside
[Milestone 9.17.4](./milestone-9.17.4.md). It follows the proprietary CAD M0
demo. M0 uses the certified focused Query foundation and implements only
additional Query surfaces demanded by its real cube/extrusion journey. It does
not wait for this semantic language. The completed M0 consumer then supplies
the concrete geometry code and measured repetition this milestone will replace.
The first extrusion and the existing Bank world already show the same defect:
operation authority, admission reads, schema installation, lifecycle rules,
dependency paths, producers, invalidation, and publication consequences are
repeated across independent surfaces.

This milestone does not reopen the certified 9.17.3 runtime, 9.17.4 Phase 1
publication foundation, Pre-M0 certification, or M0 product proof. Remaining
9.17.4 migration work resumes through this model.
[Milestone 9.18](./milestone-9.18.md) consumes the same installed operation and
recovery contracts.

Query's declaration, installation, and execution crates own the semantic
language, canonical application program, core IR, validation, lowering,
installation, explanation, and semantic revision adoption. Audience facades
only re-export the surfaces appropriate to their callers. Query-agnostic
application crates own domain vocabulary and algorithms; entry crates own the
bindings from that meaning into Query. CAD owns BREP meaning, topology grammar,
geometry, roles, tolerance, and geometric predicates. Bank owns account,
payment, posting, approval, and financial meaning.

## Current Boundary And Demonstrated Defect

The runtime owners are correct. Their contracts are still the ordinary
authoring interface.

In CAD:

- operation traits and schema installation repeat read, write, create, link,
  and emission authority;
- bindings repeat input, result, denial, scope, source, handler, identities,
  and candidate limits;
- each profile repeats a large output-role inventory and its consequences in
  creation, correspondence, candidate ceilings, and preservation;
- create and preserve repeat producer and readiness families; and
- invariant targets, dependency reads, affected-scope walks, and installation
  repeat related facts.

Bank demonstrates the broader application problem:

- `InitiateBusinessPayment`, `ApprovePayment`, and `RejectPayment` are semantic
  actions, yet markers, effects, reads, preconditions, capabilities, programs,
  and membership are authored separately;
- `PaymentStatus::ApprovalRequired -> Committed | Rejected` is a real domain
  lifecycle whose transitions and admission are distributed across proposal
  and installation code;
- `ApproveBusinessFunds`, distinct-approver policy, balanced postings,
  idempotency, activity, and retained readback form one contract spread across
  owners; and
- derived account and payment projections lack one owner for dependency and
  freshness semantics.

Query already has `AuthoredQueryBundleRequest -> CanonicalQueryBundle` and typed
query predicates, paths, projections, and aggregates. The missing boundary is a
semantic application program that reuses that machinery and lowers into the
existing application runtime.

Fixed inventories are insufficient for boolean, fillet, offset, intersection,
split, and trim results whose cardinality is known only after governed
computation. Query cannot demand a static output list, and CAD cannot replace
governance with an unrestricted vector.

## Adversarial Constraint And Decisive Court

The plausible false implementation adds attractive macros while retaining
several authoritative inventories. It can pass a happy path while handler code
widens effects, schema registration disagrees with intent, a dependency or
invalidation is forgotten, an illegal transition is admitted, or direct and
generated definitions coexist for one identity.

The decisive court has three real consumers:

1. **Fixed CAD topology.** Triangle, rectangle, and pentagon extrusions lower
   from one feature family. Typed roles, counts, correspondence, candidate
   requirements, materialization routes, invariants, and membership derive
   from it. Adding pentagon changes the semantic profile family only.
2. **Generated CAD topology.** A split or boolean-like feature proposes bounded
   variable output through a protocol-bound writer. Roles, lineage, effects,
   retained bytes, and work remain inside installed envelopes before atomic
   World publication.
3. **Bank business payment.** The existing Bank entry executes initiation,
   approval, rejection, and committed readback from one semantic feature.
   `ApprovalRequired -> Committed | Rejected`, `ApproveBusinessFunds`, distinct
   approver, balanced posting, idempotency, account activity, history, and
   external-effect posture compile into the same core IR and runtime.

The hostile sequence proves:

- order permutations canonicalize identically;
- duplicate features, actions, states, roles, and core identities reject;
- a handler cannot add an undeclared read, write, create, link, emission, or
  capability;
- declarative expressions derive exact dependencies, and removing an edge
  changes the canonical program and turns proof red;
- a custom geometry predicate with a missing, ambiguous, stale, or unbounded
  dependency rejects;
- illegal payment transitions and self-approval deny before publication;
- generated CAD output with an unauthorized kind, relation, write, role, or
  lineage rejects;
- underestimated effects, bytes, or work deny and release their reservation;
- invariant failure after private candidate construction publishes nothing;
- a sibling branch progresses during hostile denial;
- retained readback sees lawful Bank and CAD publications; and
- cleanup leaves no reservation, candidate, continuation, or resource leak.

The court observes installed contracts, actual effects, World occurrences,
retained reads, sibling progress, resource counters, history, and managed
resources independently. Macro snapshots are supporting evidence only.

## Product Decision Lock

### Two compilation layers, one canonical program

Ordinary authors use:

```text
ApplicationFeatureSpec
  -> truth, actions, lifecycle, authority, derived truth, guarantees,
     lineage, history, materialization, publication, evolution
  -> CanonicalApplicationFeature
  -> semantic expansion
```

Semantic expansion produces the public advanced core IR:

```text
ApplicationOperationSpec
ApplicationOutputProtocol
ApplicationMaterializationSpec
ApplicationInvariantSpec
ApplicationModuleSpec
  -> ValidatedApplicationProgram
  -> installed runtime contracts
  -> existing Query execution owners
```

The core IR is an advanced escape hatch for semantics that cannot be derived,
including numerical geometry predicates and generated output protocols. It is
not a second runtime or competing ordinary API.

Every direct or generated definition enters one identity space and validator.
A module cannot contain a generated and direct definition for the same
identity. Generated facts cannot be overridden at the core layer. Direct core
definitions must be complete. Installation does not branch on authoring route.

| Core definition | Authoritative facts |
| --- | --- |
| `ApplicationOperationSpec` | Identity, input/result/denial, scope, source, decision reads, non-output effects, handler, idempotency, resources, output protocol |
| `ApplicationOutputProtocol` | Fixed or generated output language, roles, create/preserve/replace/retire effects, lineage, structural rules, manifest validation, resource envelope |
| `ApplicationMaterializationSpec` | Trigger or source, variants, posture, exact operation, readiness, guarantees, publication |
| `ApplicationInvariantSpec` | Identity, execution point, canonical predicate or custom binding, bounded dependencies, affected scope, resources |
| `ApplicationModuleSpec` | Explicit membership, ordering constraints, active semantic revision |

### Feature and action ownership

`ApplicationFeatureSpec` is the primary high-level owner. It owns scope,
authoritative truth, actions, domain lifecycle, application capabilities,
derived truth, guarantees, lineage, history, materialization, publication, and
semantic revision.

`ApplicationActionSpec` bridges intent to `ApplicationOperationSpec`. It owns
action identity, typed input, scope, precondition, semantic capability, state
transition or effect intent, optional custom computation, idempotency, output,
and publication meaning.

Query derives mechanically implied input/result/denial posture, reads, writes,
creates, links, affected derived facts and guarantees, publication obligations,
and admission dependencies. Facts that cannot be derived remain explicit on
the action or referenced core definition. Arbitrary Rust handler behavior is
never inferred.

Implementation receives capability-narrow, protocol-bound inputs. It cannot add
authority, effects, states, dependencies, or publication meaning absent from
the installed feature.

### One typed semantic expression language

Authority predicates, preconditions, derived truth, and declarative guarantees
share one typed expression IR. It extends Query's existing authored and
canonical expressions, predicates, paths, projections, aggregates, identities,
planner, and bounded read-access analysis.

This milestone adds no competing evaluator, planner, string expression
language, reflection system, or captured-closure authority lane.

Expressions use typed entity, field, relation, state, principal, capability,
value, unit, and aggregate references. Deterministic canonicalization and
static analysis derive bounded dependency paths, affected scopes, admission
reads, invalidation edges, and resource requirements.

Declarative guarantees and derived facts do not repeat dependency lists.
Custom predicates use a typed binding plus explicit bounded dependencies
because arbitrary code cannot be analyzed honestly.

### Lifecycle and materialization

`ApplicationLifecycleSpec` means domain state progression:

```text
PaymentIntent: ApprovalRequired -> Committed | Rejected
Revision: Draft -> Review -> Released -> Obsolete
```

It derives or constrains transition actions, illegal-transition denials,
state-dependent authority and immutability, affected derived truth, and
publication.

The old `Initial`/`Preserve` lifecycle terminology ends. Core routes use
`MaterializationPosture::{Create, Preserve, Replace, Retire}`. The cutover
renames the concept at its owners without aliases or parallel vocabulary.

### Semantic authority

`ApplicationAuthoritySpec` and typed `ApplicationCapability` references express
domain permission in terms of application truth. They do not mint platform
authority or replace `worth-proof`.

Expansion compiles capabilities and predicates to required reads, purpose and
disclosure posture, installed ability requirements, and admission rules.
Runtime evaluates them with fresh concrete principal and platform authority. A
role name alone opens no door.

For Bank, `ApproveBusinessFunds` remains an installed ability scoped to the
payment. The semantic predicate also requires an approval-eligible payment and
a principal distinct from the initiator. Handler code cannot bypass either.

### Derived truth and invalidation

`ApplicationDerivedFactSpec` distinguishes authoritative from reproducible
truth. It declares:

- a typed expression or custom producer;
- dependency and affected-scope semantics;
- on-demand, synchronous-correctness, or asynchronous managed materialization;
- freshness, staleness, and read posture;
- resource bounds; and
- lineage and history posture.

The compiler derives producer inputs, dependencies, readiness, and invalidation.
Application code has no imperative invalidation call. Existing Signal,
producer, candidate, publication, retention, and resource owners govern
materialized facts. Derived state never becomes independent authority.

### Guarantees

`ApplicationGuaranteeSpec` owns a business or domain law. Declarative
guarantees compile expressions into canonical invariant predicates,
dependencies, affected scopes, and read authority. Custom guarantees bind code
only through a core invariant with explicit bounded dependencies and work.

Protocol structural checks, application guarantees, candidate invariants, and
external-effect posture remain distinct stages.

### Output protocols and one proposed delta

`ApplicationOutputProtocol` has two postures:

- `FixedOutputProtocol` expands a typed generator into an exact role manifest
  and exact structural quantities.
- `GeneratedOutputProtocol` declares a typed authority envelope, structural
  grammar, role and lineage contracts, and bounded resources. Execution
  produces a deterministic realization manifest.

Both share role identity, manifest validation, correspondence, admission,
invariant progression, and publication. A domain validator sees only proposed
effects and manifest, declared source and prior-output projections, and
installed policy. It cannot perform ambient reads.

The manifest is an immutable projection of the existing application effect
program. A protocol-bound writer appends effects with role and lineage metadata
as one operation. Effects and manifest cannot diverge.

```text
installed operation + admitted request
  -> reserved resources
  -> proposed application delta
  -> protocol-validated delta
  -> isolated owner candidate
  -> invariant-validated candidate
  -> coordinated World publication
```

The manifest grants no authority. Installation supplies the maximum envelope;
admission narrows it. Actual effects, bytes, and work are charged during
construction. An overrun denies and releases resources.

### Materialization compilation

`ApplicationMaterializationSpec` owns:

```text
trigger or source product x variant x materialization posture
  -> exact ApplicationOperationSpec
  -> source expectation and output protocol
  -> required guarantees and publication
```

Producer, readiness, conditional, applicability, and route identities derive
from it. Installation rejects missing or out-of-module operations, duplicate
routes, source/protocol mismatch, and unsatisfied lineage.

### Semantic revision and adoption

Changing the declaration is governed. This milestone ships:

- `ApplicationSemanticRevision`, the identity of a canonical program;
- `ApplicationSemanticDiff`, a deterministic explanation of change;
- `ApplicationAdoptionImpact`, classifying unchanged, additive, restricting,
  meaning-changing, and state-migration-required changes; and
- `ApplicationMigrationRequirement`, the proof or rewrite required before
  installation.

One revision and one owner-issued application-program generation are active per
installed module. An installation is explicitly branch-scoped or it governs an
owner-issued inventory of product branches. Adoption identifies the exact
current revision, proposed revision, application module, owner generation, and
every governed branch with its current World head. Validation runs the new
program against that exact coverage. The installation owner then compares and
advances the program generation together with World-owned semantic-adoption
occurrences. Branch creation, branch-head movement, coverage change, or program
movement returns a typed stale-adoption denial.

Every admitted operation, candidate, continuation, page/live reader, and
publication attempt carries the generation under which it was admitted. Once a
new generation activates, old admitted mutations and candidates cannot publish,
and old continuations and live/page readers close with a typed
semantic-revision-changed outcome that requires fresh admission. Adoption
either drains bounded in-flight custody before activation or advances the
generation and makes every remaining old handle fail its next owner check. It
cannot silently leave captured authority usable.

Already-performed occurrences and their recovery obligations are different
from callable application APIs. Adoption inventories them and requires a typed
disposition: satisfied, explicitly carried by the new revision, or retained as
an exact occurrence-bound recovery contract. The latter is usable only through
current recovery/correction authority for that performed occurrence; it does
not reinstall or expose the old application operation. Adoption blocks if any
obligation has no lawful disposition.

A revision cannot replace the current revision when retained state may violate
new semantics until migration and validation proof succeeds. Historical
revisions are descriptive evidence only. They are not ordinarily callable,
authoritative, co-installed, or exposed through parallel APIs.

This phase does not build an automatic migration engine. It builds the diff,
exact-basis adoption gate, generation transition, outstanding-custody and
recovery disposition, typed requirement, and proof seam.

### Membership and explanation

`ApplicationModuleSpec` is the explicit membership root. Features derive their
core definitions; direct advanced definitions are named once beside them.
Registration remains explicit Rust composition. No filesystem, Cargo package,
or source discovery is introduced.

Validated and installed programs explain semantic facts, identities, derived
core definitions, authority, dependencies, lifecycle, materialization, output
and resource posture, membership, revision, and adoption impact.

The library explanation model is required. A thin `worth explain` view may use
an existing suitable tool host. This milestone does not justify a general CLI
framework. If no host exists during planning, an executable example proves the
model and command presentation remains a later choice.

## Required Public DX

Exact syntax may change in phase planning. Semantic ownership and authored
information may not.

Ordinary CAD authoring must be approximately:

```rust
worth_feature! {
    ExtrusionRealization {
        scope: ExtrusionFeature,
        truth: [ExtrusionProfile, ExtrusionExtent],
        action: RealizeExtrusion {
            source: ExtrusionRead,
            compute: RealizeExtrusionHandler,
            produces: CadBodySet {
                protocol: BrepTopology,
                lineage: Required,
            },
        },
        variants: [Triangle, Rectangle, Pentagon],
        materialization: [Create, Preserve],
        guarantees: [
            ValidTopology,
            ManifoldSolid,
            ValidTrims,
            GeometryConsistent,
        ],
        publication: BodySetPublished,
    }
}
```

Ordinary Bank authoring must be approximately:

```rust
worth_feature! {
    BusinessPayment {
        scope: PaymentIntent,
        truth: [PaymentAmount, PaymentSource, PaymentDestination, PaymentStatus],
        lifecycle: {
            PaymentIntent: ApprovalRequired -> Committed | Rejected,
        },
        actions: [
            InitiateBusinessPayment,
            ApprovePayment {
                when: payment.status == ApprovalRequired,
                requires: ApproveBusinessFunds(payment),
                guarantee: approver != payment.initiator,
                transition: payment.status -> Committed,
            },
            RejectPayment {
                when: payment.status == ApprovalRequired,
                requires: ApproveBusinessFunds(payment),
                guarantee: rejecting_principal != payment.initiator,
                transition: payment.status -> Rejected,
            },
        ],
        derived: [AccountBalance, PaymentSummary],
        guarantees: [BalancedPosting, ExactlyOncePayment],
        history: Complete,
        publication: AccountActivity,
    }
}
```

These examples express ownership. Bank money proposals and CAD geometry remain
custom computations behind narrow bindings.

The advanced/core API remains approximately:

```rust
worth_application_operation! {
    RealizeExtrusion {
        input: RealizeExtrusionInputBinding,
        result: RealizeExtrusionResultBinding,
        denial: RealizeExtrusionDenialBinding,
        scope: ExtrusionFeature by ExtrusionFeatureKey,
        source: ExtrusionReadQuery,
        reads: extrusion_source_reads(),
        output: ExtrudedProfileProtocol,
        emits: [BodySetPublished],
        handler: RealizeExtrusionHandler,
        resources: ExtrusionResourcePolicy,
    }
}
```

The public flow is:

```text
feature declaration
  -> canonicalize
  -> validate
  -> explain or diff
  -> install
  -> execute through existing entry
```

Semantic adoption must be similarly direct:

```rust
let proposed = BusinessPaymentFeature::canonicalize()?;
let diff = installed.diff(&proposed)?;
let requirement = installed.adoption_requirement(&diff, current_world)?;
let prepared = application.prepare_adoption(
    installed.revision(),
    proposed,
    current_world.governed_branch_coverage(),
    current_authority,
    requirement.satisfied_by(migration_proof)?,
)?;
let adopted = world.publish_semantic_adoption(prepared)?;
```

The exact API may change, but the caller names the proposed program and current
authority while owner-issued values carry the exact source revision, program
generation, governed branch coverage, World heads, migration proof, and
recovery dispositions. The returned adoption identifies the single newly
active generation.

Errors identify the feature, semantic clause, generated core definition,
violated rule, and repair boundary. Authors can inspect why an action reads or
writes a fact, an invariant runs, derived truth invalidates, or adoption blocks.

## Authority And Truth Ownership

- `worth-query-declaration` owns semantic and core declaration vocabulary,
  canonicalization, semantic expansion, validation contracts, and descriptive
  explanations.
- `worth-query-installation` owns validated-to-installed lowering, semantic
  adoption, compiled dependency impact, and installed explanations.
- `worth-query-execution` and the existing narrower runtime owners retain
  admission, candidate progression, resource accounting, graph mutation,
  publication, retained readback, Signal, and managed-resource behavior.
- `worth-query-decl` and `worth-query-host` are audience facades. They re-export
  the declaration and installed/runtime surfaces their audience is allowed to
  consume; they do not implement or own those semantics.
- `worth-proof` concrete authority remains mandatory.
- Query-agnostic schema/domain crates own domain types, states, units, and pure
  laws. Entry-band crates own Query feature, action, capability, guarantee,
  protocol, and handler bindings. Domain algorithms remain in their semantic
  owners and are called through those bindings.
- cert crates own reconstruction and replay proof.

Canonical and installed programs are immutable. Execution does not
re-canonicalize meaning. Markers, strings, roles, historical revisions,
explanations, and diffs grant no authority.

## Replacement And Cutover Contract

This milestone is a cutover. Each phase names a real consumer slice. After its
new declaration produces the installed contract and its court passes, the
superseded traits, schema calls, bindings, producer/readiness wiring,
dependencies, invalidation code, and registration for that slice are deleted in
the same phase.

No adapter preserves old authoring paths. No old and new declaration for one
identity coexist. No historical revision remains callable. Tests may use a
pre-cutover contract as a temporary oracle during a phase; certification
deletes it and proves runtime observations.

## Destination Directory And Module Skeleton

```text
workspaces/worth-query/crates/worth-query-declaration/src/application_program/
  feature/{authored,canonical,validation,explanation}.rs
  action/{authored,canonical,expansion}.rs
  lifecycle/{authored,transition,expansion}.rs
  authority/{capability,predicate,expansion}.rs
  derived_fact/{authored,truth_posture,expansion}.rs
  guarantee/{declarative,custom,expansion}.rs
  expression/{authored,canonical,dependency_extraction,validation}.rs
  evolution/{revision,semantic_diff,adoption_impact,migration_requirement}.rs
  core/{operation,output_protocol,materialization,invariant,module}.rs
  program/{canonical,validation,explanation}.rs
  semantic_expansion/{operation,lifecycle,authority,derived_fact,guarantee,module}.rs

workspaces/worth-query/crates/worth-query-installation/src/application_program/
  installation/{validated_program,installed_program,semantic_adoption}.rs
  inspection/{program,semantic_diff}.rs

workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/
  application_attempt/output_protocol/{writer,manifest,validation,denial}.rs
  application_installation/{operation_view,materialization_route,invariant_impact}.rs

workspaces/worth-query/crates/worth-query-decl/src/facade.rs
  re-export declaration authoring only
workspaces/worth-query/crates/worth-query-host/src/facade.rs
  re-export installed and execution audience only

workspaces/worth-query-bank-world/crates/bank-server/src/application_definition/
  business_payment_feature.rs
  business_payment_actions.rs
  business_payment_lifecycle.rs
  business_payment_guarantees.rs
  business_payment_derived_truth.rs

worth-proprietary/crates/worth-cad-entry/src/application/
  extrusion_feature.rs
  extrusion_actions.rs
  extrusion_output_protocol.rs
  extrusion_materialization.rs
  extrusion_guarantees.rs
  split_feature.rs
  split_output_protocol.rs
```

The Bank and CAD entry destinations contain Query bindings over domain-owned
types and algorithms; they do not absorb pure banking, BREP, geometry, or
material meaning. No catch-all module, generic compiler bag, proc-macro crate,
build script, generated source tree, package scanner, or package-management
framework is permitted by this milestone.

## Ordered Phase Plan

M0 closure is the entry condition for Phase 1. Each phase begins with
plan-implementation, includes DX and directory
placement, implements one consumer-backed batch, receives substantial review,
and closes only after certification. Reuse compiled binaries and cleared
findings until changed code invalidates them.

### Phase 1: Core Program, Operation Spec, And Minimal Fixed Protocol

- freeze the completed M0 cube/extrusion journey as the real before-cutover
  behavior and compile-cost baseline;
- establish the canonical core program and one lowering path;
- ship `ApplicationOperationSpec` and the minimum fixed protocol for a real
  create operation;
- cut one CAD rectangle operation and one public fixture to the core IR;
- prove authority, installation, binding, admission, World publication,
  retained readback, sibling progress, and cleanup; and
- delete migrated manual declarations.

### Phase 2: Feature, Action, And Typed Expression IR

- ship feature, action, truth references, and canonical feature;
- extend existing Query expression and canonical query machinery;
- lower one real Bank payment action into the same operation contract;
- prove handler code cannot widen reads or effects; and
- delete migrated Bank repetition after runtime equivalence.

### Phase 3: Lifecycle, Semantic Authority, And Declarative Guarantees

- ship lifecycle, capabilities, authority predicates, and declarative
  guarantees with required core invariant support;
- cut Bank payment initiation, approval, and rejection;
- derive state preconditions, `ApproveBusinessFunds`, distinct-approver denial,
  effects, affected guarantees, and publication; and
- prove illegal transition, stale permission, and self-approval denials through
  the real Bank entry.

### Phase 4: Derived Truth And Invalidation

- ship derived facts, truth posture, freshness, resources, and automatic
  dependency/invalidation;
- ship the minimal `ApplicationMaterializationSpec` lowering required for
  on-demand, synchronous-correctness, and asynchronous managed derived facts;
- cut real Bank balance and payment-summary projections;
- prove stale derived truth is unobservable under its declared posture,
  including retained history and restart/rebuild evidence;
- reuse Signal and producer owners; and
- delete migrated invalidation wiring.

### Phase 5: Complete Fixed And Generated Output Protocols

- complete fixed roles and triangle/rectangle extrusion correspondence,
  lineage, structural resources, and preservation;
- prove pentagon's exact manifest, counts, and correspondence at the protocol
  boundary without installing its producer/materialization route;
- ship generated grammar, resource envelope, estimator, protocol-bound writer,
  and manifest validation;
- cut one real split or boolean-like CAD operation; and
- prove hostile role, lineage, effect, estimate, work, and invariant denials.

### Phase 6: Materialization And Custom Invariant Compilation

- complete `ApplicationMaterializationSpec` for CAD variant/posture route
  expansion and rename old lifecycle modes without compatibility vocabulary;
- derive CAD producer, readiness, conditional, and applicability routes;
- install pentagon through the completed materialization compiler;
- complete custom invariant bindings and bounded geometry dependencies;
- prove impact, bounded fan-out, candidate invariants, publication, readback,
  sibling progress, and cleanup; and
- delete migrated Cartesian wiring and dependency inventories.

### Phase 7: Semantic Evolution And Explanation

- ship revision identity, diff, adoption impact, migration requirement, and
  proof seam;
- preserve authored-fact provenance through expansion and installation;
- prove one lawful additive adoption succeeds on an exact current basis;
- prove a tightened Bank guarantee cannot adopt over incompatible retained
  state without migration proof, concurrent movement makes adoption stale, an
  old admission cannot publish, old readers require fresh admission, and every
  performed recovery obligation has a lawful disposition;
- prove one incompatible governed sibling branch blocks adoption, while
  compatible sibling heads retain their exact state and can progress after a
  lawful adoption;
- prove only one revision is active and old APIs are absent; and
- provide library explanation plus the narrow executable presentation selected
  during phase planning.

### Phase 8: Module Closure And Full Cutover

- make the module the one membership root;
- finish Bank payment, fixed CAD, and generated CAD consumers;
- remove every superseded route in migrated scope;
- run the three-consumer court, examples, compile-fail authority proofs,
  residue checks, compile-cost audit, boundary checks, and line-cap guard; and
- update roadmap, consumer, and `AI_README.md` references to describe current
  ordinary and advanced APIs directly.

## Documentation Deliverables

Update the Query API guide, Bank payment journey, CAD fixed/generated authoring
guide, roadmap handoffs, and operator inventory where operators actually ship.
Update `AI_README.md` as a normal current reference for where meaning,
algorithms, core IR, installation, and proofs belong. Documentation does not
narrate removed surfaces, migration history, or abandoned architecture.

## Acceptance, QA, And Cost Discipline

The milestone closes only when:

- a nontrivial feature is authored mainly as truth, actions, lifecycle,
  authority, derived truth, and guarantees without manual operation authority,
  dependency paths, producer/readiness wiring, membership, or invalidation
  where derivable;
- implementation-only changes cannot widen installed application meaning;
- direct core IR and feature expansion converge on one canonical program;
- declarative expressions reuse Query and derive bounded dependencies;
- custom predicates declare honest bounded dependencies and work;
- fixed and generated CAD reach World publication and retained readback with
  sibling progress and cleanup;
- real Bank initiation, approval or rejection, balanced/idempotent
  consequences, history, and denials pass through its entry;
- derived truth is never observed stale outside its declared posture and
  remains reconstructible;
- semantic adoption blocks incompatible state until migration proof;
- lawful adoption advances one owner generation on an exact World basis, stale
  admissions cannot publish, and performed recovery obligations remain
  reachable only through current occurrence-bound recovery authority;
- module-wide adoption validates exact current coverage of every governed
  branch, and an incompatible sibling blocks the change;
- one active revision and one membership root remain;
- no superseded route remains in migrated scope; and
- public examples compile against the real facade.

Use focused owner tests, affected integration proofs, and the smallest decisive
cross-crate court. Compile and test times are primary costs. Reuse compiled
binaries, do not repeat unaffected lanes, and do not add tests that mirror
implementation.

Measure cold compilation of changed declaration/installation/execution owners,
warm rebuild after touching one feature declaration, warm rebuild after
touching one domain protocol, and the affected Bank or CAD consumer rebuild.
Runtime counters cover proposed effects, bytes, dependency hops, invariant
work, candidates, and publications. Canonicalization, semantic expansion,
diffing, explanation, and whole-program validation remain cold installation
work rather than ordinary request work.

Before each boundary-relevant phase closes, run:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
scripts/ci/check_workspace_rust_line_caps.sh dirty
```

The historical broad CI workflow is not reinstated. The current quick line
check remains unless separately changed.

Certification blockers are specification violations, authority widening,
dishonest fixtures, absent runtime observations, resource leaks, stale derived
truth, revision bypass, competing authorities, and failed required checks.
Optional ergonomics and unrelated debt remain separate.

Repeated failure at one boundary triggers a fresh boundary plan stating root
cause, replacement design, DX, and directory placement. Complete the cutover
and reopen only evidence invalidated by changed code.

## Must Ship And Must Preserve

Ship the high-level language, advanced core IR, one canonical program, typed
semantic expressions through Query, lifecycle and authority compilation,
derived truth, guarantees, fixed/generated output protocols, materialization,
semantic revision adoption, explanation, and decisive Bank/CAD consumers.

Preserve concrete `worth-proof` authority, entry-only Query consumption,
cert-only replay, existing World/occurrence/candidate/Signal/publication/
retention/recovery/resource owners, bounded ordinary work, typed roles and
lineage, stable identity and currentness, domain-owned algorithms, and the
pause on Worth UI work until its separately planned update is available.

## Successor Handoff

Remaining [Milestone 9.17.4](./milestone-9.17.4.md) phases migrate Bank, server,
and other consumers through this facade. Missing Query surfaces are implemented
when a real feature needs and can test them.

[Milestone 9.18](./milestone-9.18.md) receives one active revision, one
canonical installed program, explicit migration requirements, and unchanged
correction/recovery authority. It may not create a parallel authoring, history,
authority, or application runtime lane.
