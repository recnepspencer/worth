# Canonical Basis Sequences And Entry Grammar

## What This Feature Is

This feature gives foundational data one stable, ordered form that different
producers can reproduce exactly. A canonical basis sequence is the semantic
authority for Milestone 2. It is the thing you compare, export, and later
digest.

## Why You Use It

- Use this when two runtimes need the same source meaning to lower into the
  same machine-stable form.
- Use this when you want canonical comparison or export to start from admitted
  basis artifacts instead of raw structs.
- Use this when you need a digest later but do not want the digest to become
  the meaning surface.
- Use this when Milestone 1 surfaces such as contracts, masks, patches, state,
  identities, locators, and compatibility-lowered state need one shared
  canonical lowering lane.

## Stable Entry Points

Common path:

- `canonicalization().basis().at(...)`
- `.from_contract(...)`
- `.from_mask(...)`
- `.from_patch(...)`
- `.from_state(...)`
- `.from_identity(...)`
- `.from_aspect_locator(...)`
- `.from_aspect_field_locator(...)`
- `.from_aspect_contract_locator(...)`
- `.from_value_locator(...)`
- `.from_source_locator(...)`
- `.from_mismatch_locator(...)`
- `.from_transition_locator(...)`
- `.from_boundary_artifact_locator(...)`
- `.bundle(...)`

Lower lane:

- `prepare_canonical_basis_sequence(...)`
- `prepare_owned_canonical_basis_sequence_admitted(...)`
- `prepare_canonical_basis_bundle(...)`
- `CanonicalBasisSequence`
- `CanonicalBasisBundle`
- `CanonicalBasisReadyArtifact`
- `CanonicalBundleReadyArtifact`
- `CanonicalBasisEntry`
- `CanonicalBasisEntryKind`
- `CanonicalBasisDomain`

Good to know:

- `canonicalization_api::common_path` is the recommended grouped public lane.
- `canonicalization_api::lower_lane::basis` is the inspectable lower lane.

### Preparation With Runtime Resource Admission

Owners with a live resource budget can move an already-owned entry vector into
the same construction authority:

```rust
let ready = prepare_owned_canonical_basis_sequence_admitted(
    version, domain, entries,
    |work, bytes| request_budget.admit(work, bytes),
)?;
```

The caller owns entry construction and its allocation before this boundary.
Foundation admits measurement visits, compared payload widths, sorting moves,
scratch storage, duplicate-locus denial copies, and the rule-version copy before
performing them. The actual ordinary `sort_by` still supplies canonical ordering
and its recorded comparison count. Resource admission uses a checked upper bound
and preserves the caller's original refusal in `CanonicalBasisPreparationStop`;
a refusal cannot produce a Ready artifact.

The sorting bound is reviewed against Rust 1.94.0, commit
`4a4ef493e3a1488c6e321570238084b38948f6db`, including both its ordinary and
size-optimized standard-library sort implementations. A different configured
compiler returns `UnsupportedSortImplementation` on the admitted preparation
port until its bound is reviewed. Ordinary canonical preparation continues to
use its existing compiler and semantics.

## Core Mental Model

A canonical basis sequence is not just a serialized blob. It is an ordered
list of typed entries in a named domain under a named rule version.

That is why basis is the authority here:

- a digest can be recomputed from it
- an export bundle can be published from it
- a mismatch can point back into it
- a blind consumer can inspect it without guessing what the producer meant

If two producers mean the same thing, they should be able to produce the same
basis sequence even if their local construction history differs.

## How It Executes

The normal flow is:

1. pick the canonicalization rule version once
2. lower one real foundational surface into a ready basis artifact
3. optionally bundle several ready basis artifacts together
4. pass the ready artifact to comparison, export, or digest lanes

The front door is shaped this way on purpose. It makes version choice explicit
once, then keeps the semantic lowering step louder than the lower-level entry
grammar.

The shipped basis builders intentionally cover the main Milestone 1 surfaces:

- aspect contracts
- aspect masks
- authoritative patches
- authoritative state
- identity input
- locator input

That coverage matters because Milestone 2 is not just a generic canonical
grammar. It is also the canonical lowering lane for the earlier foundational
meaning surfaces.

## Small Example

```rust
use worth_foundational::{canonicalization, CanonicalizationRuleVersion};
use worth_proof::TransitionOutcome;

let ready = match canonicalization()
    .basis()
    .at(CanonicalizationRuleVersion::V1)
    .from_state(state)
{
    TransitionOutcome::Success(ready) => ready,
    other => return Err(format!("basis preparation failed: {other:?}").into()),
};
```

This is the smallest honest example because the common path starts from a real
foundational surface and ends at a ready basis artifact.

## Real Example

```rust
use worth_foundational::{canonicalization, CanonicalizationRuleVersion};
use worth_proof::TransitionOutcome;

let version = CanonicalizationRuleVersion::V1;

let contract_ready = match canonicalization().basis().at(version).from_contract(contract) {
    TransitionOutcome::Success(ready) => ready,
    other => return Err(format!("contract lowering failed: {other:?}").into()),
};

let state_ready = match canonicalization().basis().at(version).from_state(state) {
    TransitionOutcome::Success(ready) => ready,
    other => return Err(format!("state lowering failed: {other:?}").into()),
};

let bundle_ready = match canonicalization()
    .basis()
    .at(version)
    .bundle([contract_ready, state_ready])
{
    TransitionOutcome::Success(bundle) => bundle,
    other => return Err(format!("bundle preparation failed: {other:?}").into()),
};
```

What is authoritative here is the ready basis artifact or bundle. Downstream
lanes should not reconstruct the same meaning from raw inputs a second time.

## How It Relates To Other Features

- [Equivalence And Mismatch Classification](./equivalence-and-mismatch-classification.md)
  compares ready basis artifacts.
- [Export Bundles And Producer Shape](./export-bundles-and-producer-shape.md)
  turns ready basis bundles into published canonical exports.
- [Digest Derivation And Slot Semantics](./digest-derivation-and-slot-semantics.md)
  derives digest output from ready basis artifacts instead of treating raw
  digests as the meaning surface.

## Inspection And Debugging

Inspect these first:

- `canonicalization_api::lower_lane::basis` when you need exact basis entry and
  readiness vocabulary
- `CanonicalBasisConstructionDenial` when lowering fails
- the basis domain and rule version when two producers should match but do not
- the ordered entry list when a mismatch feels surprising

If basis preparation fails, the problem is usually domain coherence or source
shape, not digest policy.

## Anti-Patterns

- Do not treat raw digests as if they were the same thing as canonical basis.
- Do not compare producer-local structs directly when a ready basis artifact is
  the supported semantic surface.
- Do not hide rule-version choice in a global default.

## Current Limits

- This layer stabilizes meaning and ordering. It does not yet tell you whether
  two sequences are equivalent under a chosen equivalence basis.
- Ready basis artifacts are still milestone-scoped canonicalization surfaces,
  not universal runtime interchange packets.

## Related Docs

- [Equivalence And Mismatch Classification](./equivalence-and-mismatch-classification.md)
- [Grouped Public Lanes And Front-Door Usage](./grouped-public-lanes-and-front-door-usage.md)
