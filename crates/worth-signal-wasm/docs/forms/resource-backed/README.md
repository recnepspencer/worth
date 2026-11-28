# Resource-Backed Forms

Use a resource-backed form when the source is already represented by a Worth
resource line and form actions should lower through that same resource
authority. This keeps fetching, projected server truth, optimistic effects,
settlement, and form patches on one declared boundary.

```ts
const projectLine = projectDetail.line({ projectId: "project-42" });

const form = signals.form({
  source: signals.form.source.resourceLine(projectLine, {
    id: "project-42",
  }),
  fields: ({ field }) => ({
    name: field<string>("name"),
    status: field<string>("status"),
  }),
  actions: ({ submit, action }) => ({
    submit: submit({
      resourceEffectProfile: signals.resource.effects.branchNative(),
    }),
    refresh: action("refresh", {
      resourceAction: { kind: "refresh" },
    }),
  }),
});
```

`resourceLine(...)` reads the line's projected value and retains resource
identity in the form source diagnostics. A field write still changes only the
form draft. The submit action lowers the current patch through the resource
line; it does not bypass the resource runtime or call a hidden endpoint.

## Narrow Patches Need Real Loci

When form paths match supported resource fields, ordinary scalar patches can
lower directly. Renamed paths, regions, JSON paths, item aspects, and summaries
need a declared `resourceLocus`. Repeated collection writes require a
`collectionItems` locus and stable item identity. Reorder can honestly become a
broad whole-resource replacement when narrow order semantics are unavailable.

## Reset, Replay, And Rollback Are Different

- `form.reset()` clears controller-local draft.
- Resource refresh or revalidation asks the line for newer server truth.
- `rollbackLastResourceEffect()` targets retained resource effect history.
- Exact replay and restore require retained compatible resource authority.

The latter operations can return unavailable. That is the correct result when
history was not retained, compatibility changed, or the selected effect profile
cannot prove the operation.

## Collaboration And Transfers

Resource-backed collaboration projects resource-owned locks, leases, branches,
comments, presence, and transfer truth where supported. The form controller is
not a collaboration server and does not merge independent drafts.

## Go Deeper

- [Resource Line Source](./resource-line-source.md)
- [Resource Settlement](./resource-settlement.md)
- [Resource Drift](./resource-drift.md)
- [Resource Reset](./resource-reset.md)
- [Replay And Restore](./replay-and-restore.md)
- [Resource Action Execution](./resource-action-execution.md)
- [Resources](../../resources/index.md)
