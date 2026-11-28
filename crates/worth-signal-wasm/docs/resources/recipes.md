# Resource Recipes

This is the task-first companion for the rewritten resource docs.

## Quick Answers

- first resource:
  [Your First Resource](./start-here/your-first-resource.md)
- choosing detail vs collection vs paged:
  [Choose A Resource Shape](./start-here/choose-a-resource-shape.md)
- fetch one record:
  [Fetch A Single Record](./fetching/fetch-a-single-record.md)
- fetch a list:
  [Fetch A Collection](./fetching/fetch-a-collection.md)
- fetch accumulating pages:
  [Fetch A Paged List](./fetching/fetch-a-paged-list.md)
- auth, headers, basis, or correlation ids:
  [Request Auth And Context](./fetching/request-auth-and-context.md)
- retry or stale posture:
  [Request Policies](./fetching/request-policies.md)
- create, update, or remove:
  [Write A Resource](./updating/write-a-resource.md)
- local patch or broad replacement:
  [Submit Patches And Replacements](./updating/submit-patches-and-replacements.md)
- choosing optimistic vs cautious write posture:
  [Choose An Effect Profile](./updating/choose-an-effect-profile.md)
- understanding what the runtime exposes after a write:
  [What Happens After A Write](./updating/what-happens-after-a-write.md)
- understanding what a write response changed:
  [Understand Mutation Responses](./responses/understand-mutation-responses.md)
- partial canonical truth or fallback reconciliation:
  [Handle Partial Canonical Truth](./responses/handle-partial-canonical-truth.md)
  and [Handle Fallback Reconciliation](./responses/handle-fallback-reconciliation.md)
- updating one row without replacing the whole list:
  [Update One Item Without Replacing Everything](./lists/update-one-item-without-replacing-everything.md)
- visible list truth or speculative row state:
  [Visible Selection](./lists/visible-selection.md)
- stale lines, refresh, or revalidation:
  [Caching And Refresh](./caching/README.md),
  [Stale, Pending, And Settled State](./caching/stale-pending-and-settled-state.md), and
  [Invalidation And Refresh](./caching/invalidation-and-refresh.md)
- partial updates or automatic derived views:
  [Partial Updates And Derived Views](./partial-updates/README.md),
  [Automatic Derived Views](./partial-updates/automatic-derived-views.md), and
  [Update One Region, Field, Or Item](./partial-updates/update-one-region-field-or-item.md)
- upload preparation or deferred processing:
  [Uploads And Transfers](./transfers/README.md),
  [Upload Files](./transfers/upload-files.md), and
  [Track Processing Jobs](./transfers/track-processing-jobs.md)
- builder-owned downloads or multipart export handoff:
  [Downloads And Binary Data](./downloads/README.md),
  [Offer Downloads](./downloads/offer-downloads.md), and
  [File, Media, And Export Downloads](./downloads/file-media-and-export-downloads.md)
- grouped line inspection or retained-history debugging:
  [Inspecting And Debugging Resources](./debugging/README.md),
  [Inspect A Resource Line](./debugging/inspect-a-resource-line.md), and
  [Restore, Replay, And Recover](./debugging/restore-replay-and-recover.md)
- optimistic effects or rollback posture:
  [Effects And Recovery](./effects/README.md),
  [Branch-Native Effects](./effects/branch-native-effects.md),
  [Concurrent Optimistic Effects](./effects/concurrency-and-dependencies.md), and
  [Rollback And Recovery](./effects/rollback-and-recovery.md)
- resource-backed forms, settlement or drift readback:
  [Using Resources In Forms](./forms/README.md),
  [Use A Resource As Form Source](./forms/use-a-resource-as-form-source.md), and
  [Resource Drift](../forms/resource-backed/resource-drift.md)
- route-bound resources, prefetch, or warmup:
  [Using Resources In Routes](./router/README.md),
  [Declare Route Resources](./router/declare-route-resources.md), and
  [Prefetch And Warmup Route Resources](./router/prefetch-and-warmup-route-resources.md)
- raw family declarations or direct line authoring:
  [Advanced Resource Modeling](./advanced/README.md),
  [Resource Family Identity](./advanced/resource-family-identity.md), and
  [Raw Resource Lines](./advanced/raw-resource-lines.md)
- proof packages, topology proof, or support matrices:
  [Verification And Proof](./verification/README.md),
  [Verification Packages](./verification/verification-packages.md), and
  [Mutation-Response Closeout Matrix](./verification/mutation-response-closeout-matrix.md)

## Recipe: First Detail Resource

```ts
const userDetail = signals.api({
  baseUrl: "/api",
}).url("/users/:userId").detail({
  load: ({ userId }) => ({ id: userId, name: `User ${userId}` }),
});

const line = userDetail.line({ userId: "u1" });
console.log(line.summary());
```

## Recipe: Fetch A Collection

```ts
const tasks = signals.api({ baseUrl: "/api" })
  .url("/workspaces/:workspaceId/tasks")
  .items((item: { id: string; title: string }) => item.id)
  .list({
    load: ({ workspaceId }) => [{ id: `${workspaceId}:1`, title: "First" }],
  });

const line = tasks.line({ workspaceId: "demo" });
console.log(line.value());
```

## Recipe: Create A Resource

```ts
const createUser = signals.api({ baseUrl: "/api" })
  .url("/users")
  .create({
    load: ({ body }) => ({ id: body.userId, name: body.name }),
  });

const line = createUser.line({
  body: { userId: "u1", name: "Ada" },
});

console.log(line.request().method);
console.log(line.value());
```

## Recipe: Shared Request Context

```ts
const workspaceApi = signals.api({
  auth: resourceAuth.workspace(),
}).scope({
  requestContext: ({ workspaceId }) =>
    resourceRequestContext({
      headers: { "x-workspace-id": workspaceId },
      correlationId: `workspace:${workspaceId}`,
    }),
});
```

## Recipe: Branch-Native Local Patch

```ts
const tasks = signals.api({
  effects: signals.resource.effects.branchNative(),
}).url("/tasks")
  .items((item: { id: string; title: string }) => item.id)
  .aspect("title", (item) => item.title, (item, title: string) => ({
    ...item,
    title,
  }))
  .list({
    load: () => [{ id: "t1", title: "First" }],
  });

const line = tasks.line({});
line.patch(
  tasks.patch.itemAspect({
    itemId: "t1",
    aspect: "title",
    value: "Updated",
  }),
);
```
