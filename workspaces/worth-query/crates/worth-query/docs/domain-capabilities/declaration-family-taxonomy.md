# Declaration Family Taxonomy

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Declaration family taxonomy is the Query-owned classification layer over
downstream-owned family identity.

The important split is:

- your downstream domain owns the concrete family noun and semantic family key
- Query owns the closed vocabulary that classifies that family
- the family marker ties those together without making Query own your domain
  ontology

This keeps family meaning out of raw strings and host-local branch logic.

## Why You Use It

- classify declaration families without making Query own your domain nouns
- freeze authority, signal, and grouped posture before capability review,
  legality review, and routing
- carry family posture forward as real retained declaration meaning
- let other Query capability surfaces derive structural witness availability from one
  sealed vocabulary

## Stable Entry Points

- `WorthQueryDeclarationFamilyMarker`
- `WorthQueryDeclarationFamilyTaxonomy`
- `WorthQueryDeclarationPrimaryAuthorityFamily`
- `WorthQueryGroupedDeclarationPosture`
- `WorthQuerySignalCompatibilityPosture`
- `WorthQueryDescriptiveOnlyAuthority`
- `WorthQueryRelationalTruthAuthority`
- `WorthQueryBridgeContinuationAuthority`
- `WorthQueryMixedAuthority`
- `WorthQuerySignalNotCompatiblePosture`
- `WorthQuerySignalCompatiblePosture`
- `WorthQuerySignalDeferredPosture`
- `WorthQuerySingleOnlyGrouping`
- `WorthQueryNeighborhoodCapableGrouping`
- `WorthQueryBatchCapableGrouping`
- `WorthQueryNeighborhoodAndBatchCapableGrouping`

Good to know:

- Query still does not ship concrete family nouns like `SplitEdge`
- the associated type tags are structural inputs for later witness gating
- the runtime taxonomy object is still the public inspection authority
- family markers may also declare required capability families and config
  sections
- the same family marker also carries additional declaration-side contracts, so
  legality, progression, and routing stay explicit instead of being inferred
  from taxonomy
- temporal declaration clauses such as `stale_after`, `interval`, or rolling
  windows do not belong here; they are declaration content, not taxonomy tags
- if a family wants to admit temporal clauses at all, it declares that through
  `temporal_declaration_support()` on the family marker
- async/resource declaration clauses such as source family, request identity,
  loading posture, and failure posture also do not belong here; they are
  declaration content, not taxonomy tags
- if a family wants to admit async/resource clauses at all, it declares that
  through `async_declaration_support()` on the family marker

## API Reference

The main public taxonomy surfaces are:

Family marker contract:
- `semantic_family_key() -> &'static str`
- `required_capability_families() -> &'static [WorthQueryCapabilityFamily]`
- `required_config_sections() -> &'static [WorthQueryConfigSectionFamily]`
- `taxonomy() -> WorthQueryDeclarationFamilyTaxonomy`
- `legality_contract() -> WorthQueryDeclarationLegalityContract`
- `progression_contract(handle_identity_digest, operating_context_identity_digest) -> WorthQueryDeclarationProgressionContract`
- `route_contract() -> WorthQueryDeclarationRouteContract`
- `temporal_declaration_support() -> WorthQueryTemporalDeclarationSupport`
- `async_declaration_support() -> WorthQueryAsyncDeclarationSupport`

Runtime taxonomy object:
- `WorthQueryDeclarationFamilyTaxonomy::new(primary, signal, grouped) -> WorthQueryDeclarationFamilyTaxonomy`
- `WorthQueryDeclarationFamilyTaxonomy::from_type_tags::<P, S, G>() -> WorthQueryDeclarationFamilyTaxonomy`
- `primary_authority_family() -> WorthQueryDeclarationPrimaryAuthorityFamily`
- `signal_compatibility() -> WorthQuerySignalCompatibilityPosture`
- `grouped_posture() -> WorthQueryGroupedDeclarationPosture`

Taxonomy enums:
- `WorthQueryDeclarationPrimaryAuthorityFamily::as_str() -> &'static str`
- `WorthQuerySignalCompatibilityPosture::as_str() -> &'static str`
- `WorthQueryGroupedDeclarationPosture::as_str() -> &'static str`

Posture tag families:
- `WorthQueryDescriptiveOnlyAuthority`
- `WorthQueryRelationalTruthAuthority`
- `WorthQueryBridgeContinuationAuthority`
- `WorthQueryMixedAuthority`
- `WorthQuerySignalNotCompatiblePosture`
- `WorthQuerySignalCompatiblePosture`
- `WorthQuerySignalDeferredPosture`
- `WorthQuerySingleOnlyGrouping`
- `WorthQueryNeighborhoodCapableGrouping`
- `WorthQueryBatchCapableGrouping`
- `WorthQueryNeighborhoodAndBatchCapableGrouping`

## Core Mental Model

Think of family identity as three connected layers:

1. **downstream semantic family identity**
   - "what family does this domain say this declaration belongs to?"
2. **Query taxonomy posture**
   - "what kind of family is it?"
3. **family-specific support requirements**
   - "what support/config posture must exist before this family can admit?"

The first layer stays domain-owned. The second and third layers stay
Query-owned.

Temporal meaning is intentionally separate from taxonomy. A declaration can use
the same family taxonomy and still change identity when its freshness or window
clauses change.

Async/resource meaning is intentionally separate from taxonomy too. A
declaration can use the same family taxonomy and still change identity when its
source family, request identity, loading posture, or failure posture changes.

For example, a geometry domain may define `AttachFaceMaterial` while Query classifies it
as:

- relational-truth
- signal-compatible
- neighborhood-capable
- requiring historical evaluation support

## How It Executes

1. your domain defines a family marker implementing
   `WorthQueryDeclarationFamilyMarker<YourDomain>`
2. that marker supplies:
   - `semantic_family_key()`
   - associated posture tags
   - optional required capability families
   - optional required config sections
3. Query derives the runtime taxonomy object from those type-level tags
4. declaration inputs point at the marker through their associated `Family`
   type
5. canonical declarations retain the semantic family key and runtime taxonomy
6. other Query capability surfaces derive structural witness availability from the
   same family marker

The practical consequence is that taxonomy is not an advisory label. It is part
of canonical declaration meaning and later support behavior.

## Small Example

```rust
use worth_query::facade::foundation::{
    WorthQueryCapabilityFamily, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationLegalityContract,
    WorthQueryNeighborhoodCapableGrouping, WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
};

struct GeometryDomain;
struct AttachFaceMaterial;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for AttachFaceMaterial {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "attach-face-material"
    }

    fn required_capability_families() -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::HistoricalEvaluation]
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
}
```

The family key remains canonical retained vocabulary, but it is not the
primary app-facing targeting story. Declaration-entry and binding
surfaces should treat dynamic context and semantic aspect contracts as the
ordinary geometry mental model while preserving canonical family identity as
internal proof shape.

## Real Example

```rust
use worth_query::facade::foundation::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily,
    WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityContract,
    WorthQueryNeighborhoodCapableGrouping,
    WorthQueryRelationalTruthAuthority, WorthQuerySignalCompatiblePosture,
};

struct GeometryDomain;

struct AttachFaceMaterial;

impl WorthQueryDeclarationFamilyMarker<GeometryDomain> for AttachFaceMaterial {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "attach-face-material"
    }

    fn required_capability_families() -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::HistoricalEvaluation]
    }

    fn required_config_sections() -> &'static [WorthQueryConfigSectionFamily] {
        &[WorthQueryConfigSectionFamily::Relational]
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
}

struct AttachMaterialForActiveSelection;

impl WorthQueryDeclarationInput<GeometryDomain> for AttachMaterialForActiveSelection {
    type Family = AttachFaceMaterial;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![
            WorthQueryDeclarationCanonicalEntry::text(
                "selection_scope",
                "active-face-selection",
            ),
            WorthQueryDeclarationCanonicalEntry::text(
                "material_edit_intent",
                "attach-material-from-current-selection",
            ),
        ]
    }
}
```

What this example is showing:

- the family noun stays domain-owned
- taxonomy posture stays Query-owned and closed
- support requirements now live beside family posture instead of in ad hoc
  side tables
- temporal declaration meaning is not stuffed into the taxonomy layer just to
  make it visible later

## How It Relates To Other Features

- Installed Domain Declarations retain
  the family key and taxonomy in the canonical declaration artifact, and carry
  temporal clauses as declaration-local meaning rather than taxonomy
- Declaration Family Capability Matrix
  turns family posture and requirements into support rows, checked admission,
  and witness availability
- Declaration Legality consumes the retained
  family taxonomy together with one explicit family legality contract
- Declaration Route Plans consumes admitted
  progression proof plus one explicit family route contract instead of
  rebuilding lower-authority participation from taxonomy folklore
- grouped posture here is classification, not grouped execution
- signal posture here is classification, not signal execution

## Inspection And Debugging

Inspect the canonical declaration artifact to see:

- `declaration_family_key()`
- `declaration_taxonomy()`
- `declaration_primary_authority_family()`
- `declaration_grouped_posture()`
- `declaration_signal_compatibility()`

Inspect the admitted handle to see:

- `family_support::<F>()`
- `family_support_checked::<F>()`

Use those surfaces when two declarations seem similar but admit differently or
expose different follow-on family surfaces.

## Anti-Patterns

- making Query own your domain family nouns
- treating `signal` as a peer authority family instead of a compatibility
  posture
- treating grouped posture as if it already means grouped execution
- rebuilding family meaning from strings after Query has already frozen it
- putting family-specific support requirements in host-local branch logic
- encoding temporal freshness or window semantics as fake family taxonomy

## Current Limits

Declaration family taxonomy now feeds legality review directly. It still does
not decide:

- grouped execution semantics
- actual lower-authority crossings after route planning
- Query boundary receipts over those crossings
- Query boundary envelopes over retained receipt truth
- Query relational truth routing from retained envelope truth
- Query bridge continuation routing from retained envelope truth
- signal execution or continuation participation beyond the retained family posture

It freezes the classification and support vocabulary other Query declaration
features consume.

## Related Docs

- Installed Domain Declarations
- Runtime-Installed Domain Handles
- Declaration Family Capability Matrix
- Declaration Legality
- Declaration Route Plans
- [Declaration Boundary Receipts](./declaration-boundary-receipts.md)
- [Declaration Boundary Envelopes](./declaration-boundary-envelopes.md)
- [Declaration Relational Truth Routing](./declaration-relational-truth-routing.md)
- [Declaration Bridge Continuation Routing](./declaration-bridge-continuation-routing.md)
- [Declaration Signal Compatibility](./declaration-signal-compatibility.md)
- Platform Entry
