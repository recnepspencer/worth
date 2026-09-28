# Milestone 3.13: Query Binding and Projection Consumption Substrate

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Status and Placement

Status: Complete. Phases 1 through 5 closed on 2026-07-29.

All `QP-01` through `QP-10` rows are proved. The permanent Platform Pulse
renders pending and current Query values, preserves its predecessor across an
incompatible schema, recovers visibly, and closes every governed resource.
Compiled public examples, documentation topology, compile contracts,
dependency boundaries, line caps, formatting, strict workspace lint, and the
ordinary certification lane are green. The 65,536-row Query world remains an
explicit closure-stress lane rather than inflating the ordinary edit loop.

Milestone 3.12 is complete. It established the one observation turn, bounded
affected-scope planning, atomic successor publication, and the permanent
Platform Pulse rebind world. Milestone 3.13 adds data-bearing Query projection
facts beneath those authorities. It does not reopen the observation turn,
planner, mounting identity, publication boundary, or native-window harness.

Milestone 3.9.2 already proved the narrower pattern this milestone must
generalize:

- Query owns installation, execution, settlement, live-resource lifecycle,
  invalidation, result state, warnings, and exact native projection access;
- `worth-ui-query-binding` validates exact Query authority and emits compact
  UI-owned facts and consequences;
- the UI runtime consumes those artifacts without importing Query;
- selected native access is contract-derived and bounded by selected fact
  count, not projection width; and
- printable equality, field scans, positional fact bags, and reporting
  projections cannot substitute for authority.

Milestone 3.13 broadens that seam from measurement and collection-change
consequences into reusable scalar and collection presentation. Milestone 6
still owns rich data products, large surfaces, joins, pagination UX, and
serious product-specific consumption.

## Goal and Central Claim

A file- or Rust-authored UI may display a Query-owned value only through this
progression:

```text
declared projection requirement
-> installed Query view and exact consumer contract
-> pair-bound Query compatibility witness
-> admitted UiProjectionBinding
-> Query-owned execution, publication, and projection outcome
-> Query consumed progression carrying WorthQueryConsumedProjectionAuthority
-> contract-keyed native projection consumption
-> UiProjectionFactReceipt
-> 3.12 Query observation and affected-scope plan
-> mounted semantic content tied to a node receipt
-> host presentation
-> externally visible pixels
```

The visible value, result posture, logical field or row identity, generation,
and mounted identity must remain causally correlatable across that chain.

No raw Query operation, workspace, result, patch, warning, field string,
position, diagnostic object, renderer callback, test fixture, or locally
paired basis/receipt/fact tuple may skip a stage or mint later authority. A
Query-free application must construct none of this path and pay no Query or
projection-consumption cost.

The milestone is complete only when the permanent Platform Pulse visibly
progresses from a genuine Query-owned non-current posture to a genuine current
native value in one real product process, and the supporting portfolio proves
schema compatibility, stale/revalidating state, collection identity,
wrong-world rejection, bounded invalidation, lifecycle cleanup, and
compile-time anti-bypass boundaries.

## Current Boundary and Exact Gap

The repository already has installed snapshot/live views; exact Query evidence
and contract-derived native access; retained measurement facts and UI-owned
collection consequences; the 3.12 observation/rebind/publication path; mounted
identity, filled-rectangle host paint, visual tracing; and the cumulative
Platform Pulse process and teardown contract.

The missing product contract is not a binding registry. It is the complete
data-bearing path from an installed Query projection to mounted semantic
content.

- `WorthUiSettledSnapshotFact` is measurement-specific;
- `WorthUiCollectionChangeConsequence` describes collection change effects,
  not a general displayed collection value;
- the snapshot request currently selects the literal native field name
  `"value"` and cannot become the general binding grammar;
- result-state reachability is not equally proven for every existing Query
  operation lane;
- the public Query-binding guide describes an execution extension that the
  `worth_ui::facade::query_binding` product facade intentionally does not
  expose; and
- the mounted host projection and egui adapter can paint filled rectangles but
  cannot yet present semantic text.

Private mechanics may change. A generic fact bag, local status enum, renderer
query, or diagnostic-only demonstration cannot close these gaps.

## Decisive Product Courtroom: QP-01

### Real Entry Surface

`QP-01` extends the existing `worth-ui-platform-pulse` executable-world target:
the Cargo-built production binary, checked-in `main.wui`, ordinary `main`,
existing 3.12 runtime/host/visual path, independent client-area capture, and
native close. It adds `--query-source-root <path>`; the application adapter
observes exactly `<path>/platform-pulse-value.json`, converts its bytes to
native Query input, and submits it through the Query host audience. The Pulse
composition root declares and installs meaning through `worth-query-decl` and
`worth-query-host`; only `worth-ui-query-binding` advances the installed
consumer progression reached through `worth_ui::facade::query_binding`.

JSON may be used as the external Pulse adapter's transport format. It must be
parsed into native Query input before projection. Query projection consumption
may not serialize through JSON, debug text, or a product-local value clone.
The checked-in sample records use the shape
`{"status":"ONLINE","revision":1}` and
`{"status":"UPDATED-LONG","revision":2}`. Those bytes are lawful world input
because the product adapter and Query consume them; neither runner nor UI code
extracts their values.

The runner may atomically publish data bytes and use the existing 3.12 source
edit. It may not construct Query/UI authority, mounted content, or host effects,
nor call a product-local completion command.

This is the next state of the same cumulative Pulse journey, not a replacement
journey. It retains the 3.10.3 isolated world and process-bound observation,
the 3.11 mounted-identity trace and overlay round trip, and the 3.12
valid/malformed/recovery source sequence. It still uses one product process,
one native window, one observation stream, one isolated world, no blind retry,
and the existing executable-world target. Phase 1 freezes the expanded exact
lifecycle-event count from this contracted sequence before implementation; it
may not derive the expected count from observed output. The journey performs
exactly seven external data/source file mutations, two overlay actions, and
ten adjudicated native captures: pending, first-current, overlay, cleared,
second-current, green, malformed-preserved, blue-recovered, schema-stopped,
and final-recovered. Every awaited transition remains at most 5 seconds, first
publication retains the 500 ms liveness hold, full journey time is at most
45 seconds, failure teardown is at most 5 seconds, lifecycle evidence remains
at most 256 events and 1 MiB, and retained failure artifacts remain at most
64 MiB. Each capture retains the 3.11 limits of at most 4x DPI scale and
983,040 pixels.

### Nondegenerate World

The page retains the blue background and yellow identity target. A new scalar
status group has stable value and posture slots with distinct declaration,
graph, allocation, mounted-content, and sampled-pixel identities. Installed
meaning exists before freeze. With the external record absent, Query owns
`pending`. Publishing native text `status` plus non-text `revision` produces a
current projection and visible `status` without process restart. The Pulse
retains one Query-owned live owner and one move-only consumer lease; external
changes advance through owner-issued refresh/invalidation, never a polling or
fresh-execution loop.

### Hostile Sequence

1. Launch with valid `.wui` source, installed Query meaning, and no external
   value record.
2. Prove Query owns the pending result; capture the page, value and background
   points, mounted-content receipt, application generation, and process
   identity; then prove the process remains live for 500 ms.
3. Atomically publish the first valid external record.
4. Await Query-issued and product-issued progression evidence rather than a
   sleep; prove the same binding and mounted logical node present the exact
   current native value.
5. Run the inherited snapshot/identity courtroom against both value and
   background points. Publish the mounted overlay, prove its external pixels
   and identity correlation, clear it, and prove restoration.
6. Publish the second valid record with materially different glyph geometry.
7. Prove bounded Query invalidation, exactly one 3.12 turn, stable logical binding,
   3.12-admitted mounted identity, a new fact generation, and the second value.
8. Apply the inherited valid green source replacement and prove the Query
   binding, value, and Query fact generation remain unchanged while only
   admitted source consumers change.
9. Apply the inherited malformed source replacement and prove typed denial,
   exact green/value preservation, and no successor authority; restore the
   canonical blue source and prove recovery.
10. Apply a valid `.wui` edit that binds the text slot to non-text `revision`.
    Prove exact schema mismatch, predecessor-value policy, and mounted
    diagnostic correlation without an invented value; restore `status` and
    prove recovery.
11. Close the native window and prove zero surviving source-adapter, Query
    attempt, live resource, lease, retained projection, projection receipt,
    observation, rebind, mounted-content, host, visual, process, or
    installation resources.

### Required Verdict

The initial page visibly shows a Query-owned pending posture. The first valid
record visibly produces the exact current value. The second valid record
changes the glyph region and the projection fact generation while preserving
the admitted logical binding and mounted instance.

The incompatible binding edit produces no fabricated value and no widened
graph work. Its typed stop is correlated with the exact authored requirement,
installed Query contract, mounted diagnostic reference, and retained
predecessor policy. Recovery uses the same source and Query lanes.

The yellow identity target remains stable throughout. The background remains
blue across Query-data and schema-stop transitions; only the inherited valid
source replacement changes it to green, the malformed edit preserves green,
and canonical recovery restores blue. Those controls convict implementations
that replace the whole page, repaint unrelated nodes, or mistake process
liveness for data closure.

### Independent Oracles and Mutations

No single oracle closes `QP-01`. It combines Query result/native-access
evidence; binding receipts and counters; 3.12 scope, plan, preservation, and
publication; mounted content/diagnostic receipts; external screenshots and
glyph-region adjudication; stable controls and process identity; and manually
readable retained failure screenshots. Expected `ONLINE` and `UPDATED-LONG`
values are frozen in the scenario contract independently of product lifecycle
output; lifecycle text, renderer text, or a screenshot generated by the
product may not become its own expected value.

At least these mutations must turn the courtroom red:

- replace the Query value with a product literal;
- make pending/current a UI-local enum;
- select a field by scanning names or using the literal `"value"`;
- read Query from the renderer or host adapter;
- accept an equal-looking projection from another world;
- derive collection row identity from position;
- drop binding generation from receipt admission;
- route invalidation around the 3.12 observation turn;
- publish diagnostics without mounting the value or posture;
- skip an inherited overlay or source-rebind transition while Query still
  succeeds;
- derive the expected value from product lifecycle or renderer output;
- hard-code the first Pulse value;
- suppress the second value while emitting success events;
- repaint the background instead of the content region; or
- leak the live owner or retained projection on close.

## Supporting Proof Portfolio

| ID | World and adversary | Required verdict and independent control |
| --- | --- | --- |
| `QP-02` | Drive a production Query result current -> stale/revalidating -> current, then every failed, cancelled, retried, superseded, denied, unsupported, remasked, and basis/generation-drift result in the Phase 1 mapping ledger. | Preserve distinct posture/recovery meaning and predecessor truth. A Query-issued result is the stimulus; no WUI-local status producer or string oracle is allowed. |
| `QP-03` | Cross compatible/incompatible installed view replacement with field-family, row-identity, payload-family, cardinality, and foreign-world mismatch. | Preserve identity only with Query compatibility proof; otherwise return the exact schema-, payload-, or rebind-stop and mint no successor receipt. Equal-looking controls must differ only in the governing proof. |
| `QP-04` | Through the real lease/window path, run 0, 1, 1,024, and 65,536 rows with changed sets of 0, 1, and 64 across insert, update, reorder, remove, explicit reset, partial/complete, and continuation changes; replay duplicate, reordered, superseded, foreign, and wrong-window patches. | Apply the exact Query patch before UI consequences. Query-authorized row identity survives reorder; every hostile patch yields its exact denial and zero UI effect; work follows selected/changed rows, not collection size. Independent expected keyed sets and boundary counters reject positional identity, raw patches, or full diff. |
| `QP-05` | Present equal printable declarations, keys, values, digests, and generations from the right world, a foreign world, a stale generation, and a reporting-only projection. | Only the exact operational proof admits effects. Every twin returns the exact stop with zero plans, publications, or retained authority. |
| `QP-06` | Deterministically permute Query change, source edit, and viewport observation in one turn while one rebind is effecting; exercise queue occupancy 0, 15, 16, and 17. | All permutations yield the frozen canonical order and one atomic publication; the seventeenth queued observation is a typed capacity stop. Per-source indexed counters reject global invalidation, OS-timing oracles, and a second queue/executor. |
| `QP-07` | Inject denial, cancellation at every 3.12 safe point, replacement, retry, retirement, and shutdown around scalar and collection attempts, receipts, continuations, and diagnostic detail. | A production resource census proves every owner/lease/projection/receipt is disposed exactly once, retry starts clean, and repeated cleanup is harmless. |
| `QP-08` | Compare Query-free, unchanged, one-change, and 16-change turns against projection widths and mounted graphs of 1, 1,024, and 65,536. | Frozen structural counters prove zero Query/content work on the first two and work proportional to admitted selections/changes on the latter two; elapsed time is diagnostic only. |
| `QP-09` | In the existing two-session compile owner, pair each valid use with attempts to construct or clone authority, reassemble consumed authority from projections, cross scalar/collection APIs, import raw Query outside the owner, make Pulse depend on the owner, import Query in host/runtime, or use diagnostics as authority. | Valid twins compile and each invalid twin fails for the intended type or dependency reason. Query Consumer Kit support pins, prohibitions, and residue reports close downstream adoption without a local scanner, new compile crate, target, or nested Cargo session. |
| `QP-10` | Generate DSL/Rust pairs with whitespace, declaration-order, and import-order permutations, then change view, field, shape, native family, or lifecycle requirement one axis at a time. | An independent canonical requirement model proves non-semantic variants equal and semantic variants unequal; compiler output cannot generate its own oracle. |

Each row enters the milestone ledger separately. `QP-01` cannot be substituted
with certification-only evidence, and supporting rows cannot be marked closed
because `QP-01` happens to pass.

The Phase 1 proof contract contains exactly `QP-01`--`QP-10`, in order, and
reuses the 3.12 ledger fields
`id,production_claim,fixture_provenance,typed_result,mutation_control,structural_cost,teardown,evidence_command,status,evidence`.
The existing topology audit rejects missing, duplicated, reordered, reopened,
command-drifted, or fabricated rows. Parameterization or a shared process is
allowed only when each row retains an independently failing verdict and
mutation control; no new test target or compile island is allowed.

## Product Decision Lock

### 1. Binding, Result, and Stop Are Different Axes

Roadmap posture words are not one enum.

```text
binding admission
  ready(admitted binding)
  stopped(binding-stop receipt)

fact observation after ready
  unavailable(pending or transition receipt)
  present(current or retained-stale value plus activity)
  stopped(fact-stop receipt)
```

`stale + revalidating` must be representable. `pending + current`,
`wrong-world + admitted`, and collection continuation on a scalar binding must
be unrepresentable.

The compile-time shape is:

```rust
pub enum UiProjectionAvailability<V> {
    Unavailable(UiProjectionUnavailableReceipt),
    Present(UiPresentProjection<V>),
    Stopped(UiProjectionFactStopReceipt),
}

pub enum UiPresentProjection<V> {
    Current(V),
    RetainedStale {
        value: V,
        activity: UiProjectionRetainedActivityReceipt,
    },
}
```

Fields and evidence constructors remain private. Scalar and collection receipts
specialize `V`; only collection receipts add completeness and continuation.

The binding owner derives this view losslessly from Query's primary, async,
cause, basis, remask, and failure-retention posture. Pending, current,
retained-stale, revalidating, failed, cancelled, retried, superseded, denied,
unsupported, remasked, and basis/generation drift remain distinguishable where
Query gives them different lifecycle or recovery meaning. They cannot collapse
to loading/error strings. A Phase 1 exhaustive mapping ledger fixes each
reachable Query posture to one nested UI outcome and predecessor-value policy;
elapsed time, missing UI data, or renderer behavior cannot decide it.
The normative mapping is
`milestone-3.13-phase-1-result-posture-ledger.csv`; implementation and
certification must consume every row without adding an unledgered local state.

### 2. Shape Is Compiler-Visible

The public contract has shape-specific types:

- `UiScalarProjectionBinding`
- `UiCollectionProjectionBinding`
- `UiScalarProjectionFactReceipt`
- `UiCollectionProjectionFactReceipt`

`UiProjectionBinding` and `UiProjectionFactReceipt` are the common opaque
identity/provenance cores. Their constructors remain binding-owner private.
Shape-specific wrappers expose only lawful operations. A scalar fact receipt
contains an exhaustive state view: pending, present, or stopped. Present then
contains current or stale currency; stale contains idle or revalidating
refresh. Collection receipts add completeness and continuation. A sealed shape
marker may be used internally, but no generic authority-marker bound may open
a governed path.

Scalar bindings carry exactly one contract-derived selected field. Collection
bindings carry a contract-derived row identity field, selected value fields,
and explicit completeness/continuation requirements. A scalar cannot request
rows or continuation. A collection cannot silently collapse to one positional
value.

### 3. Schema and Field Identity Are Contract-Derived

The canonical declaration records:

- installed view identity;
- scalar or collection shape;
- exact selected native field identities;
- required native value families;
- row identity contract for collections;
- lifecycle requirement;
- schema/view-shape compatibility requirement;
- payload-shape requirement; and
- consumed UI fact/aspect contracts.

Source strings are parsed and resolved before binding admission. Runtime
consumption receives Query-issued native-access keys from the exact installed
consumer contract. It performs `k` keyed accesses for `k` selected fields,
independent of unrelated projection width `w`.

Equal names, labels, digests, ordinals, or value representations do not prove
field, view, schema, binding, or world identity. Compatible replacement exists
only when Query issues compatibility evidence covering the exact old/new
contract and selected fields.

WUI presentation and allocation needs travel in Query's separate consumer
boundary. They participate in pair admission but cannot rewrite the installed
operation's Query requirements, projection mask, or lifecycle posture.

### 4. Native Values Remain Native

The binding owner accepts exactly one Query consumed progression carrying its
sealed downstream authority by value and derives compact UI projection facts
once. It does not extract and reconstruct that authority from basis, lineage,
facts, receipts, requirements, labels, or digests. Native-access keys come
from the exact Query consumer contract. The UI runtime receives an opaque
binding-owned native value view or immutable value artifact; it does not
import Query types, clone Query result state, construct keys, or scan a
retained projection.

JSON, debug strings, diagnostic formatting, and serialized inspection
projections are not the operational value lane.

Milestone 3.13's authored visible content is deliberately narrow:

- direct native text scalar presentation; and
- a bounded collection of direct native text values.

The binding substrate may preserve other Foundational native families for
future consumers, but this milestone does not invent formatting expressions,
coercions, transforms, conditional fallback, joins, or locale policy.
Milestone 3.17 owns authored expression and formatting semantics.

### 5. Projection Facts Are Derived, Affine Evidence

A `UiProjectionFactReceipt` proves:

- one admitted binding consumed one exact Query projection outcome through its
  Query-minted consumed progression and sealed downstream authority;
- the exact Query world, binding, view, schema, selected field set, source
  generation, and result generation were validated;
- pending or stopped state came from that exact Query transition, or a present
  payload satisfied the declared shape and native family;
- the result posture was exhaustively translated; and
- the enclosed UI fact may enter the declared 3.12 Query observation family.

It authorizes no Query execution, continuation, refresh, installation,
recovery, or mutation. It cannot be deserialized into authority, rebound to a
different application generation, or admitted twice after consumption.

Diagnostic and inspection views may cite the receipt. They cannot recreate or
readmit it.

Binding admission stops and result-consumption stops use separate sealed
receipt types even when they share a human-facing category such as
`schema-mismatch` or `wrong-world`. A category is not authority to cross a
different boundary.

### 6. Invalidation Reuses the 3.12 Compiler

Query invalidation produces an owner-issued projection observation/fact. It
enters the existing admitted observation set, classification owners,
fact/aspect indexes, immutable rebind plan, presentation, and atomic
publication.

Milestone 3.13 adds Query projection fact families and binding strategies
beneath those extension points. It does not add:

- a binding refresh loop;
- a renderer subscription;
- a host callback;
- a generic event bus;
- direct graph mutation;
- a second publication queue; or
- current-workspace rediscovery during planning or rendering.

Mixed source changes use the canonical 3.12 ordering. A failed or stale Query
candidate preserves the last admitted application and value according to the
typed Query policy.

### 7. Collection Identity and Delivery Stay Query-Owned

Collection delivery generalizes the existing admitted-source pattern without
exporting raw Query patches.

The binding owner advances through the Query-owned live owner and move-only
lease. It derives and readmits the exact capability-bound invalidation delta,
binds that delta to the current consumer window, and applies the exact Query
patch before deriving UI consequences. It then emits UI-opaque row references.
Runtime consequences may describe insert, update, remove, move, reset,
completeness, and continuation changes, but only from that applied delivery.
The UI may widen its own response; it cannot downgrade Query's required
disposition or reconstruct lease, generation, ordering, cursor, row identity,
or patch posture.

The ordinary lane may touch only changed rows plus explicitly affected
consumers. It may not diff the whole collection, hash all values, infer reset
from volume, or use an array index as identity. Reset is explicit and preserves
no identity that Query did not preserve.

### 8. Semantic Text Is a Mounted Host Contract

Human-visible data requires a new host-neutral semantic text projection. The
runtime completes each text row from:

- a mounted node receipt;
- admitted semantic text content;
- allocation and clip basis;
- an existing resolved text-color/theme contract;
- the frozen `UiSemanticTextProfile::BodyDefault` size, weight, wrap, and
  baseline posture;
- semantic layer order;
- frame, surface, and binding generations; and
- the current mounted-text schema version.

The host contract carries completed mechanics, not Query identity or UI
binding authority. The egui adapter validates protocol, schema, row basis,
node receipt, allocation, and capability before translating text to native
paint. Intrinsic size and baseline use exact evidence from the existing host
measurement contract; the adapter cannot invent geometry from local font
defaults.

The adapter may choose native glyph mechanics consistent with the negotiated
host profile. It may not choose the value, posture, field, fallback, semantic
color, identity, or clipping policy.

3.13 ships only `BodyDefault`; 3.16 broadens appearance without changing the
content or receipt path.

Filled-rectangle and text tables remain separate because their mechanics,
capabilities, cost, and future growth differ. A generic paint payload or
renderer command list is forbidden.

### 9. Diagnostics Explain; They Do Not Decide

Schema mismatch, payload mismatch, denial, unsupported, wrong-world, and
rebind-required outcomes carry stable typed references. Where the declaration
requests visible posture, the mounted node contains semantic posture content
and an existing mounted diagnostic reference.

An attempt-scoped binding stop may update the active page's posture slot
through the existing 3.12 observation and atomic presentation path while the
active application generation, admitted binding, and last good value remain
unchanged. The stop receipt and presentation generation are distinct from a
successor binding receipt; no failed candidate identity becomes mounted
authority.

The ordinary receipt retains only compact causal references and bounded
summary data. Rich Query or UI diagnostic material is loaded lazily through
the inspection/diagnostic policy and remains subject to scope, relevance,
disclosure, retention, and byte budgets.

No diagnostic code, message, label, serialized projection, or AI-oriented
summary may be accepted back as binding, result, compatibility, or rebind
authority.

### 10. Product Dependency Grammar Is Explicit

Application code uses:

- `worth_ui::facade::query_binding` for WUI projection declarations,
  registration, binding outcomes, and UI-facing receipts;
- `worth_query_decl::facade` for Query declaration meaning; and
- `worth_query_host::facade` for production Query installation and execution.

The Platform Pulse may add direct dependencies on `worth-query-decl` and
`worth-query-host`. It may not depend directly on `worth-query`,
`worth-ui-query-binding`, `worth-ui-runtime`, certification, replay, or test
support.

Within the Pulse package, Query declaration/host imports are confined
mechanically to `query_source/**` and the composition-root installation seam.
`native_frame`, visual, observation-projection, and host-adapter modules cannot
import or receive a Query workspace, operation, result, or live handle.

`worth-ui-query-binding` remains the only WUI production crate importing the
Query engine. The `worth-ui` product facade may orchestrate audience-facade
types and re-export UI-owned contracts, but it may not re-export raw Query
execution, settlement, patch, lease, or workspace internals.

The misleading `WorthUiQueryWorkspaceExt` product-guide example must be
replaced by the final facade grammar. The runtime's broader internal
Query-binding facade is not a product API.

### 11. Capacity and Cost Are Contractual

Every consumption turn carries `UiProjectionConsumptionBudget` with independent
limits for:

- bindings admitted;
- scalar fields accessed;
- collection rows and change operations;
- continuation work;
- native bytes retained;
- diagnostic summary bytes; and
- rich diagnostic materialization.

The Platform Pulse profile is exactly:

- one scalar binding;
- one selected field;
- zero collection rows;
- at most one Query projection change per observation turn; and
- at most two mounted text rows: one stable value slot and one posture slot.

Supporting collection scenarios declare their own bounded `n` and `k`.

Required ordinary complexity is:

```text
native access          O(k), independent of unrelated projection width w
binding classification O(changed bindings + indexed consumers)
scalar presentation    O(changed scalar values)
collection update      O(changed rows + declared move/reset work)
unchanged frame        O(1) dispatch with zero Query/content work
```

Counters must separately expose Query execution, selected native accesses,
binding admissions, result transitions, changed rows, affected consumers,
content rows completed, host text rows presented, suppressed work, and rich
diagnostic materialization. A single aggregate "work" counter is insufficient.

All queues, retained generations, continuations, diagnostic detail, and live
resources are bounded and disposed by named owners.

## Public Developer Experience

The target WUI registration surface is:

```rust
use worth_ui::facade::{
    app::WorthUi,
    query_binding::{
        UiProjectionFieldRequirement,
        UiScalarProjectionRegistration,
    },
};

let status = UiScalarProjectionRegistration::text(
    installed_query_view,
    UiProjectionFieldRequirement::declared("status")?,
)?;

let app = WorthUi::app()
    .register_scalar_projection(status)?
    .freeze()?;
```

`installed_query_view` is an audience-safe reference to declared/installed
meaning, not an operating-world handle or consumption authority. Host assembly
declares and installs through `worth-query-decl` and `worth-query-host`.
`worth-ui-query-binding` then uses Query's installed consumer surface to obtain
the exact consumer contract and consumed progression carrying its sealed
authority. The continuing guide must show those distinct steps. WUI
registration accepts the safe semantic reference and declared field
requirement; it does not accept a raw operation, workspace, result, field-name
scan, locally assembled authority, or closure that can query during rendering.

The 3.13 DSL surface is direct and intentionally small:

```text
query_scalar platform.pulse.status {
  view platform.pulse.status
  field status
  require text
}

component platform.pulse.projected_status {
  content projection platform.pulse.status
}
```

This spelling is normative unless Phase 1 discovers a collision with already
shipped grammar; any correction must preserve the same explicit view, field,
shape, native-family, and content-consumption meaning and be amended here
before Phase 2.

The DSL cannot author:

- a Query expression or operation body;
- a runtime handle or workspace;
- a loading/error state machine;
- a renderer callback;
- a field fallback scan;
- formatting, coercion, transformation, conditional, or join logic;
- a collection row key based on position; or
- host- or egui-specific text mechanics.

Rust-authored and file-authored declarations lower to the same canonical
projection requirement and receive the same binding, compatibility,
invalidation, and diagnostic behavior.

## Compile-Time and Mechanical Enforcement

The earliest honest boundary must reject misuse:

- admitted bindings, compatibility proofs, fact receipts, row references, and
  mounted text mechanics have no public constructors;
- public outcomes are exhaustive and `#[must_use]`;
- shape-specific wrappers prevent scalar/collection API crossover;
- prepared admissions/transitions and fact receipts do not implement `Copy` or
  `Clone` and are consumed by value; shareable binding references carry
  identity only and cannot duplicate execution or consumption authority;
- diagnostic and inspection projections expose no conversion into operational
  types;
- serialized forms contain identifiers and summaries only, never authority;
- runtime and host crates have no Query dependency;
- the Pulse has no direct binding-crate, runtime, certification, replay, or raw
  Query dependency;
- source-boundary checks confine the Pulse's declaration/host imports to its
  Query-source and composition-root modules;
- `worth-ui-query-binding` remains the sole allowed WUI Query importer;
- schema/view/field compatibility requires concrete Query-issued evidence, not
  a generic marker bound; and
- new enums are exhaustively mapped from Query-owned sources without wildcard
  fallthrough.

Query-owned downstream proof comes from
`worth_query::facade::consumer_kit`: typed support snapshots and pinning,
hard-prohibition and residue audits, and the real in-memory Query test runtime
where needed. WUI evidence may assert those returned artifacts; it may not
replace them with local report structs, required-row lists, source scanners,
support loops, or fabricated workspaces.

Compile-fail coverage must include attempted:

- direct binding and fact-receipt construction;
- receipt reuse after consumption;
- foreign-world admission;
- scalar continuation access;
- collection scalar collapse without an explicit selected field;
- diagnostic-to-authority conversion;
- runtime/host Query imports;
- Pulse import of internal WUI binding types; and
- renderer-side Query access.

Boundary check must add the Pulse audience-facade dependencies explicitly and
deny all broader edges. Agent-context generation must describe the new
contracts. No handwritten `AGENT_CONTEXT.md` edit is allowed.

## Architectural Destination

### Ownership

| Owner | Owns in 3.13 | Must not own |
| --- | --- | --- |
| `worth-ui-dsl` | authored scalar/collection requirement syntax and canonical lowering | installed Query identity or runtime binding |
| `worth-ui-query-binding` | pair-bound compatibility, affine carriage of Query-consumed authority, native consumption, posture translation, compact UI facts, row references, lifecycle | Query meaning, reconstructed Query authority, UI graph, renderer mechanics |
| `worth-ui-runtime` | admission of projection facts, indexed consequences, 3.12 turn integration, semantic content completion | Query execution or local result state |
| `worth-ui-host-contract` | mounted text mechanics and protocol schema | UI or Query meaning |
| `worth-ui-host-egui` | validated egui text translation | value, posture, identity, fallback, or semantic styling decisions |
| `worth-ui` | stable product registration and outcome facade | raw Query/runtime internals |
| `worth-ui-inspection` | immutable projection-consumption evidence views | operational construction |
| Platform Pulse | real product composition and external data adapter | framework authority or test injection |
| `worth-ui-certification` | adversarial and compile/dependency proof | production truth |

### Destination Tree

Only slice-relevant paths are shown. `create` means populated in this
milestone; no empty placeholders are created.

```text
workspaces/worth-ui/
  crates/
    worth-ui-query-binding/src/
      declaration/                              [modify]
        {projection_requirement,projection_shape,schema_requirement}.rs [create]
      projection_binding/                       [create: authority axis]
        {mod,admission,binding,compatibility,stop}.rs
      projection_consumption/                   [create: derived-value axis]
        {mod,fact_receipt,posture,scalar,collection,native_value}.rs
      projection_invalidation/                  [create: lifecycle axis]
        {mod,observation,lifecycle}.rs
      collection_delivery/                      [modify; do not replace]
      application_binding/                      [preserve measurement lane]
      inspection.rs                             [modify]
      entry/                                    [modify]

    worth-ui-runtime/src/
      source/                                   [modify: canonical requirement]
      fact_contract/produced/query.rs           [modify]
      runtime/observation/
        {admission,classification/owner}/query.rs [modify]
      runtime/invalidation_narrowing/selection/ [modify: shape strategies]
      mounting/projection/
        semantic_text/{mod,seed,completion}.rs   [create]
        {frame_storage,lowering}/                [modify]

    worth-ui-host-contract/src/
      mounted_projection/
        semantic_text/{mod,mechanic,table,schema}.rs [create]
        {participation,tables,view}.rs           [modify]
      mounted_frame/protocol.rs                 [modify]

    worth-ui-host-egui/src/adapter/
      native_text.rs                            [create]
      {mounted_presentation,mounted_projection_preparation,presentation_cost}.rs
                                                  [modify]

    worth-ui-dsl/src/
      semantic/projection_requirement/{mod,scalar,collection}.rs [create]
      source/{parse,lower/file_authored,lower/rust_authored}/    [modify]

    worth-ui/src/facade/
      query_binding.rs                          [modify: product grammar]

    worth-ui-inspection/src/
      projection_consumption/
        {mod,binding,fact,posture}.rs            [create: evidence views]
    worth-ui-certification/tests/
      projection_consumption/
        {posture,compatibility,collection,authority,cost,lifecycle}.rs
                                                 [create: QP-02--QP-10]

  apps/platform-pulse/
    app/main.wui                                [modify]
    query_samples/                             [create: product world inputs]
      {status-v1,status-v2}.json
    src/
      query_source/{mod,installation,external_value,lifecycle}.rs
                                                 [create: product adapter]
      observation_contract/                     [modify]
      {application,launch_configuration}.rs     [modify]
    tests/executable_world/                     [modify: QP-01; same target]
```

Dominant axes are declaration meaning, Query-bound authority, projection
consumption, invalidation lifecycle, runtime semantic effect, and host
mechanics. Files may be regrouped within those axes to obey the composition
laws, but implementation may not flatten them into `projection.rs`,
`query_helpers.rs`, `common.rs`, a generic paint module, or the existing
measurement fact.

Committed successor destinations, not created in 3.13, are:

- 3.14 `fact_contract/produced/intent.rs`;
- 3.16 `mounting/projection/appearance/`;
- 3.17-3.18 `source/{expression,composition}/`; and
- 3.19-3.22 `worth-ui-inspection/src/{diagnostics,replay,inspector}/`.

None may require moving the Query binding facade, reopening the 3.12 turn, or
letting the host consume Query.

## Ordered Phases

### Phase 1: Contract, Reachability, and Boundary Freeze

- Establish the orthogonal posture types, shape-specific public contract,
  private constructors, facade grammar, budgets, topology, and dependency
  enforcement.
- Prove real pending/current and stale-revalidating/current producers through
  Query audience facades; pin every relied-on support row through the Query
  Consumer Kit. If absent, amend Query rather than simulate WUI state.
- Exit on compile-fail/dependency proof plus green boundary and agent-context
  checks. Phase 2 may trust producer reachability and dependency direction.

### Phase 2: Canonical Declaration and Binding-Owned Consumption

- Make DSL/Rust scalar and collection requirements converge, admit exact
  installed contracts, and emit affine facts through keyed native access.
- Close `QP-02`--`QP-05` and `QP-10`, including width-independent counters and
  exhaustive compatibility, stop, result, and continuation posture.
- Preserve 3.9.2 measurement contracts or share only honest lower primitives.
  Phase 3 may trust compact facts without importing or revalidating Query.

### Phase 3: Observation, Rebind, Mounted Content, and Host Presentation

- Admit receipts through the 3.12 Query observation family, produce indexed
  scalar/collection plans, complete mounted semantic text, and translate it
  through negotiated host protocol.
- Close `QP-06`; reject cross-frame/surface/binding/node/allocation/generation
  content and protocol/schema/capability mismatch before paint.
- Prove zero unchanged-frame content work and non-authoritative visual/identity
  evidence. Phase 4 may trust the in-process value-to-pixel path.

### Phase 4: Permanent Platform Pulse Closure

- Install real Query meaning in the existing binary; drive it through the
  application source adapter; and pass `QP-01` across the native process,
  window, content, pixel, recovery, and close boundaries.
- Correlate pending, two current values, schema-mismatch preservation, and
  recovery with exact receipts while background/identity controls remain
  stable.
- Preserve protocol fixture compatibility and forbid runner injection or a
  test-only branch. Phase 5 may trust the decisive product claim.

### Phase 5: Torture, Cost, Documentation, and Deletion

- Close `QP-07`--`QP-09`, lifecycle, capacity, mixed-turn, Query-free, cost,
  documentation, and anti-bypass claims.
- Keep the ordinary loop focused and timed. Split or segment any broad
  certification lane whose warm wall time exceeds 60 seconds, without adding a
  test target or compile island; reserve the complete lane for phase closure.
- Delete the literal-field path, false facade guidance, and superseded
  convenience lanes; leave an additive 3.14 handoff.
- Exit with a zero-open-row ledger, compiled public examples, all canonical
  gates green, and no new test target or compile island.

## Documentation Deliverables

| Document | Continuing audience and required change | Verification |
| --- | --- | --- |
| `workspaces/worth-ui/README.md` | Application authors: canonical dependency direction and product import grammar. | Dependency audit and compiled minimal example. |
| `workspaces/worth-ui/AI_README.md` | Agents: discovery route for projection binding, posture, mounted content, and evidence. | Named paths and facade exports checked against source. |
| `workspaces/worth-ui/docs/query-binding.md` | Application authors: scalar/collection registration, production Query installation, native values, posture axes, invalidation, compatibility, lifecycle, cost, and anti-patterns. Remove the false workspace-extension import. | Compile ordinary and advanced examples; prove Pulse path. |
| `workspaces/worth-ui/docs/application-lifecycle.md` | Product integrators: cumulative Pulse Query installation, external data transition, visible posture/value, recovery, and close. | `QP-01`. |
| `workspaces/worth-ui/docs/architecture.md` and `docs/runtime-subsystems.md` | Contributors: ownership, dependency direction, projection fact flow, semantic text host boundary, and no session data cache. | Topology and dependency audit. |
| `workspaces/worth-ui/docs/inspection.md` | Human/agent tools: projection evidence identities, causal links, lazy detail, disclosure, retention, and non-authority. | Production inspection query and denial tests. |
| `plans/worth-ui/worth-ui-dsl-vision.md` | Language designers: exact 3.13 direct binding syntax, canonical lowering, and deferred expressions/composition. | DSL/Rust convergence tests and compiled examples. |
| `plans/worth-ui/ai-diagnostics.md` | Diagnostic-system designers: projection evidence family, support reporting, mounted diagnostic correlation, budgets, and non-authority. | Evidence-schema and lazy-materialization tests. |
| `workspaces/worth-ui/docs/worth-ui-readme.md` | Advanced contributors: replace deferred 3.13 descriptions with the shipped authority, cost, and lifecycle model. | Documentary topology audit. |

No milestone closeout summary may substitute for these continuing-audience
documents.

## Must Ship

- Opaque `UiProjectionBinding`/`UiProjectionFactReceipt` cores with
  shape-specific scalar/collection types and orthogonal posture.
- Exact schema, field, row, native-family, payload, compatibility, keyed-access,
  continuation, invalidation, budget, counter, and lifecycle contracts.
- 3.12-integrated facts, mounted semantic text with `BodyDefault`, egui
  presentation, typed diagnostics, and the canonical facade/dependency grammar.
- Direct DSL/Rust declarations, `QP-01`, and supporting
  stale/revalidating/collection/wrong-world/mixed-turn/Query-free/anti-bypass
  proof.

## Must Preserve

All closed 3.9.2 and 3.10-3.12 guarantees remain, especially:

- Query authority in Query and UI meaning/mounted identity in UI, with the
  binding crate as the sole WUI Query importer;
- one 3.12 observation/rebind/publication path and predecessor truth on every
  denial, stale/reset/cancellation, incompatibility, or interrupted effect;
- native values without serialization, scans, positional identity, printable
  authority, raw patches, or renderer queries;
- Query-free zero cost, unchanged-frame zero work, and diagnostic/replay/visual
  products off the ordinary lane; and
- the one Pulse binary/process/window/runner/failure contract and existing
  compile topology.

## Acceptance Evidence

Milestone 3.13 closes only when `QP-01`--`QP-10` have independent,
mutation-sensitive evidence and the ledger has no open rows. Together they
must prove:

- orthogonal posture and compile-time scalar/collection separation;
- exact contract-derived access, compatibility-only identity preservation, and
  no fabricated value on schema/payload stops;
- bounded collection/graph work, Query-free and unchanged-frame zero work, and
  complete terminal disposal;
- value/posture receipt correlation through mounted identity to external
  pixels; and
- compiled continuing documentation plus green constitutional, boundary,
  agent-context, line-cap, format, lint, test, certification, and
  executable-world gates without a new compile island.

## Non-Goals and Successor Handoff

Milestone 3.13 does not implement:

- arbitrary Query authoring in the DSL;
- rich tables, joins, grouping, sorting, filtering, pagination UX, or large
  data-product surfaces;
- authored expressions, formatting, coercion, localization, fallback, or
  conditional presentation;
- intent or mutation callbacks;
- broad appearance, typography, focus, motion, portal, or accessibility
  systems;
- generic renderer commands or a retained widget tree;
- causal diagnosis, replay, or the final inspector UI;
- a local result-state/cache/recovery model; or
- a second Platform Pulse harness.

Successors insert additively:

| Milestone | May add | May not reopen |
| --- | --- | --- |
| 3.14 | admitted intents whose completed mutations may later produce Query invalidation | renderer callbacks, Query result receipts as mutation authority, or the 3.12 turn |
| 3.15-3.16 | services, portals, focus, motion, accessibility, and broad appearance around the same mounted content | host-owned UI meaning or data binding |
| 3.17 | authored expressions and formatting that consume declared projection facts | renderer evaluation, hidden dependencies, or Query rediscovery |
| 3.18 | richer composition and modules lowering to the same canonical projection requirements | a second source/binding path |
| 3.19-3.22 | diagnostic, replay, and inspector products over retained projection evidence | diagnostic/replay authority or ordinary-lane tax |
| Milestone 6 | serious data surfaces and product-scale projection consumption | local Query/result-state clones, raw Query patches, or positional identity |

The 3.14 plan may trust that visible data has one honest, typed, bounded path
from installed Query meaning to mounted pixels and that an admitted intent can
later cause data change without inventing a second data or publication model.
