# Validation And Authoritative State Admission

## What This Feature Is

This feature validates aspect values against contracts and then admits validated
entries into authoritative record state. It is the main Milestone 1 authority
boundary.

## Why You Use It

- Use this when raw values need to become contract-validated artifacts before
  they can enter state.
- Use this when you need one canonical authoritative state surface instead of
  producer-local maps.
- Use this when struct and scalar values should follow the same authority flow.

## Stable Entry Points

Common path:

- `aspects().validate().against(...)`
- `.value(...)`
- `aspects().authoritative_state().admit(...)`

Lower lane:

- `validate_aspect_value(...)`
- `admit_authoritative_record_aspect_state(...)`
- `ContractValidationInput`
- `ContractValidatedAspectArtifact`
- `AuthoritativeRecordAspectStateArtifact`
- `AuthoritativeRecordAspectState`
- `ContractValidationDenial`
- `AuthoritativeStateAdmissionDenial`

Good to know:

- validation and state admission are separate phases on purpose.
- a raw `AspectValue` is not the same thing as a validated aspect artifact.
- raw scalar-like values and raw struct values already split before validation
  through `ContractValidationInput`.
- authoritative state admission fails closed on empty entry sets.

## Core Mental Model

Milestone 1 draws a hard line between:

- raw input
- validated contract-shaped meaning
- admitted authoritative state

That separation matters because authoritative state is where later canonical,
profile, transition, and diagnostic work starts trusting the data.

## How It Executes

The normal flow is:

1. build or obtain the contract
2. build the value or struct value
3. validate the value against the contract
4. admit one or more validated artifacts into authoritative state
5. hand the resulting authoritative artifact to later lanes

The admission lane is intentionally narrower than a generic map constructor:

- duplicate aspect keys are denied
- empty admission requests are denied
- later patch application may still yield an empty resulting state after legal
  clears, but the admission boundary itself requires at least one validated
  entry

## Small Example

```rust
use worth_foundational::{aspects, AspectValue};
use worth_proof::TransitionOutcome;

let TransitionOutcome::Success(validated) = aspects()
    .validate()
    .against(&contract)
    .value(AspectValue::Int64(3))
else {
    panic!("expected validated value");
};
```

This is the smallest honest example because validation is the first boundary
raw values must cross.

## Real Example

```rust
use worth_foundational::{aspects, AspectValue};
use worth_proof::TransitionOutcome;

let struct_value = aspects()
    .vocabulary()
    .struct_value()
    .with_field("title", AspectValue::String("Ship it".into()))
    .with_field("label", AspectValue::String("origin".into()))
    .finish()?;

let TransitionOutcome::Success(validated) =
    aspects().validate().against(&contract).value(struct_value)
else {
    panic!("expected validated struct value");
};

let TransitionOutcome::Success(state) = aspects().authoritative_state().admit([validated]) else {
    panic!("expected admitted state");
};

assert!(state.payload().get(contract.key()).is_some());
```

What is authoritative here is the admitted state artifact, not the original raw
value.

## How It Relates To Other Features

- [Aspect Keys, Values, And Scalar Contracts](./aspect-keys-values-and-scalar-contracts.md)
  provides the contract meaning validation consumes.
- [Aspect Shapes And Value Carriers](./aspect-shapes-and-value-carriers.md)
  explains the raw input carriers that validation accepts.
- [Authoritative Patches And Apply Flow](./authoritative-patches-and-apply-flow.md)
  starts from authoritative state.
- [Compatibility Lowering And JSON Bridges](./compatibility-lowering-and-json-bridges.md)
  can produce the same admitted state from explicit JSON bridge inputs.

## Inspection And Debugging

Inspect these first:

- `ContractValidationDenial` when the incoming value fails before state
- `AuthoritativeStateAdmissionDenial` when multiple validated entries cannot be
  admitted together
- the final `AuthoritativeRecordAspectState` payload when later steps disagree
  about what state was admitted

If state admission fails, verify the validated artifacts first. Many problems
start before the state phase.

## Anti-Patterns

- Do not let raw values enter authoritative state directly.
- Do not collapse validation and state admission into one hidden helper.
- Do not treat struct and scalar values as separate authority systems.

## Current Limits

- This layer admits authoritative state. It does not describe later patch,
  canonicalization, profile, or diagnostics behavior.
- Admission preserves the milestone’s explicit authority boundary; it is not a
  generic map update helper.

This is not the public path for minting an "empty but authoritative" state
shell.

## Related Docs

- [Aspect Shapes And Value Carriers](./aspect-shapes-and-value-carriers.md)
- [Authoritative Patches And Apply Flow](./authoritative-patches-and-apply-flow.md)
- [Compatibility Lowering And JSON Bridges](./compatibility-lowering-and-json-bridges.md)
