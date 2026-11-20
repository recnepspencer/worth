# Authoritative Patches And Apply Flow

## What This Feature Is

This feature builds authoritative patches and applies them to authoritative
state. It covers whole-aspect updates, field-level updates, and the explicit
apply flow that the hardened common path now exposes.

## Why You Use It

- Use this when authoritative state changes should be expressed as typed set and
  clear operations.
- Use this when field-level mutation should be checked against the contract and
  mutation mask.
- Use this when patch construction and patch application should remain visible
  authority steps instead of one hidden state rewrite.

## Stable Entry Points

Common path:

- `aspects().patch().whole_aspect()`
- `.set(...)`
- `.clear(...)`
- `.finish()`
- `aspects().patch().field_level(...)`
- `.set_field(...)`
- `.clear_field(...)`
- `.finish()`
- `aspects().authoritative_state().apply_patch(...)`

Lower lane:

- `AuthoritativeRecordAspectPatch`
- `WholeAspectPatchBuilder`
- `FieldLevelPatchBuilder`
- `FieldLevelAspectPatch`
- `AuthoritativePatchConstructionDenial`
- `AuthoritativePatchApplicationDenial`

Good to know:

- whole-aspect and field-level patching are separate first-class lanes.
- field-level patching is contract-aware and mutation-mask-aware by design.

## Core Mental Model

A patch is not "diff some JSON." It is a typed request to:

- set one or more validated whole-aspect values
- clear one or more whole aspects
- or set and clear specific struct fields under mutation-mask law

Patch application is a second explicit step because construction legality and
state-application legality are related but not identical concerns.

## How It Executes

The normal flow is:

1. start from validated values or a contract-plus-mutation-mask pair
2. build either a whole-aspect patch or a field-level patch
3. finish the patch and inspect any construction denial
4. apply the patch to authoritative state through the state front door

Field-level construction is intentionally stricter than the generic whole-aspect
lane:

- an empty `AuthoritativeRecordAspectPatch::empty()` is still the canonical
  whole-patch no-op surface
- a field-level patch request with no admitted sets and no admitted clears is
  denied, because it does not express any mutation under field-mask law

## Small Example

```rust
use worth_foundational::aspects;
use worth_proof::TransitionOutcome;

let TransitionOutcome::Success(patch) =
    aspects().patch().whole_aspect().set(validated).finish()
else {
    panic!("expected whole-aspect patch");
};
```

This is the smallest honest example because patch construction is the first
distinct boundary in the patch story.

## Real Example

```rust
use worth_foundational::{aspects, AspectValue};
use worth_proof::TransitionOutcome;

let label_key = aspects().vocabulary().field_key("label")?;
let note_key = aspects().vocabulary().field_key("note")?;

let TransitionOutcome::Success(field_patch) = aspects()
    .patch()
    .field_level(&contract, &mutation_mask)
    .set_field(label_key, AspectValue::String("moved".into()))
    .clear_field(note_key)
    .finish()
else {
    panic!("expected field-level patch");
};

let TransitionOutcome::Success(applied) = aspects()
    .authoritative_state()
    .apply_patch(state.payload(), &field_patch)
else {
    panic!("expected applied patch");
};
```

What is authoritative here is the applied state artifact after patch
application, not the caller’s local expectation of what the patch "should have"
done.

## How It Relates To Other Features

- [Projection, Mutation, And Diagnostic Masks](./projection-mutation-and-diagnostic-masks.md)
  provides the mutation-mask surface field-level patching depends on.
- [Validation And Authoritative State Admission](./validation-and-authoritative-state-admission.md)
  provides the authoritative state the patch applies to.
- [Struct Contracts, Fields, And Field Paths](./struct-contracts-fields-and-field-paths.md)
  provides the field keys field-level patching targets.

## Inspection And Debugging

Inspect these first:

- `AuthoritativePatchConstructionDenial` when the patch cannot even be formed
- `AuthoritativePatchApplicationDenial` when a constructed patch fails against
  real authoritative state
- the whole-aspect set and clear collections or field-level set and clear
  collections when a patch looks incomplete

If a field-level patch fails, verify the mutation mask and field keys before
suspecting state corruption.

## Anti-Patterns

- Do not treat whole-aspect and field-level patches as the same operation with
  optional flags.
- Do not bypass mutation-mask law for field-level updates.
- Do not build empty field-level patch requests and expect them to stand in for
  a no-op whole patch.
- Do not treat patch construction as if it automatically applied the change.

## Current Limits

- This layer standardizes set and clear semantics. It does not become a general
  diff engine.
- Patch application still depends on authoritative-state law; it is not a loose
  map transformation helper.

## Related Docs

- [Projection, Mutation, And Diagnostic Masks](./projection-mutation-and-diagnostic-masks.md)
- [Validation And Authoritative State Admission](./validation-and-authoritative-state-admission.md)
