# Struct Contracts, Fields, And Field Paths

## What This Feature Is

This feature gives struct-shaped aspects a real contract surface. It covers
field declarations, required or optional field law, field keys, field paths,
and struct value authoring.

## Why You Use It

- Use this when an aspect is more than one scalar and needs named fields.
- Use this when field targeting should be explicit and canonical instead of
  relying on JSON pointer folklore.
- Use this when later masks, validation, and field-level patches need a stable
  struct shape to refer to.

## Stable Entry Points

Common path:

- `aspects().struct_fields()`
- `.required(...)`
- `.optional(...)`
- `.defaulted(...)`
- `.finish()`
- `aspects().vocabulary().field_key(...)`
- `aspects().vocabulary().field_path(...)`
- `aspects().vocabulary().struct_value()`
- `.with_field(...)`
- `.finish()`
- `aspects().contract().for_key(...).identified_by(...).at_revision(...).struct_aspect(...)`
- `.struct_with(...)`

Lower lane:

- `StructAspectShape`
- `FieldDeclaration`
- `FieldKey`
- `FieldRequirement`
- `CanonicalFieldPath`
- `StructAspectValue`

Good to know:

- struct aspects are not an advanced side lane; they are a core Milestone 1
  capability.
- field paths are the shared targeting surface masks, locators, and patches
  build on.
- the common path only authors single-field Milestone 1 targets through
  `aspects().vocabulary().field_path(...)`; broader canonical field paths stay
  in the lower lane for locator and canonicalization work.

## Core Mental Model

A struct aspect contract says:

- which fields exist
- which fields are required, optional, or defaulted
- what scalar family each field carries
- which field paths later tooling may legally target

That is why struct law lives in the contract surface itself. If field meaning
is vague here, every later surface becomes vague too.

## How It Executes

The normal flow is:

1. build the struct field declarations
2. finish the struct shape
3. create the enclosing aspect contract with that shape
4. construct struct values with named fields
5. reuse field keys and field paths in masks, locators, and field-level patches

The front door is intentionally narrower than the lower lane here:

- `aspects().vocabulary().field_path(...)` is for one declared field target
- `CanonicalFieldPath::new(...)` still exists below the front door for typed
  canonical-path work that is not pretending Milestone 1 struct authoring is a
  nested object engine

## Small Example

```rust
use worth_foundational::{aspects, ScalarAspectType};

let shape = aspects()
    .struct_fields()
    .required("title", ScalarAspectType::String)
    .optional("label", ScalarAspectType::String)
    .finish()?;
```

This is the smallest honest example because field law is the first thing that
makes a struct aspect meaningful.

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
    .defaulted("done", ScalarAspectType::Bool)
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

let value = vocabulary
    .struct_value()
    .with_field("title", worth_foundational::AspectValue::String("Ship it".into()))
    .with_field("label", worth_foundational::AspectValue::String("origin".into()))
    .with_field("done", worth_foundational::AspectValue::Bool(true))
    .finish()?;
```

What is authoritative here is the struct shape and its field declarations, not
the producer’s local map layout.

## How It Relates To Other Features

- [Aspect Shapes And Value Carriers](./aspect-shapes-and-value-carriers.md)
  explains why struct aspects use `StructAspectValue` instead of `AspectValue`.
- [Projection, Mutation, And Diagnostic Masks](./projection-mutation-and-diagnostic-masks.md)
  uses field keys and field paths to define legal visibility and mutation.
- [Validation And Authoritative State Admission](./validation-and-authoritative-state-admission.md)
  validates struct values against the finished contract.
- [Authoritative Patches And Apply Flow](./authoritative-patches-and-apply-flow.md)
  reuses field targeting for field-level patches.

## Inspection And Debugging

Inspect these first:

- `FieldDeclaration` and `FieldRequirement` when a field seems to be admitted or
  rejected unexpectedly
- `FieldKey` when producers disagree about field naming
- `CanonicalFieldPath` when a mask, locator, or patch should target the same
  field but does not

If struct handling surprises you, check the declared shape first. Many later
problems start as field-shape drift.

## Anti-Patterns

- Do not model struct aspects as untyped maps with informal field rules.
- Do not treat field paths as throwaway strings.
- Do not bury required or defaulted field behavior in validation code alone.

## Current Limits

- Struct fields are scalar-family fields today; this layer is not a general
  nested object engine.
- Common-path field authoring is single-field only.
- Broader field-path law still exists in the lower lane for locator and
  canonicalization vocabulary.

## Related Docs

- [Aspect Shapes And Value Carriers](./aspect-shapes-and-value-carriers.md)
- [Projection, Mutation, And Diagnostic Masks](./projection-mutation-and-diagnostic-masks.md)
- [Authoritative Patches And Apply Flow](./authoritative-patches-and-apply-flow.md)
