# Milestone 2: WORTH-Native App Facade And Direct Typed Product Surface

## Goal

Ship the first-class WORTH-native `worth-server` surface so applications can
consume typed Query-backed declarations, reads, mutations, state, inspection,
projection facts, lease declaration, and delivery-contract negotiation directly
through the server without rebuilding ordinary product meaning as handwritten
endpoint glue, local status taxonomies, or caller-owned support folklore.

## Why This Milestone Exists

After Milestone 1, the server has one honest forced-entry path. The next naive
mistake would be to keep that path internal while WORTH-native apps continue to
rebuild their product surface as:

- handwritten endpoint families
- generated-client sprawl
- route-local request/response wrappers
- client-owned cache invalidation folklore
- bespoke live-sync plumbing above the same Query meaning

That would preserve the transport boundary while wasting the architectural
value of the server for the most important consumer family: WORTH-native
applications.

Milestone 2 exists to stop that failure by making direct server-managed
product consumption the ordinary WORTH-native path.

## Governing Summaries

- `MENTALITY.md`: solve the hostile "endpoint glue survives forever" and
  "caller-owned pseudo-Query grows above the server" problems first, not the
  easiest demo of one pretty direct-call API.
- `arch_laws.md`: the WORTH-native surface must stay one facade over proof-
  bearing server boundaries rather than becoming a parallel semantic entrypath.
- `composition_laws.md`: facade entry, operation families, capability posture,
  lease declaration, and certification need separate named homes rather than
  one giant direct-consumption file.
- `domain_structure_laws.md`: direct-consumption authority, derived delivery
  views, denial artifacts, and compatibility bridges must stay structurally
  distinct.
- `perf_laws.md`: WORTH-native ergonomics must not hide broad scans, rich-path
  work, rediscovered support posture, declaration reinterpretation, or
  transport-local fallback behavior.
- `milestone-1.md`: the direct surface is allowed only if it remains a real
  consumer of the Milestone 1 forced-entry path rather than a second semantic
  runtime above it.
- `worth_server_roadmap.md`: Milestone 2 belongs immediately after the forced-
  entry milestone because WORTH-native direct consumption is now the first real
  product-facing surface the server should own.
- `test-requirements.md`: Milestone 2 is not closed by API plausibility. It
  must satisfy the shared pipeline non-bypass, WORTH-native no-glue
  equivalence, and compatibility path-honesty certification suites with narrow
  artifacts and exact zero assertions where required.
- `AI_README.md`: Query is the ordinary domain-facing runtime, so the direct
  server facade must project Query-owned declaration identity, support and
  admission posture, reads, state, inspection, projection consumption, async
  result-state, intent, and delivery meaning instead of inventing a second
  product runtime.
- `milestone-9.4-closeout.md`: Query 9.4 already closes runtime-backed
  temporal/async/mixed-cause/downstream-delivery semantics on ordinary Query
  surfaces, so Milestone 2 should consume those surfaces directly and keep
  durable-later debt explicit.

## Adversarial Constraint

For the same authenticated principal, tenant/workspace target, branch/basis
posture, remask posture, diagnostics posture, canonical Query declaration
identity, and canonical Query intent, a WORTH-native application using the
direct server facade must resolve, admit, deny, read, inspect, mutate,
materialize projection facts, declare leases, and negotiate delivery through
the same canonical server-owned artifacts and capability posture as any other
surface.

This milestone fails if the WORTH-native surface:

- bypasses request-context, middleware, Query-handoff, response, or evidence
  boundaries behind ergonomic helpers
- invents a second meaning model for reads, mutations, state, or delivery
- invents caller-owned declaration identity, support posture, async state, or
  typed fact consumption above Query
- hides runtime-backed versus durable-later capability posture behind "it just
  works" convenience
- requires product teams to recreate ordinary product meaning as endpoint glue
  anyway
- or leaks branch, remask, provenance, or denial posture because the direct
  surface flattened them into convenience structs

## Product Decision Lock

- Milestone 2 is not a generated SDK milestone. It is a typed server facade
  milestone.
- WORTH-native applications should consume server-managed Query semantics
  directly when they stay inside admitted server contracts.
- The direct surface must remain visibly server-owned, not a thin alias for
  Query types and not a hidden bypass around Milestone 1.
- The direct surface must preserve canonical Query declaration identity and
  admitted family posture instead of teaching support from autocomplete or
  from visible method names.
- Reads, state, inspection, mutations, lease declaration, and delivery
  negotiation are separate direct-consumption responsibilities even when they
  share one facade root.
- Projection-fact consumption, async result-state, and time-aware retained
  posture are ordinary direct-consumption contracts when Query already admits
  them; they are not optional "advanced Query" escape hatches.
- Runtime-backed-now versus durable-later posture must stay explicit on the
  direct facade just as strongly as on the compatibility surface.

## Phase Plan

### Phase 1: Direct WORTH-Native Facade Root And Session-Bound Entry Boundary

Freeze the direct WORTH-native facade root so applications enter one typed
server-owned surface instead of reconstructing ordinary product access through
endpoint wrappers or ad hoc client scaffolding.

**Relevant subsystems**
- WORTH-native facade root
- client/session-bound facade handles
- direct surface registration and export boundaries

**Relevant APIs**
- `WORTHServerWORTHNativeFacade`
- `WORTHServerWORTHNativeSession`
- `WORTHServerWORTHNativeSurfaceRoot`
- `WORTHServer`

**Relevant Query surfaces**
- None. This phase should not introduce a Query-facing shortcut.

**Shared crate usage**
- None. This phase should consume the Milestone 1 facade/bootstrap boundary as
  already proven rather than creating a second proof or foundational vocabulary
  layer around direct-session entry.

**Warnings**
- Do not let applications reach past the direct facade into raw request-
  context, middleware, or Query-handoff internals.
- Do not make the direct facade a broad bag of unrelated convenience methods.

**Test requirements**
- Add a facade-entry parity test proving equivalent WORTH-native entry flows
  produce the same canonical request-context and admission artifacts as the
  already-admitted server path.
- Add a hidden-bypass test proving a WORTH-native caller cannot construct a
  direct session or direct operation handle without crossing the same typed
  server boundary as other surfaces.
- This phase must contribute to `Shared Pipeline Non-Bypass Torture Test` in
  [test-requirements.md](./test-requirements.md) by proving the direct session
  root cannot become a second semantic entrypoint.

**Engineering decisions**
- The WORTH-native surface is a real server facade, not a generated client and
  not a direct Query alias.
- Session-bound or caller-bound facade handles may exist, but they must carry
  server-owned proof, not ambient convenience-only state.

**Open questions**
- None.

### Phase 2: Canonical Declaration Identity, View Shape, And Support-Admission Intake

Freeze the direct intake boundary that lets WORTH-native applications enter the
server with canonical Query declaration identity, admitted family posture, and
view-shape-aware intent instead of rebuilding those contracts through local
builders, pseudo-support snapshots, or endpoint DTO vocabularies.

**Relevant subsystems**
- direct declaration intake
- support/admission posture projection
- view-shape-aware direct declaration binding
- saved-query and template consumption seams where admitted

**Relevant APIs**
- `WORTHServerDirectDeclaration`
- `WORTHServerDirectSupportSnapshot`
- `WORTHServerDirectViewShape`
- `WORTHServerWORTHNativeSession`

**Relevant Query surfaces**
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`
- `workspace.read(...)`
- `workspace.state(...)`
- admitted saved-query, scope, template, and view-shape authoring surfaces
- `WORTHQueryRuntimeFacadeFamily`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Declaration intake must
  consume existing request-context and Query-admission artifacts rather than
  creating a direct-only declaration review ladder.
- Use no new `worth-foundational` surfaces in this phase. The direct intake
  boundary should project server-owned support posture through existing
  admission and denial artifacts rather than minting a second support ontology.

**Warnings**
- Do not make the direct facade accept anonymous read builders whose canonical
  identity exists only in caller memory.
- Do not flatten admitted-versus-visible family posture into "method exists"
  ergonomics.
- Do not treat view shape as UI sugar if it affects delivery, invalidation, or
  retained-state semantics downstream.

**Test requirements**
- Add a declaration-identity parity test proving equivalent direct and
  compatibility-side declaration intake preserves the same canonical Query
  identity, admitted family classification, and denial posture.
- Add a support-posture localization test proving unsupported or
  visible-but-not-admitted families fail through typed server posture artifacts
  instead of ambient "not implemented" errors.
- Add a view-shape intake parity test proving direct table/detail/grouped or
  other admitted view-shape declarations preserve the same canonical Query
  meaning as the underlying admitted Query declaration family.
- This phase must contribute to `WORTH-Native No-Glue Equivalence Test` in
  [test-requirements.md](./test-requirements.md) by proving the direct facade
  owns declaration intake instead of product-local pseudo-Query layers.

**Engineering decisions**
- Canonical declaration identity is part of the direct server surface, not an
  internal Query detail callers are expected to rediscover.
- Support and admission posture must be consumable through typed direct-surface
  artifacts before ordinary product code performs meaningful work.

**Open questions**
- None.

### Phase 3: Direct Read, State, Inspection, And Retained Async-Time Posture Families

Freeze the ordinary direct-consumption read surface so WORTH-native
applications can express reads, state, inspection, and admitted runtime-backed
async/time-aware retained posture against server-managed Query meaning without
rewrapping them as route-local requests.

**Relevant subsystems**
- direct read operation family
- direct state operation family
- direct inspection operation family
- retained async/time posture projection
- Query-first lowering through the server

**Relevant APIs**
- `WORTHServerDirectRead`
- `WORTHServerDirectState`
- `WORTHServerDirectInspection`
- `WORTHServerDirectAsyncResultState`
- `WORTHServerDirectTemporalState`
- `WORTHServerQueryHandoff`
- `WORTHServerResponseEnvelope`

**Relevant Query surfaces**
- `workspace.read(...)`
- `workspace.state(...)`
- `workspace.inspect(...)`
- `workspace.read_live_intent(&view).review()?.admit()?.execute()`
- `workspace.inspect_intent(target).review()?.admit()?.execute()`
- admitted runtime-backed temporal live surfaces
- admitted async/resource result-state surfaces
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`
- `WORTHQueryRuntimeFacadeFamily`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. The direct read/state/
  inspection families should consume `WORTHServerRequestContext`,
  `WORTHServerAdmission`, and `WORTHServerQueryHandoff` rather than introduce a
  second progression model.
- Use `worth-foundational::facade::DiagnosticRichnessProfile` when the direct
  state or inspection artifacts expose diagnostics posture. Do not mint a
  direct-surface-only richness enum.

**Warnings**
- Do not merge reads, state, and inspection into one generic "fetch" helper.
- Do not let the direct read surface conceal whether an operation is one-shot,
  state-bearing, or inspection-bearing when that distinction affects meaning.
- Do not flatten retained async/resource posture into direct-surface-local
  `loading`, `retrying`, or `stale` enums when Query already owns those facts.
- Do not bypass Query support posture just because the caller is WORTH-native
  and in-process.

**Test requirements**
- Add a direct-read parity test proving equivalent WORTH-native reads and
  compatibility-surface reads compare equal on canonical Query-facing meaning
  where overlap exists.
- Add a direct-state parity test proving retained posture from
  `workspace.state(...)` remains canonical across direct and compatibility
  surfaces for the same retained handle family.
- Add an async/time posture parity test proving admitted runtime-backed
  temporal or async result-state survives on the direct surface without being
  reclassified into local convenience status taxonomies.
- Add a direct-surface localization test proving unsupported state or
  inspection combinations fail through typed server denial/support posture
  rather than broad "unsupported operation" convenience errors.
- This phase must satisfy the direct-read portions of `WORTH-Native No-Glue
  Equivalence Test` and the overlap portions of `Compatibility Surface
  Path-Honesty Test` in [test-requirements.md](./test-requirements.md).

**Engineering decisions**
- Read, state, and inspection stay separate direct operation families because
  they differ in semantic role even when they reuse the same underlying
  pipeline.
- The direct surface consumes server-owned Query handoff and response shaping;
  it does not reinterpret Query meaning locally.
- Retained async and time-aware posture belong on the same direct-surface truth
  family as other admitted retained-state contracts instead of on a parallel
  helper lane.

**Open questions**
- None.

### Phase 4: Direct Projection Consumption And Typed Fact Boundary

Freeze the direct projection-consumption surface so WORTH-native applications
can consume typed Query-materialized facts through the server without reopening
authority in relational truth, bridge internals, or caller-owned caches.

**Relevant subsystems**
- direct projection-fact consumption
- materialization-bound fact receipts
- basis- and policy-aware fact projection

**Relevant APIs**
- `WORTHServerDirectProjectionConsumption`
- `WORTHServerDirectFactReceipt`
- `WORTHServerDirectMaterializationDigest`
- `WORTHServerResponseEnvelope`

**Relevant Query surfaces**
- admitted projection-consumption declarations and receipts
- `workspace.state(...)`
- `workspace.inspect(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Projection consumption must
  consume the proof-bearing receipts and basis posture already established by
  Query and Milestone 1 server boundaries.
- Use `worth-foundational::facade::DiagnosticRichnessProfile` if direct fact
  receipts expose policy- or provenance-shaped richness posture. Do not create
  a direct-fact-only richness taxonomy.

**Warnings**
- Do not let the direct surface fish for facts in lower-authority runtime
  artifacts when Query already materialized them.
- Do not return anonymous payload bags when the fact family is semantically
  typed.
- Do not sever fact receipts from the materialization digest, basis, policy, or
  view-shape posture that made them valid.

**Test requirements**
- Add a projection-fact parity test proving equivalent direct and underlying
  Query projection-consumption paths preserve the same typed fact identity,
  materialization digest, basis posture, and denial surface.
- Add a hostile authority-reopening test proving the direct facade cannot bypass
  projection-consumption receipts to reconstruct facts from lower-runtime
  internals.
- Add a fact-remask localization test proving masked or policy-denied facts fail
  through typed server artifacts rather than arriving as partial anonymous
  payloads.
- This phase must contribute to `WORTH-Native No-Glue Equivalence Test` in
  [test-requirements.md](./test-requirements.md) by proving product code can
  consume typed materialized facts without rebuilding authority folklore.

**Engineering decisions**
- Projection consumption is part of the ordinary direct product surface when
  Query already owns typed fact receipts.
- Fact consumption must remain receipt- and basis-bound, not cache- or
  convenience-bound.

**Open questions**
- None.

### Phase 5: Direct Mutation Surface And Provenance-Bearing Result Boundary

Freeze the direct mutation surface so WORTH-native applications can issue
authoritative Query-backed mutations through the server without rebuilding
mutation wrappers or weakening typed denial, provenance, and capability
posture.

**Relevant subsystems**
- direct mutation operation family
- mutation admission and lowering
- direct mutation result shaping

**Relevant APIs**
- `WORTHServerDirectMutation`
- `WORTHServerPipelineIntent`
- `WORTHServerQueryOperation`
- `WORTHServerSuccessEnvelope`
- `WORTHServerDenialEnvelope`

**Relevant Query surfaces**
- `workspace.write_intent(command).review()?.admit()?.execute()`
- `workspace.write_batch_intent(commands).review()?.admit()?.execute()`
- `workspace.write_intent(command).execute()`
- `workspace.write_batch_intent(commands).execute()`
- `workspace.inspect(&receipt)`
- `workspace.write(...)` as an expert lower-level seam, not the ordinary direct
  surface

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Direct mutation must consume
  the Milestone 1 middleware and handoff proofs rather than create a second
  mutation-review ladder.
- Use no new `worth-foundational` surfaces in this phase. Direct mutation
  should project the response/provenance artifacts already established in
  Milestone 1 Phase 6 rather than wrap them in a parallel direct-only evidence
  model.

**Warnings**
- Do not make direct mutation a raw pass-through to lower runtime mutation
  calls.
- Do not collapse validation, authorization, and capability denial into a
  single "mutation failed" direct-surface result.
- Do not teach `workspace.write(...)` as the normal WORTH-native direct path.

**Test requirements**
- Add a direct-mutation parity test proving equivalent WORTH-native and
  compatibility-surface mutations preserve the same canonical mutation-facing
  response, provenance, and support posture.
- Add a hostile mutation-denial test proving invalid, forbidden, or
  capability-mismatched direct mutations localize to the correct server
  boundary rather than returning client-convenience error fog.
- Add an expert-seam denial test proving direct mutation does not silently
  degrade from intent-shaped mutation into the lower-level `workspace.write(...)`
  path under unsupported-intent pressure.
- This phase must satisfy the mutation-bearing lanes of `WORTH-Native No-Glue
  Equivalence Test` and `Compatibility Surface Path-Honesty Test` in
  [test-requirements.md](./test-requirements.md).

**Engineering decisions**
- Direct mutation remains server-owned network/runtime behavior even when the
  caller is in-process and WORTH-native.
- The direct surface may expose a more ergonomic call shape than HTTP, but it
  may not reduce proof-bearing denial and provenance structure.

**Open questions**
- None.

### Phase 6: Lease Declaration And Delivery-Contract Negotiation Surface

Freeze the direct WORTH-native lease and delivery-negotiation surface so
applications can declare server-managed live product needs directly without
dropping down into HTTP-shaped subscription glue.

**Relevant subsystems**
- direct lease declaration
- direct downstream-delivery contract negotiation
- freshness-mode request shaping
- delivery-class and resume-basis request shaping
- runtime-backed resume posture exposure

**Relevant APIs**
- `WORTHServerDirectLeaseDeclaration`
- `WORTHServerDirectDeliveryContract`
- `WORTHServerDirectFreshnessMode`
- `WORTHServerDirectDeliveryClass`
- `WORTHServerQueryRequestedResume`
- `workspace.public_downstream_delivery_contract()`
- `workspace.downstream_delivery(...)`

**Relevant Query surfaces**
- `workspace.public_downstream_delivery_contract()`
- `workspace.downstream_delivery(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`
- `workspace.state(...)`
- `workspace.inspect(...)`
- admitted freshness-mode and delivery-class Query-facing contracts
- `WORTHQueryRuntimePublicApiFamilyContract`
- `WORTHQueryLowerRuntimeSupportPosture`

**Shared crate usage**
- Use no new `worth-proof` surfaces in this phase. Lease declaration and
  delivery negotiation should consume the existing server query-handoff and
  support posture artifacts rather than introduce a direct-only resume proof
  family.
- Use no new `worth-foundational` surfaces in this phase. The direct lease
  surface should stay a server facade over Query delivery contracts, not a new
  evidence ontology.

**Warnings**
- Do not let lease declaration become connection-local or UI-widget-local
  state.
- Do not collapse delivery class, freshness mode, and resume-basis requests
  into one opaque "subscribe" convenience blob.
- Do not mislabel runtime-backed resume as durable restart-stable resume on the
  direct surface.
- Do not let the direct facade negotiate delivery without consulting Query
  support posture first.

**Test requirements**
- Add a lease-declaration parity test proving equivalent WORTH-native lease
  declarations and ordinary server lease declarations compare equal on
  canonical identity and capability posture.
- Add a delivery-request parity test proving direct freshness-mode,
  delivery-class, and requested-resume contracts preserve the same canonical
  Query-facing negotiation meaning as the underlying handoff.
- Add a hostile delivery-negotiation denial test proving stale, unsupported, or
  durable-later resume requests fail typed and visible on the direct surface.
- Add a support-posture parity test proving direct negotiation preserves the
  same `WORTHQueryLowerRuntimeSupportPosture` classification as the underlying
  Query-facing handoff.
- This phase must satisfy the capability-honesty parts of `WORTH-Native
  No-Glue Equivalence Test` in [test-requirements.md](./test-requirements.md),
  especially the rule that direct ergonomics may not hide runtime-backed
  versus durable-later posture.

**Engineering decisions**
- Direct lease declaration is still server state, not client-owned state with a
  nicer constructor.
- Delivery negotiation stays Query-first and contract-first; the direct surface
  is an admission facade over those contracts.
- Freshness, delivery class, and resume requests are named direct-surface
  contracts even before the later sync runtime milestones close transport
  delivery itself.

**Open questions**
- None.

### Phase 7: Branch, Basis, Remask, And Provenance Direct-Consumption Closure

Freeze the direct-surface artifact family that makes branch, basis, remask, and
provenance posture explicit and client-consumable instead of hidden behind
WORTH-native convenience defaults.

**Relevant subsystems**
- direct branch/basis targeting vocabulary
- remask-aware direct-consumption artifacts
- provenance-bearing direct result views
- direct support-posture shaping

**Relevant APIs**
- `WORTHServerWorkspaceTarget`
- `WORTHServerBranchTarget`
- `WORTHServerQuerySupportPosture`
- `WORTHServerProvenance`
- `WORTHServerOperatorEvidenceRecord`

**Relevant Query surfaces**
- `workspace.state(...)`
- `workspace.inspect(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`
- `workspace.downstream_delivery(...)`
- `WORTHQueryRuntimeFacadeFamily`
- `WORTHQueryRuntimePublicApiFamilyContract`

**Shared crate usage**
- Use `worth-foundational::facade::DiagnosticRichnessProfile` for direct
  diagnostics posture whenever branch, remask, or provenance visibility depends
  on richness selection.
- Use `worth-foundational::facade::FoundationalBoundaryEvidenceReceiptFrontDoor`
  and `FoundationalBoundaryEvidenceProvenanceFrontDoor` when the direct surface
  needs a server-owned projection over existing response/provenance artifacts
  rather than raw envelope passthrough.
- Use no new `worth-proof` surfaces in this phase. Direct-consumption closure
  should expose the existing proven server posture rather than generate a
  second branch/remask proof family.

**Warnings**
- Do not hide branch, basis, or remask posture just because the caller is
  WORTH-native and "trusted."
- Do not export raw foundational or Query artifacts directly if the direct
  surface needs a server-owned visibility contract around them.
- Do not remask after direct result shaping.

**Test requirements**
- Add a direct-consumption parity test proving equivalent branch-aware
  WORTH-native and compatibility flows compare equal on canonical basis,
  provenance, and remask-visible meaning where overlap exists.
- Add a hostile remask/support test proving the direct surface preserves typed
  denial, support, or remask narrowing under permission or basis pressure
  rather than silently flattening visible truth.
- Add a provenance-richness test proving reduced diagnostics richness trims
  detail without changing branch, remask, or support classification.
- This phase must satisfy the branch, basis, remask, and diagnostics-richness
  portions of `WORTH-Native No-Glue Equivalence Test` in
  [test-requirements.md](./test-requirements.md).

**Engineering decisions**
- Ergonomics may compress ceremony, but they may not erase server-owned
  posture.
- Provenance and support posture are ordinary direct-consumption contracts, not
  optional debugging add-ons.

**Open questions**
- None.

### Phase 8: WORTH-Native Ergonomics And No-Endpoint-Glue Composition Boundary

Freeze the composition boundary that makes the WORTH-native facade genuinely
replace ordinary endpoint glue for common product work instead of merely
wrapping the same glue in a nicer API.

**Relevant subsystems**
- direct surface composition model
- operation-family aggregation
- product-local facade integration points
- declaration/support snapshot composition
- projection-fact and retained-state composition
- capability/cost visibility

**Relevant APIs**
- `WORTHServerWORTHNativeFacade`
- `WORTHServerDirectDeclaration`
- `WORTHServerDirectRead`
- `WORTHServerDirectProjectionConsumption`
- `WORTHServerDirectMutation`
- `WORTHServerDirectLeaseDeclaration`
- `WORTHServerDirectDeliveryContract`

**Relevant Query surfaces**
- `workspace.read(...)`
- `workspace.state(...)`
- `workspace.inspect(...)`
- `workspace.write_intent(...)`
- admitted projection-consumption surfaces
- `workspace.public_downstream_delivery_contract()`
- `workspace.public_support_matrix()`

**Shared crate usage**
- None. This phase should compose the direct surface from the earlier server and
  Query artifacts already admitted instead of inventing new proof or
  foundational seams.

**Warnings**
- Do not turn ergonomics into magic by hiding broad work behind cheap-looking
  helpers.
- Do not force product code to stitch together five low-level steps when the
  direct surface claims to replace endpoint glue.
- Do not let convenience helpers bypass explicit support posture or basis
  posture just to keep the call shape short.
- Do not require caller-owned glue to translate canonical declaration identity,
  typed fact receipts, or retained async posture into product-usable shapes.

**Test requirements**
- Add a no-endpoint-glue equivalence test proving a representative WORTH-native
  product flow can be expressed through the direct facade without a parallel
  endpoint family while preserving canonical server meaning.
- Add a cost-honesty test proving direct convenience calls still preserve
  explicit capability posture, basis posture, and exact narrow counters where
  the operation boundary is expensive or denial-bearing.
- Add a composition-residue test proving direct ergonomics do not require
  endpoint-like DTO layers or route-only glue to recover ordinary state,
  provenance, or denial meaning.
- Add a retained-posture composition test proving product-facing direct flows
  can carry support posture, async/time result-state, and typed fact receipts
  without introducing caller-owned status enums or cache folklore.
- This phase is the local implementation home for the core pass condition of
  `WORTH-Native No-Glue Equivalence Test` in
  [test-requirements.md](./test-requirements.md): reducing glue without
  reducing semantic honesty.

**Engineering decisions**
- The direct facade should compose the right server-owned artifacts into a
  product-usable shape, but only at explicit semantic seams.
- Convenience belongs at the facade root; semantic truth still belongs to the
  typed operation/result artifacts underneath.

**Open questions**
- None.

### Phase 9: Hostile Direct-Surface Certification Closure

Close Milestone 2 with certification that proves the WORTH-native facade is a
real server-owned product surface rather than a friendly bypass around the
existing pipeline.

**Relevant subsystems**
- WORTH-native certification harness
- cross-surface parity bundles
- no-endpoint-glue certification
- direct-surface sabotage guards

**Relevant APIs**
- direct-surface certification bundles
- canonical declaration, request-context, handoff, response, and support
  digests
- counter and provenance certification artifacts

**Relevant Query surfaces**
- `workspace.read(...)`
- `workspace.state(...)`
- `workspace.inspect(...)`
- `workspace.write_intent(...)`
- admitted projection-consumption surfaces
- `workspace.public_downstream_delivery_contract()`
- `workspace.downstream_delivery(...)`
- `workspace.public_support_matrix()`
- `workspace.admit_public_api_family(...)`

**Shared crate usage**
- Use `worth-foundational::facade::FoundationalBoundaryArtifactCompileFailBoundary`,
  `FoundationalBoundaryEvidenceCompileFailBoundary`, and
  `FoundationalPerformanceCompileFailBoundary` to classify direct-surface
  compile-fail, evidence, and forbidden-counter certification boundaries.
- Use `worth-foundational::facade::FoundationalBoundaryArtifactProductionTestReadyArtifact`,
  `FoundationalBoundaryEvidenceProductionTestReadyArtifact`, and
  `FoundationalPerformanceProductionTestReadyArtifact` if this phase emits
  milestone-close certification artifacts beyond local tests.
- Use no `worth-proof` surfaces unless the certification harness itself becomes
  a proof-bearing transition family rather than a hostile comparison harness.

**Warnings**
- Do not close the milestone on one successful in-process call path.
- Do not use broad high-level result equality as the primary proof that the
  direct surface is honest.
- Do not certify the direct surface without comparing it to the compatibility
  overlap lanes where semantics should match.
- Do not omit declaration identity, support posture, typed fact receipts, or
  retained async/time posture from certification just because the happy path
  read result looked correct.

**Test requirements**
- Add one mixed-hostility WORTH-native certification matrix varying branch,
  basis, remask, diagnostics posture, runtime-backed versus durable-later
  requests, declaration/view-shape intake, and equivalent compatibility-surface
  overlap while asserting exact canonical declaration, request-context,
  handoff, response, support, provenance, and counter digests.
- Add one direct-surface sabotage suite that attempts raw Query access,
  skipped middleware/admission use, WORTHd direct-session handles, and direct
  lease/dependency construction, and proves each attempt fails at the narrowest
  expected boundary with exact zero assertions for forbidden success and
  forbidden evidence residue.
- Add one no-endpoint-glue certification lane proving the direct facade can
  express the representative product flow without hidden endpoint families and
  without semantic drift in the certified artifacts.
- Add one retained-posture and typed-fact certification lane proving direct
  async/time result-state and projection-fact consumption remain parity-safe
  and receipt-bound without caller-owned reinterpretation.
- This phase must close Milestone 2 against the named suites
  `Shared Pipeline Non-Bypass Torture Test`, `WORTH-Native No-Glue Equivalence
  Test`, and the Milestone 2 overlap portions of `Compatibility Surface
  Path-Honesty Test` in [test-requirements.md](./test-requirements.md).

**Engineering decisions**
- Milestone 2 is only closed if WORTH-native ergonomics are parity-safe and
  mechanically non-bypass.
- The proof bar is "no endpoint glue without semantic loss," not "friendlier
  API with the same hidden architecture debt."

**Open questions**
- None.

## Must Ship

- one typed WORTH-native server facade root for ordinary product consumption
- direct declaration intake carrying canonical Query identity, view-shape
  posture, and support/admission posture
- direct Query-backed read, state, inspection, and admitted retained
  async/time-posture operation families
- direct projection-consumption and typed fact-receipt surfaces where Query
  already admits them
- direct Query-backed mutation operation families with typed result posture
- direct lease declaration and downstream-delivery negotiation surfaces
- explicit branch, basis, remask, provenance, and capability artifacts on the
  direct surface
- ergonomic composition strong enough to replace large families of handwritten
  endpoint glue for ordinary WORTH-native work
- hostile certification proving the direct surface is parity-safe and
  mechanically non-bypass

## Must Preserve

- the WORTH-native facade remains server-owned rather than Query-owned or
  client-owned
- Milestone 1 forced-entry boundaries remain mandatory for every direct
  operation
- Query remains the semantic authority for ordinary product meaning
- canonical declaration identity, typed fact receipts, and retained async/time
  posture remain Query-owned semantics projected through server-owned surfaces
- runtime-backed-now versus durable-later posture remains explicit on the
  direct surface
- ergonomics do not conceal cost, denial, remask, provenance, or support
  posture

## Acceptance Evidence

- direct WORTH-native and compatibility overlap flows compare equal on
  canonical declaration, request-context, handoff, response, support,
  provenance, and materialization-bound fact artifacts where they should
- typed denial artifacts localize invalid, unsupported, forbidden, remasked,
  or basis-mismatched direct operations to the correct server boundary
- admitted async/time result-state and projection-fact consumption remain
  parity-safe and do not degrade into caller-owned status or cache folklore
- representative WORTH-native product flows can be expressed without parallel
  handwritten endpoint families for ordinary product work
- hostile certification proves direct-surface bypass attempts fail with exact
  zero assertions for forbidden success and forbidden evidence residue

## Sequencing Notes

Milestone 2 belongs immediately after Milestone 1 because the server now has an
honest forced-entry path but not yet a first-class product-facing surface for
the most important consumer family: WORTH-native applications.

It belongs before Milestone 3 because the roadmap intentionally treats
WORTH-native direct consumption and compatibility HTTP as separate lanes, and
WORTH-native ergonomics are now a first-class architectural goal rather than a
later convenience layer on top of compatibility endpoints.
