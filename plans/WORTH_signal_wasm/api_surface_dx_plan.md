# worth-signals-wasm API Surface DX Hardening Plan

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Prerequisite milestone:** [api_surface_closeout.md](./api_surface_closeout.md)
>
> **Adjacent milestone:** [router_navigation_projection_plan.md](./router_navigation_projection_plan.md)
>
> **Core vision:** [plans/worth_signal/worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
>
> **Resource certification parent:** [test-requirements.md](./test-requirements.md)

## Goal

Harden the shipped `worth-signals-wasm` API surface so normal application
authoring becomes materially easier, clearer, and less repetitive without
changing the already-closed runtime semantics underneath it.

This milestone is not about inventing a new async engine, a generic fetch
helper, or a Laravel-style magical resource abstraction.

It is about making the existing resource line model pleasant enough that a
serious frontend team would reach for it first instead of feeling forced back
to TanStack Query plus ad hoc API-client glue.

The governing standard is:

> The DX layer is a declaration compiler. It compresses repetition, but it
> never shrinks semantic truth.

The target product shape is:

- shared API defaults such as base URL, auth, and headers are declared once
- request defaults can be layered at app, section, and endpoint scope
- common declarations start with one obvious route entrypoint: `url(...)`
- route-first semantic finalizers carry the main semantic intent for the
  common lane
- explicit ids and explicit route intent remain visible in code
- path parameters stay readable and explicit
- request-side URL parameters use `params`, not `query`
- signed upload, multipart upload, deferred processing, and similar advanced
  paths stay inside the same authoring grammar as common reads
- the low-level family declaration surface remains available as the escape
  hatch rather than the default experience

## Why This Milestone Exists

Milestone 6 closed the semantics and proof burden for the resource/API line.
That was the correct first job.

But the current public authoring surface still looks too much like stabilized
substrate and not enough like an application-facing product.

Today the common lane still forces developers to repeat or over-think:

- `resourceParams(...)`
- `normalizeParams(...)`
- `resourceParamIdentity(...)`
- per-resource auth and header setup
- raw route construction inside `load(...)`
- large declaration objects even for ordinary CRUD-shaped reads

That means the current surface is semantically strong but ergonomically weak.

The package has already done this kind of hardening successfully once for the
main signal surface:

- semantics stayed strict
- authoring became much nicer
- the explicit/spec lane stayed available

The API surface now needs the same treatment.

The practical success condition is not just shorter declarations.

It is that a normal frontend engineer can open a feature module with five to
ten endpoints and understand:

- shared API posture
- route shape
- identity-bearing inputs
- common line usage

without needing to think in raw family declarations, manual param
normalization, `resourceParamIdentity(...)`, or handwritten request
construction unless the module is intentionally escaping the common lane.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the hostile DX failure
  first, not just shortening examples. The spec therefore starts from large-app
  authoring pressure, repetition, and adversarial nonstandard endpoints rather
  than from cosmetic builder sugar.
- `arch_laws.md`
  The most important laws here are 2, 7, 20, 33, 40, and 41. The new DX layer
  must preserve the existing authority split, keep route/request/builder
  semantics self-describing, and encode proof-bearing distinctions instead of
  flattening them into convenience blobs.
- `perf_laws.md`
  The most important thing it protects is cost honesty. Builder-shaped APIs
  must not hide broad invalidation, broad refresh, repeated reparsing, or a
  second request-planning engine behind cheap-looking calls.
- `domain_laws.md`
  The most important thing it protects is subsystem clarity. Shared API
  defaults, declaration-site route authoring, request parameter serialization,
  upload posture, and raw-family escape hatches need distinct homes instead of
  one mega-builder.
- `worth_signal_vision.md`
  The most important thing it protects is that `worth-signal` remains derived
  execution substrate. This milestone may improve authoring, but it must still
  lower to the same runtime-owned line, lifecycle, and diagnostics truth.
- `wasm_product_roadmap.md`
  The most important thing it protects is sequencing. DX hardening belongs
  after API-surface semantic closeout and before the router, because the router
  should consume a humane API lane instead of compensating for resource
  declaration ceremony.
- `api_surface_plan.md`
  The most important thing it protects is the line model and semantics already
  closed. This milestone must not reopen lifecycle, reconciliation, delivery,
  branch, replay, or compatibility semantics just to make declarations shorter.
- `api_surface_closeout.md`
  The most important thing it protects is closure honesty. The API resource
  milestone is closed semantically; this work is a new product hardening step,
  not an admission that the original closeout was fake.
- `router_navigation_projection_plan.md`
  The most important thing it protects is downstream consumption. The router
  needs a nice API surface for route-local resources and request defaults; it
  should not become the place where API authoring ergonomics are fixed by
  router-specific sugar.
- `resource_response_lens_contracts_plan.md`
  The most important thing it protects is response topology automation. The DX
  lane should hand off advanced response-shape patching to response lenses
  rather than expanding into manual per-route reconciliation folklore.
- `worth-signals-wasm/test-requirements.md`
  The most important thing it protects is proof quality. The new authoring lane
  must certify raw-vs-builder equivalence, inheritance honesty, and hostile
  nonstandard endpoint behavior instead of only shipping pleasant docs.

## Adversarial Constraint

This milestone must survive the following hostile condition:

> A large TypeScript application with shared auth and header posture, explicit
> named endpoints, conventional detail/list/paged reads, nested tenant or
> workspace routes, custom action endpoints, signed uploads, multipart uploads,
> deferred processing, pushed delivery, and a small number of ugly
> non-resourceful backend routes must converge to the same family identity, the
> same line lifecycle truth, the same diagnostics/history truth, and the same
> request-shaping truth regardless of whether the resource was authored through
> the new DX `url(...)` lane or the existing raw family declaration surface.

If the DX lane can produce:

- different family or line identity from the raw lane
- different request, freshness, upload, delivery, or restore semantics
- hidden broad work behind cheap-looking builder calls
- repeated per-endpoint header/auth duplication because inheritance is weak
- a pleasant CRUD lane that collapses when one endpoint becomes nonstandard
- or a second route/request planning abstraction that later fights the router

then the milestone has failed.

## Product Decision Lock

- keep explicit endpoint and declaration names visible in application code
- do not add a magical `resource("products")` abstraction as the default lane
- prefer one small route declaration grammar over giant declaration objects
- no `.build()` stage is part of the default lane
- default read/write intent should be expressible from `url(...)` plus semantic
  finalizers without redundant verb or shape restatement
- the common lane completes declarations through route-first semantic
  finalizers such as `.detail<User>()`, `.list<User>()`, `.paged<User>()`,
  `.create<User, Body>()`, `.update<User, Body>()`, and `.remove()`
- prefer `list` over `index`
- prefer `params` over `query` for request-side URL parameter vocabulary
- preserve explicit detail, list, paged, create, update, and remove
  distinctions
- shared API defaults such as base URL, headers, and auth should be declared
  once and inherited automatically
- nested API scopes should be able to add section-specific headers, auth, base
  URL fragments, and request conventions once and have them lower
  automatically
- the DX lane must degrade one step at a time for weird endpoints rather than
  forcing an immediate fall through to raw `load(...)`
- signed upload, multipart upload, deferred processing, and related advanced
  cases must remain inside the same builder grammar as ordinary reads
- the raw family declaration lane remains the escape hatch and semantic anchor
- the DX lane must lower to the exact same closed resource family and line
  model already certified under Milestone 6

Normative consequence:

- any DX surface that changes runtime semantics relative to the closed raw lane
  is out of spec
- any DX surface that hides costful or broad work behind deceptively cheap API
  shape is out of spec
- any DX surface that makes conventional CRUD pretty but adversarial custom
  endpoints miserable is out of spec
- any DX surface that replaces explicit names and route intent with opaque
  resource magic is out of spec

## Architectural Model

### Ownership split

This milestone freezes the intended ownership boundary:

1. **raw resource family surface**
   - remains the semantic authority at the wasm product boundary
   - continues to describe canonical family, line, lifecycle, reconciliation,
     delivery, download, replay, and restore behavior
2. **API DX declaration surface**
   - owns pleasant authoring, shared defaults, scoped inheritance, path
     interpolation, and common request vocabulary
   - lowers into the raw resource family surface
   - does not define independent lifecycle or cache semantics
3. **future router surface**
   - consumes the DX surface and the raw line model
   - does not own API authoring ergonomics

The DX layer is therefore not:

- a second resource runtime
- a second route system
- a generic HTTP client detached from line semantics
- a replacement for the raw lane's semantic authority

### Authoring model

The intended ergonomic direction is one explicit route entrypoint plus
route-first semantic finalizers.

Representative shape:

```ts
const getUser = url("/users/:userId").detail<User>();

const getUsers = url("/users")
  .params<{ search?: string; page?: number }>()
  .list<User>();

const createUser = url("/users").create<User, CreateUserBody>();

const updateUser = url("/users/:userId").update<User, UpdateUserBody>();

const uploadReceipt = url("/receipts/upload")
  .create<PreparedUpload>()
  .signedUpload({ finalize: true })
  .processing("poll");
```

Important constraints:

- the route string establishes path-param inference before semantic completion
- the semantic finalizer closes the family in one obvious place
- `url(...)` remains explicit and grepable
- shared defaults are inherited unless overridden
- weird endpoints stay in the same grammar by adding small explicit modifiers
  instead of switching to a second declaration API
- the result of the chain is already the usable family
- the common lane does not rely on declaration-binding annotations to complete
  family meaning

### Parameter model

The API must distinguish:

1. **path params**
   - values consumed by placeholders in `url("/products/:productId")`
   - should be inferred where possible
2. **request params**
   - extra serialized URL parameters for list/search/filter/paging flows
   - should use `params`, not `query`
3. **body payload**
   - explicit for non-GET-style operations when the endpoint takes a request
     body

The authoring goal is:

- infer path params from the URL where possible
- let semantic finalizers such as `.detail<T>()` and `.list<T>()` complete the
  common meaning without requiring repetitive identity ceremony
- keep request params explicit and named as `params`
- do not force developers to re-declare trivial identity by hand for the 80%
  case

The boundary must also stay explicit:

- path params are not request params
- request params are not body payload
- request params are not automatically identity-bearing merely because they are
  serialized

Normative consequence:

- `params(...)` defines request-side URL parameter vocabulary
- any request params that affect family identity, freshness partitioning,
  reconciliation visibility, or cache partitioning must be deterministically
  included by the lowering rules or explicitly classified by the API lane
- the DX surface must make it obvious whether `search`, `page`, `sort`, and
  similar request params create a different line identity

### Shared-default model

The DX declaration lane must support one explicit API root for shared request
posture:

- base URL
- shared headers
- shared auth
- shared serializers or request conventions
- shared inheritance rules

The intention is that most applications declare headers once and reuse them for
nearly every endpoint, with only explicit local overrides where needed.

The scoped inheritance model must also support intermediate API sections so
large apps can define shared request posture once for one part of the product
without contaminating unrelated routes.

Representative shape:

```ts
const api = signals.api({
  baseUrl: "/api",
  headers: () => ({
    Authorization: `Bearer ${session.token()}`,
  }),
});

const tenantApi = api.scope({
  headers: ({ tenantId }) => ({
    "x-tenant-id": tenantId,
  }),
});

const adminTenantApi = tenantApi.scope({
  headers: () => ({
    "x-admin-area": "true",
  }),
});

const getUser = tenantApi.url("/users/:userId").detail<User>();

const exportUsers = adminTenantApi
  .url("/users/export")
  .create<ExportJob>()
  .headers(() => ({
    "x-export-mode": "full",
  }));
```

The intended common lane is therefore:

```ts
const getUser = tenantApi.url("/users/:userId").detail<User>();
```

Required merge semantics:

- app-root defaults apply first
- nested API scopes apply in lexical order
- endpoint-local overrides apply last
- collisions resolve deterministically
- diagnostics and request inspection must reveal what was inherited versus what
  was overridden

Deterministic merging is necessary but not sufficient.

The product must also preserve source explainability. A large application team
must be able to answer why one line carried a given base URL, header, auth
posture, or serializer choice without reconstructing scope ancestry by hand.

Representative direction:

```ts
line.requestSummary();
```

should be able to explain not just the admitted value, but where that value
came from across root, scope, and endpoint overrides.

### Advanced-path model

The builder lane must handle adversarial but common real-world cases without
forcing immediate escape:

- signed upload preparation
- multipart upload preparation
- finalize-required upload contracts
- deferred processing via poll, callback, or webhook
- nested tenant/workspace/project routes
- custom action endpoints
- nonstandard but still explicit request shapes
- the occasional brutal endpoint that mixes path params, request params,
  headers, body, upload preparation, deferred completion, and push delivery

The degradation ladder must be:

1. standard `url(...)` path
2. `url(...)` plus one extra override or advanced step
3. explicit advanced `url(...)` shape with transport modifiers
4. raw family declaration escape hatch

Not:

1. standard builder path
2. immediate fall to raw `load(...)`

### Line-consumption model

The DX milestone must not stop at declaration compression.

The shipped product is still line-centered, and a real application team will
spend more time reading, inspecting, patching, delivering, and publishing
lines than authoring declarations.

That means the pleasant lane must also improve the normal read path for:

- current state
- request posture summaries
- upload, processing, and download summaries
- first-step diagnostics and explainability
- collection and paged patching helpers

The intent is not to delete the precise low-level line surface.

It is to ensure the common lane teaches and supports a smaller grouped read
path so ordinary UI code does not need to stitch together five unrelated calls
just to answer:

- is this line pending or stale
- what request posture actually applied
- what happened last
- is there upload, processing, or download state I should render

Normative consequence:

- declaration DX without line-consumption DX is incomplete
- any pleasant declaration lane that still leaves line reads feeling like raw
  substrate has underdelivered on the product goal
- every pleasant declaration must imply a pleasant first read
- cheap-looking grouped line reads must stay honest about cost and must not
  materialize richer replay or retained-history work accidentally

### Reconciliation and delivery model

The DX lane must treat list-shaped resources as first-class citizens rather
than as a pleasant detail-read lane plus raw collection escape.

That means collection and paged authoring must eventually support a compressed,
readable lane for:

- item identity
- narrow patch capability declaration
- item-aspect declaration
- summary declaration
- family-scoped patch helpers
- family-scoped delivery helpers

The minimum viable common truth must stay small.

If the line value is already a direct `Task[]`-style list, the pleasant lane
should not require a mini reconciliation DSL merely to unlock common
replace/update/delete helper ownership. Envelope-shaped values should pay more;
plain list-shaped values should stay tiny.

Representative direction:

```ts
const tasks = api
  .url("/workspaces/:workspaceId/tasks")
  .list<Task>()
  .items((item) => item.id)
  .reconcile((list) =>
    list
      .items((value) => value.items, (value, items) => ({ ...value, items }))
      .aspect("title", (item) => item.title, (item, title) => ({
        ...item,
        title,
      })),
  );

line.patch(
  tasks.patch.itemAspect({
    itemId: "t1",
    aspect: "title",
    value: "Updated",
  }),
);

line.deliver(
  tasks.delivery.patch({
    packetId: "pkt-1",
    basisId: null,
    patch: tasks.patch.itemAspect({
      itemId: "t1",
      aspect: "title",
      value: "Delivered",
    }),
  }),
);
```

The compatibility lane may remain more explicit and raw than the native lane.
But native collection/paged delivery must not require global helper soup once a
family already owns reconciliation truth.

### Binary and download model

The DX lane must explicitly include resource-binary and download-bearing
resources.

The package docs already teach that value and downloadable artifacts are
separate first-class surfaces. The hardening work must therefore avoid
producing a beautiful declaration lane for CRUD reads while leaving downloads
as a raw-only afterthought.

Representative direction:

```ts
const reportDetail = api
  .url("/reports/:reportId")
  .detail<Report>()
  .downloads((download) =>
    download.file("report-pdf", {
      fileName: ({ reportId }) => `${reportId}.pdf`,
      mediaType: "application/pdf",
    }),
  );
```

Normative consequence:

- download-bearing resources are in scope for pleasant authoring
- the pleasant lane must preserve the explicit split between `line.value()` and
  `line.download()` rather than flattening them into one magical payload

### Feature-module and host-default model

The DX lane must scale beyond isolated endpoint snippets.

That means the milestone must consider how shared API scopes and DX-authored
families live inside real feature modules, including:

- controller-local API scopes
- graph publication from resource lines
- feature-local grouping of related families
- host-derived auth and header defaults without ambient hidden magic

Representative direction:

```ts
const workspaceApi = signals.api({
  baseUrl: "/api",
  auth: signals.api.auth.sessionToken(() => session.token()),
  headers: ({ workspaceId }: { workspaceId: string }) => ({
    "x-workspace-id": workspaceId,
  }),
});

const productDetail =
  workspaceApi.url("/workspaces/:workspaceId/products/:productId").detail<Product>();

const products =
  workspaceApi.url("/workspaces/:workspaceId/products").list<Product>();
```

Normative consequence:

- shared defaults must remain explicit and typed even when they are derived
  from host or session truth
- the pleasant lane must scale to serious feature-module organization rather
  than only beautifying isolated examples

## Canonical Before / After

This milestone should be visually defensible in one glance.

Before:

```ts
const getUser = signals.resource.detail({
  params: resourceParams<{ userId: string }>(),
  normalizeParams: ({ userId }) =>
    resourceParamIdentity({ userId }, userId),
  auth: resourceAuth.authenticated(),
  requestContext: resourceRequestContext({
    headers: {
      Authorization: `Bearer ${session.token()}`,
    },
  }),
  load: ({ userId }) => fetchUser(userId),
});
```

After:

```ts
const api = signals.api({
  baseUrl: "/api",
  auth: sessionAuth,
});

const getUser = api.url("/users/:userId").detail<User>();
```

The milestone is only successful if the second form lowers to the same raw
family truth as the first form while remaining easier to read in a normal
feature module.

## Phases

### Phase 1: Request Scope Foundation

Purpose:

- remove repeated auth/header/base-URL boilerplate
- make shared request posture a first-class product concept

This phase must ship:

- one explicit API root surface for shared defaults
- nested API scope surfaces for section- or feature-local defaults
- inheritance for base URL, auth, and headers
- inheritance for request-side serializers and related request conventions where
  declared
- explicit local override semantics
- diagnostics-visible evidence of inherited versus overridden request posture

Phase 1 gate:

- no later phase begins until common shared-request posture can be declared
  once, nested per section, and proved to lower identically across multiple
  endpoint families

### Phase 2: Route Kernel Foundation

Purpose:

- establish one deterministic route grammar before endpoint semantics,
  transport modifiers, or router consumption build on top of it
- freeze the path/request identity boundary early so later DX layers do not
  reopen it

This phase must ship:

- one standard `url(...)` entrypoint for the common lane
- structural route parsing rather than regex-shaped string tricks
- compile-time route validity for segment structure and placeholder legality
- path-param inference from route placeholders where possible
- explicit `params(...)` support for request-side URL parameters
- clear separation between path params, request params, and body payloads
- canonical lowering into the closed family/member identity model
- typed and runtime-honest base-URL composition for route-authored families
- request-target inspection that stays semantically honest enough for later
  router consumption

Phase 2 gate:

- no later phase begins until URL declaration, path-param inference, request
  param serialization, and base-URL composition can be certified without
  hidden identity drift or request-target ambiguity

### Phase 3: Common Endpoint Grammar

Purpose:

- replace giant declaration objects for the common lane
- make conventional endpoint semantics explicit without reintroducing ceremony

This phase must ship:

- route-first semantic finalizers such as:
  - `.detail<T>()`
  - `.list<T>()`
  - `.paged<T>()`
  - `.create<T, Body>()`
  - `.update<T, Body>()`
  - `.remove()`
- semantic type families such as `Detail<T>`, `List<T>`, `Paged<T>`,
  `Create<T>`, `Update<T>`, and `Remove<T>` remain the conceptual categories
  expressed by the finalizers rather than by declaration-binding annotations
- no `.build()` in the common lane
- preserved explicit endpoint identity and readable call-site naming
- equivalence with the raw family declaration lane for common endpoint shapes
- compile-time proof that route-first semantic finalization preserves required
  `line(...)` call-site pressure and does not weaken path-param inference
- support for explicit custom action routes inside the same grammar
- hostile proof that weird endpoints degrade one step at a time rather than
  forcing immediate raw-lane escape

Phase 3 gate:

- no later phase begins until standard and nonstandard endpoint declarations
  can coexist in one grammar without semantic or readability collapse

### Phase 4: Collection-Owned DX

Purpose:

- make list-shaped resources first-class citizens in the pleasant lane instead
  of leaving them as a raw-only escape after nice detail reads

This phase must ship:

- a compressed pleasant lane for collection and paged reconciliation authoring,
  including item identity and narrow patch declarations
- family-scoped patch helper ownership for DX-authored collection and paged
  families
- family-scoped native delivery helper ownership for DX-authored collection and
  paged families
- a tiny common truth for direct `Task[]`-style lists, with richer
  reconciliation declaration only where envelopes or summaries actually demand
  it
- parity between builder-authored collection/paged declarations and raw
  reconciliation legality

Phase 4 gate:

- no later phase begins until list-shaped resources no longer require raw-only
  reconciliation ergonomics for the common lane and builder-authored patch or
  delivery helpers do not introduce a second mutation or delivery engine

### Phase 5: Upload, Processing, And Advanced Transfer Builders

Purpose:

- make advanced transfer contracts pleasant without reopening transport
  semantics

This phase must ship:

- `signedUpload(...)`
- `multipartUpload(...)`
- finalize-required upload declaration
- `processing("poll" | "callback" | "webhook", ...)`
- pleasant download-bearing declaration support for binary/resource descriptor
  cases
- explicit advanced modifiers such as `verb(...)`, `body(...)`, `headers(...)`,
  and related transport-shaping steps for truly nonstandard endpoints
- equivalence with the existing upload and processing posture semantics
- denial and diagnostics parity for advanced transfer builders

Phase 5 gate:

- no later phase begins until `url(...)`-authored upload and processing flows
  can be certified as semantically identical to the raw lane, and download-
  bearing resources fit the same pleasant grammar without semantic flattening

### Phase 6: Consumption, Escape Hatch, And Closeout

Purpose:

- close the hardening work without trapping the package in one authoring style
- make the pleasant lane feel pleasant after declaration, not only during it

This phase must ship:

- a clean raw-family escape hatch that remains first-class and documented
- diagnostics and history parity between builder and raw declarations
- a compressed line-consumption lane for common state, request-summary, and
  first-step explainability reads, while preserving the precise low-level line
  surface
- family-scoped delivery helper parity for DX-authored native collection and
  paged resources
- compile-time and runtime proof that builders do not overclaim capability
- docs that teach the pleasant lane first and the raw lane second

Phase 6 gate:

- the milestone is not closed until route-first `url(...)` declarations and
  raw-authored resource families converge exactly under hostile certification,
  the grouped line-consumption lane stays cost-honest, and the docs teach a
  clear default path across declaration, line usage, patching, delivery,
  transfers, downloads, and explainability

## Current Progress Snapshot

- `Phase 1: Request Scope Foundation`
  - status: materially complete
  - shipped: shared API root, nested scopes, inherited auth/headers/base URL,
    deterministic overrides, request-source explainability, raw-lane
    equivalence proofs
- `Phase 2: Route Kernel Foundation`
  - status: materially complete
  - shipped: deterministic `url(...)` route kernel, compile-time route
    validity, path-param inference, explicit `params(...)`, canonical identity
    lowering, base-URL composition, semantically honest request-target
    inspection
- `Phase 3: Common Endpoint Grammar`
  - status: partially complete
  - shipped: route-first read finalizers for `.detail(...)`, `.list(...)`, and
    `.paged(...)`, plus raw-lane equivalence and hostile route/param proofs
  - still open: `.create(...)`, `.update(...)`, `.remove()`, and explicit
    custom action support inside the same pleasant grammar
- `Phase 4: Collection-Owned DX`
  - status: started but not close to closed
  - shipped direction: builder-authored collection and paged families now own
    phase-specific DX work rather than being treated as a later afterthought
  - still open: compressed reconciliation authoring, family-scoped patch and
    delivery ownership, and proof that list-shaped resources no longer require
    raw-only ergonomics
- `Phase 5: Upload, Processing, And Advanced Transfer Builders`
  - status: not started in the DX lane
  - note: raw transport semantics exist and are certified, but the pleasant
    authoring grammar is still open work
- `Phase 6: Consumption, Escape Hatch, And Closeout`
  - status: not started as a coherent DX phase
  - note: some parity groundwork exists from the current resource surface, but
    the grouped consumption lane, raw escape-hatch teaching, and hostile
    closeout proofs remain open

## Must Ship

- one shared API root for common auth, headers, base URL, and inheritance
- nested API scopes for section-specific request defaults
- route-first semantic finalizers for common read and write intent
- explicit `url(...)` declaration as the common entrypoint
- explicit `params(...)` request parameter vocabulary
- path-param inference where possible
- no `.build()` in the normal lane
- compressed reconciliation authoring for common collection and paged cases
- family-scoped patch helpers for DX-authored list-shaped resources
- native family-scoped delivery helpers for DX-authored list-shaped resources
- signed upload, multipart upload, and deferred processing builder support
- pleasant binary/download declaration support
- custom action and nonstandard endpoint support inside the same grammar
- a compressed common lane for line state and first-step explainability reads
- a documented raw-lane escape hatch
- diagnostics, type, and runtime parity between builder and raw lanes

## Must Preserve

- the closed resource family and line semantics from Milestone 6
- explicit naming and explicit route intent
- one canonical line model
- runtime-owned lifecycle, freshness, retry, timeout, reconciliation,
  delivery, replay, and restore semantics
- cost honesty at the builder boundary
- future router consumption without router-owned API semantics

## Required Named Proof Families

- `The Shared Request Inheritance Equivalence Test`
- `The Scoped Request Defaults Inheritance Equivalence Test`
- `The Nested Request Defaults Override Honesty Test`
- `The URL And Raw Detail Equivalence Test`
- `The URL And Raw List Equivalence Test`
- `The URL And Raw Paged Equivalence Test`
- `The Declaration Type And Runtime Lowering Equivalence Test`
- `The URL Path Param Inference And Stable Identity Test`
- `The Route-First Semantic Finalizer Type Precision Test`
- `The Request Params Serialization And Identity Boundary Test`
- `The Conventional CRUD Declarations Parity Test`
- `The Nonstandard Endpoint Degradation Test`
- `The Reconciliation Builder And Raw Collection Equivalence Test`
- `The Family-Scoped Patch Helper Parity Test`
- `The Native Delivery Builder Parity Test`
- `The Signed Upload Builder Parity Test`
- `The Multipart Upload Builder Parity Test`
- `The Deferred Processing Builder Parity Test`
- `The Binary Download Builder Parity Test`
- `The Line Consumption Summary Honesty Test`
- `The DX Declaration Diagnostics And History Honesty Test`
- `The DX Declaration Capability Overclaim Compile-Time Boundary Test`

## Acceptance Evidence

This milestone is complete only when the wasm product surface can prove:

- shared API defaults lower identically across multiple endpoint families
- nested API scopes lower identically to their explicit fully-written request
  posture equivalents
- route-first semantic-finalizer authoring and raw-authored resources
  converge to the same family identity, line identity, lifecycle truth, and
  diagnostics/history truth
- a conventional feature module with five to ten endpoints can stay entirely in
  the pleasant lane without mentioning raw family declarations, manual param
  normalization, or request construction unless it is intentionally escaping
- path-param inference and request-param declaration do not destabilize
  canonical identity
- route-first semantic finalizers preserve `line(...)` call-site correctness
  for required path params and do not regress TypeScript inference quality
- conventional CRUD-shaped declarations are materially shorter without semantic
  drift or redundant verb/shape restatement
- collection and paged reconciliation declarations can be authored through a
  compressed lane without changing narrow-patch legality or runtime truth
- family-scoped patch and delivery helpers do not introduce a second mutation
  or delivery engine
- adversarial custom endpoints still fit the same grammar without forcing
  immediate raw-lane escape
- signed upload, multipart upload, and deferred processing builder steps remain
  exact semantic consumers of the closed transfer substrate
- download-bearing resources fit the pleasant lane without flattening value and
  artifact semantics together
- inherited versus overridden request posture is visible enough in diagnostics
  and request inspection that teams can reason about section-local defaults
- the common line-read lane remains honest about cost and does not materialize
  richer history or replay work just to provide a cheap-looking summary
- the docs recommend one obvious pleasant lane while preserving the raw lane as
  the explicit escape hatch

## Architectural Notes

- the builder surface should live as a separate authoring subsystem over the
  existing resource facade, not as a rewrite of the resource runtime itself
- route and request declaration should remain explicit enough to be grepable and
  reviewable
- the builder layer should prefer additive lowering helpers over a giant
  conversion blob
- the docs should teach a migration of thought:
  declare shared posture once, declare route shape, attach semantic intent, use
  the line, and only drop lower when the endpoint earns it
- line-consumption DX should prefer grouped read helpers over magical state
  fusion and must preserve the canonical underlying line surfaces
- the DX lane should likely become the default teaching lane for the later
  router milestone

## Sequencing Notes

This milestone belongs after API-surface closeout because:

- the semantic line model had to be real before ergonomics could safely harden
- the transfer, delivery, and compatibility lanes are now concrete enough to be
  wrapped honestly instead of guessed at

This milestone belongs before the router because:

- the router should consume a pleasant API declaration lane rather than invent
  its own route-local API sugar
- route-local resources will be much easier to author once shared request
  defaults and the `url(...)` declaration lane already exist

This milestone now also belongs before response lens contracts because:

- response lenses should deepen the pleasant API resource lane after the
  route/request authoring grammar exists
- collection-owned DX provides the immediate predecessor for broader
  response-shape automation

Current judgment:

- treat this as a new product-hardening milestone, not as a rewrite of the
  closed API milestone
- keep the raw lane as the semantic anchor
- make the pleasant lane the default recommendation for real app code
