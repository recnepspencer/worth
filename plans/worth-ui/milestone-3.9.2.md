# Milestone 3.9.2: Query 9.14 Native Consumer and Identity Cutover

Status: Complete on 2026-07-23. Phases 1 through 7 are closed.

## Goal

Complete Worth UI's ordinary-consumer cutover to the finished Query 9.14
surface before mounted receipts are introduced. Worth UI must select and read
installed native values through Query-minted declaration-indexed keys, retain
Query operational identity only behind its owning binding boundary, and
translate admitted Query settlements and patches exactly once into UI-owned
graph, measurement, allocation, and presentation consequences.

The milestone ends with a Query-native, identity-honest input to Milestone
3.10. It does not create mounted receipts, teach hosts Query concepts, or
pre-build Milestone 3.13's broad projection-product substrate.

## Why This Milestone Exists

Milestone 3.9.1 modernized Worth UI against Query 9.14 Phases 1 through 14 and
deliberately contained the then-missing lifecycle, lease, invalidation, window,
and patch capabilities. Query 9.14 is now complete. The managed-live
compatibility seam has already been removed and the ordinary Query-owned
operation lifecycle is present, but two predecessor assumptions still survive
in Worth UI:

1. measurement facts are found by string-kind tests, broad fact-family scans,
   local scalar refinement, positional copying, and index access instead of
   Query's declaration-indexed native-access contract; and
2. printable Query binding and settlement identities still participate in
   graph-world identity, eligibility, fact keys, touch authority, replacement,
   planning, and execution evidence.

Those assumptions would be especially dangerous after 3.10. Once mounted
receipts become the only host-renderable artifact, any counterfeit identity or
reconstructed Query fact below that line would be harder to see and easier to
treat as authoritative UI meaning. This milestone closes the source boundary
first.

## Governing Summaries

- `MENTALITY.md` protects causal closure under hostile ambiguity. The cutover
  must remove the competing sources of access and identity authority, not
  merely rename or hide them.
- `arch_laws.md` requires typed, owner-minted phase progression and semantic
  identity stronger than representation. Query keys and owner-specific
  validation must cross the boundary; strings and digests cannot authorize
  work.
- `composition_laws.md` requires one named semantic responsibility per file
  and function. Native selection, access, UI refinement, identity readmission,
  and consequence translation must remain distinct instead of becoming one
  modernization adapter.
- `domain_structure_laws.md` requires the filesystem and crate graph to expose
  who owns truth. `worth-ui-query-binding` remains the sole production Query
  importer; Query authority, UI consequence, and host output remain physically
  distinguishable.
- `perf_laws.md` requires carried proof and delta-bounded work. Reading `k`
  declared UI facts must perform `k` keyed accesses, not scan or clone an
  unrelated projection of width `n`; Query counters and UI refinement counters
  remain separately attributable.
- `worth_ui_roadmap.md` requires stronger Query lanes to be consumed rather
  than rebuilt, a hot-lowered UI path, Query-free hosts, and mounted receipts
  only after runtime-owned meaning is settled.

## Adversarial Constraint

Two settled projections may expose identical printable binding labels,
settlement labels, native bytes, UI view IDs, warning counts, and current
visible output while belonging to different Query runtimes, installations,
generations, bases, operation contracts, capabilities, or leases. They must
never share a retained slot, satisfy each other's graph-world admission,
preserve each other's plan links, validate each other's touch origin, or
authorize each other's replacement, refresh, or patch.

Conversely, adding `n` unrelated display fields, derived fields, rows, views,
or diagnostic labels must not increase the ordinary work required to obtain
`k` declared UI measurement facts. The binding must resolve Query-minted
native-access keys from the exact consumer projection contract, carry them
through the exact consumption request, and perform exactly the admitted keyed
accesses against the settled or refreshed projection. No field-family string
dispatch, whole-projection scan, positional native-fact copy, or local
field-path selector may become hidden authority.

A Query patch may affect UI state only after the exact live owner, move-only
lease, capability-bound invalidation delta, current workspace, collection
window, and Query-shaped patch have admitted it. Worth UI may then mint its
own consequence artifact from that receipt. Neither the patch nor its
reporting projection may cross into host authority or future mounted receipts
as a Query object.

The milestone therefore has three simultaneous obligations:

1. Query owns native selection, native access, operational identity,
   compatibility, lifecycle, invalidation, rows, ordering, windows, patches,
   and Query work counters;
2. Worth UI owns authored binding intent, graph identity, measurement meaning,
   source coordinates, allocation, invalidation consequences, presentation,
   and UI work counters; and
3. every crossing weakens upstream authority and requires the receiving owner
   to validate the exact source before minting a narrower local artifact.

## Product Decision Lock

- The manually adjudicated
  [Query 9.14 cutover inventory](./milestone-3.9.2-query-9.14-cutover-inventory.csv)
  is the subsystem migration reference. The separately adjudicated
  [boundary and edge matrix](./milestone-3.9.2-boundary-edge-matrix.csv) is the
  authority-flow reference.
- Search discovers candidates. Human review decides whether each subsystem is
  replaced, retained, refined, diagnostic-only, deleted, or unrelated.
- No checked-in test may claim the migration succeeded because an old symbol,
  string, or path disappeared. The CSVs are design evidence, not runtime
  manifests or mechanical sentinels.
- `worth-ui-query-binding` remains the only Worth UI production crate allowed
  to import `worth-query`.
- Query operational identity remains behind that binding owner. Downstream UI
  crates receive UI-minted opaque references or evidence only after exact
  Query validation. Printable Query labels and digests are terminal
  inspection data.
- Query's native-access key is resolved from the exact consumer projection
  contract and is never synthesized from a UI field path, fact-family string,
  ordinal, or copied fact.
- Query-owned denial and counter topology is preserved at the binding
  boundary. Worth UI adds only domain-local consequence and refinement
  outcomes; it does not mirror Query's taxonomy.
- Query-free Worth UI applications remain Query-free and ceremony-free.
- Query replay, aftermath, reversal, lineage, and persistent naming remain
  outside the ordinary UI consumer lane.
- Existing operation-native live ownership, leases, invalidation readmission,
  collection windows, and Query-shaped patch application are retained. The
  milestone removes predecessor residue around them; it does not fork them.
- Milestone 3.10 still owns mounted-node and mounted-frame receipts plus the
  host observation contract. Milestone 3.13 still owns the broad scalar,
  collection, product, continuation, and result-state binding substrate.
- New proof is consolidated into existing compiled scenario owners and the
  Query reusable certification kit. This milestone adds no nested Cargo build,
  generated fixture workspace, per-phase integration target, or private target
  directory.

## Boundary Contract

The ordinary snapshot chain is:

```text
authored UI measurement dependency
-> exact installed-operation consumer projection contract
-> contract-derived native selection and WorthQueryNativeAccessKey
-> bound projection request carrying that selection
-> execute -> publish -> consume_bound(request) -> settle
-> keyed native_value(key, row) access against the exact settlement
-> binding-owner validation of exact Query source
-> UI-minted binding evidence and measurement consequence
-> generation-owned UI fact slot and compact plan link
-> requested allocation or execution ingress
```

The ordinary live collection chain is:

```text
Query live owner
-> move-only consumer lease
-> delivered invalidation
-> capability-bound consumer invalidation delta
-> current-workspace readmission
-> current collection consumer and admitted window
-> Query-shaped patch plan
-> exact apply_patch receipt
-> one UI-owned graph / measurement / allocation consequence
-> existing framework-turn publication boundary
```

The authority may weaken in the downstream direction, but it may never be
reconstructed in the upstream direction:

```text
Query operational artifact
    -- exact owner validation -->
Worth UI binding evidence / consequence
    -- lowering and mounting -->
UI execution or mounted receipt
    -- projection -->
host rendering and observation
```

The host direction has no return edge. A UI receipt, digest, diagnostic label,
mounted identity, native value, or source coordinate cannot be readmitted as
Query authority.

## Phase Plan

### Phase 1: Adjudicated Post-9.14 Migration Authority

Phase status: Complete on 2026-07-23. The manually reviewed authority contains
29 subsystem dispositions and 16 boundary edges. The review corrected the
initial plan-link lifetime model: plan links remain binding-slot scoped while
fact, touch, and allocation evidence is settlement-revision scoped.

Freeze the semantic scope of the cutover before production edits begin. Broad
search seeds the two CSVs, but every row represents a manually resolved
subsystem or authority crossing rather than a token occurrence.

**Relevant subsystems**

- Query-facing production and test surfaces involving native values,
  projections, settlement, binding identity, graph-world identity, touch
  authority, replacement, invalidation, windows, patches, and diagnostics
- the 3.9.1 modernization inventory and boundary matrix
- the canonical road-1 dependency contract and generated crate contexts
- existing application-contract, topology-contract, compile-contract, and
  Query Consumer Kit scenario owners

**Relevant artifacts**

- `milestone-3.9.2-query-9.14-cutover-inventory.csv`
- `milestone-3.9.2-boundary-edge-matrix.csv`
- `milestone-3.9.1-query-modernization-inventory.csv`
- `milestone-3.9.1-boundary-edge-matrix.csv`
- `tools/boundary-check/config/road1.toml`

**Warnings**

- A row is a semantic subsystem, not a file or search match. Several files may
  implement one identity decision, while one file may contain both retained
  diagnostic projection and obsolete operational comparison.
- `resolved` means the human disposition is decided. It does not mean the
  implementation is complete.
- A new ambiguous search cluster updates the CSV. It does not justify a
  permanent grep-count or zero-match test.
- Historical 3.9.1 rows remain evidence of the earlier migration. Do not
  rewrite them to pretend the Phase 15-26 surface existed when 3.9.1 closed.
- Query types in certification or the sole binding crate are not automatically
  violations. Query-shaped authority in runtime, host, graph, or mounting
  crates is.

**Test requirements**

- Parse both CSVs during phase QA and reject duplicate IDs, missing owners,
  missing manual resolutions, invalid classifications, empty target homes,
  edges without a failure owner, edges without a cost contract, or edges
  without a forbidden shortcut. This checks document integrity only.
- Rerun broad searches for `native_fact`, `native_value`, `display_fields`,
  `derived_fields`, `query_binding_identity`, `settlement_identity`,
  `SettledQueryBinding`, `WorthQuery`, `consumer_invalidation_delta`,
  `plan_patch`, `apply_patch`, `replay`, `aftermath`, and `lineage`. Manually
  map every semantic cluster to a row or add a row.
- Put a legitimate terminal diagnostic label and an operational string
  identity comparison in the same review sample. The former must remain
  diagnostic-only and the latter must be replaced, proving spelling is not the
  decision rule.
- Put the correct operation-native patch path and a predecessor native-fact
  scan in the same review sample. The former must be retained or refined and
  the latter replaced, proving “new Query code” is not the decision rule.

**Engineering decisions**

- The CSVs are versioned design and review evidence. They do not generate Rust,
  allowlists, test matrices, or runtime registries.
- Allowed subsystem dispositions are `replace`, `retain`, `refine`,
  `diagnostic-only`, `delete`, and `unrelated`.
- Permanent enforcement targets dependency direction, sealed construction,
  exact owner validation, typed denial, behavioral outcomes, and exact
  counters. Historical migration vocabulary remains review provenance.
- Phase completion updates each CSV row only after the named evidence exists.

**Open questions**

- None.

### Phase 2: Declaration-Indexed Native Selection and Access

Phase status: Complete on 2026-07-23. The binding owner now builds the bound
projection request and native-access key from one exact consumer contract,
drives the ordinary zero-argument snapshot progression through
`consume_bound`, and performs only keyed native access. Raw Query requests,
keys, projections, and positional fact collections no longer cross the public
Worth UI boundary.

Replace measurement-family string dispatch, broad consumed-fact scans,
positional copies, and index reads with Query's installed declaration-indexed
native-access contract.

**Relevant subsystems**

- `worth-ui-query-binding/application_binding/measurement_fact_observation`
- snapshot consumer-contract construction and bound projection request
- retained settled and refreshed projections
- query execution/frame evidence that currently exposes positional native facts
- the installed snapshot-measurement operation and its declared result shape

**Relevant Query APIs**

- exact consumer projection contract fact selection
- `WorthQueryNativeAccessKey`
- bound projection request construction
- `consume_bound(...)`
- settled and refreshed `native_value(&key, row)` access
- Query-native access denials and counter snapshots

**Warnings**

- A native-access key must be derived from the exact consumer projection
  contract used by the same bound operation. A key reconstructed from a UI
  field path, result-shape string, fact-family name, or ordinal is counterfeit.
- The key and request are pair-bound. Do not resolve a key from one contract
  and consume or access a projection from another merely because declarations
  compare equal.
- Do not replace one scan with a cached map owned by Worth UI. Query already
  owns declaration-indexed access, compatibility, and currentness.
- `as_float32()` on a broadly selected consumed fact is not the ordinary
  contract. Read the admitted native value by key, then perform the narrow
  UI-owned conversion required by the declared measurement consequence.
- Absence, null, unsupported shape, foreign capability, stale generation,
  wrong runtime, wrong row, and missing declaration remain distinct at their
  owning boundary. Do not collapse them into “missing measurement.”

**Test requirements**

- Execute a real installed snapshot operation whose projection contains one
  declared measurement value and many unrelated display and derived values.
  Assert one contract-derived key and one keyed access for the measurement;
  increasing unrelated projection width must not increase access or UI
  refinement counts.
- Resolve two keys from semantically similar declarations installed in
  different Query runtimes or generations. Each key must work only with its
  exact settled projection; swapping them must return Query's typed denial
  before UI refinement.
- Feed equal scalar bytes through different native shapes, including an
  admitted float, null or absence, and an unsupported non-float value. Only
  the admitted declared shape may mint a UI measurement observation, and each
  denial must remain attributable.
- Refresh a settled live projection and prove the current declared key accesses
  the refreshed value while a stale or foreign key cannot. No fallback scan,
  ordinal lookup, or copied native fact may recover the value.
- Compile-fail or visibility proof must show downstream runtime and host crates
  cannot construct `WorthQueryNativeAccessKey`, call raw Query access, or
  receive a broad Query fact collection through the Worth UI facade.

**Engineering decisions**

- The installed Worth UI measurement operation declares the exact native
  selection once. The binding owner obtains the exact consumer contract,
  creates its `projection_request()`, calls the installed request builder's
  typed native-field selection method, builds the bound request, resolves the
  selection through `resolve_native_key(...)`, and carries the returned key
  with that request/settlement relationship.
- The key remains opaque Query authority inside `worth-ui-query-binding`. UI
  runtime state may carry a binding-owned fact reference, never the key itself.
- The ordinary installed path uses `consume_bound(...)`; the predecessor
  unbound `consume(..., WorthQueryProjectionDeclaration)` surface is removed
  from Worth UI's production snapshot progression unless an independently
  adjudicated non-native consumer still requires it.
- Positional `native_fact(index)` and copied
  `Box<[ConsumedFieldValueFact]>` surfaces are removed from ordinary Worth UI
  evidence. Terminal inspection receives a reporting projection only when
  explicitly needed.
- UI measurement conversion is a named downstream refinement step with its own
  counters; it does not rediscover which Query fact was selected.
- The first implementation slice should establish the exact key/request shape
  at operation consumption before deleting predecessor accessors, so compiler
  failures identify every downstream consumer that needs a principled
  replacement.

**Open questions**

- None.

### Phase 3: Opaque Query Identity Readmission

Phase status: Complete on 2026-07-23. Owner-only admission now mints distinct
opaque references for an installed binding epoch and a settlement revision.
Graph identity, measurement admission, fact receipts, touch authority,
allocation, replacement, and live evidence no longer use printable Query
identities operationally. Stable plan links remain binding-slot scoped while
revision-specific evidence is rejected across foreign or superseded
settlements.

Remove printable Query binding and settlement identities from every
operational UI decision. The binding owner validates the exact retained Query
source, then mints two narrower UI artifacts with different lifetimes:

1. an admitted binding-authority reference for the installed binding epoch;
   and
2. an admitted settlement reference for one current settled or refreshed
   source revision.

These references are opaque outside `worth-ui-query-binding`. They are not
Query proof, but their construction proves that the binding owner checked the
real Query artifact before allowing downstream UI work.

**Relevant subsystems**

- exact settled snapshot evidence and downstream settlement retention
- exact operation-live resource evidence and copied lease identity
- `WorthUiQueryBindingEvidence` and replacement authority drift
- `UiGraphWorldProfile::SettledQueryBinding`
- query measurement eligibility and settled fact receipts
- graph touch origin and touch-authority readmission
- allocation mapping, plan input, executable schema, virtualized data lane,
  host-observation diagnostics, and plan inspection
- all `query_binding_identity` and `settlement_identity` string comparisons,
  digests, keys, and copied fields

**Relevant authority operations**

- exact installed-reference currentness
- Query's named same-installation, compatible-basis, replacement, rebind, and
  lifecycle relationships where each exact question applies
- binding-owned admission of a retained settled or refreshed projection
- UI-authored binding identity and result-shape comparison
- terminal reporting projection after operational decisions are complete

**Warnings**

- One opaque reference cannot represent both a binding epoch and a settlement
  revision. Doing so would either preserve stale facts or force an unnecessary
  binding identity change on every refresh.
- Pointer equality, slot ordinals, source generations, labels, digests, and
  serialized tokens are not Query identity. They may index a candidate only
  after the binding owner validates the exact retained source.
- Do not wrap a Query string in a newtype and call it opaque. The constructor
  must require owner-held exact evidence unavailable to callers.
- `WorthUiQueryBindingIdentity` is legitimate UI-authored identity. It states
  which UI binding and shape are intended; it cannot state which Query
  installation, capability, or settlement is current.
- Query's named relationships are not interchangeable. A same-installation
  witness cannot authorize replacement, rebind, sharing, or settlement
  currentness.
- Diagnostics may still show Query-provided labels and digests, but operational
  code must not be able to read those reporting projections back into
  admission, equality, hashing, plan equivalence, or touch validation.

**Test requirements**

- Create two real Query runtimes whose installed operations and settlements
  expose identical printable labels and native values. Their admitted binding
  and settlement references must differ, and each must fail graph, fact,
  replacement, and touch readmission against the other before UI work.
- Refresh one binding twice so the binding-authority reference remains valid
  while the settlement reference changes. The predecessor fact, touch origin,
  and allocation mapping must become stale without pretending the installed
  binding or its stable plan-slot link changed. Dereferencing that stable link
  must resolve only the current consequence.
- Bind two different UI declarations to the same exact Query source. Query
  authority may be shared only where Query admits it, while UI replacement and
  plan equivalence must still distinguish the UI-authored identities and
  result shapes.
- Mutate, collide, omit, or replay terminal Query labels and diagnostic
  digests. No operational outcome, currentness decision, or plan equivalence
  may change.
- Attempt to construct either admitted reference from public runtime, graph,
  host, certification-fixture, or application code. Sealed construction or
  compile-fail proof must make the promotion unavailable.
- Attempt to use the admitted UI reference with Query lifecycle, native-access,
  patch, replay, or lineage APIs. The type topology must provide no such
  reverse promotion.

**Engineering decisions**

- The binding owner exposes named validation methods that accept its exact
  retained installed reference, settlement, or operation-native resource and
  return a sealed UI-local reference or a typed owner-specific denial.
- Binding-authority and settlement references carry private owner-minted
  identity. Public equality, ordering, or hashing is provided only where the
  downstream UI role needs candidate indexing after validation; no reporting
  representation participates.
- Graph-world identity stores the UI binding identity plus the admitted
  binding/settlement references appropriate to its currentness contract.
  `Box<str>` Query identity fields and text-digest authority are removed.
- Replacement asks the binding owner the exact installed/lifecycle
  relationship, then combines that answer with independently checked UI
  meaning. It never compares copied Query labels.
- Fact keys, touch origins, and allocation mappings carry the admitted
  settlement reference. Plan inputs and executable rows retain a compact
  binding-slot reference and resolve its current consequence through the
  binding owner. Neither carries Query strings or Query artifacts.
- Inspection projects labels only from the still-retained exact source and
  marks them as reporting. It cannot reconstruct admitted references from the
  projection.

**Open questions**

- None.

### Phase 4: UI Measurement Derivation and Retained Settlement Topology

Phase status: Complete on 2026-07-23. Each successful real Query settlement
now yields one complete UI-owned measurement consequence containing opaque
source references, canonical UI observations, exact owner-attributed counters,
and diagnostic posture. Settlement retention publishes that consequence as a
single slot value; derivation denial stops before fact, graph, plan, or live
resource mutation, and downstream execution observes compact UI consequences
rather than Query-native values.

Rebuild the measurement-fact path on the Phase 2 keyed access and Phase 3
identity artifacts. Retain the exact settled Query projection once, derive one
complete UI measurement consequence at admission or refresh, and let all
downstream rows reference the UI-owned consequence rather than Query facts.

**Relevant subsystems**

- `WorthUiSettledSnapshotProjection` and settled snapshot retention
- `WorthUiSettledSnapshotFact` and measurement fact batch
- source generation and source order
- exact settled snapshot evidence
- measurement eligibility, fact consumption, allocation mapping, graph touch,
  plan lowering, virtualized data execution, and inspection
- snapshot execution and framework-turn refresh publication

**Required topology**

```text
one exact settled Query projection
    + exact contract-derived native keys
    + admitted binding and settlement references
-> one binding-owned derivation attempt
-> one complete UI measurement consequence
-> one generation-owned consequence slot
-> stable binding-slot plan links
-> revision-specific fact / touch / allocation references
```

**Warnings**

- The retained Query projection is not a public fact bag. It exists so the
  binding owner can perform exact validation, lifecycle, refresh, cleanup, and
  terminal inspection.
- Do not clone every `ConsumedFieldValueFact` into
  `WorthUiSettledSnapshotFact`, expose `native_fact(index)`, or preserve a
  positional escape hatch for tests.
- Source generation and order are UI publication coordinates. They can reject
  stale UI consequences within one binding slot, but they cannot replace the
  admitted Query settlement reference.
- A partial Query result, warning, or typed denial must remain structured at
  the binding boundary. If UI presentation needs a local posture, derive a
  named UI consequence without copying Query's enum as operational authority.
- Failed derivation or refresh must not mutate the active consequence slot,
  graph world, plan link, source order, or operation-native resource.
- Application replacement and in-generation source refresh remain distinct
  transactions.

**Test requirements**

- Settle one real installed projection, derive one UI measurement consequence,
  and fan it out to many plan rows. Assert one retained Query projection, one
  derivation, one consequence slot, and compact downstream references; no
  Query fact or native key may be cloned per row.
- Refresh that projection successfully and prove the complete consequence slot
  changes atomically and its source revision advances once. Prior fact, touch,
  and allocation references deny as stale; the stable binding-slot plan link
  resolves the new consequence without plan relowering; unrelated binding
  slots and plan regions remain untouched.
- Force keyed-access denial, unsupported UI conversion, partial Query result,
  warning-bearing success, and source-coordinate exhaustion. Each outcome must
  preserve the prior complete active consequence and retain its distinct
  diagnostic meaning.
- Scale dependents per consequence and unrelated settled bindings
  independently. Derivation work must remain proportional to newly admitted
  facts; frame work must remain proportional to requested compact references.
- Remove the positional native-fact API and migrate existing hostile virtual
  data tests to assert the declared UI consequence at the active plan edge.
  Tests may not reach around the binding owner to inspect copied Query facts.
- Retire or replace an application generation and prove exact-once disposal of
  the retained Query projection and no surviving consequence, mapping, touch
  origin, or plan reference from that generation.

**Engineering decisions**

- The settlement retention owner stores each exact Query projection once and
  stores each derived UI consequence once. A slot contains a complete value;
  partial field mutation is unavailable.
- The measurement consequence contains UI measurement family, canonical UI
  value, UI source coordinates, admitted UI source references, and separately
  projected presentation/diagnostic posture. It contains no copied Query fact
  collection.
- The existing compact generation-scoped plan-reference pattern is retained as
  a binding-slot route. It does not carry settlement revision. Every
  dereference resolves the slot's current admitted consequence, while
  revision-specific downstream evidence carries the settlement reference.
- Query result state, warnings, cost snapshots, and reporting labels remain
  observable through an inspection projection owned by the binding crate. They
  do not participate in graph identity, eligibility, allocation, or plan
  equivalence.
- Query-free and non-measurement nodes continue through their current paths
  without settlement slots or dummy evidence.

**Open questions**

- None.

### Phase 5: Operation-Native Change to UI Consequence Handoff

Phase status: Complete on 2026-07-23. A real Query collection owner now
progresses lease, drain, delta, readmission, patch planning, and patch
application before minting one sealed UI consequence. The runtime publishes
that exact retained consequence only inside an admitted framework turn;
dropped handoffs, callback unwind, reset-required delivery, wrong worlds,
window shifts, and interrupted close preserve the prior truth and recover the
same authority rather than reconstructing it.

Finish the downstream side of the already-correct Query 9.14 live path. Retain
the real owner, lease, invalidation-delta readmission, collection window,
patch-plan, and patch-application progression, but replace the current
Query-shaped “UI consequence” with a sealed, genuinely UI-owned change
consequence.

This phase does not implement the 3.12 rebind planner or the 3.13 general
collection projection lane. It establishes the only admitted source artifact
those milestones may consume.

**Relevant subsystems**

- `operation_live/resource`
- `collection_delivery`
- operation-live retention, succession, and retirement
- framework-turn Query source capability and publication transaction
- collection graph, measurement, allocation, virtualization, and diagnostic
  consequence inputs
- Query's compiled-impact-selected patch scope and the applied receipt's
  descriptive Foundational invalidation projection
- Query operation-native reference-consumer certification

**Required progression**

```text
shared live owner
-> exact move-only consumer lease
-> drain delivery
-> consumer_invalidation_delta(delivery)
-> admit_consumer_invalidation_delta(delta, current workspace)
-> bind_shared_target(admitted delta, current workspace)
-> plan_patch(admitted delta, current workspace)
-> apply_patch(exact patch)
-> validate exact patch receipt at the binding owner
-> mint one sealed Worth UI collection-change consequence
-> hand off through the framework-turn source boundary
```

**Warnings**

- Query rows, row handles, entity identities, native-access keys, continuations,
  warnings, result-state enums, patch operations, and maintenance ordinals
  remain Query artifacts. A public enum prefixed `WorthUi` does not make copied
  Query types into UI ownership.
- The binding owner may mint an opaque UI row/source reference after validating
  the exact Query row and patch receipt. It must not synthesize row identity
  from labels, positions, entity strings, or native bytes.
- A reset-required patch is not a partial mutation list. It produces one typed
  UI reset consequence and preserves the previous admitted UI truth until the
  future rebind owner completes a replacement.
- “Preserve mounted identity” is premature before Milestone 3.10. This phase
  may preserve admitted UI row or allocation identity; only mounted receipts
  may later claim mounted identity.
- Do not diff the full collection, reinterpret raw CDC, replay a historical
  delta, infer ordering, recompile Query dependency impact from UI dependency
  metadata, or contact providers after an earlier typed stop.
- Translating a patch receipt does not authorize applying graph mutations from
  arbitrary caller code. The sealed consequence enters only through the
  framework-turn source boundary; 3.12 owns changed-fact classification and
  preserve/remount planning.

**Test requirements**

- Run a real operation-native live view through promotion, managed lease,
  invalidation drain, delta creation, current-workspace readmission, window
  binding, patch planning, exact application, and UI consequence minting. Prove
  one Query patch receipt produces one sealed UI consequence and no copied
  Query capability escapes.
- Create equivalent-looking rows in different Query runtimes, capabilities,
  windows, generations, and leases. Their UI row/source references must not
  alias, and a swapped patch or admitted delta must deny before consequence
  minting.
- Exercise every patch operation produced by Query's ordinary collection-
  application path: insert, remove, move, update, window shift, continuation,
  and native-fact changes. The UI consequence must contain only the graph,
  measurement, allocation, and diagnostic effects Worth UI owns; terminal
  Query posture remains inspection-only. Keep result-state and warning mapping
  exhaustive, but do not fabricate a receipt or add a WUI-side state source to
  claim behavioral reachability before Query has an ordinary producer.
- Present an exact patch whose compiled Query impact is narrower than the UI's
  broad candidate dependency set. Worth UI may further narrow its local
  response but may not widen upstream delivery, reinterpret the descriptive
  Foundational invalidation projection as authority, or rerun Query impact
  compilation locally.
- Deliver reset-required, stale lease, wrong workspace, wrong generation,
  out-of-order patch, duplicate patch, interrupted disposal, and no-semantic-
  delivery cases. Prior Query consumer state and prior admitted UI consequence
  must remain complete whenever Query does not return an applied receipt.
- Attempt to apply a sealed UI consequence outside the framework-turn source
  boundary or turn it back into a Query patch/delta. Construction, visibility,
  and type topology must make both routes unavailable.
- Dispose, retry a stopped disposal, replace, preserve, and retire live
  resources through public lifecycle APIs. Prove exact-once lease release and
  no orphan UI source reference after terminal retirement.

**Engineering decisions**

- Existing Query-owned `WorthUiOperationLiveResource` progression is retained
  and decomposed only where file responsibility requires it.
- The patch translator lives in `worth-ui-query-binding` because it must inspect
  the exact Query receipt. Its output is a sealed UI consequence containing
  UI-owned row/source references and narrow graph, measurement, allocation,
  and diagnostic effects.
- Query warning, result-state, continuation, and counter projections may be
  retained in a terminal inspection payload. They are not graph mutations or
  operational UI identity.
- Query's applied receipt fixes the upstream affected scope. Worth UI derives
  downstream consequences only inside that scope; the receipt's Foundational
  invalidation projection may guide or explain local mapping but cannot be
  readmitted as Query invalidation authority.
- The framework-turn source owner accepts the sealed consequence as source
  evidence and preserves atomicity. Phase 3.12 later decides the narrowed
  invalidation and rebind plan; Phase 3.13 later generalizes collection facts
  and posture.
- No mounted-node or mounted-frame receipt type is introduced, and no host
  adapter receives the consequence.

**Open questions**

- None.

### Phase 6: Exact Cost and Scale Closure

Phase status: Complete on 2026-07-23. Query-owned access and patch counters
remain separately inspectable from UI derivation and consequence counters.
Real narrow and 128-row collection patches perform identical patch-local work;
one declared measurement performs identical indexed access and derivation at
projected widths 1 and 128; Query-free, snapshot-only, and operation-live
postures construct only their admitted subsystems. Settlement admission and
per-frame resolution now share the projection's one derived UI fact by stable
pointer, and the independent thread-scoped allocator observer records zero
allocations for exact retained-fact resolution.

Make the new boundary's cost law explicit and counter-backed. Query's access,
invalidation, window, patch, and lifecycle counters remain Query-owned; Worth
UI counts only its key selection, identity readmission, measurement
refinement, consequence minting, slot publication, and compact-reference
resolution.

**Relevant subsystems**

- Query consumption-cost snapshot and native-access result/denial counters
- Worth UI measurement refinement and retained-slot observations
- operation-live refresh and patch-application counters
- application replacement, affected-binding, affected-plan-row, framework-
  turn, allocation-ingress, and frame counters
- certification allocator and compile-topology evidence

**Required cost contracts**

- Native request construction and access are `O(k)` for `k` declared native
  selections, with `O(1)` access per admitted key and row.
- Measurement derivation is `O(k)` in admitted UI measurement facts and
  independent of unrelated projected width, total application width, and
  downstream fan-out.
- Binding and settlement readmission are `O(1)` per exact crossing and perform
  no text digest, serialization, registry scan, or global slot search.
- Consequence translation is `O(p + f)` for `p` applied Query patch operations
  and `f` patch facts actually reported by the exact receipt. It performs no
  full-collection diff or unrelated-row scan.
- Replacement and framework-turn publication remain bounded by changed
  bindings plus affected dependents. Steady execution remains bounded by
  requested compact references.
- Query-free paths perform zero Query binding, native-access, settlement,
  operation-live, and patch work.

**Warnings**

- A counter copied from the implementation under test is not independent proof
  of an allocation or zero-scan claim. Use Query's sealed counters for Query
  work and the existing independent allocator observer where allocations are
  claimed.
- Do not combine Query and UI counters into one attractive total that hides
  which owner performed the work.
- A warm targeted test time is not a substitute for structural cost evidence,
  and a cold compile is not a runtime performance test.
- Do not create a benchmark crate, custom target directory, nested Cargo
  invocation, or one executable per scale case.
- Exact counts should describe the named scenario. Avoid brittle assertions on
  incidental standard-library allocations or unrelated compiler behavior.

**Test requirements**

- For `k` declared measurement keys and independently varied unrelated width
  `n`, assert Query reports exactly `k` indexed accesses and the expected
  refinement checks, with zero fact scans, row scans, and path parses. Worth UI
  must report exactly the admitted derivations, not `n`.
- Hold `k` constant while scaling downstream dependents and unrelated
  application nodes. Settlement retention and derivation counts remain
  constant; plan-link creation scales only with affected dependents; one frame
  touches only requested rows and facts.
- Apply equivalent semantic patches to a narrow and a very broad collection.
  Query and UI work must scale with patch operations and affected facts, not
  collection width. A reset-required outcome performs no partial UI mutation.
- Compare Query-free, snapshot-only, and operation-live applications in the
  same compiled scenario owner. Each must construct and pay only for its
  admitted subsystems.
- Use the independent thread-scoped allocator observer to prove removal of the
  per-settlement native-fact clone and absence of per-frame Query proof cloning.
  Exact allocation claims must be scoped to the operation being observed.
- Record warm targeted, warm fast-lane, and isolated cold compile timings after
  target topology stabilizes. Fail the milestone if new test topology causes a
  material regression without an explicit, reviewed reason.

**Engineering decisions**

- Query counter snapshots are projected for diagnostics without being reminted
  as Worth UI authority.
- UI counters are attached to the exact local operation that owns them and
  remain separately inspectable.
- Existing compiled scenario owners absorb the scale matrix so code generation
  and dependency monomorphization are shared.
- The settled projection is retained once for authority and cleanup; native
  values are borrowed during derivation and are not cached merely to make
  counters look smaller.

**Open questions**

- None.

### Phase 7: Facade, Documentation, and Certification Closure

Phase status: Complete on 2026-07-23. The product Query-binding facade now
exposes only authored and installed view declaration/registration
capabilities; execution progression, settled projections, live resources,
sealed collection consequences, native keys, leases, deltas, patches, and
Query inspection artifacts remain at their owning boundary. Certification
code imports the binding owner directly instead of widening the product
facade.

The existing disk-backed replacement owner now writes and observes real
`.wui` bytes, prepares and activates the application, installs and binds the
real Query operation, executes and settles it, frames through egui, applies a
real operation-native update, publishes its sealed UI consequence, inspects,
replaces, removes, reintroduces, and disposes the application. The existing
application-contract aggregate owns this proof; no phase-specific integration
target was added.

Hostile evidence remains with the owner that can lawfully possess the
capability:

- WUI binding and runtime owners deny equal-label foreign installations, wrong
  binding meaning, stale settlement references, wrong workspaces,
  reset-required delivery, callback-unwind publication, and stopped disposal
  while retaining predecessor truth or the exact retryable resource.
- Query's installed-operation hostile owners deny swapped declaration,
  capability, contract, revision, family, runtime, generation, and settlement
  native keys before indexed access. They also deny foreign or superseded
  leases and deltas before mutation. WUI cannot construct these cases without
  adding the authority leak this milestone removes.
- The current compile aggregate proves the product facade cannot receive a
  settled projection, live resource, or sealed collection consequence. The
  physical inventory retains 251 real cases while executing 14 fail and 9 pass
  targets in two Cargo sessions.

Current Query 9.14 collection planning copies terminal posture from the prior
admitted window while incrementally deriving rows and continuation. Its
ordinary collection-application path therefore has no real producer for
result-state or warning patch operations. WUI retains compiler-exhaustive
terminal mapping for those receipt variants but does not fabricate a receipt,
rerun the installed operation after each patch, or create a parallel WUI state
source to claim behavioral coverage. A Query-owned producer that carries
executor-authoritative successor posture is the explicit prerequisite for the
additional behavioral case.

Close the public and documentary contract only after the real boundary is
working. Remove stale predecessor inventory references, narrow the Worth UI
facade around owner-controlled operations, and certify the full path through
existing real lifecycle owners.

**Relevant subsystems**

- `worth-ui-query-binding` public facade and module topology
- `worth-ui-runtime`, graph, inspection, certification, and host dependency
  surfaces
- application-contract, topology-contract, compile-contract, and Consumer Kit
  scenario owners
- compile-contract case inventory
- Milestones 3.9.1, 3.9.2, 3.10, 3.12, and 3.13 roadmap entries
- subsystem inventory and boundary-edge matrix

**Public DX target**

```rust
let installed = worth_ui_query_binding::install_domain(&mut query_runtime)?;
let binding = application.bind_query_view(&installed, authored_view)?;

let prepared = binding.prepare_snapshot(&query_workspace)?;
let settled = prepared.execute_and_settle(&mut query_workspace)?;
let consequence = binding.admit_measurement(settled)?;

framework_turn.admit_query_consequence(consequence)?;
```

The exact names may follow the existing facade vocabulary, but the shape is
mandatory: application code names UI intent; the binding owner performs Query
progression and validation; runtime code receives a sealed UI consequence;
host code receives neither.

**Warnings**

- Do not re-export raw operating worlds, Query proof constructors, native keys,
  broad settled projections, leases, deltas, patches, or owner-internal
  identity artifacts for facade convenience.
- Do not keep predecessor accessors “for diagnostics” if they still permit
  operational decisions. Terminal inspection APIs must be named and typed as
  observation.
- Do not add a compile case per removed constructor or an integration target
  per phase. Compile proof belongs in the existing positive and negative
  aggregate owners.
- A stale CSV path is a real test-harness defect even when that row happens not
  to run in the usual filter. Remove or replace the row with a current
  authority-bound case; do not point it at a deleted fixture.
- Documentation must distinguish historical 3.9.1 closure from the new
  3.9.2 cutover rather than rewriting the older phase plan.

**Test requirements**

- Extend the existing real `.wui` lifecycle scenario: write bytes to disk,
  observe them through production filesystem/watcher ingress, prepare and
  activate the application, install and bind the real Query operation, resolve
  native keys, execute, publish, consume-bound, settle, derive UI measurement,
  lower, frame, refresh through an operation-native patch, inspect, replace,
  and clean up through public facades.
- In that same scenario family, exercise equal labels across foreign runtimes,
  swapped native keys, stale settlement references, wrong UI meaning, stale
  leases, reset-required patches, interrupted publication, and stopped
  disposal. Every case must deny at its owner while prior active UI truth
  remains complete.
- Compile proof must show `worth-ui-runtime`, graph, host, and downstream apps
  cannot import Query, construct admitted source references, receive Query
  native keys or patches, or mint UI consequences. Positive cases must prove
  the intended lifecycle remains ergonomic.
- Run the Query reusable reference-consumer certification kit through the
  Worth UI binding owner and correlate its exact authority/counter results with
  UI-local consequence evidence. Do not manufacture Query receipts or bypass
  real file ingress.
- Parse the compile-contract CSV and prove every active case path exists.
  Remove the stale deleted managed-live fixture row and manually adjudicate any
  replacement coverage.
- Review every inventory and edge row against production code and real proof,
  then record honest completion evidence. Rerun searches as review provenance;
  no passing test depends on their count.
- Run boundary check, generated agent-context check, the WUI-scoped Rust
  line-cap check (`scripts/ci/check_workspace_rust_line_caps.sh worth-ui`),
  formatting, clippy, workspace tests, the two compile-contract invocations,
  and the existing targeted certification owners. Also record the canonical
  workspace-wide line-cap result as external baseline evidence; failures in
  non-WUI owners are not silently reported as this milestone's passing gate.

**Engineering decisions**

- The production facade exposes lifecycle-ordered Worth UI capabilities rather
  than Query assembly ingredients.
- Terminal diagnostics project labels, Query posture, and counters from the
  still-retained exact source; they never accept them back as authority.
- The 3.9.1 spec receives a dated closeout amendment stating its upstream exit
  trigger has fired and naming 3.9.2 as the successor. Its historical phase
  descriptions remain intact.
- The roadmap inserts 3.9.2 between 3.9.1 and 3.10 and sharpens the 3.10,
  3.12, and 3.13 non-overlap.
- Closure creates no new ordinary dependency edge on `worth-query`, no nested
  build, and no test-target explosion.

**Open questions**

- None.

## Must Ship

- a manually adjudicated subsystem inventory and boundary-edge matrix
- exact contract-derived Query native-access keys on the measurement path
- keyed settled and refreshed native access without whole-fact scans or copies
- binding-owner validation of Query operational identity followed by
  UI-minted opaque evidence
- UI-owned measurement and live-change consequences that retain no power to
  act as Query authority
- exact Query and UI counters proving bounded work
- public-facade, documentation, and certification closure without new compile
  topology

## Must Preserve

- every closed Milestone 3.9 and 3.9.1 truth that is not the obsolete access
  or identity mechanism
- atomic application replacement and framework-turn source publication
- prior active truth on every denial, stale input, reset requirement, or
  interrupted replacement
- Query-free applications without Query construction or cost
- Query-owned lifecycle, lease, invalidation, window, patch, denial, warning,
  result-state, provenance, and counter meaning
- UI-owned declaration, graph, measurement, allocation, source-coordinate,
  dependency, and presentation meaning
- one Query importer and Query-free runtime and host surfaces
- current compiler-session, integration-target, and cold-build budgets

## Acceptance Evidence

- every inventory and edge row is manually resolved and backed by real
  structural or behavioral evidence
- equal printable representation cannot substitute for exact Query authority
- exact Query authority cannot substitute for different UI-authored meaning
- `k` declared native facts require `k` keyed accesses independent of unrelated
  projected width
- no ordinary path scans or clones a settled projection to rediscover selected
  native facts
- live invalidation and patch delivery produce one admitted UI consequence and
  preserve prior truth on denial or reset
- Query objects and identity representations do not reach host authority or
  future mounted-receipt contracts
- boundary, agent-context, WUI-scoped line-cap, format, clippy, test, compile-
  contract, and existing certification gates pass without a new compile
  island
- the canonical workspace-wide line-cap check remains separately visible as
  an external baseline: 114 non-WUI Rust files exceed the cap, while no WUI
  Rust file does

## Sequencing Notes

- Milestone 3.9 remains historically closed. Milestone 3.9.1 remains the
  completed Phases 1-14 consumer-path modernization.
- Query 9.14's completion satisfies 3.9.1's explicit Phase 17/19/23/24 exit
  trigger. This named 3.9.2 milestone owns the resulting native-access,
  operational-identity, and operation-native delivery cutover.
- Phase order is mandatory: adjudicate scope; establish key-driven access;
  close identity readmission; rebuild UI derivation on those two truths;
  close live consequence delivery; prove exact cost; then close facades,
  documentation, and certification.
- Milestone 3.10 starts only after Phase 7 because mounted receipts must consume
  already-settled UI meaning.
- Milestone 3.13 remains responsible for broadening the proven consumer pattern
  beyond the current measurement and collection seams.
