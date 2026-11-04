# Aspect Keys, Values, And Scalar Contracts

## What This Feature Is

This feature lets you define what one aspect means before any runtime can treat
its values as authoritative. It covers aspect keys, scalar value families,
reference and opaque value families, and scalar-first contract authoring.

## Why You Use It

- Use this when a boundary value needs explicit interpretation law instead of a
  loose field name and guessed type.
- Use this when the same aspect key should mean the same thing across producers.
- Use this when you want scalar value validation to be driven by a real
  contract, not by ad hoc type checks.
- Use this when reference or opaque aspect shapes should stay distinct from
  ordinary scalar value contracts.

## Stable Entry Points

Common path:

- `aspects().vocabulary().key(...)`
- `aspects().vocabulary().identity(...)`
- `aspects().vocabulary().revision(...)`
- `aspects().contract().for_key(...)`
- `.identified_by(...)`
- `.at_revision(...)`
- `.scalar(...)`
- `.scalar_with(...)`
- `.reference_entity()`
- `.reference_with(...)`
- `.opaque_token()`
- `.opaque_with(...)`

Lower lane:

- `AspectKey`
- `AspectIdentity`
- `AspectContractRevision`
- `AspectValue`
- `ScalarAspectType`
- `ReferenceAspectType`
- `OpaqueAspectType`
- `AspectContract`
- `AbsenceLaw`
- `AspectEquivalenceBasis`
- `AspectEvolutionPolicy`
- `EntityId`
- `ContentRefId`
- `CanonicalString`
- `CanonicalDecimal`
- `CanonicalTimestamp`

Good to know:

- `aspects()` is the hardened common lane for Milestone 1 authoring.
- scalar contracts are still real contracts; they just use the simplest shape.
- `opaque_with(...)` is intentionally narrower than the other custom shape
  builders: it fails closed if the supplied mask contract is not diagnostic
  only.

## Core Mental Model

A scalar contract answers five things together:

- which aspect key is being defined
- which stable identity and revision it carries
- which scalar family the value belongs to
- how absence is treated
- how equality and evolution are judged

That is why a scalar contract is more than "key plus type." It is the full
interpretation law for that aspect.

Milestone 1 also ships two neighboring contract families that should not get
lost to history:

- reference shapes such as entity references
- opaque shapes such as token-like values that are intentionally not interpreted
  as ordinary scalar payloads

Those shapes travel through the same contract lane, but they are not just
"special scalar values."

## How It Executes

The normal flow is:

1. construct a valid aspect key
2. choose a stable identity and revision
3. choose the scalar value family
4. either use the standard scalar contract or provide explicit mask, absence,
   equivalence, and evolution law
5. if the aspect is reference-shaped or opaque, use the matching shape helper
   instead of pretending it is scalar
6. hand the finished contract to validation or compatibility lowering

## Small Example

```rust
use worth_foundational::{aspects, ScalarAspectType};

let vocabulary = aspects().vocabulary();

let contract = aspects()
    .contract()
    .for_key(vocabulary.key("retry.count")?)
    .identified_by(vocabulary.identity(7))
    .at_revision(vocabulary.revision(1))
    .scalar(ScalarAspectType::Int64);
```

This is the smallest honest example because it shows the minimum contract
identity and shape needed before validation can begin.

## Real Example

```rust
use worth_foundational::{
    aspects, AbsenceLaw, AspectEquivalenceBasis, AspectEvolutionPolicy,
    ScalarAspectType,
};

let vocabulary = aspects().vocabulary();

let contract = aspects()
    .contract()
    .for_key(vocabulary.key("build.status")?)
    .identified_by(vocabulary.identity(11))
    .at_revision(vocabulary.revision(2))
    .scalar_with(
        ScalarAspectType::String,
        aspects().mask_contract().scalar(),
        AbsenceLaw::Required,
        AspectEquivalenceBasis::ExactCanonicalValue,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    );
```

What is authoritative here is the contract, not the callerâ€™s local idea of
what `"build.status"` ought to mean.

The same is true for entity-reference and opaque shapes: their contract
defines their meaning. Content uses `ContentRef` fields in a flat Struct aspect.
The Native schema binds each such field to a format and version; Native admission
proves available bytes, integrity and domain validity. A descriptive scalar
`ContentRefId` proves none of that, and no standalone content-shape builder exists.

Opaque contract authoring also preserves one extra law at the front door:

- `opaque_token()` always chooses the milestone's diagnostic-only opaque mask
  contract
- `opaque_with(...)` exists only for explicit opaque authoring, and it rejects
  any non-diagnostic mask contract instead of silently constructing an
  incoherent opaque contract

## How It Relates To Other Features

- [Aspect Shapes And Value Carriers](./aspect-shapes-and-value-carriers.md)
  explains the exact Rust carrier types these contracts validate.
- [Struct Contracts, Fields, And Field Paths](./struct-contracts-fields-and-field-paths.md)
  covers the richer shape lane for non-scalar aspects.
- [Validation And Authoritative State Admission](./validation-and-authoritative-state-admission.md)
  uses the contract to validate incoming values.
- [Compatibility Lowering And JSON Bridges](./compatibility-lowering-and-json-bridges.md)
  can lower JSON into the same contract-driven meaning.

## Inspection And Debugging

Inspect these first:

- the `AspectKey`, `AspectIdentity`, and `AspectContractRevision` when two
  producers should align but do not
- the scalar family when a value is rejected unexpectedly
- `AspectEquivalenceBasis` and `AspectEvolutionPolicy` when parity or change
  classification feels wrong
- the shape family itself when a value seems to be treated as scalar, reference,
  or opaque at the wrong boundary

If a scalar value is rejected, the problem is usually contract meaning, not the
state-admission lane.

## Anti-Patterns

- Do not treat aspect keys as informal strings with no contract behind them.
- Do not use plain scalar types as a substitute for a real aspect contract.
- Do not collapse scalar, reference, and opaque shapes into one informal
  "primitive value" bucket.
- Do not treat opaque authoring as if it could legally reuse scalar mutation or
  projection mask contracts.
- Do not let equality or evolution rules live only in comments.

## Current Limits

- This layer defines meaning. It does not validate or admit state by itself.
- Scalar contracts are not the whole milestone; struct and mask law are separate
  first-class surfaces.
- Reference and opaque contracts share the lane but keep their own meaning.

## Related Docs

- [Aspect Shapes And Value Carriers](./aspect-shapes-and-value-carriers.md)
- [Struct Contracts, Fields, And Field Paths](./struct-contracts-fields-and-field-paths.md)
- [Validation And Authoritative State Admission](./validation-and-authoritative-state-admission.md)
