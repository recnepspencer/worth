# Use A Resource As Form Source

## What This Feature Is

This is the normal path for building a form on top of a live resource line.

## Why You Use It

Use it when you want the form to inherit:

- resource status and freshness
- visible selection
- transfer and processing state
- mutation-response readback
- retained resource verification digest

## Stable Entry Points

- `signals.form.source.resourceLine(line, { id })`
- `signals.form({ source: ... })`
- `form.resourceSource()`

## Core Mental Model

The form does not replace the resource line. It projects the line into a
form-friendly report and keeps draft truth beside it.

## How It Executes

`form.resourceSource()` exposes:

- `status` and `freshness`
- `shape` and patch-lowering posture
- `transfer`, `lifecycle`, and `settlement`
- `visibleSelection`
- `history` availability
- verification digests

## Small Example

```ts
const form = signals.form({
  source: signals.form.source.resourceLine(taskLine, { id: "task-form" }),
  fields: ({ field }) => ({
    title: field("title"),
  }),
});

console.log(form.resourceSource()?.status.kind);
```

## Real Example

```ts
const report = form.resourceSource();

console.log(report.request.method);
console.log(report.shape.familyKind);
console.log(report.shape.patchLowering);
console.log(report.visibleSelection.kind);
console.log(report.verification.packageDigest);
```

## How It Relates To Other Features

- Use [Reflect Resource Settlement In A Form](./reflect-resource-settlement-in-a-form.md)
  for pending/confirmed/failed posture.
- Read [Resource Drift](../../forms/resource-backed/resource-drift.md) to
  inspect how remote changes affect a local draft.
- Use the form-side [Resource Line Source](../../forms/resource-backed/resource-line-source.md)
  page when you want the same topic from the forms doc tree.

## Inspection And Debugging

Start with `form.resourceSource()`. It is the common read. Only drop to the raw
line when you need deeper resource-only inspection.

## Anti-Patterns

- Do not duplicate resource status in a separate form-owned cache.
- Do not treat the form as the owner of resource freshness or delivery truth.

## Current Limits

Plain forms still return `null` for `form.resourceSource()`. This surface is
only present for resource-backed sources.

## Related Docs

- [Resource Line Source](../../forms/resource-backed/resource-line-source.md)
- [Inspecting And Debugging Resources](../debugging/README.md)
