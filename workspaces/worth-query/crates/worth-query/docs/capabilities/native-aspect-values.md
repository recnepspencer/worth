# Native Aspect Values

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../foundations/ordinary-application-front-door.md).

## What This Feature Is

Native aspect values let application code author, query, and consume the exact
value vocabulary defined by `worth-foundational`. Query adds runtime admission
and projection authority around those values without replacing them with a
smaller scalar model or a consumer-owned encoding.

## Why You Use It

- Store an exact numeric, temporal, reference, null, or struct value without
  converting it to text or JSON.
- Build predicates whose operand family is checked against the declared aspect
  contract before execution.
- Carry scalar or whole-struct projection facts into a downstream subsystem
  without reopening source or basis authority.
- Distinguish whole-value set, field set, field clear, explicit null, and
  optional absence during mutation.

## Stable Entry Points

Import native value types from `worth_foundational::facade`:

- `AspectValue` is the complete scalar algebra.
- `StructAspectValue` is an ordered, typed struct value.
- `ScalarAspectType` declares a scalar contract family.

Import the Query roles from the capability namespace that owns the operation:

- `worth_query::facade::mutation::{declare, WorthQueryAuthoredAspectValue}`
- `worth_query::facade::read::{WorthQueryPredicateOperand,
  ConsumedNativeValueView, ConsumedNativeRefinementDenial}`
- `worth_query::facade::read::SchemaFieldView` for contract-derived predicate
  validation

`WorthQueryAuthoredAspectValue` and `WorthQueryPredicateOperand` are proofless
authoring roles. They do not validate a value or become authority merely by
being constructed.

## Core Mental Model

Foundational owns value meaning and aspect contracts. Query owns the journey
that admits authored meaning against the active contract, carries the admitted
value through execution, and seals selected result facts with source and basis
authority.

```text
AspectValue or StructAspectValue
-> Query authoring role
-> active Foundational contract and mask validation
-> admitted native mutation or predicate
-> native result value
-> proof-bearing consumed fact
-> borrowed refinement
```

Null is a value. Absence is the lack of a value. Clearing an optional field is
therefore different from setting that field to `AspectValue::Null`, and Query
preserves that distinction.

## How It Executes

1. The consumer authors an exact `AspectValue` or `StructAspectValue`.
2. Query resolves the active aspect contract for the target.
3. Foundational validates family, struct shape, mutation mask, absence law,
   and contract revision before lower-runtime execution.
4. Relational commits the native patch or Query returns a typed stop without a
   partial mutation.
5. Reads, retained rows, live rows, and projection consumption keep the native
   scalar or struct shape intact.
6. A consumer may borrow the value through `native_value()` or an exact
   refinement method. Refinement never widens or converts the value.

## Small Example

```rust
use worth_foundational::facade::{AspectKey, AspectValue};
use worth_query::facade::mutation::{declare, WorthQueryAspectTouch};

let declaration = declare(|builder| {
    builder
        .set_aspect(
            WorthQueryAspectTouch::whole_aspect(AspectKey::new("measurement")?),
            AspectValue::UInt64(42),
        )
        .build_insert("Measurement")
})?;
```

The caller supplies value meaning and mutation intent. The active runtime
contract decides whether `UInt64` is legal; constructing the value does not.

## Real Example

```rust
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, StructAspectValue};
use worth_query::facade::mutation::{declare, WorthQueryAspectTouch};
use worth_query::facade::read::ConsumedNativeValueView;

let profile = StructAspectValue::new([
    (FieldKey::new("title")?, AspectValue::String("Draft".into())),
    (FieldKey::new("rank")?, AspectValue::UInt32(7)),
])?;

let declaration = declare(|builder| {
    builder
        .set_aspect(
            WorthQueryAspectTouch::whole_aspect(AspectKey::new("profile")?),
            profile,
        )
        .build_insert("Candidate")
})?;

// After an ordinary read and admitted projection consumption:
for fact in authority.facts().display_fields() {
    match fact.native_value() {
        ConsumedNativeValueView::Scalar(value) => observe_scalar(value),
        ConsumedNativeValueView::Struct(value) => observe_profile(value),
    }
}
```

The struct is authoritative only after runtime admission. The consumed fact
then keeps the admitted value together with its source row, source family, and
projection authority. `fact.as_struct()` is available when the consumer expects
a struct and needs a typed denial for any other shape.

## How It Relates To Other Features

- Pair native values with [Writes And Intent Boundaries](../execution/writes-and-intents.md)
  for ordinary mutation execution and typed stops.
- Use [Projection Consumption](projection-consumption.md) when another runtime
  needs selected values together with operational authority.
- Use [Read Composition](../authoring/read-composition.md) to select the fields
  that become available to projection consumption.
- Use [Aspects And Authority Lanes](../modeling/aspects-and-authority-lanes.md)
  for contract, mask, absence, and authority semantics.

## Inspection And Debugging

Mutation stops identify contract validation failures before lower-runtime work.
For a consumed field fact, inspect `field_path()`, `source_family()`,
`source_identity()`, and `projection_authority()`. A
`ConsumedNativeRefinementDenial` additionally reports `expected()` and
`actual()` native shapes while retaining the same source evidence.

## Anti-Patterns

- Formatting native values into strings or debug text for identity, equality,
  storage, or transport.
- Recreating a local scalar enum, row carrier, or JSON envelope beside
  `AspectValue` and `StructAspectValue`.
- Treating a proofless authoring wrapper as contract validation authority.
- Converting a struct to a field map and thereby losing whole-set, field-set,
  field-clear, absence, or explicit-null meaning.
- Rebuilding projection values from rows after a typed projection denial.

## Current Limits

- The ordinary runtime-backed path is supported. Store-backed durability and
  cross-process Query continuation remain governed by the support matrix.
- Projection consumption begins with a completed Query read and only exposes
  fields admitted by that read's result shape.
- Native refinement is exact and borrowed. It intentionally does not provide
  numeric widening, text parsing, or struct coercion.

## Related Docs

- [Projection Consumption](projection-consumption.md)
- [Writes And Intent Boundaries](../execution/writes-and-intents.md)
- [Read Composition](../authoring/read-composition.md)
- [Aspects And Authority Lanes](../modeling/aspects-and-authority-lanes.md)
- [Support Matrix And Admission](../foundations/support-matrix-and-admission.md)
