# Attach Material With Declaration-Scoped Contributions

> **Internal engine surface.** This page documents `WorthQueryWorkspace` (`worth_query::facade`), the engine surface that `worth-ui-query-binding` uses. Application code uses `worth-query-decl` and `worth-query-host`; start with the [application front door](../../foundations/ordinary-application-front-door.md).

## What This Recipe Covers

This recipe shows how to attach material to an active-face selection while also
keeping declaration-scoped support, explanation, or workflow meaning on the
same Query run.

Use it when one declaration and its contribution posture belong together.

The geometry family is the one declaration family that ships family-native
helpers today, so this recipe uses its vocabulary. Any other family reaches
the same result through the generic
[Contribution-Composed Orchestration](../contribution-composed-orchestration.md)
lane that this helper lowers onto.

## When To Use It

Use this when:

- the declaration already exists as one active-face material attachment job
- the run also needs support, explanation, or workflow contribution meaning
- you want one composed result that preserves declaration truth plus
  contribution truth together

Do not use this when:

- you only need the declaration-side run
- you have no contribution intents at all
- you want grouped contributions over a neighborhood instead of one declaration

## The Smallest Useful Path

```rust
let input = WorthQueryGeometryMaterialAttachmentInput::new(face_selection_input)
    .with_support_contribution(
        WorthQuerySupportContributionAuthoring::declaration_traceability(
            "geometry.traceability.face",
            "face selection remains traceable through material attachment",
        ),
    );

let composed = handle
    .geometry_helpers()
    .orchestrate_material_attachment_for_active_face_selection(input)?;
```

This gives you one composed declaration-plus-contribution result through the
geometry-native helper surface.

## A Richer Path With Multiple Contribution Kinds

```rust
let input = WorthQueryGeometryMaterialAttachmentInput::new(face_selection_input)
    .with_support_contribution(
        WorthQuerySupportContributionAuthoring::declaration_traceability(
            "geometry.traceability.face",
            "face selection remains traceable through material attachment",
        ),
    )
    .with_explanation_contribution(
        WorthQueryExplanationContributionAuthoring::requires_context(
            "geometry.material.context",
            "material attachment depends on the current active-face context",
        ),
    )
    .with_workflow_contribution(
        WorthQueryWorkflowContributionAuthoring::preview_only(
            "geometry.material.preview",
            "keep material attachment in preview-only posture",
        ),
    );

let outcome = handle
    .geometry_helpers()
    .orchestrate_material_attachment_for_active_face_selection_outcome(input);
```

Use the ordinary outcome lane when the app wants one compact result over the
composed run.

## If It Goes Wrong

```rust
let outcome = handle
    .geometry_helpers()
    .orchestrate_material_attachment_for_active_face_selection_outcome(input);

if let Some(recovery) = handle.recover_from_outcome(&outcome) {
    let _ = recovery.recommended_action();
    let _ = recovery.authority_surface();
}
```

If you need per-intent retained context, move to the checked or proof lane on
the underlying contribution-composed surface.

## What This Reuses

This helper path still lowers onto:

- contribution-composed orchestration
- the declaration-entry envelope ceiling
- typed-stop remediation guidance

The helper only packages the declaration family and common contribution kinds
into one easier call shape.

## Related Docs

- [Family Helpers](../family-helpers.md)
- [Contribution-Composed Orchestration](../contribution-composed-orchestration.md)
- [Grouped Contributions](../grouped-contributions.md)
