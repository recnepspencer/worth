# Standalone Local Truth Branch Merge And Signal Projection Plan

> **Status:** Planned engineering spec
>
> **Roadmap parent:** [wasm_product_roadmap.md](./wasm_product_roadmap.md)
>
> **Supersedes:**
> [branch_merge_materialization_foundation_plan.md](./branch_merge_materialization_foundation_plan.md)
>
> **Predecessor milestone:**
> [concurrent_resource_effect_branch_dag_plan.md](./concurrent_resource_effect_branch_dag_plan.md)
>
> **Successor milestone:**
> [resource_mutation_response_reconciliation_plan.md](./resource_mutation_response_reconciliation_plan.md)

## Goal

Make manual aspect-level branch resolution honest in standalone
`worth-signals-wasm` without turning native `worth-signal` into an application
truth store.

The TypeScript product runtime owns one explicit, in-memory local truth
authority for browser-only application values. It plans and commits branch
merges. Native `worth-signal` owns derived execution, invalidation,
recomputation, speculative execution branches, lifecycle, and proof of the
work it performed after consuming a committed local-truth artifact.

The gear demo is a certification consumer. It does not classify conflicts,
compose values, or authorize commits.

## Why This Milestone Exists

Demo 5 proved that independent optimistic effects need effect-owned branches,
semantic dependency proof, arbitrary-order settlement, and a rebuildable
visible projection. It also exposed an authority ambiguity: the WASM product
needs a confirmed local application-value basis when no relational runtime is
present, but native Signal must remain a derived runtime over host state.

Manual merge resolution makes that ambiguity load-bearing. Putting value
history, aspect deltas, conflict decisions, or custom resolved application
values into `worth-signal` would violate Signal's role in the larger Rust
platform. Putting them in React would create UI-owned truth. This milestone
creates the missing TypeScript authority explicitly and connects it to Signal
through a one-way commit-to-derivation boundary.

## Governing Summaries

- `MENTALITY.md` protects adversarial-first foundation work. The plan must
  survive stale reviews, repeated partial merges, sibling and parent/child
  histories, and failed derived refresh before demo polish.
- `arch_laws.md` protects one canonical artifact, authority/derivation
  separation, proof-bearing phase transitions, decision logs, and lowered
  execution. Local truth commits once; Signal consumes that commit and never
  promotes a derived receipt back into truth authority.
- `composition_laws.md` protects named semantic steps. Truth declaration,
  commit admission, ancestry, delta extraction, conflict classification,
  review, lowering, publication, Signal projection, and presentation must not
  collapse into one coordinator file.
- `domain_structure_laws.md` protects physical authority topology. TypeScript
  local truth, speculative intent, Signal derivation, worker translation,
  diagnostics, and UI presentation require distinct modules and dependency
  directions.
- `perf_laws.md` protects semantic-delta-bounded work. Planning and projection
  must scale with ancestry, selected loci, conflicts, committed loci, and
  invalidated dependents rather than total truth size or graph size.
- `wasm_product_roadmap.md` protects sequencing. This work follows the
  concurrent effect branch DAG and precedes mutation-response reconciliation.
- `worth-query/docs/AI_README.md`, explicitly requested during design,
  protects the larger platform split: Query owns the ordinary public runtime
  workflow, Relational owns authoritative platform history and merge
  execution, the bridge carries committed change, and Signal evaluates derived
  work over host snapshots without owning truth storage.

## Adversarial Constraint

Generate in-memory histories containing 2 to 32 live local-truth branches,
1 to 128 entities, and 1 to 64 declared aspects. Branches may fork from parent
heads or retained checkpoints; edit overlapping and disjoint loci; form sibling
or parent/child relationships; merge selected aspects repeatedly; use mixed
source, target, and custom resolution choices; and advance either reviewed
head before execution. After any authoritative commit, Signal projection may
fail, restart, replay, or rebuild.

For every admitted history:

- the TypeScript truth layer publishes exactly one canonical commit or
  publishes nothing
- every review and executable plan is bound to the exact base, source head,
  target head, schema, scope, and policy
- disjoint aspects compose without erasure and unresolved or unselected loci
  remain unchanged
- repeated partial merges preserve per-locus integration lineage and do not
  manufacture false conflicts from already-integrated source changes
- a Signal projection failure never rolls back or corrupts committed local
  truth; it produces typed stale/rebuild-required derivation posture
- destroying all Signal branches, projections, caches, and diagnostics permits
  deterministic reconstruction from local truth alone
- worker-first and explicit main-thread compatibility converge on identical
  truth commits, decisions, derived values, and proof digests
- cost scales with visited ancestry plus changed, selected, conflicted,
  committed, and invalidated loci

Any implementation that stores authoritative value history in Rust Signal,
lets React compose the merged object, treats a Signal snapshot as local-truth
authority, infers aspects from arbitrary object differences, publishes a stale
review, or rolls truth back because derivation failed fails the milestone.

## Product Decision Lock

- Native `worth-signal` is never the authority for application values, value
  merge policy, changed-value history, manual choices, or custom resolutions.
- The standalone TypeScript truth layer is explicitly in-memory and
  process-local. It does not claim relational durability, MVCC parity,
  cross-process recovery, or platform Query authority.
- Platform Rust applications continue to use Query -> Relational -> Bridge ->
  Signal. This milestone does not add a parallel platform entry lane.
- One `LocalTruthCommit` is the canonical artifact for each admitted local
  mutation or merge. Branch heads, indexes, projections, histories,
  diagnostics, Signal transactions, and UI views derive from it.
- TypeScript truth commits precede Signal derivation. A derivation failure
  cannot invalidate an already-published truth commit.
- V1 admits declared top-level plain-object aspect fields. Nested paths,
  collections, identity migration, deletion topology, and arbitrary
  user-supplied merge functions remain typed unavailable until they have
  dedicated truth materializers.
- Mixed `AdoptSource`, `PreserveTarget`, and custom choices are legal.
- A custom value is authored on a local-truth resolution branch through the
  same declared aspect mutation lane used for ordinary edits. React cannot
  submit a precomposed target object.
- Signal branch merge may remain available for reconstructible derived
  execution state, but it cannot authorize or materialize application truth.
- A native proof may attest branch lifecycle, targeted execution, invalidation,
  recomputation, or derived reuse. It cannot be accepted where a
  `LocalTruthBasis` or `LocalTruthCommit` is required.
- Demo 5's confirmed local resource state is migrated onto or explicitly
  adapted to the same authority contract before Demo 6 ships.

## Phase Plan

### Phase 1: Freeze And Enforce The Authority Matrix

This phase makes the agreed ownership split executable before adding merge
behavior.

**Relevant subsystems**

- TypeScript product runtime authority topology
- native Signal facade and visibility boundaries
- worker-first deployment boundary
- Demo 5 resource-effect coordinator

**Relevant APIs**

- new `LocalTruthAuthority` product facade
- new `LocalTruthAuthorityKind`
- existing `WorkerBranchBasisReceipt`
- existing `SignalBranchBasisArtifact`

**Warnings**

- `canonical`, `truth`, `basis`, and `commit` names must identify which
  authority issued them; an unqualified `canonicalValueAuthority` is no longer
  acceptable across the product boundary.
- A TypeScript type alias around a native Signal basis does not create local
  truth authority.
- A convenience store inside resources, router, forms, or React would recreate
  the same ambiguity under a narrower name.

**Test requirements**

- Run a package-boundary residue audit that rejects imports or APIs allowing
  native Signal receipts, snapshots, branch heads, or digests to construct a
  `LocalTruthBasis`, `LocalTruthCommit`, or executable truth plan.
- Destroy every Signal branch and projection after several local commits and
  prove the local-truth branch heads and values remain complete and readable.
- Attempt to publish local truth from the main-thread UI while worker-first
  authority is active and prove the boundary denies it without worker mutation.

**Engineering decisions**

- Add a dedicated `local_truth` product-runtime subtree organized by authority,
  history, merge planning, publication, and Signal projection responsibilities.
- Keep native Rust changes limited to generic derived-execution capabilities
  required by the projection consumer; no Rust type may contain application
  value merge choices or a local truth journal.
- Replace Demo 5's coordinator-local authority shadow with a named adapter onto
  the local truth facade, or prove it is a read-only projection of that facade.
- Add hard-prohibition tests for `Signal* -> LocalTruth*` authority promotion.

**Open questions**

- None.

### Phase 2: Declare Local Truth Schemas And Aspect Materializers

This phase defines which application values the standalone authority can
understand and prevents merge logic from inferring semantic aspects from raw
JavaScript object differences.

**Relevant subsystems**

- TypeScript local truth declarations
- resource/controller authoring integration
- aspect-to-field materialization registry
- declaration diagnostics

**Relevant APIs**

- new `LocalTruthSchemaDeclaration<T>`
- new `LocalTruthAspectDeclaration<T, K>`
- new `DeclaredLocalTruthSchema<T>`
- new `LocalTruthAspectMaterializer<T>`
- new `LocalTruthSchemaIdentity`

**Warnings**

- An aspect id is semantic identity, not a dotted path guessed from an object.
- Two fields may not claim the same exact aspect locus unless the declaration
  provides one canonical composite materializer.
- Schema callbacks must not become arbitrary merge functions; they extract,
  validate, compare, and replace one declared locus.

**Test requirements**

- Declare a four-aspect gear value and prove extraction followed by
  materialization preserves all unselected fields byte-for-byte.
- Reject duplicate aspect ids, duplicate field ownership, missing comparator
  posture, non-plain-object V1 values, and materializers that mutate outside
  their declared field.
- Register equivalent schemas in different declaration orders and prove the
  same canonical schema identity and aspect order.

**Engineering decisions**

- V1 uses explicit top-level field declarations with canonical aspect order.
- Every aspect declaration carries extraction, value validation, equivalence,
  and replacement behavior plus an explicit cost class.
- Development and certification builds verify locality by comparing before and
  after values outside the declared field; production consumes the sealed
  declaration without repeating broad validation on every hot-path commit.
- Schema identity is part of every truth basis, commit, review, and executable
  merge plan.

**Open questions**

- None.

### Phase 3: Establish The Canonical Local Truth Commit Pipeline

This phase creates the one authoritative mutation artifact from which every
later branch, merge, Signal, history, and UI surface derives.

**Relevant subsystems**

- local truth mutation admission
- immutable snapshots and branch heads
- commit publication
- authority decision log and counters

**Relevant APIs**

- new `LocalTruthBasis`
- new `LocalTruthMutationRequest`
- new `ValidatedLocalTruthMutation`
- new `PlannedLocalTruthCommit`
- new `LocalTruthCommit`
- new `LocalTruthCommitOutcome`

**Warnings**

- `LocalTruthCommit` must not be independently reconstructed from a patch,
  digest, branch id, or Signal receipt.
- Diagnostics and history views derive from the commit; they are not sibling
  canonical records.
- Validation, planning, publication, and derivation scheduling must not occur
  inside one untyped `commit()` body.

**Test requirements**

- Inject failure at validation, planning, reconstruction, digesting, and
  publication boundaries; prove either one complete commit becomes visible or
  branch head, snapshot, journal, decision log, and indexes remain unchanged.
- Replay the same validated mutation against the same basis and prove stable
  commit identity; replay it after the head advances and prove typed stale-basis
  denial with zero mutation.
- WORTH a commit-shaped object or reuse a commit from another authority instance
  and prove it cannot advance a branch.

**Engineering decisions**

- The proof chain is `raw -> validated -> planned -> staged -> committed` with
  sealed construction at each transition.
- One immutable commit contains authority identity, schema identity, branch,
  parent commit, before/after snapshot identity, exact aspect operations,
  decision trace, integrity digest, and structural counters.
- Publication atomically installs the immutable snapshot, advances the branch
  head, appends the canonical commit, and updates derived indexes.
- Mutation outcomes preserve success, advisory, denied, unavailable, and failed
  postures instead of flattening them into promise rejection strings.

**Open questions**

- None.

### Phase 4: Build Local Truth Branch History And Per-Locus Lineage

This phase gives the TypeScript authority the history required for sibling,
parent/child, repeated partial, and resolution-branch merges without creating a
Rust aspect journal.

**Relevant subsystems**

- local truth branch catalog and ancestry
- checkpoint and bounded operation journal
- per-locus integration lineage
- retention and compaction

**Relevant APIs**

- new `LocalTruthBranchHandle`
- new `LocalTruthBranchForkRequest`
- new `LocalTruthBranchReceipt`
- new `LocalTruthCheckpoint`
- new `LocalTruthLocusLineage`
- new `LocalTruthHistorySegment`

**Warnings**

- A whole-branch merge parent is insufficient for partial merges because it
  falsely marks unselected source loci as integrated.
- Per-locus lineage belongs inside canonical merge commits and checkpoints; a
  mutable side map cannot become unreplayable authority.
- A branch display name, Signal branch id, or snapshot label is not ancestry
  proof.

**Test requirements**

- Generate sibling and parent/child trees, fork from retained checkpoints, and
  prove the same ancestry and values reconstruct from checkpoint plus bounded
  commit segment after all derived indexes are deleted.
- Merge aspect A from a source, later advance source aspects A and B, and prove
  the next plan treats only the post-integration change to A plus the unmerged B
  delta as new source work.
- Remove a required checkpoint, corrupt a segment, WORTH a parent link, or
  reference an authority-foreign branch and prove typed unavailable or denial
  before merge planning.

**Engineering decisions**

- Ordinary commits carry one parent; merge commits additionally carry exact
  per-locus source integration lineage.
- Derived ancestry, branch-child, head, and locus-lineage indexes are
  disposable and rebuildable from checkpoints plus canonical commits.
- Retention may compact history only by producing a checkpoint that preserves
  branch heads, exact values, ancestry reachability, per-locus integration
  lineage, and a digest over the compacted segment.
- History APIs expose typed positions and segment identities rather than
  parsing commit labels or rendered receipts.

**Open questions**

- None.

### Phase 5: Resolve Merge Bases And Extract Exact Aspect Deltas

This phase computes the comparison basis and candidate loci entirely from local
truth authority.

**Relevant subsystems**

- local truth ancestry queries
- per-locus integration lineage
- selected-node and selected-aspect scope
- sparse delta indexes and counters

**Relevant APIs**

- new `LocalTruthMergeScope`
- new `ResolvedLocalTruthMergeBasis`
- new `LocalTruthAspectDelta`
- new `LocalTruthDeltaProof`
- new `LocalTruthMergeBasisOutcome`

**Warnings**

- Source fork point alone is not the correct basis for every direction or after
  prior partial integration.
- One global base may establish structural ancestry while each locus has a
  later integrated source basis; both facts must remain visible.
- Delta extraction may not scan every entity or compare whole snapshots when
  exact commit operations and locus indexes already exist.

**Test requirements**

- Prove sibling, child-into-parent, and parent-into-child histories resolve the
  expected structural base and exact per-locus effective bases independently of
  the currently visible Signal branch.
- Generate repeated selected-aspect merges and prove extracted deltas contain
  exactly new source work, target work since its relevant basis, and no
  previously integrated loci.
- Deny unrelated branches, ambiguous retained ancestry, stale scope, missing
  lineage, duplicate loci, and forged bases before value comparison.
- Assert exact ancestry-node, commit-segment, entity, and aspect visit counters
  under sparse histories.

**Engineering decisions**

- V1 admits the unique nearest retained structural ancestor and exact
  commit-recorded per-locus integration bases.
- Multiple incomparable structural ancestors remain typed unavailable until a
  deliberate criss-cross merge policy exists.
- Scope is canonicalized once into exact entity/aspect keys before extraction.
- The basis artifact binds authority, schema, source head, target head,
  structural ancestor, per-locus bases, scope, and counters.

**Open questions**

- None.

### Phase 6: Classify Automatic Decisions And Reviewable Conflicts

This phase turns base/source/target local truth into independent, reviewable
aspect decisions without consulting native Signal values.

**Relevant subsystems**

- local truth merge policy registry
- aspect equivalence contracts
- conflict classification
- structural-unavailability classification

**Relevant APIs**

- new `LocalTruthMergePolicyDeclaration`
- new `LocalTruthMergeLocusClassification`
- new `LocalTruthConflictRecord`
- new `LocalTruthConflictAlternative`
- new `LocalTruthMergePreviewOutcome`

**Warnings**

- Native Signal merge records may explain derived graph compatibility, but they
  cannot classify application-value conflicts.
- Structural identity, deletion, schema, or topology conflicts must not be
  disguised as ordinary field alternatives.
- Display equality is not equivalence unless the declared aspect comparator
  proves it.

**Test requirements**

- On a four-aspect gear object, prove source-only, target-only, equivalent,
  unchanged, and overlapping changes classify independently against their
  exact effective bases.
- Prove selected-aspect scope emits no decision outside its admitted loci even
  when both branches changed other aspects.
- Remove comparator proof, change schema identity, or introduce unsupported
  structure and prove conservative conflict or typed unavailable posture rather
  than silent adoption.
- Shuffle declaration, commit, and candidate order and prove canonical conflict
  ids, alternatives, and preview digests remain identical.

**Engineering decisions**

- Classification variants are `Unchanged`, `AdoptSource`, `PreserveTarget`,
  `Equivalent`, `ResolutionRequired`, and `UnsupportedStructure`.
- Conflict ids bind authority, schema, effective basis, source/target heads,
  entity, aspect, values' evidence digests, and policy.
- Automatic policy resolution is distinct from manual review and records its
  exact selection basis.
- Preview produces executable intent when every locus is resolved, a review
  artifact when safe choices remain, or a typed outer denial/unavailability.

**Open questions**

- None.

### Phase 7: Issue Review Artifacts And Admit Manual Resolution

This phase makes human resolution a runtime-owned protocol rather than a modal
that edits objects.

**Relevant subsystems**

- local truth review registry
- conflict alternative admission
- resolution branch authoring
- attribution and decision evidence

**Relevant APIs**

- new `LocalTruthMergeReview`
- new `LocalTruthResolutionSelection`
- new `LocalTruthResolutionSubmission`
- new `LocalTruthResolutionBranchReceipt`
- new `AdmittedLocalTruthResolution`

**Warnings**

- Resolution-required is not a generic failure and must not be conveyed by
  parsing an exception string.
- React may submit conflict ids and runtime-issued alternative ids; it may not
  submit a merged target object.
- Custom resolution values require an ordinary schema-validated commit on a
  dedicated local-truth resolution branch.

**Test requirements**

- Resolve 1 to 64 conflicts with random source, target, and custom-resolution
  branch choices; prove every conflict is covered exactly once and no unrelated
  locus changes.
- Omit, duplicate, WORTH, cross-review, or add an extra selection and prove
  admission denies before staging.
- Author a custom value on the reviewed target basis, then advance the
  resolution branch or touch an out-of-review aspect and prove the receipt is
  stale or ineligible.
- Serialize a review through the worker boundary and prove the main thread
  cannot manufacture executable authority from the serialized view alone.

**Engineering decisions**

- Each review carries exact bases, scope, policy, conflicts, alternatives,
  schema, expiry posture, and a canonical digest.
- `AdoptSource` and `PreserveTarget` are runtime-issued alternatives.
- A custom alternative references a resolution-branch commit and exact aspect
  locus; raw custom JSON is not part of the resolution submission.
- Actor, reason, and correlation metadata remain explicitly host-asserted and
  cannot be confused with authenticated identity.

**Open questions**

- None.

### Phase 8: Lower One Executable Local Truth Merge Plan

This phase separates policy and review decisions from mutation by producing the
only artifact the truth executor accepts.

**Relevant subsystems**

- policy precedence and manual selection lowering
- target reconstruction plan
- per-locus lineage update plan
- stale-basis admission

**Relevant APIs**

- new `ResolvedLocalTruthMergeDecision`
- new `LoweredLocalTruthMergePlan`
- new `LocalTruthMergePlanCounters`
- new `LocalTruthExecutableMergeIntent`

**Warnings**

- The executor must not re-run equivalence, choose policy, inspect the UI
  submission, or rediscover scope.
- A target-preserved choice still needs decision evidence even when it emits no
  value write.
- A source-adopted or custom locus must update integration lineage even if its
  value compares equal at execution time.

**Test requirements**

- Mix schema defaults, request policy, automatic equivalence, source/target
  choices, and resolution-branch choices; prove one deterministic precedence
  and selection basis for every locus.
- Advance source, target, schema, policy registry, or resolution branch after
  preview and prove executable-intent admission denies before reconstruction.
- Shuffle all input maps and prove byte-equivalent lowered plan, decision trace,
  target operations, lineage updates, and counters.
- Submit an intent from another local truth authority instance and prove it
  cannot enter execution.

**Engineering decisions**

- The lowering chain consumes only validated bases, classified loci, and an
  admitted resolution artifact.
- The plan contains exact target aspect replacements, target-preserved loci,
  source integration lineage updates, expected heads, expected schema/policy
  identities, and boundedness counters.
- Runtime-issued sealing combines TypeScript's private construction surface
  with worker-authority registration so structural object forgery cannot execute.
- Planning is side-effect free and may be discarded without branch residue.

**Open questions**

- None.

### Phase 9: Publish One Atomic Local Truth Merge Commit

This phase executes the lowered plan entirely inside the TypeScript authority
and publishes one canonical merge commit.

**Relevant subsystems**

- target snapshot reconstruction
- authority-local staging
- commit publication and decision log
- branch retention and resolution-branch retirement

**Relevant APIs**

- new `StagedLocalTruthMerge`
- new `CommittedLocalTruthMerge`
- new `LocalTruthMergeCommitOutcome`
- new `LocalTruthMergeDecisionReceipt`

**Warnings**

- Native Signal mutation is not part of the authoritative commit transaction.
- Source branches are not destructively cleared after partial merge; canonical
  lineage records what was integrated.
- Dropping a staged merge must leave branch heads, snapshots, history, lineage,
  diagnostics, and Signal state untouched.

**Test requirements**

- Inject failure after each reconstruction step and immediately before
  publication; prove zero target, history, lineage, and decision-log mutation.
- Commit mixed decisions, then repeat partial merges after later source and
  target edits; prove exact values and per-locus integration bases without lost
  or falsely repeated work.
- Execute the same plan twice and prove one success followed by typed stale or
  duplicate posture, never a second commit.
- Retire a resolution branch only after its referenced value is captured by the
  merge commit and prove replay no longer depends on live branch state.

**Engineering decisions**

- Staging constructs the full immutable target snapshot, commit record,
  decision trace, lineage update, index delta, and counters before publication.
- Publication is one synchronous authority-local move with no awaited work.
- The merge commit records both structural ancestry and exact per-locus source
  integration lineage.
- Any post-publication diagnostics are derived from the canonical commit and
  may be rebuilt independently.

**Open questions**

- None.

### Phase 10: Project Committed Local Truth Into Native Signal Execution

This phase defines the one-way authority crossing from committed TypeScript
truth into rebuildable Rust Signal state.

**Relevant subsystems**

- local truth to Signal projection planner
- worker branch targeted transactions
- aspect-aware invalidation and recomputation
- stale derivation and rebuild lifecycle

**Relevant APIs**

- new `LocalTruthSignalProjectionPlan`
- new `LocalTruthSignalProjectionReceipt`
- new `LocalTruthDerivationPosture`
- existing `BranchTargetedTransactionRequest`
- existing `WorkerBranchBasisReceipt`

**Warnings**

- Projection consumes a committed truth artifact; it cannot alter, reject, or
  reinterpret that commit.
- Native Signal merge must not be called to decide application values. The
  target execution branch receives exact committed aspect values and recomputes
  downstream derived work.
- Signal success does not strengthen truth authority, and Signal failure does
  not weaken it.

**Test requirements**

- Apply one merge commit touching two of four aspects and prove the linked
  Signal execution branch receives only those aspect invalidations while
  downstream recomputation matches a clean rebuild from the committed snapshot.
- Inject targeted-transaction, evaluator, worker interruption, and delivery
  failure after truth publication; prove truth remains committed and derivation
  enters typed stale/rebuild-required posture.
- Delete all native branches and derived state, rebuild from local truth branch
  heads, and prove identical public derived values and proof digests.
- Attempt to pass a Signal projection receipt where a truth basis is required
  and prove mechanical denial.

**Engineering decisions**

- Every local truth branch has a derived `SignalBranchProjectionBinding` that
  may be destroyed and reacquired.
- Projection plans are derived only from canonical commit aspect operations and
  declared schema bindings.
- V1 prefers exact targeted transactions plus recomputation over native merge
  reuse. Derived-state merge optimization may be added later only with rebuild
  equivalence proof and no effect on truth outcomes.
- Projection outcomes distinguish `Current`, `CommittedDerivationPending`,
  `RebuildRequired`, `Unavailable`, and `Failed`.

**Open questions**

- None.

### Phase 11: Close Worker-First And Compatibility Product Protocols

This phase keeps local truth authority singular across deployment postures and
prevents the main thread from becoming a second store.

**Relevant subsystems**

- worker-owned local truth runtime
- main-thread bridge messages
- explicit main-thread compatibility runtime
- public merge and inspection facade

**Relevant APIs**

- new `signals.localTruth(...)` declaration entry
- new `localTruth.branch(...)` facade
- new `localTruth.previewMerge(...)`
- new `localTruth.resolveMerge(...)`
- new `localTruth.inspect(...)`

**Warnings**

- Serializable receipts are observations, not transferable authority tokens.
- Worker-first and compatibility modes may share semantic modules but not two
  simultaneously writable authority instances for one runtime.
- Public APIs must not expose mutable branch maps, commit registries, lineage
  indexes, or Signal projection bindings.

**Test requirements**

- Execute generated histories in worker-first and explicit main-thread
  compatibility modes and prove identical commit, review, decision, projection,
  replay, and counter digests.
- Replay, duplicate, reorder, truncate, and WORTH bridge messages; prove request
  identity, expected basis, and authority registration prevent duplicate or
  out-of-order mutation.
- Terminate the worker after truth commit but before projection delivery and
  prove restart reconstructs truth and derived Signal state without a second
  commit.

**Engineering decisions**

- Worker-first mode owns the TypeScript local truth authority beside the WASM
  runtime worker; the main thread receives immutable outcome views.
- Compatibility mode instantiates the same authority lifecycle in one process
  behind the same public facade.
- Commands use typed request identities and expected truth bases; observations
  use immutable envelopes with explicit non-authoritative posture.
- Bridge counters expose serialized breadth, commit count, projection count,
  rebuild count, and round-trip count.

**Open questions**

- None.

### Phase 12: Migrate Demo 5 Onto The Explicit Local Truth Boundary

This phase removes the authority ambiguity identified during architectural
review without changing Demo 5's concurrent optimistic behavior.

**Relevant subsystems**

- resource effect branch DAG
- confirmed resource-line state
- optimistic projection coordinator
- settlement and reconciliation policy

**Relevant APIs**

- replacement for coordinator-local `canonicalValueAuthority`
- local truth resource-line adapter
- revised resource reconciliation decision receipt
- existing effect and projection inspection surfaces

**Warnings**

- The migration must not reintroduce shared-snapshot rollback or React-side
  compensating patches.
- `serverRevisionThenAdmissionSequence` is a standalone resource reconciliation
  policy, not native Signal merge policy.
- Existing projection branches remain derived and must still be destroyable.

**Test requirements**

- Re-run the ten-request mixed success/reject/cancel certification and prove the
  same visible behavior with one explicit confirmed local truth authority and
  no coordinator-local shadow.
- Destroy and rebuild the optimistic projection after every settlement
  permutation; prove equivalence from confirmed local truth plus open effect
  envelopes.
- Scan product sources and docs for unqualified claims that Signal branches own
  server or application truth and fail certification on residue.

**Engineering decisions**

- Confirmed server observations become local truth commits before resource
  Signal projection.
- Effect envelopes remain canonical speculative intent artifacts; they do not
  become confirmed truth records.
- Binding values, canonical-value signals, native branches, and UI model strips
  become explicit projections of the local truth commit plus open effects.
- Keep the existing sibling/dependency DAG and retirement machinery; this phase
  corrects authority placement rather than replacing concurrency infrastructure.

**Open questions**

- None.

### Phase 13: Replace Demo 6 With The Gear Manual-Merge Certification

This phase proves the public capability through a premium, understandable gear
scenario while keeping all decisions in runtime-owned artifacts.

**Relevant subsystems**

- Worth Signals Demo 6 data and scenario controller
- local truth merge facade and worker bridge
- Signal-derived gear projections
- conflict review UI and code sample

**Relevant APIs**

- gear `LocalTruthSchemaDeclaration`
- sibling and parent/child branch scenario builders
- merge preview, review, resolution, commit, and derivation outcomes
- runtime inspection receipts rendered by the demo

**Warnings**

- Three.js or React may render committed and previewed gear state but may not
  calculate merged transforms, materials, ratios, or conflict classifications.
- The demo must distinguish confirmed local truth, speculative branches,
  resolution branch values, and derived Signal output visually and textually.
- A polished animation cannot substitute for stale-basis, failure, and rebuild
  proof.

**Test requirements**

- Show disjoint edits such as tooth count, material, rotation, and label across
  sibling branches; prove automatic composition preserves every independent
  aspect.
- Show a same-aspect conflict, choose source, target, and a custom resolution
  branch in separate runs, and prove the displayed result is read from the
  committed merge receipt rather than UI composition.
- Advance a branch while the review is open and prove the UI receives a clear
  stale-review outcome with no commit.
- Interrupt Signal projection after commit and prove the demo reports committed
  truth plus rebuilding derived state, then converges without recommitting.
- Run browser automation against worker-first and compatibility modes and
  compare displayed receipt identities to packaged runtime outcomes.

**Engineering decisions**

- The primary story uses siblings for independent work and a focused conflict;
  an advanced panel demonstrates parent/child lineage and repeated partial
  merge.
- Large explanatory copy is minimized. The UI shows branch ownership, changed
  aspects, conflict choices, committed result, and derivation posture directly.
- Code samples call the public local truth facade and never contain an object
  spread merge, field-selection compositor, or fallback patch.
- Demo source receives a no-shortcut residue test covering React, scenario data,
  bridge fixtures, and code samples.

**Open questions**

- None.

### Phase 14: Documentation, Certification, And Platform Boundary Closeout

This phase makes the capability safe to adopt and prevents standalone semantics
from being mistaken for the larger Rust platform.

**Relevant subsystems**

- feature documentation and API reference
- architecture and migration guidance
- hostile certification bundle
- package types, examples, and support posture

**Relevant APIs**

- local truth feature index entry
- branch merge and manual resolution guide
- standalone-versus-platform authority guide
- generated certification manifest

**Warnings**

- Documentation must never claim relational durability, restart-stable local
  history, authenticated actor identity, or platform Query admission.
- Rust Signal docs must describe only derived branch/execution capabilities;
  TypeScript truth docs own value history and manual merge semantics.
- Demo-only availability must not be advertised as package support.

**Test requirements**

- Execute documentation examples and compare their commit, decision,
  derivation, and denial outputs with packaged public types.
- Run hostile residue scans for Rust local truth journals, UI merge
  compositors, raw-id authority reconstruction, unqualified canonical-truth
  wording, and claims that Signal owns application values.
- Generate random branch histories with mixed resolutions, retention,
  interruption, and rebuild; certify replay equivalence and exact structural
  counters across deployment postures.
- Verify public type smoke tests cannot construct sealed truth, review,
  executable plan, or commit artifacts directly.

**Engineering decisions**

- Documentation presents two explicit diagrams: platform
  `Query -> Relational -> Bridge -> Signal` and standalone
  `TypeScript Local Truth -> Signal`.
- Support posture labels the local truth layer in-memory and process-local.
- Certification is organized by authority, merge semantics, derivation,
  worker parity, boundedness, interruption recovery, and UI no-shortcut proof.
- Closeout requires all public docs, types, package manifests, examples, and
  Demo 6 to agree on the same authority vocabulary.

**Open questions**

- None.

## Must Ship

- explicit TypeScript `LocalTruthAuthority` with declared schemas, immutable
  snapshots, canonical commits, branch heads, checkpoints, bounded history,
  and per-locus integration lineage
- sibling, parent/child, selected-aspect, repeated partial, and resolution-branch
  merge semantics
- typed preview, review, manual resolution, lowering, atomic commit, denial,
  unavailability, and decision-log artifacts
- one-way commit-to-Signal projection with exact aspect invalidation,
  recomputation, stale derivation posture, and deterministic rebuild
- worker-first and explicit compatibility-mode public protocol parity
- Demo 5 authority hardening without regression of concurrent effect behavior
- complete gear-based Demo 6 manual merge experience
- feature docs, platform-boundary guidance, public types, examples, hostile
  certification, and boundedness evidence

## Must Preserve

- native `worth-signal` as a deterministic derived-computation runtime over host
  state rather than an application truth store
- Query, Relational, and bridge authority in the larger Rust platform
- existing native branch lifecycle, targeted transaction, retirement,
  invalidation, recomputation, diagnostics, and replay capabilities
- Demo 5's effect-owned sibling/dependency branches, arbitrary-order closeout,
  and rebuildable optimistic projection
- one canonical artifact at each authority boundary and asymmetric derived
  projections downstream
- exact worker/main-thread support posture and explicit typed unavailability
- semantic-delta-bounded execution with visible counters

## Acceptance Evidence

The milestone closes only when one sealed certification bundle proves:

- no Rust production type, journal, branch receipt, or merge API owns local
  application-value history or manual resolution authority
- every local truth mutation and merge publishes exactly one canonical commit
  or leaves all authority state unchanged
- sibling, parent/child, same-aspect, disjoint-aspect, repeated partial, and
  custom-resolution histories converge under generated operation order
- stale source, target, schema, policy, review, and resolution-branch bases deny
  before publication
- per-locus lineage prevents already-integrated work from reappearing as new
  source change
- committed truth survives every injected Signal projection failure and all
  derived state rebuilds equivalently from truth alone
- worker-first and compatibility modes produce identical semantic and proof
  digests
- Demo 5 retains its certified ten-request behavior through the explicit local
  truth boundary
- Demo 6 renders only runtime-issued truth, conflict, decision, and derivation
  artifacts and contains no merge compositor
- exact counters prove bounded ancestry, delta, conflict, commit, projection,
  invalidation, and bridge breadth
- docs and package types state the standalone and platform authority boundaries
  without overclaiming durability or Signal truth ownership

## Sequencing Notes

- This milestone follows the concurrent resource effect branch DAG because it
  reuses explicit fork basis, targeted transactions, retirement, worker branch
  commands, effect dependency proof, and disposable projections.
- It supersedes the earlier materialization-foundation plan because that plan
  incorrectly placed application merge truth and aspect history in native
  Signal.
- It precedes mutation-response reconciliation because create, update, remove,
  identity migration, and multi-family write convergence need one honest local
  truth commit and derivation boundary before they can reconcile values.
- Generic aspect-capacity work remains a native Signal dependency for breadth,
  but it does not move local value authority into Rust.
- Durable or shared authoritative merge remains a Query/Relational platform
  capability and is not implemented by extending this local truth layer.
