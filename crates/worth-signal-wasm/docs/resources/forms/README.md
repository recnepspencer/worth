# Using Resources In Forms

Use this section when a resource line is the source of a form or when a form
needs to explain resource-backed drift, settlement, or mutation-response
truth.

This is the resource-side guide. The form-side deeper docs still live in
[`forms/resource-backed/`](../../forms/resource-backed/README.md).

## Start Here

- [Use A Resource As Form Source](./use-a-resource-as-form-source.md)
  The stable entry points and what `form.resourceSource()` really gives you.
- [Reflect Resource Settlement In A Form](./reflect-resource-settlement-in-a-form.md)
  Pending, confirmed, failed, retry, and visible-selection posture.
- [Resource Drift](../../forms/resource-backed/resource-drift.md)
  Inspect how remote changes affect a local draft.
- [Read Mutation Responses In Forms](./read-mutation-responses-in-forms.md)
  The form-side digest of resource mutation-response completion.
- [Replay, Restore, And Reset Resource-Backed Forms](./replay-restore-and-reset-resource-backed-forms.md)
  Exact replay, exact restore, and reset paths when a resource line owns source truth.

## The Main Mental Model

The resource line still owns resource truth.
The form owns draft truth, field projection, and submit/readiness behavior.

`form.resourceSource()` is the stable bridge between those two surfaces.

## Related Docs

- [Resource-Backed Forms](../../forms/resource-backed/README.md)
- [Branch-Native Effects](../effects/README.md)
- [Handling Server Responses](../responses/README.md)
