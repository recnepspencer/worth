# Milestone 9.18: Tree-Based Semantic Undo And Redo

## Goal

Replace Milestone 9.16's provisional linear current-head undo/redo experiment
with an accepted tree-based correction product over the composite runtime-world
history governed by [Milestone 9.17](./milestone-9.17.md) and completed through
[Milestone 9.17.3](./milestone-9.17.3.md), using the application contract hardened
by [Milestone 9.17.4](./milestone-9.17.4.md). Undo and redo
select exact committed occurrences and publish newly admitted composite commits
on an exact product branch. They never erase history, move a hidden stack
cursor, replay old authority, reinterpret a receipt as permission, or assume
that reversing Relational truth alone restores the selected Signal world.

One correction may advance Relational while retaining the exact Signal basis,
advance or reconcile Signal while retaining Relational, or coordinate changes
in both. Every unchanged component remains pinned to its exact prior basis.

Later cross-runtime milestones extend this accepted single-parent correction
contract through semantic merge, rebase, multi-parent publication, durable
recovery, offline synchronization, and distributed collaboration rather than
replacing it.

## Roadmap Placement

Milestone 9.16 accepted aftermath classification, retained pre-images,
retained governed input, external-effect causality, recovery handles, and
publication while explicitly withholding product acceptance from its existing
undo/redo lane. Milestone 9.17.1 supplies exact component bases and Relational
branch-local MVCC; 9.17.2 supplies the product branch, composite single-parent
history, and coordinated target-head authority; 9.17.3 supplies complete Query
carriage and public branch/history authority. Milestone 9.17.4 then supplies
entry-owned bindings, fresh request execution, installed domain handlers, and
the retained-history/recovery experience this milestone extends. Correction
must use that same application entry, not restore caller-owned phase plumbing
or the provisional API.

9.17.4 retires Bank's provisional undo/redo server commands, HTTP routes,
user-node actions and experimental-only tests together. Accepted aftermath
inspection and owner recovery remain available. This milestone must install
new tree-correction intents and complete Bank transport/user-node journeys;
it cannot assume a supported linear correction route survived the cutover.

```text
exact committed source occurrence
    -> exact source composite world commit
    -> selected target product branch and exact composite head
    -> installed correction and per-component posture
    -> current principal, capability, purpose, policy, and definitions
    -> owner-local inverse, compensation, reconciliation, or reapplication plans
    -> composite applicability and invariant admission
    -> owner-local preparation
    -> Runtime World coordinated compare-and-publish
    -> new composite commit with exact component bases
    -> typed aftermath and next actions
```

## Current Boundary

- Query owns installed operation meaning, authority admission, invariant
  execution, aftermath, and the provisional correction surface.
- Relational owns authoritative truth changes and the exact inverse or
  compensation inputs installed for those changes.
- Signal owns Signal definition and derived-execution branch semantics; its
  branch state is not inferred from Relational history.
- the base Runtime Bridge owns installed semantic correspondence; the dedicated
  Runtime World owner owns composite branches, history, and publication
  progression completed by Milestone 9.17.2 and carried through Query by
  Milestone 9.17.3.

The missing product law is not merely “apply the inverse.” It is:

> Given one exact historical product world and one exact current product branch,
> which component changes, retained component bases, compensations, and derived
> reconciliations can lawfully produce a new current product world?

## Adversarial Constraint

A source composite commit changed Relational truth and caused a corresponding
Signal branch advance. It has two descendants:

- one product branch contains an intervening disjoint Relational change while
  retaining the source Signal basis; and
- another contains a conflicting Relational change plus an independently
  advanced Signal definition branch and expired caller capability.

An external effect from the source cannot be reversed, but its installed
compensation remains available. The caller attempts to:

- undo a copied receipt rather than the exact source occurrence;
- target an equal-version foreign Relational or Signal branch;
- reverse Relational state while silently selecting ambient latest Signal;
- reuse the source Signal snapshot after its definition basis became
  incompatible;
- redo with replacement input or original authority;
- hide divergent alternatives behind a linear redo stack;
- publish after the selected composite target head advances; and
- cancel after one component prepares but before composite publication.

The honest path must:

- preserve every original component and composite commit;
- admit a lawful disjoint correction as a new composite commit;
- retain an unchanged component at the exact selected basis;
- require Signal owner reconciliation when the corrected Relational or
  definition meaning invalidates the selected Signal basis;
- classify conflicting or incompatible divergence before effects;
- require fresh authority;
- compensate rather than counterfeit reversal;
- retain every redo alternative after divergence;
- expose partial preparation without product-visible half currentness; and
- lose the atomic race when the composite target head changes.

A Relational-only inverse test, hidden linear stack, bridge-bypassing component
publication, or Signal `latest` lookup must fail this courtroom.

## Product Decision Lock

1. Undo and redo are canonical new-history operations. Neither deletes,
   rewrites, uncommits, or moves past history.
2. The base Runtime Bridge owns installed semantic correspondence. The Runtime
   World owner owns composite commits, product branch references, composite
   currentness, and coordinated compare-and-publish.
3. Relational owns authoritative truth changes, Relational branch history,
   owner-local inverse inputs, conflicts, and publication candidates.
4. Signal owns Signal branch history, definition-bound execution meaning,
   reconciliation, derived lifecycle, and owner-local publication candidates.
5. Query owns installed correction meaning, fresh admission, operation lowering,
   typed progression, public DX, and aftermath projection. It owns no component
   or composite head.
6. Domain packages own semantic inverse and compensation meaning. Query,
   Runtime Bridge, and Runtime World cannot infer an inverse from touched scope, a before/after
   diff, or apparent component equality.
7. Every correction names an exact source operation occurrence, source
   composite commit, target product branch, target composite head generation,
   component bases, operation version, and current actor authority.
8. Every component receives one explicit posture: retain exact basis, apply
   owner-issued inverse, apply compensation, reconcile/rebuild derived state,
   reapply retained meaning, or reject as unavailable/irreversible. Omission is
   not “unchanged.”
9. Retaining a component means retaining the exact source or target basis
   selected by the admitted plan. It never means resolving that component's
   ambient current head.
10. Signal derived values are not reversed as authoritative truth. They are
    retained only under exact basis equivalence or reconciled/rebuilt through
    Signal authority from the corrected authoritative and definition basis.
11. A lawful Signal definition change may itself have an owner-declared inverse
    or reapplication. Query cannot treat definition rollback as cache invalidation.
12. A tree has no implicit redo top. Reapplication selects an exact prior
    operation or correction lineage and preserves alternative descendants.
13. Redo is fresh execution of retained governed meaning. It is not replay,
    cached output reuse, retry, or reuse of prior authorization.
14. Relevant divergence produces typed cleanly-applicable,
    revalidation-required, component-reconciliation-required,
    conflict-requiring, stale-head, non-invertible, unavailable, or
    indeterminate posture before effects.
15. Current authentication, capability, purpose, disclosure, elevation,
    conflict-of-interest, definitions, invariants, and idempotency are re-entered.
16. External-effect reversal is admitted only by the external owner. Otherwise
    Query exposes compensation, reconciliation, or irreversibility honestly.
17. Owner-local preparations become product current only through Milestone
    9.17.2's composite publication progression as carried by 9.17.3. A failed
    or losing correction
    leaves no product-visible half correction.
18. The provisional Milestone 9.16 lane is evidence, not authority. It is
    accepted, revised, or deleted only through this milestone's cutover.
19. Merge, rebase, multi-parent correction, offline synchronization, and
    distributed crash recovery remain in the cross-runtime roadmap.

## Destination Topology

```text
worth-query-installation/src/application_aftermath/correction/
    contract.rs
    component_posture.rs
    recorded_inverse.rs
    compensation.rs
    reconciliation.rs
    reapplication.rs

worth-query-admission/src/application_aftermath/correction/
    source.rs
    target.rs
    component_plan.rs
    applicability.rs
    authority.rs

worth-query-execution/src/domain_computation/application_aftermath/
    reversal/
        intent.rs
        admission.rs
        progression.rs
        evidence.rs
    reapplication/
        intent.rs
        admission.rs
        progression.rs
        evidence.rs
    correction_history/
        causality.rs
        alternatives.rs
        retention.rs

worth-runtime-world/src/correction/
    preparation.rs
    component_outcomes.rs
    coordination.rs
    publication.rs

worth-relational/
    existing correction inputs and owner-local candidate publication

worth-signal/
    existing basis compatibility, reconciliation, rebuild, and branch lifecycle

worth-query-publication/src/application_aftermath/
    reversal.rs
    reapplication.rs
    correction_outcome.rs

worth-query-certification/tests/application_aftermath/
    composite_reversal.rs
    composite_reapplication.rs
    divergence.rs
    signal_reconciliation.rs
    external_effects.rs
    authority.rs
```

The correction tree is organized by semantic meaning in Query, owner-local
mechanics in Relational and Signal, and cross-runtime coordination in Runtime
World. Forbidden placements include a Query-local history store, a Runtime World-
implemented domain inverse, a Relational-owned Signal selection, and a generic
undo manager that hides component posture.

## Phase Plan

### Phase 1: Correction Contract And Occurrence Identity

Install exact operation-level inverse, compensation, reconciliation,
reapplication, retained input, pre-image, and irreversibility contracts. Bind
every candidate to the exact committed occurrence, source composite commit,
component bases, and operation schema version. Scope-only or receipt-only
correction becomes unrepresentable.

### Phase 2: Composite Tree Selection And Applicability

Expose owner-backed inspection of source composite commits, target product
branches, current composite heads, exact component bases, and correction
alternatives. Compile applicability against relevant intervening composite and
component history, current definitions, policy, retention, and authority before
effects begin.

### Phase 3: Owner-Local Component Correction Plans

Lower the admitted correction into explicit per-component retain, inverse,
compensation, reconciliation, rebuild, reapplication, or denial plans. Each
runtime validates and prepares only its own meaning. Query and Runtime World
may coordinate those plans but cannot recreate them.

### Phase 4: Coordinated Reversal And Compensation Publication

Execute the owner-local plans through ordinary runtime boundaries and publish
one new composite commit through Runtime World's coordinated
compare-and-publish progression. Preserve the source and every descendant.
Denial, cancellation, stale head, and conflict move no product head; retained
owner effects without product movement receive the exact
`ProductUnpublishedOwnerEffects` lifecycle posture.

### Phase 5: Reapplication And Divergent Redo

Admit reapplication from exact retained governed meaning under current
authority and current component definitions. Preserve multiple alternatives
after divergence; never collapse them into a mutable redo stack or silently
discard them after a new edit.

### Phase 6: Public Facade, Documentation, And Provisional Cutover

Publish the composite branch/history/aftermath workflow through
`worth-query-decl` and `worth-query-host`. Revise the application-aftermath
feature guide to teach composite source and target selection, component
retention, Signal reconciliation, fresh authority, compensation, divergence,
and external-effect limits. Delete or migrate every provisional surface and
test that encodes the rejected linear or Relational-only product.

### Phase 7: Hostile Certification

Use an independent composite-history oracle across copied receipts,
equal-version foreign component branches, stale target heads, disjoint and
conflicting divergence, incompatible Signal definitions, replacement inputs,
expired authority, compensation, irreversible effects, cancellation, response
loss, partial preparation, and concurrent correction. Mutation of occurrence
binding, exact component retention, reconciliation admission, fresh authority,
or composite compare-and-publish must turn the court red.

## DX Target

```rust
let choices = bank
    .history(product_branch)
    .corrections(committed_operation)
    .inspect()
    .await?;

let undone = choices
    .reverse_on(product_branch.head())
    .as_principal(principal)
    .purpose(purposes::operator_correction())
    .execute()
    .await?
    .require_committed()?;

let redone = undone
    .reapplications()
    .select(original_operation)
    .against(product_branch.head())
    .as_principal(principal)
    .execute()
    .await?;
```

Ordinary callers select product meaning, not lower-runtime branch ids. Advanced
inspection may explain which components are retained, reversed, compensated,
or reconciled, but cannot let callers fabricate component plans or publication
authority.

## Performance Contract

- Ordinary commits with no correction request retain exact-zero correction
  planning and history-navigation work.
- Correction admission scales with the selected source, target, changed
  component set, declared dependency closure, and relevant intervening changes,
  not total product or component history.
- An unchanged component performs no inverse, rebuild, or branch advance; it
  carries and validates its exact retained basis.
- Signal reconciliation scales with the authoritative semantic delta plus the
  declared Signal dependency closure, with an explicit dense fallback when
  sparse maintenance is no longer profitable.
- Listing alternatives is explicit history work with a declared bound and
  continuation; it is never hidden in an outcome getter.
- Retained pre-images, governed inputs, component candidates, and correction
  pins have installed byte and lifetime bounds.
- Counters distinguish composite history traversal, per-component
  applicability, inverse construction, compensation, Signal reconciliation,
  invariant work, owner preparation, and composite publication retries.

## Must Preserve

- the accepted Milestone 9.16 aftermath, recovery, external-effect, retention,
  and publication foundation;
- the completed Milestone 9.17 umbrella: 9.17.1 component authority and
  independent-branch progress, 9.17.2 composite branch/history/currentness,
  and 9.17.3 Query carriage and public-facade authority;
- original component and composite commits plus complete causal history;
- fresh authorization, current definitions, and invariant execution for every
  correction;
- derived Signal state remaining rebuildable and non-authoritative; and
- the distinction between redo, replay, reuse, retry, reconciliation, rebuild,
  rollback, and external compensation.

## Explicit Non-Goals

- semantic merge or rebase;
- automatic resolution of conflicting divergence;
- multi-parent correction or merge publication;
- offline or replicated branch synchronization;
- Store-backed durable recovery; and
- using history navigation, component correspondence, or retained evidence as
  authorization.

## Acceptance Evidence

Milestone 9.18 closes only when the provisional inventory is fully classified,
the accepted public facade exposes no linear-stack or Relational-only fiction,
every correction is an independently observed new composite commit, unchanged
components retain exact bases, required Signal reconciliation is owner-issued,
stale and hostile attempts apply nothing, external effects retain honest
posture, ordinary commits pay zero correction work, documentation compiles
against the real facade, and residue checks find no Query-owned head or
competing composition/history chain.

## Handoff

[Milestone 9.19](./milestone-9.19.md) may now require advanced search, bulk,
path, attachment, and reuse operations to preserve the accepted composite
tree-based aftermath contract. It cannot defer their correction semantics to
provisional Phase 8 behavior or treat a Relational branch as the complete
product world.
