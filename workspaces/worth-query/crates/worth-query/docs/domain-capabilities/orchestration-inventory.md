# Orchestration Inventory

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Surface Is

The orchestration inventory is the Query-owned semantic registry for the
public orchestration surface.

Use it when you need to answer:

- which orchestration verbs actually ship
- which checked/proof/transcript lane backs each verb
- which aspect posture that surface owns
- which basis, preview, policy, and lower-authority posture it depends on
- which contribution families or strategy families it can compose with
- which docs page and certification suite are supposed to cover it

This is not a second orchestration engine. It is the anti-drift boundary that
keeps exported verbs, retained proof lanes, semantic attachments, docs, and
certification synchronized.

## Why It Exists

Query now exposes more than one kind of orchestration surface:

- declaration-entry orchestration
- progressed route/receipt/envelope products
- continuation preparation and execution
- signal-compatibility orchestration
- contribution-composed orchestration
- grouped/helper projections over those same generic lanes

If those surfaces are inventoried only by name, later growth can drift in ways
that are easy to miss:

- a helper surface can silently hide weaker aspect semantics
- a collaborative or merge-aware follow-on can attach bridge meaning through
  prose instead of typed registry rows
- contribution-composed verbs can stop advertising which declaration-scoped
  contribution families they actually compose with
- docs and certification can keep talking about a surface after the live
  semantic attachments changed

The registry prevents that by making the live semantic shape queryable and
auditable.

## Core Inventory Concepts

Each row records:

- verb identity
- orchestration family
- visibility lane
- binding projection
- checked/proof/transcript contract
- support surface
- aspect posture
- basis posture
- policy/tenant posture
- lower-authority attachment
- strategy attachment
- contribution compatibility
- collaborative extension posture
- doc reference
- certification reference

The inventory is therefore the canonical answer to both:

- "does this verb exist?"
- "what kind of public contract does this verb actually own?"

## Row Semantics

The current families are:

- `DeclarationEntry`
- `RouteFromProgressed`
- `ReceiptFromProgressed`
- `EnvelopeFromProgressed`
- `ContinuationPrepareTarget`
- `ContinuationPrepareContext`
- `ContinuationExecute`
- `SignalCompatibilityOrchestration`
- `ContributionComposedOrchestration`
- `GroupedNeighborhoodOrchestration`

The current visibility lanes are:

- `Ordinary`
- `OrdinaryOutcome`
- `Checked`
- `ProofVisible`

Helper verbs are inventoried as first-class rows. They do not get to invent a
second semantics path. A geometry helper still declares:

- which generic family it projects onto
- which aspect posture it keeps
- which lower-authority families it leans on
- whether it participates in contribution or grouped orchestration semantics

## Aspect, Basis, And Authority Attachments

The important semantic accessors are:

- `aspect_posture()`
- `basis_posture()`
- `policy_tenant_posture()`
- `lower_authority_attachment()`
- `strategy_attachment()`
- `collaborative_extension_posture()`

These are Query-owned descriptors over lower truth, not replacement
taxonomies.

Examples:

- declaration-entry rows register declared aspect contract posture and
  declaration-entry basis posture
- continuation rows register aspect-sensitive readmission and explicit
  readmission basis posture
- signal orchestration rows register retained aspect contract/coverage plus
  signal/bridge strategy relevance
- contribution-composed rows register category-scoped aspect composition and
  declaration-scoped contribution posture

The lower-authority attachment does not restate relational, signal, bridge, or
foundational semantics. It tells you which lower authorities this Query surface
is actually adapting.

## Contribution And Strategy Attachments

The contribution compatibility surface answers whether a row:

- has no contribution semantics
- composes declaration-scoped contribution families
- stays outside contribution composition entirely, even if it is a grouped lane

The strategy attachment answers whether a row is relevant to:

- relational merge/correspondence strategy families
- signal merge / invalidation / delivery strategy families
- bridge preview / merge / writeback strategy families
- foundational materialization profiles

This is the main registry hook for collaborative extension work. Query does not
re-own those lower strategy systems, but it does expose which public surfaces
lean on them.

For example:

- contribution-composed orchestration is foundational-profile-aware because
  summary materialization is part of its public contract
- grouped neighborhood orchestration is not implicitly contribution-compatible
  just because it coordinates multiple member declarations; the helper row now
  advertises explicit grouped-neighborhood contribution compatibility instead
  of pretending grouped contribution support does not exist
- grouped helper rows can advertise retained contract-and-coverage posture once
  grouped admission and grouped orchestration keep member-local aspect witness
  visible

## Querying The Inventory

Start from the canonical inventory:

```rust
let inventory = WorthQueryOrchestrationSurfaceInventory::current();

let row = inventory
    .row_for_public_name("orchestrate_declaration_with_contributions")
    .expect("surface should be inventoried");

assert_eq!(
    row.aspect_posture(),
    WorthQueryOrchestrationAspectPosture::CategoryScopedAspectComposition,
);
assert_eq!(row.basis_posture().as_str(), "declaration_scoped_contribution");
assert!(row
    .lower_authority_attachment()
    .includes_runtime_bridge());
assert!(row
    .contribution_compatibility()
    .supports(WorthQueryDeclarationEntryContributionCategoryFamily::WorkflowPreview));
assert_eq!(
    row.collaborative_extension_posture().as_str(),
    "collaborative_phases_ready",
);
```

You can also inspect strategy-aware signal rows directly:

```rust
let row = WorthQueryOrchestrationSurfaceInventory::current()
    .row_for_public_name("orchestrate_signal_compatibility")
    .expect("signal row should exist");

assert_eq!(
    row.aspect_posture(),
    WorthQueryOrchestrationAspectPosture::RetainedContractAndCoverage,
);
assert!(row.strategy_attachment().is_merge_strategy_aware());
assert!(row
    .lower_authority_attachment()
    .includes_signal());
```

## Running The Audit

`WorthQueryOrchestrationInventoryAudit` checks both structural and semantic
drift.

It still verifies the established anti-drift classes:

- duplicate public names
- uninventoried public verbs
- missing docs references
- missing checked/proof metadata
- missing certification metadata
- missing support-surface linkage
- dishonest binding projection
- family/visibility gaps

It now also verifies semantic attachment coverage through
`semantic_attachment_gaps()`.

That semantic gap surface is intentionally exacting:

- a missing generic semantic attachment can also surface helper-drift findings
  when the helper rows no longer match their canonical base family
- contribution-composed rows fail not only when compatibility is `None`, but
  also when declaration-scoped compatibility is present without any admitted
  family set
- grouped rows are allowed to advertise explicit non-participation in
  contribution composition instead of being forced into fake compatibility

```rust
let audit = WorthQueryOrchestrationInventoryAudit::current();

assert!(audit.uninventoried_public_verbs().is_empty());
assert!(audit.missing_binding_projection_rows().is_empty());
assert!(audit.family_visibility_gaps().is_empty());
assert!(audit.semantic_attachment_gaps().is_empty());
```

If a new public surface is added without semantic registration, the audit is
the closure failure you are meant to hit.

## How This Supports Collaborative Extensions

Collaborative or conflict-aware extensions need to ask questions like:

- does this surface preserve retained aspect truth or only declared contract?
- is this surface already bridge- and signal-aware?
- is preview posture explicit here?
- can this surface compose declaration-scoped contribution families?
- should collaborative conflict or recovery layers even be allowed to build on
  this row?

The inventory now reserves those answers in typed row metadata instead of
forcing a follow-on feature to reconstruct them from docs or lower-crate
spelunking.

That does not mean collaborative semantics are already implemented here. It
means the registry grammar is already ready for them.

## What This Surface Does Not Own

The inventory does not become the authority for:

- relational merge or correspondence semantics
- signal merge, invalidation, or delivery semantics
- bridge writeback, preview, or merge semantics
- foundational materialization profile semantics
- contribution category meaning itself

Those remain lower-authority concerns. The inventory only records which public
Query surfaces attach to them.

## Related Docs

- [Domain Capabilities](./README.md)
- [Declaration Entry Orchestration](./declaration-entry-orchestration.md)
- [Continuation Pipeline](./continuation-pipeline.md)
- [Signal Compatibility Orchestration](./signal-compatibility-orchestration.md)
- [Contribution-Composed Orchestration](./contribution-composed-orchestration.md)
- [Family Helpers](./family-helpers.md)
- [Grouped Authoring](./grouped-authoring.md)
- [Typed Stops And Remediation Guidance](./typed-stops-and-remediation-guidance.md)
- [Runtime-Installed Domains And Operations](./runtime-installed-domains.md)
- [Public Doc Coverage](./public-doc-coverage.md)
