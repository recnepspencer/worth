# Fetch And Write API Resources

Use this page when you want the normal route-first lane for fetching and
writing server-backed state.

## What This Covers

- `signals.api(...)`
- `api.scope(...)`
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
- advanced `.verb(...)`, `.body<T>()`, and `.headers(...)`

## Happy Path

```ts
import { createSignals } from "worth-signals-wasm";

const signals = await createSignals();

const api = signals.api({
  baseUrl: "/api",
  headers: {
    authorization: "Bearer shared-token",
  },
});

const users = api.url("/users")
  .params()
  .items((item: { id: string; name: string }) => item.id)
  .list({
    load: ({ params }) => [
      { id: `u:${params.search ?? "all"}`, name: "Ada" },
    ],
  });

const createUser = api.url("/users").create({
  load: ({ body }) => ({
    id: body.userId,
    name: body.name,
  }),
});

const listLine = users.line({ params: { search: "ada" } });
const createLine = createUser.line({
  body: { userId: "u1", name: "Ada" },
});

console.log(listLine.summary());
console.log(createLine.value());
```

## What `load(...)` May Return

A line value is a JSON value. Whatever `load(...)` resolves is committed as the
JSON it would serialize to: `toJSON` is honored exactly as `JSON.stringify`
honors it (a `Date` becomes its ISO string, a class with `toJSON` becomes what
it returns), `undefined` members are omitted, and non-finite numbers become
`null`. A value JSON would misrepresent is refused instead of stored as
something it is not: a `Map` or `Set` (serialized as `{}`), a typed array, a
boxed primitive, a function, or cyclic data settles the line `rejected` with a
message naming the class and the path (`resource line value cannot represent
Set at $value.sizes: ...`). Decode wire data to plain JSON before returning it,
or give rich types a `toJSON`.

## When To Use The Standard Finalizers

- `.detail(...)` for one resource member
- `.list(...)` for one collection snapshot
- `.paged(...)` for visible-window or accumulated-page flows
- `.create(...)` for standard POST-style writes
- `.update(...)` for standard PUT-style writes
- `.remove(...)` for standard DELETE-style removal
- `.mutation({ semantics, method?, ... })` when the transport method and the semantic mutation class should be declared honestly in one surface
- `.command({ semantics, method?, ... })` for command-style routes that should stay fallback-only instead of pretending to be visible CRUD

If those already match the endpoint, prefer them over `verb(...)`.

## When To Use Advanced Request Shaping

Use `.verb(...)`, `.body<T>()`, or `.headers(...)` when the endpoint is real
but nonstandard.

```ts
const exportUsers = api.url("/users/export")
  .verb("POST")
  .body<{ format: "csv" | "json" }>()
  .headers({ "x-export-mode": "full" })
  .detail({
    load: ({ body }) => ({ acceptedFormat: body.format }),
  });
```

That stays inside the same line model. It is still one family, one line, and
the same request inspection surface.

## Where To Go Next

- collections, reconcile, patch, delivery:
  [Collections And Delivery](./collections-and-delivery.md)
- uploads and processing:
  [Transfers](./transfers.md)
- downloads:
  [Downloads](./downloads.md)
- low-level route reference:
  [Route Authoring Reference](../api-reference/route-authoring.md)
