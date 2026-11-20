# Projection, Mutation, And Diagnostic Masks

## What This Feature Is

This feature defines what parts of an aspect may be projected, mutated, or
exposed for diagnostics. It covers mask contracts, concrete masks, and field
path legality.

## Why You Use It

- Use this when visibility and mutation boundaries need to be part of contract
  meaning.
- Use this when field-level updates or diagnostics should be mechanically
  constrained.
- Use this when you want scalar, struct, and opaque aspects to advertise
  different legal mask behavior.

## Stable Entry Points

Common path:

- `aspects().mask_contract().scalar()`
- `aspects().mask_contract().struct_fields()`
- `aspects().mask_contract().opaque_diagnostic_only()`
- `aspects().projection_mask().whole_aspect()`
- `aspects().projection_mask().fields(...)`
- `aspects().mutation_mask().whole_aspect()`
- `aspects().mutation_mask().fields(...)`
- `aspects().diagnostic_mask().whole_aspect()`
- `aspects().diagnostic_mask().fields(...)`

Lower lane:

- `AspectMaskContract`
- `AspectMask<_>`
- `ProjectionMask`
- `MutationMask`
- `DiagnosticMask`
- `MaskAdmissibilityDenial`

Good to know:

- masks are contract law, not presentation sugar.
- the common path keeps mask categories separate so projection, mutation, and
  diagnostic semantics do not blur together.
- the common path authors field masks as sets of single declared fields; it is
  not a nested-path mask authoring API.

## Core Mental Model

A mask says what part of an aspect is legal to talk about or change in a given
mode.

There are three separate questions:

- what may be projected
- what may be mutated
- what may be exposed diagnostically

Those answers are not interchangeable, so the mask types are not
interchangeable either.

## How It Executes

The normal flow is:

1. choose the mask contract that belongs to the aspect shape
2. construct a concrete projection, mutation, or diagnostic mask
3. admit that mask against the aspect contract
4. reuse the admitted mutation mask for field-level patching or the admitted
   diagnostic mask for explicit diagnostic targeting

At the front door, field masks are intentionally single-field-path masks built
from declared field keys. If you need a broader canonical path vocabulary for
later locator or digest work, use the lower lane instead of widening the mask
authoring contract.

Opaque aspects are stricter than the other shape families:

- the common path exposes `opaque_diagnostic_only()` as the canonical opaque
  mask contract
- opaque contract authoring now fails closed if a caller tries to pair an
  opaque shape with a non-diagnostic mask contract

## Small Example

```rust
use worth_foundational::aspects;

let mutation = aspects().mutation_mask().fields(["label", "note"])?;
```

This is the smallest honest example because it shows the mode-specific mask
surface directly.

## Real Example

```rust
use worth_foundational::{
    aspects, AbsenceLaw, AspectEquivalenceBasis, AspectEvolutionPolicy,
    ScalarAspectType,
};

let vocabulary = aspects().vocabulary();
let shape = aspects()
    .struct_fields()
    .required("title", ScalarAspectType::String)
    .optional("label", ScalarAspectType::String)
    .optional("note", ScalarAspectType::String)
    .finish()?;

let contract = aspects()
    .contract()
    .for_key(vocabulary.key("task.summary")?)
    .identified_by(vocabulary.identity(41))
    .at_revision(vocabulary.revision(1))
    .struct_with(
        shape,
        aspects().mask_contract().struct_fields(),
        AbsenceLaw::Required,
        AspectEquivalenceBasis::DeclaredStructFields,
        AspectEvolutionPolicy::AdditiveFieldsAllowed,
    );

let projection = aspects().projection_mask().fields(["title", "label", "note"])?;
let mutation = aspects().mutation_mask().fields(["label", "note"])?;
let diagnostic = aspects().diagnostic_mask().fields(["title", "label", "note"])?;

assert!(contract.admits_projection_mask(&projection).is_ok());
assert!(contract.admits_mutation_mask(&mutation).is_ok());
assert!(contract.admits_diagnostic_mask(&diagnostic).is_ok());
```

What is authoritative here is the admitted mask against the contract, not a
caller’s local idea of which fields ought to be fair game.

## How It Relates To Other Features

- [Struct Contracts, Fields, And Field Paths](./struct-contracts-fields-and-field-paths.md)
  provides the field-path surface masks build on.
- [Authoritative Patches And Apply Flow](./authoritative-patches-and-apply-flow.md)
  uses mutation masks to gate field-level patching.
- [Identities, Locators, And Blind-Consumer Addressing](./identities-locators-and-blind-consumer-addressing.md)
  uses the same field-path ideas for addressable loci.

## Inspection And Debugging

Inspect these first:

- the mask mode itself when a value seems admitted under the wrong category
- the field-path list when a field should be visible or mutable but is not
- `MaskAdmissibilityDenial` when a contract rejects a concrete mask

If a field-level operation fails later, confirm the mutation mask first.

## Anti-Patterns

- Do not treat projection, mutation, and diagnostic masks as one generic list.
- Do not make field-level patch legality independent from mutation-mask law.
- Do not leave mask behavior implicit for opaque or struct-shaped aspects.
- Do not pair opaque contracts with scalar-style projection or mutation mask
  contracts just because the mask booleans fit mechanically.

## Current Limits

- Masks operate over the explicit field-path surface Milestone 1 defines.
- This layer does not itself validate values or apply patches; it constrains
  those later steps.

## Related Docs

- [Struct Contracts, Fields, And Field Paths](./struct-contracts-fields-and-field-paths.md)
- [Authoritative Patches And Apply Flow](./authoritative-patches-and-apply-flow.md)
