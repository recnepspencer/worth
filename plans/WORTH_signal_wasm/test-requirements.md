# worth-signals-wasm Test Requirements

> **Status:** Completed certification spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Milestone parent:** [api_surface_plan.md](./api_surface_plan.md)
>
> **Formal closeout:** [api_surface_closeout.md](./api_surface_closeout.md)
>
> **Core lineage:** [\plans/worth_signal/test-requirements.md](../WORTH_signal/test-requirements.md)

## Purpose

This document defines the certification bar for the `worth-signals-wasm`
resource/API product surface.

It is not a list of example tests.
It is the proof contract that closes the API-surface milestone.

The package is not done when:

- detail, collection, paged, refresh, retry, upload, patch, diagnostics, or
  branch APIs appear to work in happy-path examples
- TypeScript types look refined
- runtime tests cover nominal behavior

The package is done only when the product surface can prove that:

- the same semantic resource declaration converges to the same local line truth
  regardless of authoring posture
- runtime-owned lifecycle, retry, timeout, supersession, continuity, replay,
  branch, and restore truth remain authoritative at the package boundary
- narrow reconciliation, delivery, diagnostics, and export-facing surfaces do
  not hide broader cost, broader authority, or a second client truth model
- later binary/download/delivery and external-system integration can plug into
  the same line model without semantic drift

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is hostile-proof design. The test suite
  must certify the adversarial constraint, not just the public nouns.
- `arch_laws.md`
  The most important thing it protects is boundary honesty. Every resource
  boundary must emit self-describing artifacts and must not merge different
  authority or cost categories into one convenience surface.
- `perf_laws.md`
  The most important thing it protects is breadth honesty. Cheap-looking reads,
  patches, refreshes, summaries, and delivery surfaces must prove their scope
  explicitly.
- `domain_laws.md`
  The most important thing it protects is subsystem clarity. Test suites and
  harnesses must mirror real proof domains instead of collapsing into giant
  resource buckets.
- `worth_signal_vision.md`
  The most important thing it protects is that `worth-signal` remains derived
  execution substrate rather than becoming truth-state storage. The wasm
  product surface must certify this boundary instead of weakening it.
- `wasm_product_roadmap.md`
  The most important thing it protects is sequencing. Resource/API proof must
  consume closed runtime semantics and preserve a clean path to downloads,
  delivery, and later external integration.
- `api_surface_plan.md`
  The most important thing it protects is one coherent line model across all
  phases. Identity, lifecycle, request posture, reconciliation, diagnostics,
  binary descriptors, and external compatibility must all converge on the same
  local materialization truth.
- `worth_signal/test-requirements.md`
  The most important thing it protects is certification rigor. This wasm
  document must require named adversarial suites, canonical artifacts, replay
  parity, compile-time boundaries, and cost proof rather than example-only
  coverage.

## Adversarial Constraint

This certification program must survive the following hostile condition:

> A long-lived application with resource families, pending reloads, retry and
> timeout policy, host-driven revalidation, partial item/aspect/summary patch
> reconciliation, branch-local history, binary/download descriptors, pushed
> updates, and later externally-authored read definitions must converge to the
> same committed local line truth, the same visible lifecycle/freshness truth,
> and the same diagnostics/history explanation regardless of whether the line
> was driven by first load, refresh, revalidate, invalidation, patch delivery,
> upload/deferred completion, branch restore, replay, or external delivery.

If two semantically equivalent histories can produce:

- different line identity for the same declaration and canonical parameters
- different visible value, status, freshness, diagnostics, or history artifacts
- silent broad replace where a narrower declared path was admissible
- hidden rich-history work behind summary-shaped APIs
- different local truth for signals-first versus externally-driven materialized
  lines
- or a second client-owned lifecycle/cache authority

then the product has failed certification.

## Certification Rules

Every required named suite in this document must:

- run with canonical artifact emission, not only assertion-style pass/fail
- define its hostile workload explicitly
- verify runtime behavior, public facade behavior, and type-surface boundaries
  where relevant
- certify replay/restore/branch parity whenever the phase claims those
  semantics exist
- certify breadth or cost honesty whenever the public API looks cheap
- include denial artifacts for ineligible, incompatible, or undeclared paths
  rather than only permissive success cases

Where a suite names a compile-time boundary, the package must maintain explicit
compile-fail proof fixtures or equivalent type-checking artifacts that stay in
sync with the public declaration surface.

## Verification Package Standard

Every broad certification family should emit a canonical verification package
containing the categories relevant to that suite.

The package vocabulary for this milestone is:

- declaration digest
- canonical parameter digest
- family and line identity digest
- request posture digest
- lifecycle digest
- freshness and invalidation digest
- continuity digest
- reconciliation digest
- patch breadth digest
- diagnostics digest
- history/replay/restore digest
- binary/download descriptor digest
- delivery provenance digest
- external-compatibility digest
- boundary performance envelope
- typed denial or incompatibility artifact

Equivalent runs must match exactly except for fields explicitly declared
non-semantic.

0. The Full Resource Hostile Replay And Branch Convergence Test

Purpose

Prove that the complete resource product surface remains one coherent system
rather than seven individually-correct subsystems that drift when combined.

Why it matters

Phase-local suites can all pass while the real product still forks into:

- one identity story for initial loads
- another lifecycle story for refresh and timeout
- another reconciliation story for delivery and patch
- another explanation story for branch restore and retention

That is exactly the failure mode this milestone exists to prevent.

What to stress

Build one medium-large application graph containing:

- detail, collection, and paged resource families
- canonical same-param and neighboring changed-param family members
- sync and async first-load lines
- refresh, retry, timeout, supersession, and host-driven revalidation
- auth, context, continuation, upload, and deferred-processing posture
- item, aspect, and summary reconciliation
- binary/download descriptors
- pushed delivery packets
- externally-driven compatibility declarations and basis refresh
- branch fork, restore, replay, and retained-history truncation

Run one hostile script with:

- repeated same-param reads and changed-param rematerialization
- narrow patch, broad replace, and invalidation interleaving
- out-of-order completions and out-of-order delivery packets
- duplicate delivery packets
- stale basis delivery attempts
- upload prepare/finalize followed by deferred processing
- branch fork before completion and restore before and after completion
- replay from retained history and replay from full canonical history

Execute the full scenario in at least:

- ordinary forward execution
- branch fork plus restore execution
- retained-history replay
- full canonical replay

What to verify

- all modes converge to identical committed local line truth
- all modes converge to identical visible lifecycle/freshness truth
- diagnostics summaries, full diagnostics, and history artifacts remain
  semantically aligned
- no path creates a second cache, lifecycle, or delivery authority
- every denial remains explicit and replay-stable

Pass condition

The verification package must emit family identity digest, request digest,
lifecycle digest, continuity digest, reconciliation digest, binary/download
digest, delivery provenance digest, external compatibility digest,
diagnostics/history digest, replay/restore digest, and boundary performance
envelope. Equivalent histories must match exactly when semantically equivalent.

## Phase Coverage Map

- Full milestone closeout additionally requires suite 0.
- Phase 1 is closed only by suites 1 through 5.
- Phase 2 is closed only by suites 6 through 9.
- Phase 3 is closed only by suites 10 through 14.
- Phase 4 is closed only by suites 15 through 19.
- Phase 5 is closed only by suites 20 through 22.
- Phase 6 is closed only by suites 23 through 25.
- Phase 7 is closed only by suites 26 through 28.

## Phase 1: Resource Family Identity And Materialization Kernel

1. The Resource Family Identity Equivalence Test

Purpose

Prove that one semantic family declaration plus one canonical parameter set
materializes exactly one authoritative local line identity.

What to stress

- detail, collection, and paged family declarations
- repeated same-parameter reads through the same family
- neighboring changed-parameter reads through the same family
- scope-local rematerialization after explicit free
- materialization before and after graph publication
- identity reads through line descriptor, line facade, and graph publication

What to verify

- same semantic family + same canonical params reuses one line identity
- changed canonical params produce explicit replacement or new-line identity
- free/rematerialize keeps family/param identity truth while changing only the
  lifecycle-owned line instance when appropriate
- graph publication does not collapse the public output contract into internal
  family identity

Pass condition

The verification package must emit declaration digest, canonical parameter
digest, family identity digest, line identity digest, publication contract
digest, and rematerialization digest. Equivalent same-param paths must match
exactly.

2. The Canonical Parameter Normalization Test

Purpose

Prove that parameter identity is stable, immutable, and replay-safe rather than
caller-reference folklore.

What to stress

- equivalent parameter objects with different property ordering
- caller mutation after materialization
- nested parameter shapes
- branded and unbranded authoring helpers
- restore/replay from retained parameter identity

What to verify

- semantic equality lowers to one canonical parameter digest
- caller mutation after admission cannot mutate canonical identity
- incompatible or non-normalizable params deny before line reuse is claimed
- replay reconstructs the same canonical identity story

Pass condition

The verification package must emit raw-input digest, normalized-parameter
digest, canonical identity digest, mutation-immunity report, and replay digest.

3. The Resource Family Shape Compile-Time Separation Test

Purpose

Prove that detail, collection, and paged families do not expose each other's
shape-specific surfaces by accident.

What to stress

- accessing collection-only patch/reconciliation APIs on detail families
- accessing paged-only surfaces on detail or plain collection families
- constructing family-specific lowered proofs outside the owning module
- unions where reconciliation or family-kind truth is only maybe present

What to verify

- the compiler rejects illegal capability access where possible
- the type surface does not overclaim narrow patch or family-shape legality
- runtime admission rejects any remaining WORTHd or widened paths before
  materialization work is constructed

Pass condition

No family-kind-specific facade, lowering proof, or patch surface may be
reachable without the correct declaration-bearing proof path.

4. The Canonical Line Facade Stability Test

Purpose

Prove that the public line facade tells one stable story across family kinds and
materialization paths.

What to stress

- `value()`, `status()`, `freshness()`, `descriptor()`, `signal()`,
  `history()`, `free()`, and disposal
- first materialization, rematerialization, refresh, invalidation, and branch
  publication contexts
- freed handle denial across every operational method

What to verify

- the same semantic line exposes one canonical facade vocabulary
- release semantics are framework-owned and deny post-free operational access
- facade reads remain self-describing and do not leak internal runtime records

Pass condition

The verification package must emit facade-shape digest, release-state digest,
denial digest, and publication-compatibility digest.

5. The Resource Line View Narrowing Test

Purpose

Prove that line-scoped views are owned by line lifecycle and remain narrower
derived consumers rather than parallel resource handles.

What to stress

- multiple views over the same line
- same-view reuse and changed-view distinction
- line free/dispose with extant views
- refresh and invalidation while views are alive

What to verify

- views do not become separate lifecycle authorities
- freeing a line disposes owned views
- view identities, values, and denials stay subordinate to the owning line

Pass condition

The verification package must emit view-identity digest, ownership digest,
dispose cascade digest, and denial digest.

### Phase 1 Closeout Requirements

Required hostile dimensions

- all family kinds must be exercised under the same proof harness, not in three
  unrelated example files
- same semantic declarations authored through direct and helper-driven paths
  must prove descriptor equivalence
- line identity, publication identity, and line-scoped view identity must be
  checked both before and after rematerialization
- every lifecycle-owned release path must be tested both by explicit `free()`
  and by disposal syntax where supported

Required owning proof lanes

- one runtime proof lane for family identity and rematerialization
- one compile-time proof lane for family-shape and capability separation
- one publication proof lane for graph-facing contract stability
- one lifecycle-ownership proof lane for views and disposal

Phase 1 pass condition

Phase 1 is not closed until the package can prove that one semantic declaration
produces one identity story across direct line reads, graph publication, line
views, rematerialization, replay, and release. If any one of those surfaces
needs its own identity exception rule, Phase 1 is not done.

## Phase 2: Runtime-Lowered Refresh, Revalidation, And Continuity

6. The Resource Refresh And Continuity Policy Parity Test

Purpose

Prove that refresh and revalidate expose runtime-owned lifecycle and continuity
truth rather than package-local status folklore.

What to stress

- refresh versus revalidate
- visible prior value preserved while pending
- rejection, timeout, and supersession with and without visible prior value
- sync and async reload paths
- repeated retry-bearing failures before final settlement
- branch fork while one reload lineage is pending
- host-driven revalidation overlapping an app-issued refresh
- same tests under detail, collection, and paged lines

What to verify

- lifecycle states, freshness, and continuity are identical across equivalent
  sync and async histories
- visible-value continuity remains policy truth, not UI-local accident
- superseded completions never become authoritative

Pass condition

The verification package must emit lifecycle digest, continuity digest,
supersession digest, visible-value digest, and denial-history digest.

7. The Resource Policy Profile Lowering Test

Purpose

Prove that named profile declarations lower to deterministic runtime policy
truth rather than becoming friendly option bags.

What to stress

- default, retry, timeout, and combined profiles
- semantically equivalent declarations through direct and named-profile paths
- incompatible profile selection or unknown profile ids
- replay and restore under compatible and incompatible policy descriptors
- profile changes across retained-history restore boundaries
- profile selection under host-driven revalidation and branch-local refresh

What to verify

- equivalent policy declarations lower to identical canonical descriptors
- incompatible or unknown profiles deny before execution work is constructed
- replay and restore honor policy compatibility explicitly

Pass condition

The verification package must emit policy registry digest, lowered profile
descriptor digest, compatibility artifact, lifecycle digest, and replay digest.

8. The Resource Invalidation And Freshness Breadth Test

Purpose

Prove that line, member, and family invalidation tell an explicit freshness
story and do not hide broader scope behind one stale bit.

What to stress

- line-local invalidation
- single-member invalidation
- family-wide invalidation
- invalidation followed by patch, refresh, revalidate, and branch restore
- duplicate invalidation events and collapsed equivalent invalidation histories
- invalidation while a retry-bearing or timeout-bearing reload is still pending

What to verify

- freshness explicitly distinguishes line, member, and family breadth
- diagnostics and history record the exact invalidation cause and scope
- invalidation breadth does not silently widen under equivalent local cases

Pass condition

The verification package must emit invalidation-scope digest, freshness digest,
diagnostics digest, history digest, and breadth envelope.

9. The Resource Initial Async Materialization Parity Test

Purpose

Prove that promise-backed first load and later reloads converge on one line
model instead of one eager model plus one async exception path.

What to stress

- initial pending with no visible value yet
- timeout and rejection before first visible value exists
- refresh overtaking an initial pending load
- same-params reuse while initial load is pending
- branch restore before first settlement and after superseded first settlement
- retained-history replay of no-visible-value-yet histories

What to verify

- first-load lifecycle is honest and nullable-value aware
- continuity with no visible value yet is explicit rather than implied
- later refresh supersession reconstructs one coherent lifecycle/history story

Pass condition

The verification package must emit initial-load digest, visible-value-absence
digest, supersession digest, same-param reuse digest, and replay digest.

### Phase 2 Closeout Requirements

Required hostile dimensions

- every lifecycle state must be exercised under both direct app-issued work and
  host-driven revalidation
- timeout, retry, supersession, and invalidation must be tested both alone and
  in overlapping mixed histories
- both visible-value continuity and no-visible-value-yet continuity must be
  certified under replay and branch restore
- policy profile equivalence must be proven across named-profile and
  semantically identical direct declarations

Required owning proof lanes

- one lifecycle proof lane for refresh/revalidate/retry/timeout/supersession
- one freshness/invalidation breadth proof lane
- one policy-lowering and compatibility proof lane
- one replay/restore proof lane dedicated to async-first line histories

Phase 2 pass condition

Phase 2 is not closed until pending, retry, timeout, supersession,
revalidation, invalidation, and continuity can be explained as one runtime
story across live diagnostics, retained history, replay, and restore. If any of
those states are only visible in one surface or only reproducible in forward
execution, Phase 2 is not done.

## Phase 3: Request Context, Auth, And Continuation Posture

10. The Auth And Request Context Lowering Test

Purpose

Prove that auth and request context are typed semantic declarations that lower
consistently into one canonical request story.

What to stress

- explicit auth declarations
- param-derived auth/context declarations
- context headers, correlation ids, branch ids, and basis ids
- secret-bearing posture such as authorization or signed header material
- equivalent declarations through different authoring helpers
- refresh, rematerialization, and replay after request posture changes
- diagnostics and history generation under retained and rich-history conditions

What to verify

- equivalent request posture declarations lower to identical request digests
- diagnostics and `line.request()` expose grouped request truth cleanly
- semantic request posture is preserved without leaking raw secret material into
  diagnostics, history, replay, or exported verification artifacts
- undeclared or incompatible posture is denied before load work begins

Pass condition

The verification package must emit request posture digest, lowered request
descriptor digest, diagnostics request summary digest, secret-safe artifact
digest, denial artifact, and replay digest.

11. The Redirect And Callback Continuation Parity Test

Purpose

Prove that continuation posture is explicit request truth rather than ambient
application folklore.

What to stress

- no continuation
- redirect continuation
- callback continuation
- webhook continuation
- equivalent continuation truth across direct and param-derived declarations
- continuation posture changing across refresh/replay boundaries
- redirect or callback completion arriving after the line has already
  superseded that request lineage

What to verify

- undeclared continuation does not silently become redirect or callback truth
- equivalent continuation declarations lower identically
- diagnostics and request summaries preserve explicit continuation categories

Pass condition

The verification package must emit continuation descriptor digest, explicit
absence artifact, diagnostics digest, and replay digest.

12. The Deferred Processing Job Lifecycle Test

Purpose

Prove that deferred completion jobs live inside the same line lifecycle model
rather than escaping into polling folklore.

What to stress

- accepted versus processing versus ready results
- polling, callback, and webhook job postures
- visible value before and after processing completes
- processing plus refresh, invalidation, and branch restore
- duplicate completion signals
- stale completion after timeout or supersession
- branch-local processing completion after restore to a pre-completion snapshot

What to verify

- `processing()` tells one honest story with lifecycle and diagnostics
- undeclared families deny deferred-processing result artifacts
- equivalent deferred histories replay and restore identically

Pass condition

The verification package must emit processing posture digest, lifecycle digest,
diagnostics digest, denial artifact, and restore/replay digest.

13. The Signed Upload Prepare Transfer Finalize Parity Test

Purpose

Prove that upload transport and later processing complete through one line model
instead of separate hidden transport state.

What to stress

- direct multipart transport
- signed upload transport
- prepared, uploaded, awaiting-processing, and ready states
- upload plus deferred processing on the same family
- finalize failure, retry, timeout, and refresh interaction
- duplicate finalize callbacks
- finalize arriving after the upload lineage was superseded
- branch restore before finalize and after finalize

What to verify

- `upload()` and `processing()` never tell conflicting lifecycle stories
- undeclared families deny upload result artifacts
- prepare, transfer, finalize, and later processing remain visible as one line
  history

Pass condition

The verification package must emit upload posture digest, upload lifecycle
digest, processing digest, combined upload+processing digest, denial artifact,
and replay digest.

14. The Request Posture Compile-Time Boundary Test

Purpose

Prove that request/auth/context/continuation/processing/upload-only surfaces do
not leak onto declarations that have not proven them.

What to stress

- using posture-specific helpers on declarations without the required proof
- forging lowered request records outside the owning module
- widening through `shape | undefined` or similar maybe-present declaration
  paths

What to verify

- the compiler rejects posture access that is not definitely declared
- runtime admission rejects any remaining WORTHd or widened posture artifacts

Pass condition

No request-only declaration helper, lowered posture proof, or line posture
surface may be reachable without the corresponding declaration-bearing proof
path.

### Phase 3 Closeout Requirements

Required hostile dimensions

- request posture must be tested under declaration-time, param-derived, and
  replay-restored paths
- secret-bearing auth/context material must be checked under diagnostics,
  history, verification artifacts, and any package-boundary export posture that
  exists by this phase
- continuation, deferred completion, and upload posture must be exercised under
  duplicate, stale, superseded, and branch-restored completions
- combined upload-plus-processing families must be tested as first-class cases,
  not as a last-minute addendum

Required owning proof lanes

- one request/auth/context proof lane
- one continuation/deferred-completion proof lane
- one upload/deferred-processing combined proof lane
- one compile-time boundary proof lane for posture access and lowered proofs

Phase 3 pass condition

Phase 3 is not closed until request shaping, auth, continuation, deferred
completion, and upload posture can all lower into one canonical request and
lifecycle story without leaking secrets, minting hidden transport state, or
splitting upload and processing into separate truths.

## Phase 4: Partial Patch Reconciliation And Collection Scope Narrowing

15. The Narrow Patch Versus Broad Refresh Equivalence Test

Purpose

Prove that declared narrow patch reconciliation converges to the same committed
truth as broad replace or refresh while touching less declared semantic surface.

What to stress

- item patch, item-aspect patch, and summary patch
- equivalent histories using narrow patch versus broad replacement
- refresh after patch and patch after refresh
- invalidation plus patch plus replay
- paged collections with moving windows, off-page items, and the same logical
  item appearing across neighboring pages
- collections with duplicate candidate identities that should deny narrow patch
- page window shifts between patch admission and patch settlement

What to verify

- equivalent histories produce identical committed local value truth
- diagnostics and history distinguish narrow path from broad path honestly
- narrow patch never applies broader semantic change than declared
- paged narrow patching never corrupts page window truth, page-local summaries,
  or neighboring-page identity

Pass condition

The verification package must emit committed-value digest, narrow breadth
digest, broad-replace digest, paged-window digest, history digest, and
equivalence digest.

16. The Automatic Narrow Patch Admission Test

Purpose

Prove that declared reconciliation shape is enough to admit narrow patching
automatically without repetitive call-site folklore.

What to stress

- declared item keys
- declared aspect names
- declared summaries
- the same patch paths across collection and paged families
- patch attempts for off-page items, duplicated visible items, and page-summary
  updates that are only legal for the current page window
- declarations where item identity is unstable or only partially declared

What to verify

- line reconciliation summaries expose exactly what narrow paths are legal
- `line.patch(...)` admits narrow forms only when declaration truth proves them
- type and runtime surfaces agree on narrow legality
- paged families do not overclaim narrow legality for off-window or
  cross-window updates

Pass condition

The verification package must emit reconciliation descriptor digest, legal
narrow-path digest, paged-admission digest, compile-boundary digest, and
runtime admission digest.

17. The Broad Replace Honest Fallback Test

Purpose

Prove that the product falls back to explicit broad replace or typed denial when
declaration truth does not justify narrower scope.

What to stress

- undeclared item structure
- undeclared aspect structure
- maybe-present reconciliation declarations
- attempted narrow patch on detail families
- item identity collisions that make narrow admission semantically unsafe
- summary declarations without enough structure to prove page-local legality

What to verify

- the type surface does not overclaim narrow legality
- runtime patch admission denies illegal narrow forms before state changes
- broad replacement remains explicit and self-describing

Pass condition

The verification package must emit denial artifact, broad-replace digest,
compile-fail digest, and no-side-effect proof.

18. The Resource Reconciliation Versus Mutation Intent Boundary Test

Purpose

Prove that resource patch APIs remain read-side reconciliation surfaces rather
than becoming an accidental mutation engine.

What to stress

- patches that attempt to encode create/delete/write intent
- summary writers that try to smuggle item mutations
- patches that would require authority the line does not own
- reconciliation requests that would need cross-line or cross-family mutation
- patch sequences that would only be valid if the line were a command surface

What to verify

- reconciliation accepts only declared read-side convergence work
- mutation-shaped work denies explicitly
- diagnostics and history never mislabel broad write behavior as narrow patch

Pass condition

The verification package must emit denial-history digest, reconciliation digest,
mutation-boundary artifact, and no-side-effect proof.

19. The Summary Reconciliation Honesty Test

Purpose

Prove that summary-local patching remains narrower than whole-line replacement
and does not silently rewrite item truth.

What to stress

- declared summary patch that preserves items
- summary patch that attempts to mutate item membership
- summary patch that attempts to mutate item values
- summary patch plus refresh and invalidation
- summary patch across page-window shifts and duplicate delivery of the same
  summary change

What to verify

- summary-local writers preserve reconciled items exactly
- illegal summary writers deny without patch side effects
- diagnostics/history record summary scope distinctly from item or replace scope

Pass condition

The verification package must emit summary-scope digest, preservation proof,
denial artifact, and no-side-effect digest.

### Phase 4 Closeout Requirements

Required hostile dimensions

- collection and paged reconciliation must both be certified under moving
  windows, duplicated identities, unstable identity candidates, and mixed patch
  plus refresh histories
- every narrow patch form must be compared against an equivalent broad replace
  history for committed-truth parity
- every denial path must prove no side effects in value, diagnostics, and
  lifecycle history
- summary-local updates must be proven not to mutate item truth even under
  replay, duplicate delivery, and page-window movement

Required owning proof lanes

- one collection reconciliation equivalence proof lane
- one paged-window and off-page admission proof lane
- one mutation-boundary and no-side-effect denial proof lane
- one summary-local honesty proof lane

Phase 4 pass condition

Phase 4 is not closed until the package can prove that declared narrow patching
is truly narrower than broad replacement in both semantics and breadth, that
paged windows stay honest under movement and off-page pressure, and that the
resource surface has not accidentally become a mutation API in disguise.

## Phase 5: Resource Diagnostics, History, Branch, And Restore Surface

20. The Resource Diagnostics And History Honesty Test

Purpose

Prove that diagnostics and history explain one coherent line story across
initial admission, refresh, retry, timeout, supersession, invalidation, patch,
upload, and deferred completion.

What to stress

- mixed lifecycle histories with retry, timeout, supersession, and invalidation
- mixed patch plus refresh histories
- upload plus deferred processing histories
- retained-history truncation and explainability unavailability

What to verify

- history records explicit lifecycle transitions rather than implied ones
- diagnostics snapshots and history trails agree on the same semantic story
- retained-history limits produce named unavailability artifacts rather than
  silent absence

Pass condition

The verification package must emit lifecycle-history digest, diagnostics digest,
retention/unavailability artifact, patch-history digest, and explanation digest.

21. The Branch Restore Resource Parity Test

Purpose

Prove that branch, restore, replay, and exact-restore availability reconstruct
the same local resource truth and the same explanation truth.

What to stress

- branch fork before completion
- restore before and after fulfillment
- restore across invalidation, patch, and upload/deferred histories
- unavailable restore posture due to missing runtime capability or head snapshot

What to verify

- equivalent branch-local histories converge to identical local line truth
- exact-restore availability is explicit rather than guessed
- replay, lineage, branch, and restore artifacts stay semantically aligned

Pass condition

The verification package must emit branch digest, restore availability digest,
replay digest, lineage digest, exact-restore artifact, and incompatibility
artifact where relevant.

22. The Diagnostics Summary Cost Honesty Test

Purpose

Prove that `diagnosticsSummary()` is a cheap summary surface rather than a
hidden rich-history materializer.

What to stress

- repeated summary reads under retained and rich-history conditions
- runtime-truth harness instrumentation where replay or lineage materialization
  would throw or count if touched
- summary reads before and after branch/restore capability is available

What to verify

- diagnostics summary groups the right concepts for first-read DX
- summary reads availability directly without materializing replay or lineage
- summary cost envelope remains narrower than full history reads

Pass condition

The verification package must emit summary digest, explainability-availability
digest, replay-touch counter, lineage-touch counter, and boundary performance
envelope. The replay and lineage touch counters must remain zero.

## Phase 6: Binary Descriptor, Download, And Live-Delivery Surface

23. The Binary Descriptor Does Not Collapse Structured Truth Test

Purpose

Prove that file/media/export references remain typed structured truth and do not
collapse into byte-transport folklore.

What to stress

- resource values that reference files, exports, and media
- lines that carry both structured value truth and binary descriptors
- patch, refresh, and invalidation applied to descriptor-bearing lines
- branch restore and replay for descriptor-bearing histories

What to verify

- binary descriptors remain a distinct semantic category from visible structured
  value truth
- structured reconciliation can update descriptor-bearing lines without turning
  bytes into implicit line state
- diagnostics/history explain descriptor changes distinctly from structured
  value changes

Pass condition

The verification package must emit structured-value digest, binary descriptor
digest, reconciliation digest, diagnostics/history digest, and replay digest.

24. The Download Descriptor And Byte Transport Boundary Test

Purpose

Prove that download-facing APIs stay transport-aware without making the resource
line the byte-owner or session-owner.

What to stress

- download-ready versus not-yet-ready descriptor states
- signed download descriptors
- stale descriptor refresh
- branch restore and replay around download readiness changes

What to verify

- line surfaces expose download posture as descriptor truth, not hidden transfer
  state
- byte transport identity remains distinct from resource identity
- download readiness and incompatibility artifacts are self-describing

Pass condition

The verification package must emit download descriptor digest, readiness digest,
transport-boundary artifact, incompatibility artifact, and replay digest.

25. The Live Delivery And Local Refresh Convergence Test

Purpose

Prove that pushed structured delivery and local refresh converge on one line
model instead of two partially overlapping truth engines.

What to stress

- local refresh racing with delivered patch packets
- delivered invalidation packets
- delivered narrow patch versus delivered broad replace
- duplicate delivered packets
- out-of-order delivered packets
- delivered packet against stale basis
- branch-local delivery histories
- restore and replay after mixed delivery and local refresh

What to verify

- delivered updates, local refresh, and local invalidation converge to one
  committed truth
- diagnostics/history attribute visible changes to local refresh, delivery, or
  reconciliation honestly
- live delivery does not bypass declared reconciliation legality
- duplicate or stale delivery does not silently widen scope or regress local
  truth
- ordering-sensitive delivery either converges exactly or emits explicit basis
  incompatibility artifacts

Pass condition

The verification package must emit delivery provenance digest, committed-value
digest, reconciliation digest, basis-compatibility artifact, lifecycle digest,
and replay/restore digest.

## Phase 7: External Integration Compatibility Surface

26. The Signals First And External Resource Convergence Test

Purpose

Prove that the initial signals-first product surface and a later
externally-driven posture can converge on one local materialization model.

What to stress

- one declaration authored natively
- one semantically equivalent declaration authored through external-definition
  compatibility posture
- refresh, patch, delivery, branch, and restore histories across both paths

What to verify

- semantically equivalent native and external definitions lower to one line
  model
- local identity, lifecycle, freshness, diagnostics, and history digests match
- the external path does not mint a second cache or lifecycle authority

Pass condition

The verification package must emit native-definition digest, external-definition
digest, lowered-materialization digest, local-line digest, lifecycle digest,
and convergence digest.

27. The External Definition Compatibility Boundary Test

Purpose

Prove that externally-authored read definitions can be admitted only through a
typed compatibility boundary rather than ad hoc callback glue.

What to stress

- missing required external definition fields
- incompatible external definition versions
- unknown external reconciliation or request posture contracts
- restore from retained history when the external definition catalog changed

What to verify

- incompatible or incomplete external definitions deny before materialization
- compatibility is typed and self-describing
- restore/replay never reinterpret old external definitions silently

Pass condition

The verification package must emit external-definition digest, compatibility
artifact, incompatibility artifact, denial digest, and replay/restore digest.

28. The External Delivery Basis Refresh Compatibility Test

Purpose

Prove that externally-delivered patches, bases, and basis refresh use one
compatibility contract with explicit basis truth instead of transport folklore.

What to stress

- external patch against current basis
- external patch against stale basis
- external basis refresh
- mixed local refresh and external basis refresh
- branch restore after basis change

What to verify

- basis compatibility is explicit in line request/history/diagnostics surfaces
- stale external delivery denies or forces explicit basis refresh rather than
  silently applying
- local refresh and external basis refresh converge on one committed truth

Pass condition

The verification package must emit basis digest, basis-refresh digest,
compatibility artifact, denial artifact, committed-value digest, and
replay/restore digest.

## Certification Closeout Rule

The `worth-signals-wasm` API-surface milestone is not closed until:

- every named suite in this document has a real owning proof lane
- suite 0 exists as a real hostile end-to-end certification lane rather than a
  narrative aspiration
- compile-time boundary suites have maintained compile-fail artifacts
- cost-honesty suites expose named counters or equivalent mechanical proof
- replay/restore/branch suites emit canonical equivalence artifacts
- equivalent native, refresh-driven, patch-driven, delivery-driven, and later
  external-driven histories converge exactly when they mean the same thing
- incompatible policy, compatibility, basis, or retention states deny
  explicitly rather than drifting silently

The milestone fails certification if any later-friendly surface:

- requires a second local truth engine
- weakens line identity or lifecycle truth
- hides broad scope behind cheap API shape
- or can be explained only by reading internal implementation details instead of
  the canonical emitted artifacts
