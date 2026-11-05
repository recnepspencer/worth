# API Route Authoring Reference

If you are trying to learn the feature surface instead of the full reference,
start with:

- [start_here.md](../start_here.md)
- [Fetch And Write Resources](../resources/fetch-and-write.md)
- [Resource Recipes](../start_here.md)

## What This Feature Is

This is the recommended way to declare API-backed resources in
`worth-signals-wasm`.

Use `signals.api(...)` when you want to:

- declare shared API defaults once
- build resources from explicit routes with `url(...)`
- keep common reads, writes, uploads, downloads, and list behavior in one
  grammar

Use `signals.apiScope(...)` when you want one stable runtime-scoped API identity
that can be reused across app features, catalogs, or helper layers without
app-authored cache wrappers.

In plain English: this is the pleasant app-facing lane for server-backed state.
It still produces the same kind of resource family and line as the lower-level
`signals.resource.*(...)` surface.

## Why You Use It

- keep auth, headers, and base URL setup out of every single resource
- declare route shape directly instead of hand-writing canonical keys first
- use one obvious lane for detail, list, paged, create, update, remove, command, and mutation
- stay in the same grammar when an endpoint needs uploads, processing, custom
  verbs, or downloads
- keep the raw family surface available as an escape hatch instead of the
  default path

## Stable Entry Points

Shared API posture:

- `signals.api(...)`
- `signals.apiScope(...)`
- `api.scope(...)`

Route-first authoring:

- `api.url(...)`
- `.params()`
- `.detail(...)`
- `.list(...)`
- `.paged(...)`
- `.create(...)`
- `.update(...)`
- `.remove(...)`
- `.mutation({ semantics, method?, ... })`
- `.command({ semantics, method?, ... })`

Collection-owned helpers:

- `.items(...)`
- `.reconcile(...)`
- `.aspect(...)`
- `.summary(...)`
- `.pageWindowSummary(...)`

Advanced request and transfer shaping:

- `.headers(...)`
- `.body<T>()`
- `.verb(...)`
- `.signedUpload(...)`
- `.multipartUpload(...)`
- `.processing(...)`
- `.downloads((params, value, download) => [...])`

## Core Mental Model

Think in four steps:

1. declare shared request posture once with `signals.api(...)`
2. use `signals.apiScope(...)` when one feature area needs a stable named API identity
3. narrow it with `scope(...)` when one feature area needs more defaults
4. declare one route with `url(...)`
5. finish it with the semantic finalizer that matches the endpoint

Example:

- `api.url("/users/:userId").detail(...)` means "one user by route param"
- `api.url("/users").params().list(...)` means "a list with request params"
- `api.url("/receipts/upload").create(...).signedUpload().processing("poll")`
  means "a write endpoint that also prepares upload and later processing"

For standard CRUD-shaped writes, the semantic finalizer is still:

- `.create(...)`
- `.update(...)`
- `.remove(...)`

When the transport method and the semantic mutation class should be expressed
honestly instead of guessed from a CRUD bucket, use:

- `.mutation({ semantics: "create" | "update" | "remove", method?, ... })`
- `.command({ semantics: "command" | "relationshipUpdate" | "aggregateMutation" | "sideEffect", method?, ... })`

`command(...)` is intentionally narrower in the response lane. It only admits
fallback-only reconciliation targets, so command routes do not pretend to prove
an exact visible-topology update they did not actually return.

The important boundary is this:

- the API route lane owns authoring ergonomics
- the resource family and line still own lifecycle, diagnostics, patching,
  delivery, and history semantics

So this surface makes declarations shorter and clearer. It does not invent a
second cache or a second runtime.

## How It Executes

When you materialize a line from a route-authored family, the runtime still
does the same real work:

1. resolve inherited defaults from API root, scopes, and endpoint overrides
2. resolve the route and path params into one stable canonical identity
3. resolve request params, body, headers, auth, and transfer posture
4. materialize the line
5. run `load(...)`
6. expose the result through the normal line surface

That means:

- `line.request()` still shows the admitted request posture
- `line.summary()` still gives the first grouped read
- `line.history()` and diagnostics still reflect the same runtime truth

## Small Example

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals();

const api = signals.api({
  baseUrl: "/api",
  headers: {
    authorization: "Bearer shared-token",
  },
});

const productDetail = api.url("/products/:productId").detail({
  load: ({ productId }) => ({
    id: productId,
    title: `Product ${productId}`,
  }),
});

const line = productDetail.line({ productId: "p1" });

console.log(line.summary());
```

Use this shape when:

- most endpoints share the same base URL or auth
- the route itself already tells the story of the resource
- you want the common lane without manual `resourceParams(...)` and
  `resourceParamIdentity(...)` ceremony

## Real Example

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals();

const workspaceApi = signals.apiScope("workspace-api", {
  baseUrl: "/api",
  headers: {
    authorization: "Bearer root-token",
  },
}).scope({
  headers: ({ workspaceId }) => ({
    "x-workspace-id": workspaceId,
  }),
});

const productDetail = workspaceApi
  .url("/workspaces/:workspaceId/products/:productId")
  .detail({
    load: ({ productId }) => ({
      id: productId,
      title: `Product ${productId}`,
    }),
  });

const products = workspaceApi
  .url("/workspaces/:workspaceId/products")
  .params()
  .items((item: { id: string; title: string }) => item.id)
  .list({
    load: ({ workspaceId, params }) => [
      {
        id: `${workspaceId}:${params.search ?? "all"}`,
        title: `Search ${params.search ?? "all"}`,
      },
    ],
  });

const exportProducts = workspaceApi
  .url("/workspaces/:workspaceId/products/export")
  .verb("POST")
  .body<{ format: "csv" | "json" }>()
  .processing("poll")
  .downloads(({ workspaceId }, _value, download) => [
      download.file("export-file", {
        fileName: `${workspaceId}-products.csv`,
        mediaType: "text/csv",
        download: download.ready({
          url: `https://downloads.example/${workspaceId}.csv`,
          method: "GET",
        }),
      }),
    ])
  .detail({
    load: ({ workspaceId, body }) => ({
      workspaceId,
      acceptedFormat: body.format,
    }),
  });

const productLine = productDetail.line({
  workspaceId: "acme",
  productId: "p7",
});

const productListLine = products.line({
  workspaceId: "acme",
  params: { search: "tools" },
});

console.log(productLine.summary());
console.log(productListLine.reconciliation());
```

What this example shows:

- `signals.apiScope(...)` gives a stable reusable API identity for the feature area
- API-root and scoped headers are inherited automatically
- route params stay explicit in the route string
- request params stay explicit in `params`
- collection helpers stay on the family once you declare item identity
- nonstandard request shaping, processing, and downloads stay in the same lane

The `download` builder inside `.downloads(...)` now covers both:

- `download.ready(...)` for plain GET/POST handoff
- `download.multipart(...)` for direct multipart host handoff with fields

## How It Relates To Other Features

- Move to [Resource Line Reference](./resource-line.md) once you
  are working with `family.line(...)`.
- Move to [Resource Request And Policy Reference](./resource-request-and-policy.md)
  when you need to reason about auth, headers, continuation, upload, or
  processing posture.
- Move to [Reconciliation Contract](../resources/responses/README.md)
  when a collection needs more advanced patch or summary behavior.
- Move to [Resource Family Authoring Reference](./resource-family-authoring.md)
  when you intentionally need the raw family declaration lane.

## Inspection And Debugging

Start with:

- `line.summary()`
- `line.request()`
- `line.diagnosticsSummary()`
- `line.history().availability`

For collection and paged families, also check:

- `family.patch`
- `family.delivery`
- `line.reconciliation()`

If shared defaults are involved, `line.request().sources` and
`line.diagnosticsSummary().request.sources` tell you where auth, headers, and
base URL came from.

## Anti-Patterns

- teaching app teams the raw `signals.resource.*(...)` lane first for ordinary
  CRUD-style endpoints
- rebuilding shared auth and header posture inside each `load(...)`
- using `verb(...)` and `body<T>()` for standard create/update/remove routes
  when the semantic finalizer already says what the endpoint is
- dropping to the raw lane as soon as one endpoint needs uploads, processing,
  or downloads

## Current Limits

- the route lane is the recommended default, not the only valid lane
- the raw family declaration surface is still the right choice when you need
  full manual control over canonical identity or are working directly with the
  lower-level compatibility surface
- exact replay and exact restore still depend on what the runtime supports;
  route authoring does not change that boundary

## Related Docs

- [Resource Overview](../resources/overview.md)
- [Resource Line Reference](./resource-line.md)
- [Resource Request And Policy Reference](./resource-request-and-policy.md)
- [Reconciliation Contract](../resources/responses/README.md)
- [Resource Binary And Download Reference](./resource-binary-and-download.md)
- [Resource Family Authoring Reference](./resource-family-authoring.md)

Transport methods include `GET`, `POST`, `PUT`, `PATCH`, and `DELETE`. For a partial-update endpoint, `.mutation({ semantics: "update", method: "PATCH", load })` preserves the actual HTTP method in its request and mutation proof.
