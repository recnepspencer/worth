# Aspect Shapes And Value Carriers

## What This Feature Is

This feature explains the actual Rust types that carry aspect shapes and raw
aspect values in `worth-foundational`. Use it when you need to know which enum
or builder you are holding before validation, state admission, or patching.

This is the missing "what types do I actually write?" layer between contract
authoring and validation. It covers `AspectShape`, `ScalarAspectType`,
`AspectValue`, `StructAspectValue`, and `ContractValidationInput`.

## Why You Use It

- Use this when you need to choose the correct raw value carrier before calling
  validation.
- Use this when contract shape and value shape are easy to confuse, especially
  for `InlineBytes`, `ContentRef`, `EntityRef`, and opaque aspects.
- Use this when you want a stable API reference for the exact Milestone 1 value
  types instead of inferring them from tests.

## Stable Entry Points

Common path:

- `aspects().vocabulary().struct_value()`
- `.with_field(...)`
- `.finish()`
- `aspects().validate().against(...)`
- `.value(...)`

Lower lane:

- `AspectShape`
- `ScalarAspectType`
- `ReferenceAspectType`
- `OpaqueAspectType`
- `AspectValue`
- `StructAspectValue`
- `ContractValidationInput`
- `AspectContract::shape()`
- `AspectValue::value_family()`
- `EntityId`
- `ContentRefId`
- `CanonicalString`
- `CanonicalDecimal`
- `CanonicalTimestamp`
- `InternedString`

Good to know:

- `AspectShape` is contract metadata, not a raw value.
- `AspectValue` is the raw scalar-like value carrier, but not every contract
  shape is represented as a direct `AspectValue` variant.
- `StructAspectValue` is its own carrier. Struct aspects do not travel as
  `AspectValue`.

## Core Mental Model

There are three different questions here, and the docs need to keep them
separate:

- what shape the contract declares
- what raw value carrier the caller constructs
- what validation input lane accepts that raw value

The carrier matrix for Milestone 1 looks like this:

- `AspectShape::Scalar(ScalarAspectType)` accepts raw `AspectValue`
- `AspectShape::Struct(StructAspectShape)` accepts raw `StructAspectValue`
- `AspectShape::Reference(ReferenceAspectType::Entity)` accepts
  `AspectValue::EntityRef(EntityId)`
- `AspectShape::Opaque(OpaqueAspectType::Token)` has a contract shape, but no
  public raw `AspectValue::Opaque(...)` variant

That last bullet is important. Opaque is a contract family, not a general raw
value constructor in Milestone 1.

`AspectValue` itself is also more specific than it may look at first glance:

- `AspectValue::String(...)` holds `InternedString`, not a plain `String`
- `AspectValue::InlineBytes(...)` carries a complete opaque inline atom
- `AspectValue::ContentRef(...)` holds a descriptive 32-byte `ContentRefId`
- Native content references occupy format-bound fields of a flat Struct aspect;
  there is no standalone content shape
- `AspectValue::EntityRef(...)` holds `EntityId`

So the API is not "generic primitive values." It is a typed carrier set with a
few intentionally sharp distinctions.

## How It Executes

The normal flow is:

1. define the contract and inspect its `AspectShape`
2. build the matching raw carrier
3. let `ContractValidationInput` wrap that carrier as either `Scalar` or
   `Struct`
4. validate it against the contract
5. admit only the validated artifact into authoritative state

In concrete terms:

1. scalar and entity-reference shapes start from `AspectValue`
2. struct shapes start from `StructAspectValue`
3. opaque shapes are contract-declared but not raw-authored through a public
   `AspectValue` variant

This is why raw value construction and contract shape are related, but not
identical.

## Small Example

```rust
use worth_foundational::{aspects, AspectValue};

let raw = AspectValue::Int64(3);

let input = aspects().validate().against(&contract).value(raw);
```

This is the smallest honest example because it shows the actual carrier a
scalar-like aspect uses before validation starts.

## Real Example

```rust
use worth_foundational::{
    aspects, validate_aspect_value, AspectContract, AspectShape, AspectValue,
    ContentRefId, EntityId, PartitionId, ScalarAspectType,
};

let entity_contract = AspectContract::reference_entity(
    aspects().vocabulary().key("entity.parent")?,
    aspects().vocabulary().identity(30),
    aspects().vocabulary().revision(1),
);

assert_eq!(
    entity_contract.shape(),
    &AspectShape::Reference(worth_foundational::ReferenceAspectType::Entity)
);

let validated_entity = validate_aspect_value(
    &entity_contract,
    AspectValue::EntityRef(EntityId::new(PartitionId::main(), 1, 0)).into(),
);

let content_contract = AspectContract::struct_aspect(
    aspects().vocabulary().key("blob.preview")?,
    aspects().vocabulary().identity(31),
    aspects().vocabulary().revision(1),
    aspects()
        .struct_fields()
        .required("payload", ScalarAspectType::ContentRef)
        .finish()?,
);
let content_value = aspects()
    .vocabulary()
    .struct_value()
    .with_field("payload", AspectValue::ContentRef(ContentRefId([9; 32])))
    .finish()?;

// This validates field shape only. Native installation must bind the payload
// field to its format/version; Native admission must resolve and certify bytes.
let validated_content = validate_aspect_value(&content_contract, content_value.into());

let struct_value = aspects()
    .vocabulary()
    .struct_value()
    .with_field("title", AspectValue::String("Ship it".into()))
    .with_field("done", AspectValue::Bool(true))
    .finish()?;

let struct_contract = AspectContract::struct_aspect(
    aspects().vocabulary().key("task.summary")?,
    aspects().vocabulary().identity(41),
    aspects().vocabulary().revision(1),
    aspects()
        .struct_fields()
        .required("title", ScalarAspectType::String)
        .required("done", ScalarAspectType::Bool)
        .finish()?,
);

let validated_struct = validate_aspect_value(&struct_contract, struct_value.into());
```

What is authoritative here is never the raw `AspectValue` or
`StructAspectValue`. Those are caller-side carriers. The authoritative boundary
starts only after validation succeeds.

`ContentRefId` appears as `AspectValue::ContentRef(ContentRefId)`. Pure scalar
value plumbing may describe this digest, but it grants no byte custody. Native
admits it only through a format-bound Struct field and the controlled content
door. See the [content capability](../../../../docs/capability-index.md).

## How It Relates To Other Features

- [Aspect Keys, Values, And Scalar Contracts](./aspect-keys-values-and-scalar-contracts.md)
  explains how contract meaning is declared before these carriers are used.
- [Struct Contracts, Fields, And Field Paths](./struct-contracts-fields-and-field-paths.md)
  explains how `StructAspectShape` and `StructAspectValue` fit together.
- [Validation And Authoritative State Admission](./validation-and-authoritative-state-admission.md)
  starts at the next boundary, where raw carriers become validated artifacts.
- [Compatibility Lowering And JSON Bridges](./compatibility-lowering-and-json-bridges.md)
  matters when you need JSON lowered into these same carriers and validated
  inputs.

## Inspection And Debugging

Inspect these first:

- `AspectContract::shape()` when the runtime seems to expect the wrong value
  family
- `AspectValue::value_family()` when a scalar-like value is rejected
- `ContractValidationDenial` when a reference, content, or struct value fails
  to cross the validation boundary
- the exact `AspectValue` variant when `InlineBytes` and `ContentRef` are easy to
  confuse

If a struct contract rejects your input, confirm you built `StructAspectValue`
instead of trying to force a struct through `AspectValue`.

If an opaque contract rejects your input, that is expected in Milestone 1. The
public API does not expose a raw opaque value constructor.

## Anti-Patterns

- Do not treat `AspectShape` as if it were the same thing as a raw value.
- Do not assume every contract shape has a direct `AspectValue` variant.
- Do not treat inline atomic bytes and a content digest as the same semantic
  lane, or treat a digest as proof that bytes were admitted.
- Do not let raw `AspectValue` skip validation and enter authoritative state
  directly.

## Current Limits

- Opaque contracts are declared, but Milestone 1 does not expose a public raw
  `AspectValue::Opaque(...)` constructor.
- Struct values are still scalar-field structs, not arbitrary nested object
  graphs.
- JSON compatibility lowering admits scalar, entity-reference and struct
  shapes, but not opaque shapes. Lowering a content digest does not resolve it.

## Related Docs

- [Aspect Keys, Values, And Scalar Contracts](./aspect-keys-values-and-scalar-contracts.md)
- [Struct Contracts, Fields, And Field Paths](./struct-contracts-fields-and-field-paths.md)
- [Validation And Authoritative State Admission](./validation-and-authoritative-state-admission.md)
- [Compatibility Lowering And JSON Bridges](./compatibility-lowering-and-json-bridges.md)
