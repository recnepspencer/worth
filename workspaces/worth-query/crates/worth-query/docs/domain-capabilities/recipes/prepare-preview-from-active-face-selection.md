# Prepare Preview From An Active Face Selection

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Recipe Covers

This recipe shows the shortest helper-driven path from one active-face
selection to a preview-facing next-step result.

Use it when your app already has one geometry declaration input for the active
face and wants Query to:

1. admit and progress that declaration
2. classify the preview-facing next step

The geometry family is the one declaration family that ships family-native
helpers today, so this recipe uses its vocabulary. Any other family reaches
the same result through the generic
[Signal Compatibility Orchestration](../signal-compatibility-orchestration.md)
lane that this helper lowers onto.

## When To Use It

Use this when:

- the declaration family already supports the active-face helper surface
- the next question is "can Query stop at compatibility or prepare the preview
  step?"
- you want the geometry-native helper path instead of stitching the signal
  compatibility input by hand

Do not use this when:

- you need a cross-family generic flow
- you already have an envelope and want the generic signal/continuation lane
- you need explicit continuation execution instead of a preview-facing next-step
  answer

## The Smallest Useful Path

```rust
let progressed = handle.geometry_helpers().progress_active_face_selection(
    face_selection_input,
)?;

let outcome = handle
    .geometry_helpers()
    .prepare_preview_for_active_face_selection_outcome(progressed);
```

This gives you the ordinary outcome lane over the same signal-facing
orchestration the generic API would use.

## A More Inspectable Path

```rust
let progressed = handle.geometry_helpers().progress_active_face_selection(
    face_selection_input,
)?;

let proof = handle
    .geometry_helpers()
    .prepare_preview_for_active_face_selection_proof(progressed);

let _ = proof.request();
let _ = proof.outcome();
let _ = proof.linked_artifacts();
```

Use the proof lane when your app or tooling needs the retained explanation for
why preview preparation stopped or succeeded.

## If It Goes Wrong

If you only need one compact repair answer:

```rust
let progressed = handle.geometry_helpers().progress_active_face_selection(
    face_selection_input,
)?;

let outcome = handle
    .geometry_helpers()
    .prepare_preview_for_active_face_selection_outcome(progressed);

if let Some(recovery) = handle.recover_from_outcome(&outcome) {
    let _ = recovery.recommended_action();
}
```

If you need stronger retained context, move to the checked or proof recovery
lane from the underlying signal-compatibility orchestration surface.

## What This Reuses

This helper path still lowers onto the canonical Query surfaces:

- declaration progression
- signal compatibility orchestration
- ordinary outcomes or proof-visible transcripts
- typed-stop remediation guidance

The helper only makes the callsite family-native.

## Related Docs

- [Family Helpers](../family-helpers.md)
- [Signal Compatibility Orchestration](../signal-compatibility-orchestration.md)
- [Envelope To Signal Or Continuation](../workflow/envelope-to-signal-or-continuation.md)
