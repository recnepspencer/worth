# Mutation Response Reconciliation

Use this page when a save, create, or delete response should update the rest
of your app state for you instead of making feature code copy server results
into related views by hand.

## What This Covers

- response-owned `.create(...)`, `.update(...)`, and `.remove(...)`
- `reconciles: [...]`
- exact detail, collection item, insert, delete, tombstone, and summary targets
- identity migration for temporary or draft ids
- partial mapping and explicit fallback results
- diagnostics, history, restore, and verification reads for mutation responses

## Why You Use It

- save a detail document and update a related list or summary in the same
  declared write
- create with a temporary id, then move to the server's real id
  without feature-local remapping
- remove or tombstone an item while keeping detail, summary, and fallback
  results explicit
- inspect what the response actually updated without reverse-engineering cache
  behavior

## Stable Entry Points

- `api.url(...).response(...).create(...)`
- `api.url(...).response(...).update(...)`
- `api.url(...).response(...).remove(...)`
- `line.mutationResponse()`
- `line.summary().diagnostics.latest`
- `line.history().verificationPackage()`
- `signals.resource.mutationResponses.closeoutMatrix()`

## Core Mental Model

A mutation response is not just "the value returned by the write."

Before Worth updates any related read line, it builds one mutation-response
plan from the server response. You do not write that plan yourself. Worth uses
it to decide what can be updated safely.

That plan records:

- which response lens proved the payload
- which read families were declared targets
- which targets reconciled exactly
- which targets stayed in a fallback result
- whether identity migrated
- what rollback and diagnostics evidence was emitted

If a declared target cannot reconcile exactly, the result must stay explicit as
`refetchRequired`, `deliveryAwaited`, `partialReconciliation`,
`placementUnavailable`, `deletionUnavailable`, or
`identityMigrationUnavailable`.

The line you hold after `.create(...)`, `.update(...)`, or `.remove(...)` is
still the write line. Worth uses the response to update declared read lines,
and it records what happened as mutation-response evidence you can inspect
later.

## How It Executes

1. The route response lens proves which response fields or shapes are available.
2. Worth lowers the write into one mutation-response plan.
3. The plan accepts or declines each declared target before visible read truth
   changes.
4. Exact targets update declared detail, collection, summary, or auxiliary
   lines.
5. Any target that cannot reconcile exactly stays explicit as a fallback
   result.
6. Diagnostics, history, restore, replay posture, and verification reads all
   derive from the same plan.

## Small Example

```ts
const taskDetail = api.url("/tasks/:taskId")
  .response(signals.resource.response.detail<{
    id: string;
    status: string;
  }>()({
    status: "status",
  }))
  .detail({
    load: ({ taskId }) => ({ id: taskId, status: "draft" }),
  });

const taskList = api.url("/tasks")
  .response(signals.resource.response.collection({
    itemId: (item) => item.id,
    items: (value) => value.items,
    replaceItems: (value, nextItems) => ({ ...value, items: [...nextItems] }),
    summaries: signals.resource.valueSummaries({
      version: {
        read: (value) => value.version,
        write: (value, version) => ({ ...value, version }),
      },
    }),
  }))
  .list({
    load: () => ({
      items: [{ id: "task:1", status: "draft" }],
      version: 1,
    }),
  });

const saveTask = api.url("/tasks/:taskId")
  .response(signals.resource.response.detail()({
    status: "status",
    version: "version",
  }))
  .update({
    reconciles: [{
      family: taskDetail,
      params: ({ taskId }) => ({ taskId }),
      fallback: "partialReconciliation",
      detail: { kind: "field", field: "status" },
    }, {
      family: taskList,
      params: () => ({}),
      fallback: "partialReconciliation",
      summary: { kind: "summary", summary: "version" },
    }],
    load: ({ taskId }) => ({
      id: taskId,
      status: "published",
    }),
  });

const line = saveTask.line({
  taskId: "task:1",
  body: {},
});

console.log(line.mutationResponse().partialAdmission);
console.log(line.summary().diagnostics.latest.mutationResponseTargetOutcomes);
```

Here the detail status updates exactly, but the summary target stays in
`partialReconciliation` because the response omitted `version`.

This is the smallest honest example because it shows the two things that make
mutation responses different from ordinary writes:

- one write can target more than one read family
- the runtime keeps partial results explicit instead of silently guessing

Notice that the detail read family uses the same response-owned detail field
lane as the mutation response. That keeps the common top-level field contract
declared once, then reused across the read line and the write-response plan.

## Save And Replace Detail Lines

Use `detail: { kind: "replace" }` when the response proves the whole detail
value exactly.

Use `detail: { kind: "field", field: "..." }`, `region`, or `path` when the
response proves only one part of the detail value and you want rollback to stay scoped to that smaller area.

This is the difference between:

- "the server proved the whole canonical detail value"
- "the server proved only this declared part of the detail value"

Both are supported. The route declaration must say which one is true.

## Update Related Collection Items

Use `collection: { kind: "item" }` when the response proves one visible item
inside a related collection or paged line.

```ts
const saveTaskStatus = api.url("/tasks/:taskId")
  .response(signals.resource.response.detail()())
  .update({
    reconciles: [{
      family: taskList,
      params: () => ({}),
      fallback: "refetchRequired",
      collection: { kind: "item" },
    }],
    load: ({ taskId }) => ({
      id: taskId,
      status: "reviewed",
    }),
  });
```

Reach for this when the server returns the latest item value and you want a
visible list row to update exactly without refetching the whole list.

This is different from a summary patch:

- collection-item update changes a resident item value
- summary patch changes collection-level metadata such as `version`, `total`,
  or another declared summary

## Patch Related Summaries

Use `summary: { kind: "summary", summary: "..." }` when the response proves
collection-level metadata but not a resident item change.

Good fits:

- `version` after a save
- `total` after create or delete
- status or modified metadata on a related count or overview resource

One write can update multiple declared read families.

Typical supported pairings:

- exact detail update plus exact collection item replacement
- exact detail update plus exact summary patch on a related collection or count
  view
- exact detail update plus auxiliary detail truth such as permissions or policy
  reads

The route declares those targets in `reconciles: [...]`, and the runtime emits
one plan naming which targets updated exactly and which targets stayed in a
fallback result.

## Real Example

This is a common production shape: save one task, update the task detail,
update the matching row in a visible list, and bump a related summary version.

```ts
const saveTask = api.url("/tasks/:taskId")
  .response(signals.resource.response.detail()({
    id: "id",
    status: "status",
    version: "version",
  }))
  .update({
    reconciles: [{
      family: taskDetail,
      params: ({ taskId }) => ({ taskId }),
      fallback: "refetchRequired",
      detail: { kind: "field", field: "status" },
    }, {
      family: taskList,
      params: () => ({}),
      fallback: "refetchRequired",
      collection: { kind: "item" },
    }, {
      family: taskList,
      params: () => ({}),
      fallback: "partialReconciliation",
      summary: { kind: "summary", summary: "version" },
    }],
    load: ({ taskId }) => ({
      id: taskId,
      status: "reviewed",
      version: 7,
    }),
  });

const line = saveTask.line({
  taskId: "task:1",
  body: {},
});

console.log(taskDetail.line({ taskId: "task:1" }).value().status);
console.log(taskList.line({}).value().items.find((item) => item.id === "task:1"));
console.log(taskList.line({}).value().version);
console.log(line.summary().diagnostics.latest.mutationResponseTargetOutcomes);
```

What this gives you:

- the detail line updates because the response proved `status`
- the visible list row updates because the response returned the latest item
- the list summary updates because the response also proved `version`
- if any one of those targets could not update safely, the line would say so
  explicitly instead of leaving a stale sibling view behind

## Create With Identity Migration

Use `identity: { ... }` when the server returns a real id that differs from
the id you submitted.

```ts
const createTask = api.url("/tasks")
  .response(signals.resource.response.detail()())
  .create({
    reconciles: [{
      family: taskList,
      params: () => ({}),
      fallback: "placementUnavailable",
      collection: { kind: "insert", placement: "append" },
    }],
    identity: {
      submitted: ({ body }) => body.id,
      response: (value) => value.id,
      canonical: (value, responseIdentity) => responseIdentity ?? value.id,
      targets: [{
        family: taskDetail,
        params: ({ body }) => ({ taskId: body.id }),
        canonicalParams: (_params, _value, canonicalIdentity) => ({
          taskId: canonicalIdentity,
        }),
        fallback: "identityMigrationUnavailable",
      }],
    },
    load: ({ body }) => ({ id: `task:${body.id}`, status: "created" }),
  });
```

The create line records both the collection placement update and the identity
migration evidence. The migrated draft line can then explain why exact restore
or replay is available or unavailable.

## Delete, Exact Removal, And Tombstone Posture

Remove responses can prove more than one honest outcome:

- exact collection deletion when the response proves which visible item left
  the view
- exact detail invalidation when the response proves the detail truth is no
  longer current
- exact tombstone posture when the response proves the item remains visible as a
  deleted record instead of disappearing
- typed `deletionUnavailable` fallback when the response does not prove an exact
  topology removal

The important rule is the same as save and create: deletion behavior must be
declared and the result must stay explicit.

## How To Choose A Lane

Pick the narrowest lane that matches what the response really proves:

- use detail `replace` when the server proved the whole detail value
- use detail `field`, `path`, or `region` when the server proved only part of
  the detail value
- use collection `item` when the response proved one visible item
- use collection `insert` or `delete` when the response proved topology
  membership changed
- use summary patch when the response proved collection-level metadata
- use identity migration when submitted and canonical ids differ
- use a fallback result when the response did not prove exact local truth

If you have to say "the app just knows this should probably update that other
view," the route is missing a declaration.

## Inspect What Happened

Use grouped line inspection for the ordinary surface:

- `line.summary().diagnostics.latest.mutationResponseTargetOutcomes`
- `line.summary().diagnostics.latest.mutationResponseConfirmationKind`
- `line.summary().diagnostics.latest.mutationResponseFallbackReasonDigest`
- `line.summary().diagnostics.latest.mutationResponseFreshnessPostureDigest`
- `line.summary().diagnostics.latest.mutationResponseNoHiddenMutationDigest`

Use history and verification when you need the full plan-level artifact:

- `line.history().lifecycle.at(-1)?.mutationResponsePlan`
- `line.history().verificationPackage().mutationResponse.plan`
- `line.history().restoreExact()`
- `line.history().replayExact()`
- `signals.resource.mutationResponses.closeoutMatrix()`

Those reads stay compact by default. They name the accepted targets and the
fallback result without reconstructing replay detail for every target.

Use them this way:

- `line.mutationResponse()` for immediate write confirmation and target results
- `line.summary().diagnostics.latest` for compact UI-safe evidence
- `line.history()` when you need restore, replay posture, or lifecycle entries
- `verificationPackage()` when you need the canonical artifact for tests,
  tooling, or deeper inspection
- `closeoutMatrix()` when you need the support categories stated literally

If you are authoring a route, stay on this page. If you are asking "is this a
normal supported lane or only a fallback/denial lane?", jump to the closeout
matrix.

## Fallback Means The Runtime Stayed Honest

If a target cannot reconcile exactly, the fallback result is part of the
contract.

Examples:

- `refetchRequired`
  The response does not prove canonical local truth. A later refresh must.
- `deliveryAwaited`
  The server accepted the write, but canonical local truth is expected from a
  later delivery packet.
- `partialReconciliation`
  Some declared targets reconciled exactly and others stayed explicit.
- `placementUnavailable`
  The create response did not prove a canonical insertion position.
- `deletionUnavailable`
  The remove response did not prove an exact topology deletion.
- `identityMigrationUnavailable`
  The server changed identity, but the declared target could not migrate
  exactly.

Do not treat those outcomes as hidden app knowledge. They are already part of
the mutation-response inspection surface.

## Multi-Family Reconciliation

The runtime supports one write response updating multiple families together,
including mixes like:

- detail plus collection item
- detail plus summary
- detail plus collection item plus summary
- detail plus auxiliary reads
- create placement plus identity migration

If some declared targets update exactly and others cannot, the outcome is still
one explicit plan. Under `allOrNone`, exact siblings are downgraded to a typed
partial result. Under `partialAllowed`, the exact targets apply and the
remaining fallback targets stay named.

## Inspection And Debugging

When something looks wrong, start with these reads:

- `line.mutationResponse().confirmation.kind`
  Shows whether the write consumed canonical truth, stayed partial, awaited
  delivery, or fell back to refetch.
- `line.summary().diagnostics.latest.mutationResponseTargetOutcomes`
  Shows each declared target, its outcome, and any fallback or partial field.
- `line.summary().diagnostics.latest.mutationResponseFallbackReasonDigest`
  Shows the compact fallback result for the whole write.
- `line.history().verificationPackage().mutationResponse.plan`
  Shows the full plan artifact when you need deeper evidence.
- `signals.resource.mutationResponses.closeoutMatrix()`
  Shows whether a lane is a happy path, denial, fallback, or out of scope.

If a route shape is denied before line creation, that is a capability boundary,
not a runtime fallback.

## Anti-Patterns

- Do not treat mutation responses like manual cache-commit callbacks.
- Do not use whole-detail replace when the route only proves one field, path,
  or region.
- Do not describe `deliveryAwaited`, `refetchRequired`, or
  `partialReconciliation` like exact support.
- Do not let undeclared related views update through app folklore.

## Current Limits

- Mutation response reconciliation only updates declared targets.
- Fallback results are honest support, but they are not the same as exact
  reconciliation.
- Out-of-scope and denied lanes stay explicit through
  `signals.resource.mutationResponses.closeoutMatrix()` rather than being
  marketed as almost-supported.

## Closeout Matrix

Use [Mutation Response Closeout Matrix](./responses/README.md)
when you need the support categories stated literally instead of inferred from
the prose below.

Supported ergonomic happy paths:

- exact detail line replace and exact granular detail updates
- exact related collection-item update plus exact summary patch where the
  topology and summary proof are declared
- exact collection item insert, delete, and tombstone posture where the
  topology proof is declared
- create placement with declared identity migration
- multi-family reconciliation with exact, partial, and explicit fallback
  outcomes
- compact diagnostics, history, verification, rollback, and branch-restore
  reads for admitted lanes

Supported precise denials:

- undeclared response fields, paths, regions, summaries, and target classes
- overclaimed placement, deletion, identity migration, and multi-target shapes
- stale target denial before visible read truth mutates

Supported typed unavailable fallbacks:

- `refetchRequired`
- `deliveryAwaited`
- `partialReconciliation`
- `placementUnavailable`
- `deletionUnavailable`
- `identityMigrationUnavailable`

Intentionally out-of-scope work:

- hidden best-effort reconciliation that mutates undeclared read truth
- pretending a denied route shape is a supported ergonomic lane
- treating fallback posture as equivalent to exact reconciliation

Deferred product ergonomics:

- none are required for the admitted Phase 9 surface; unsupported lanes must
  stay explicit rather than being described as almost-supported
- `signals.resource.mutationResponses.closeoutMatrix().deferredErgonomics`
  should therefore be empty on the shipped surface

## Related Docs

- [Fetch And Write Resources](./fetch-and-write.md)
- [Collections And Delivery](./collections-and-delivery.md)
- [Line Inspection](./line-inspection.md)
- [Mutation Response Closeout Matrix](./responses/README.md)
- [History And Restore](./debugging/restore-replay-and-recover.md)
- [Response Topology Proof](./verification/response-topology-proof.md)
