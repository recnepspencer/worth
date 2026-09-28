# Collections And Partial Updates

Use a narrow update when the server or application has proved that one part of
a resource changed and replacing the whole value would erase useful identity or
invalidate unrelated work.

## Start With Identity

A collection can support item-local behavior only when the family knows what
makes an item the same item:

```ts
const tasks = api.url("/tasks")
  .items((task: { id: string; title: string }) => task.id)
  .list({
    load: () => client.listTasks(),
  });
```

Array position is not item identity. If items can reorder, insert, or disappear,
an index cannot prove which logical item changed.

## Choose The Narrowest Declared Locus

A **locus** is the exact part of the resource a patch or response is allowed to
change. Worth supports these families of loci when their structure is declared:

- `field` — one detail field;
- `region` — one declared detail region;
- `jsonPath` — one declared nested path;
- `item` — one collection or paged item;
- `itemAspect` — one semantic view of one item;
- `summary` — one collection- or page-level summary;
- `replace` — the whole resource value.

An aspect is a named semantic slice, such as a task's title or an invoice's due
date. It lets invalidation, effects, and merge reasoning stay focused on the
meaning that changed rather than treating the entire object as one blob.

## Patch One Item Aspect

```ts
const tasks = api.url("/tasks")
  .items((task: { id: string; title: string }) => task.id)
  .aspect(
    "title",
    (task) => task.title,
    (task, title: string) => ({ ...task, title }),
  )
  .list({ load: () => client.listTasks() });

const line = tasks.line({});

await line.patch(tasks.patch.itemAspect({
  itemId: "task-42",
  aspect: "title",
  value: "Reviewed",
}));
```

The declaration supplies both the getter and the replacement rule. That is the
proof that lets the runtime update the title without guessing how the rest of
the item should be preserved.

## Partial Delivery And Server Responses

The same declared structure is used by local patches, external deliveries, and
mutation-response reconciliation. A response that proves one item or field can
update that locus while unrelated values retain identity and continuity.

If the family did not declare enough structure, the runtime must use a broader
admitted path or report the narrow operation unavailable. It does not infer
identity, fields, paths, aspects, or summary semantics from whatever object
happened to arrive.

## Derived Views

Per-item views and collection summaries can update from the same narrow change.
Use the built-in object-field lane when it expresses the real meaning. Declare a
named aspect when the semantic view is not just a property lookup or when merge
and inspection need that name.

Large deep collections are not magically constant-time. Some aggregate,
equality, and reporting reads are explicitly non-incremental and may perform
structural scans. The inspection counters exist so cost remains visible.

## Common Mistakes

- Identifying items by index or display label.
- Sending `replace` because declaring the actual narrow locus feels verbose.
- Hand-writing an object path that the family never admitted.
- Assuming a derived view owns canonical resource truth.
- Promising narrow merge behavior without declared aspects or branch proof.

## Go Deeper

- [How Partial Resource Updates Work](./how-partial-resource-updates-work.md)
- [Update One Region, Field, Or Item](./update-one-region-field-or-item.md)
- [Automatic Derived Views](./automatic-derived-views.md)
- [When To Declare Derived Views Explicitly](./when-to-declare-derived-views-explicitly.md)
- [How Partial Updates Affect Caching And Delivery](./how-partial-updates-affect-caching-and-delivery.md)
- [Working With Lists](../lists/README.md)
