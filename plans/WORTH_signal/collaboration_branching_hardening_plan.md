# WORTH Signal Collaboration Branching Hardening Plan

> **Status:** Planned engineering spec
>
> **Shared vocabulary prerequisite:** [../worth-foundational/milestone-9.md](../worth-foundational/milestone-9.md)
>
> **Downstream consumer:** Query collaboration-entry work depends on this
> substrate; this spec does not belong to Query and should be inserted into the
> next Signal-side collaboration hardening roadmap track before implementation.
>
> **Purpose:** harden the remaining `worth-signal` branch and merge substrate
> needed before Query can close its collaboration-facing witness,
> readmission/conflict-classification, and recovery/inspection seams without
> rebuilding host-local merge glue.

## Goal

Make `worth-signal` expose a collaboration-grade branching substrate that:

- preserves explicit branch and snapshot basis instead of ambient active-branch
  folklore
- admits scoped merge and cherry-pick as native proof-bearing merge semantics
  rather than caller-side record filtering
- lowers scoped merge request, admission, no-op, skipped, denial, and
  unavailable posture through the shared foundational vocabulary instead of a
  signal-local dialect
- retains enough branch / merge / strategy evidence that higher layers can
  classify collaborative posture without re-deriving signal merge meaning from
  raw host state
- leaves Query free to own the collaboration-facing witness and recovery story
  without forcing Query to become a second signal merge engine

## Why This Milestone Exists

Query `9.3.8` is closing a collaboration-facing platform-entry seam.

That seam needs lower-authority runtimes to already be honest about the branch
and merge posture they own.

`worth-signal` already has substantial branching support, replay, restore,
merge planning, and branch-local derived execution truth. The remaining gap is
not â€œadd branching.â€ The remaining gap is hardening the branch substrate so
later collaborative readmission, conflict classification, and recovery do not
depend on:

- ambient active-branch assumptions
- broad source adoption masquerading as partial merge
- caller-local cherry-pick filtering
- helper-local merge strategy folklore
- or raw branch/merge handles that do not retain enough proof for later
  strategy-aware inspection

This milestone therefore belongs before Queryâ€™s late collaboration phases and
before the sibling lower-authority hardening work in `worth-relational` and
`worth-runtime-bridge` is consumed by the same Query collaboration seam.

It should not be treated as a Query-owned roadmap item. The dependency
direction is the opposite: Queryâ€™s collaboration seam must wait for this Signal
substrate to close honestly.

## Governing Summaries

- `MENTALITY.md`: the hard problem is collaboration-safe branch truth under
  replay, restore, scoped merge, and later readmission pressure, not merely
  exposing a nicer merge helper.
- `arch_laws.md`: branch basis, scoped merge proof, strategy posture, and
  execution must remain phase-typed and authority-preserving; execution may not
  rediscover strategy or basis on the hot path.
- `composition_laws.md`: branch basis, fork admission, scoped merge request,
  candidate lowering, denial, strategy proof, and inspection support must stay
  as separate named responsibilities rather than one â€œcollaboration mergeâ€
  helper subsystem.
- `domain_structure_laws.md`: the tree must preserve the distinction between
  signal-owned branch/merge truth and Query-owned collaboration projection; the
  spec must not collapse them into a cross-crate pseudo-domain.
- `perf_laws.md`: scoped merge, replay, restore, and readmission support must
  remain bounded by selected scope, branch basis, and retained strategy width
  rather than whole-live-branch rediscovery.
- `worth-foundational/milestone-9.md`: selected scope meaning, no-op/skipped
  evidence, denial/unavailable posture, canonical basis participation, and
  locator participation must lower through one shared vocabulary before Query
  consumes collaboration-grade branch posture from Signal.
- downstream roadmap pressure: later collaboration-facing Query work needs
  lower-authority collaboration substrate from Signal before it can honestly
  freeze collaboration witness, readmission, classification, and recovery
  seams.

## Adversarial Constraint

Under branch fork, branch-local divergence, scoped cherry-pick, same-aspect and
disjoint-aspect merge pressure, snapshot restore, replay, repeated readmission,
strategy mismatch, and hostile caller attempts to reuse stale or incomplete
branch evidence, `worth-signal` must preserve the same admitted branch basis,
the same scoped merge candidate meaning, the same retained merge/strategy
posture, and the same typed denial or unavailable outcomes regardless of
whether higher layers approach the runtime through ordinary guided merge flows,
specialist proof-visible flows, or later collaborative readmission paths.

If any supported path:

- allows branch children to inherit the wrong parent basis by relying on the
  active branch implicitly
- treats cherry-pick as caller-side filtering after native planning
- widens scoped merge into broad source adoption when a narrower strategy is
  required
- hides signal merge or delivery strategy identity behind helper-local labels
- forces Query to rediscover signal merge posture from raw branch state or host
  glue
- or changes scoped merge / restore meaning between ordinary execution, replay,
  restore, and later readmission

then this milestone has failed.

## Product Decision Lock

- This milestone is a lower-authority collaboration-substrate hardening
  milestone, not a wasm milestone and not a Query product milestone.
- `worth-signal` remains authoritative for branch-local derived execution
  truth, branch basis, scoped merge candidate meaning, merge strategy posture,
  invalidation posture, and delivery strategy posture where Signal already owns
  them.
- Query will later own the collaboration-facing witness, readmission,
  classification, recovery, and operator-facing inspection surfaces over these
  lower-authority artifacts.
- Scoped merge and cherry-pick are native signal merge semantics. They are not
  caller-local filters, helper-local shortcuts, or UI-side selection folklore.
- Signal must lower scoped merge request, admitted scope, selected no-op,
  skipped scope, denial, and unavailable posture through the shared
  foundational Milestone 9 vocabulary. It may not export a parallel signal-only
  scoped-merge ontology for upper layers to learn separately.
- Branch fork basis must be explicit.
- Merge execution may consume only lowered scoped/basis/strategy proof. It may
  not rediscover branch meaning from ambient runtime state.
- Retained branch basis, scoped merge proof, strategy witness, and
  compatibility/readmission-preparation artifacts should use existing
  `worth-proof` progression and trust-boundary substrate wherever they cross
  from current runtime truth into retained, replayed, or readmitted posture.
- `TransitionOutcome` categories, `AuthorityWitness`, `FreshnessScopedBasis`,
  `BoundaryBridged`, `.bridge_trust_boundary()`, and
  `.readmit_with_authority(...)` are the default proof/readmission tools unless
  implementation discovers a concrete missing substrate.
- Where a phase-tagged retained carrier is needed, `worth-proof::Artifact` or a
  direct equivalent wrapper should be preferred over free bags of fields.
- Unsupported scoped merge or strategy combinations must fail typed and early;
  they must not silently widen into broad merge behavior.

## Expected Artifact Inventory

The implementation should converge on a concrete artifact family equivalent to:

- one explicit branch basis artifact
- one explicit branch fork request and fork receipt family
- one scoped merge request family lowered into `FoundationalMergeScope`
- one scoped merge proof family covering requested/admitted/skipped/no-op scope
- one scoped denial / unavailable family that maps onto existing
  `worth-proof::TransitionOutcome` categories instead of inventing a parallel
  top-level outcome law
- one strategy witness family covering merge, invalidation, and delivery
  posture where Signal already owns those semantics
- one compatibility/readmission-preparation witness family
- one inspection/support witness family

These names may change, but the artifact split should not.

## Explicitly Out Of Scope

This milestone does not include:

- Query-owned collaborative conflict classes
- full collaborative recovery grammar
- durable workflow continuation ownership
- product/UI merge execution flows
- widening unsupported topologies into admitted execution
- re-owning relational or runtime-bridge collaboration semantics inside Signal

## Phase Plan

### Phase 1: Explicit Branch Basis Artifact Boundary

Freeze one signal-owned branch basis artifact family that names the branch,
snapshot, restore, and head posture consumed by later fork, merge, replay, and
collaborative readmission work.

**Relevant subsystems**
- `worth-signal` history, branch, snapshot, restore, and replay surfaces
- `worth-signal` diagnostics and retained artifact surfaces
- Query `9.3.8` collaboration-entry seam as the downstream consumer

**Relevant APIs**
- branch, snapshot, restore, and replay specialist/runtime surfaces
- retained history and diagnostics artifact surfaces
- `worth-proof::Artifact`
- freshness/readmission carriers equivalent to
  `FreshnessScopedBasis<CurrentValidity, _>`

**Warnings**
- Do not let â€œcurrent active branchâ€ act as an unstated branch basis.
- Do not collapse branch identity, snapshot identity, and restore posture into
  one raw handle or string packet.

**Test requirements**
- adversarial basis equivalence test proving equivalent branch/snapshot posture
  yields the same canonical branch basis artifact across ordinary and replay
  lanes; assert identical branch-basis digest, identical snapshot/restore
  component digests, and identical current-basis/readmission posture for
  semantically equivalent inputs
- adversarial stale-or-cross-branch basis rejection test proving incomplete,
  stale, or cross-branch basis use fails typed before later fork or merge work;
  assert no branch mutation, no merge/fork side effects, and distinct typed
  rejection for stale basis versus cross-branch basis mismatch

**Engineering decisions**
- branch basis will be a first-class retained artifact, not a diagnostics-only
  reconstruction
- the artifact will always preserve separate branch, snapshot, and restore/head
  posture fields rather than one flattened branch token
- if branch basis crosses a trust boundary into retained/readmitted posture, it
  should do so through a phase-tagged proof-bearing carrier rather than an
  ad hoc struct bag

**Open questions**
- None.

### Phase 2: Explicit Branch Fork Basis Boundary

Make branch creation consume declared parent branch and optional snapshot basis
instead of inheriting fork meaning from whichever branch happens to be active.

**Relevant subsystems**
- `worth-signal` branch creation and branch switching surfaces
- `worth-signal` snapshot validation and restore surfaces
- retained branch-history artifact surfaces

**Relevant APIs**
- branch creation specialist/runtime surfaces
- snapshot lookup and restore-intent surfaces
- `worth-proof::TransitionOutcome`

**Warnings**
- Do not let branch fork semantics depend on the active branch implicitly.
- Do not mutate active-branch truth before fork-basis validation succeeds.

**Test requirements**
- adversarial explicit-fork-basis parity test proving repeated children created
  from the same parent/snapshot basis receive equivalent parent-basis proof;
  assert identical parent-basis digest, identical created-branch basis digest,
  and identical restored-active-branch proof across equivalent fork requests
- adversarial invalid-parent-or-snapshot denial test proving unknown, stale, or
  incompatible fork basis fails without mutating active branch posture; assert
  created-branch count remains unchanged, active-branch digest remains
  unchanged, and denial variants distinguish unknown parent, unknown snapshot,
  and incompatible snapshot lineage

**Engineering decisions**
- branch creation will admit a declared parent branch id and optional snapshot
  basis as the canonical fork request surface
- successful fork receipts will name parent basis, created branch basis, and
  restored active-branch posture explicitly

**Open questions**
- None.

### Phase 3: Scoped Merge Request Boundary

Define the signal-owned scoped merge request family for full-branch, selected
node, and selected aspect cherry-pick semantics.

**Relevant subsystems**
- `worth-signal` guided merge entry surfaces
- `worth-signal` merge planner request normalization
- shared collaboration-facing branch substrate consumed later by Query

**Relevant APIs**
- guided merge builder / planner surfaces
- raw merge request / policy surfaces where specialist lowering already exists
- `worth-foundational::FoundationalMergeScope`
- `worth-foundational::FoundationalMergeScopeFamily`

**Warnings**
- Do not treat cherry-pick as a presentation-only selection overlay.
- Do not accept empty, ambiguous, or helper-WORTHd scoped merge requests as if
  they were ordinary full-branch merges.

**Test requirements**
- adversarial scoped-request normalization parity test proving equivalent
  selected-node and selected-aspect requests normalize to the same native scope
  declaration across builder and proof-visible entry lanes; assert identical
  normalized scope digest, identical canonical selected-locus ordering, and
  identical full-branch/default-vs-explicit-full-branch meaning where the
  request is semantically the same
- adversarial malformed-scope rejection test proving empty, ambiguous, or
  structurally invalid selected scope fails typed before planning; assert no
  merge candidate planning occurs and rejection remains distinct for empty
  selection, malformed locus, and ambiguous selected-aspect request shapes

**Engineering decisions**
- scoped merge request vocabulary will be native signal request surface, not a
  caller-local wrapper over later result filtering
- full-branch merge remains the default only when no scoped request is declared

**Open questions**
- None.

### Phase 4: Foundational Scoped Vocabulary Lowering Boundary

Lower native scoped merge request meaning into the shared foundational Milestone
9 vocabulary before candidate planning or later collaborative consumers depend
on it.

**Relevant subsystems**
- `worth-signal` merge request and planner entry surfaces
- `worth-foundational` scoped merge request/admission vocabulary
- proof-bearing branch/merge boundary artifact surfaces

**Relevant APIs**
- native signal scoped merge request surfaces
- foundational scoped merge request and outcome vocabulary
- `worth-foundational::FoundationalMergeScope`
- `worth-foundational::FoundationalMergeCandidate::with_scope(...)`
- `worth-foundational::FoundationalMergeAdmissionOutcome`

**Warnings**
- Do not let Signal publish a parallel scoped-merge dialect above the native
  planner.
- Do not defer foundational lowering until wasm, Query, or diagnostics layers.

**Test requirements**
- adversarial cross-construction lowering parity test proving equivalent native
  full-branch, selected-node, and selected-aspect requests lower to the same
  foundational scope request meaning; assert identical foundational scope
  family, identical selected-locus digest, and identical producer-ordering
  canonicalization across independent construction paths
- adversarial no-parallel-ontology boundary test proving upper layers cannot
  observe scoped merge request meaning except through the foundational scope
  vocabulary or guided facade equivalents; assert helper-local request wrappers
  cannot bypass foundational lowering and compile-fail or typed-boundary tests
  reject raw caller-owned scoped-merge packets

**Engineering decisions**
- native scoped merge request shapes remain signal-owned, but their
  collaboration-facing boundary meaning lowers immediately into foundational
  scoped merge vocabulary
- foundational lowering is required before candidate lowering, denial
  localization, or retained proof publication can be considered stable

**Open questions**
- None.

### Phase 5: Scoped Candidate Lowering Boundary

Apply selected scope before merge policy, identity correspondence, invalidation
planning, and aspect-decision planning so scoped merge changes candidate truth
upstream of execution.

**Relevant subsystems**
- `worth-signal` merge planning and candidate construction
- branch-local mutation journal / candidate discovery surfaces
- merge policy and aspect-decision planning surfaces

**Relevant APIs**
- merge planner and candidate-construction surfaces
- merge policy / aspect decision planning surfaces
- `worth-foundational::FoundationalAdmittedMergeScopeEvidence` or direct
  equivalent lowered output family

**Warnings**
- Do not filter result records after planning and call that scoped merge.
- Do not let scoped merge widen candidate discovery into whole-live-branch
  rediscovery when branch-local mutation scope is already known.

**Test requirements**
- adversarial selected-scope candidate parity test proving selected-node and
  selected-aspect scope change the planned candidate set before policy
  resolution; assert planned candidate digests differ from full-branch plans
  only by selected scope, and assert requested/admitted/skipped/no-op widths
  reflect selected scope before policy or aspect-decision summaries are built
- adversarial whole-live-scan denial test proving scoped candidate construction
  remains bounded by branch-local mutation scope rather than ambient branch
  rediscovery; assert branch-local mutation-scope counters stay exact, whole
  live-branch scan counters remain zero or deny explicitly, and convenience
  index variation does not change candidate meaning

**Engineering decisions**
- selected scope will participate in candidate construction as an upstream
  planning fact
- planned candidate sets will distinguish requested, admitted, skipped, and
  no-op scope members explicitly

**Open questions**
- None.

### Phase 6: Scoped Merge Denial And Unavailable Boundary

Make unsupported or unsafe scoped merge posture fail through typed denial or
unavailable outcomes instead of collapsing into generic merge failure or broad
fallback.

**Relevant subsystems**
- `worth-signal` merge denial and rejection surfaces
- candidate admissibility and identity correspondence surfaces
- diagnostics and retained explanation artifacts

**Relevant APIs**
- merge planner denial/report surfaces
- merge policy / candidate admissibility surfaces
- `worth-proof::TransitionOutcome`
- `worth-foundational::FoundationalMergeAdmissionOutcome`

**Warnings**
- Do not hide unknown selected nodes, ambiguous correspondence, or non-adoptable
  selections inside generic merge failure.
- Do not degrade unsupported scoped strategy families into broad merge.

**Test requirements**
- adversarial scoped-denial localization test proving unknown selected node,
  deleted target, ambiguous correspondence, and unknown aspect remain distinct
  typed failures; assert each denial emits a distinct denial kind digest and
  selected locus digest rather than collapsing into generic merge failure
- adversarial no-side-effect unavailable test proving unsupported scoped merge
  posture fails before branch mutation, delivery, or history drift; assert no
  branch-state digest change, no delivery/invalidation side effects, and
  unavailable posture remains distinct from invalid-scope denial and selected
  no-op outcomes

**Engineering decisions**
- scoped denial and scoped unavailable remain separate outcome families
- denial receipts will preserve enough localized scope evidence for later Query
  readmission/classification work without reopening host glue
- denial and unavailable posture must map honestly onto the existing proof
  progression families rather than inventing a parallel top-level outcome shape
- `Denied`, `Deferred`, `Stale`, `RebindRequired`, and `Failed` should remain
  the governing non-success categories; scope-specific meaning belongs inside
  Signal/Foundational domain posture, not in a new top-level proof taxonomy

**Open questions**
- None.

### Phase 7: Scoped Merge Plan And Result Proof Boundary

Carry scoped merge truth through preview, execution, replay, and retained
history so later collaboration layers consume proof rather than re-deriving
merge posture.

**Relevant subsystems**
- `worth-signal` merge preview and merge execution surfaces
- retained history, replay, and diagnostics artifact surfaces
- branch restore and replay parity surfaces

**Relevant APIs**
- merge planning/execution specialist surfaces
- replay/history/digest artifact surfaces
- `worth-foundational::FoundationalMergeAdmissionOutcome`
- `worth-proof::Artifact`

**Warnings**
- Do not emit scoped merge execution results without retained scope proof.
- Do not let replay reconstruct scoped merge semantics from ambient branch state
  alone.

**Test requirements**
- adversarial scoped-merge replay parity test proving preview, execution, and
  replay preserve the same scope declaration, admitted scope, skipped scope,
  and no-op posture; assert identical scope declaration digest, admitted-scope
  digest, skipped-scope digest, no-op digest, and scope breadth counters across
  original execution and replay lanes
- adversarial restore-after-merge boundary test proving restore and repeated
  merge preserve bounded scoped-merge validity rather than widening into
  ambient branch discovery; assert restore reconstructs the same scoped merge
  proof packet, repeated merge stays bounded by retained branch-local scope, and
  no ambient branch rediscovery path changes admitted scope meaning

**Engineering decisions**
- plan/result artifacts will carry scope declaration, admitted/skipped/no-op
  posture, and bounded counters as first-class proof
- replay parity over scoped merge proof is mandatory before later collaboration
  layers may treat the substrate as stable

**Open questions**
- None.

### Phase 8: Canonical Basis, Locator, And Diagnostic Participation Boundary

Make branch basis and scoped merge artifacts participate in canonical basis,
locator, and compact diagnostic/explanation surfaces so later collaboration
layers do not have to bolt those on retroactively.

**Relevant subsystems**
- retained branch basis artifacts
- retained scoped merge request/admission/denial/unavailable artifacts
- canonical basis and locator participation surfaces
- diagnostics and explanation attachment surfaces

**Relevant APIs**
- retained history and explanation/digest surfaces
- foundational canonical basis / locator-facing lowering surfaces where Signal
  already participates in branch/merge proof
- `worth-foundational::FoundationalMergeScopeLocator`
- foundational canonical-scope lowering surfaces such as the existing
  `transitions::basis::canonical_scope` participation layer

**Warnings**
- Do not leave selected scope, skipped scope, no-op scope, or denial posture as
  proof that can only be understood from runtime-local structs.
- Do not make later Query or bridge layers invent ad hoc locators or digest
  participation for scoped merge artifacts.

**Test requirements**
- adversarial canonical-basis ordering parity test proving scoped request,
  admitted scope, skipped scope, no-op scope, denial, and unavailable posture
  produce stable canonical basis entries across independent construction paths;
  assert canonical-basis digests match across producer-order variation and that
  selected-node, selected-aspect, skipped, no-op, denial, and unavailable
  entries remain separately addressable in the basis output
- adversarial locator-and-diagnostic boundary test proving selected-node,
  selected-aspect, and denial loci remain distinct and explanation surfaces
  consume retained scoped artifacts rather than helper-local reconstruction;
  assert locator digests differ by locus family, diagnostic rows round-trip from
  retained artifacts, and compact explanation never requires runtime-local state
  to identify selected versus skipped versus denied posture

**Engineering decisions**
- branch basis and scoped merge artifacts will be digestible, locatable, and
  explanation-friendly as part of first-ship collaboration hardening rather
  than later documentation polish
- compact-by-default skipped/no-op/denial explanation is sufficient, but the
  compact surface must still be canonical and replay-honest

**Open questions**
- None.

### Phase 9: Signal Merge Strategy Identity Boundary

Freeze the signal-owned merge, invalidation, and delivery strategy identity
surface that collaboration-aware upper layers will later consume through Query.

**Relevant subsystems**
- `worth-signal` merge strategy surfaces
- invalidation and delivery strategy surfaces where branch merge meaning changes
  downstream behavior
- retained diagnostics / strategy inventory surfaces

**Relevant APIs**
- merge policy and reconciliation strategy surfaces
- invalidation and delivery strategy descriptors where specialist/runtime
  surfaces already expose them
- `worth-proof::Artifact`

**Warnings**
- Do not hide strategy identity behind helper-local labels or ad hoc enums.
- Do not collapse merge strategy posture and delivery/invalidation posture into
  one generic â€œsignal strategyâ€ bag if the runtime reasons about them
  differently.

**Test requirements**
- adversarial strategy-identity equivalence test proving equivalent admitted
  strategy posture reached through ordinary and specialist merge lanes yields
  the same retained strategy witness; assert identical merge-strategy digest,
  invalidation-strategy digest, and delivery-strategy digest where the runtime
  meaning is equivalent
- adversarial helper-WORTHd-strategy rejection test proving incomplete or
  synthetic strategy posture cannot cross as admitted signal strategy evidence;
  assert typed rejection for WORTHd or incomplete strategy packets and assert
  no retained admitted strategy witness is published on failure

**Engineering decisions**
- strategy identity will be retained as proof-bearing signal-owned artifacts,
  not rediscovered from raw merge plans or branch state
- merge, invalidation, and delivery strategy posture may compose, but they will
  remain distinguishable where later collaboration classification needs the
  difference

**Open questions**
- None.

### Phase 10: Branch Compatibility And Readmission Preparation Boundary

Provide the signal-owned compatibility/readmission preparation seam that later
Query collaboration phases can consume without Query re-owning signal merge
meaning.

**Relevant subsystems**
- retained branch basis artifacts
- retained scoped merge proof artifacts
- retained strategy witness surfaces
- inspection and compatibility-report surfaces

**Relevant APIs**
- branch/history/merge specialist surfaces
- diagnostics or proof-visible compatibility/report surfaces that can project
  retained branch posture honestly
- `worth-proof::AuthorityWitness`
- `worth-proof::BoundaryBridged`
- `.bridge_trust_boundary()`
- `.readmit_with_authority(...)`

**Warnings**
- Do not make Query rediscover compatibility by spelunking raw branch handles,
  merge receipts, and strategy labels separately.
- Do not let this phase become Query-style collaborative conflict
  classification; Signal should expose lower-authority compatibility facts, not
  own the later collaboration grammar.

**Test requirements**
- adversarial readmission-preparation parity test proving equivalent retained
  branch basis, scoped merge proof, and strategy posture yield the same
  compatibility witness across ordinary and specialist signal lanes; assert
  identical compatibility digest, identical current-basis/readmission posture,
  and identical lower-authority fact inventory across equivalent lanes
- adversarial stale-or-mismatched-preparation denial test proving missing,
  incomplete, or cross-basis retained posture fails typed rather than widening
  into ambient compatibility assumptions; assert denial remains distinct for
  stale basis, missing retained proof, and strategy mismatch, and assert no
  ambient compatibility fallback witness is produced

**Engineering decisions**
- Signal will expose a lower-authority compatibility/readmission-preparation
  witness, not a full collaboration conflict taxonomy
- the witness will preserve enough branch/merge/strategy facts for Query to
  classify replayable vs stale vs rebind/inspection needs later without
  reopening signal merge semantics from scratch
- retained branch and scoped merge artifacts must cross this seam through the
  existing current-basis / readmission posture rather than ad hoc revalidation
  flags or helper-local freshness folklore
- if implementation discovers that current `worth-proof` readmission substrate
  cannot honestly carry one of these retained postures, the missing proof
  substrate should be added there first rather than patched over in Signal

**Open questions**
- None.

### Phase 11: Inspection And Support Boundary

Close the collaboration hardening substrate with one inspection/support seam so
higher layers and operators can see what branch/merge/strategy posture Signal
actually admitted.

**Relevant subsystems**
- `worth-signal` diagnostics and history inspection surfaces
- retained branch basis, scoped merge proof, and strategy witness inventories
- support/readiness style surfaces consumed later by Query

**Relevant APIs**
- diagnostics and history specialist/runtime surfaces
- retained artifact / inventory export surfaces
- `worth-proof::ArtifactView` / `ArtifactParts` or direct equivalents where
  inspection must remain proof-bearing without exposing minting internals

**Warnings**
- Do not leave collaboration-critical branch posture visible only through test
  helpers or debug-only logs.
- Do not let inspection surfaces synthesize strategy or basis posture that the
  runtime did not actually admit.

**Test requirements**
- adversarial inspection-surface parity test proving equivalent admitted
  branch/merge/strategy posture projects to the same inspection/support witness
  across equivalent signal lanes; assert identical inspection digest, identical
  summarized branch-basis/scope/strategy rows, and identical support/readiness
  posture across equivalent retained inputs
- adversarial no-shortcut inspection test proving inspection cannot manufacture
  branch or strategy posture absent from retained signal artifacts; assert
  missing retained proof yields explicit absence or typed failure, never
  synthesized branch basis, scoped merge, or strategy rows

**Engineering decisions**
- inspection/support output will summarize retained branch basis, scoped merge
  proof, and strategy posture without flattening them into one opaque blob
- this inspection seam exists to support later Query collaboration work, not to
  replace lower-authority merge execution or replay APIs

**Open questions**
- None.

## Must Ship

- explicit branch basis artifacts
- explicit fork-from-basis branch creation
- native scoped merge / cherry-pick request vocabulary
- foundational scoped merge vocabulary lowering
- scoped candidate lowering before merge policy and execution
- typed scoped denial and unavailable posture
- retained scoped merge proof in preview, execution, replay, and history
- canonical basis, locator, and compact diagnostic participation for scoped
  branch/merge artifacts
- proof-bearing signal merge / invalidation / delivery strategy identity
- branch compatibility/readmission-preparation witness for later collaboration
  consumers
- inspection/support surfaces for retained collaboration-critical branch
  posture

## Must Preserve

- `worth-signal` remains the authority for derived execution branch truth and
  signal-side merge meaning
- Query remains the future owner of the collaboration-facing witness,
  classification, and recovery grammar
- scoped merge stays native runtime semantics, never caller-local record
  filtering
- foundational Milestone 9 remains the shared scoped merge language seen by
  upper layers rather than a parallel signal-local ontology
- branch, restore, replay, and merge parity remain mandatory
- bounded branch-local candidate discovery and retained-proof replay honesty
  remain architectural requirements, not optimization hints

## Acceptance Evidence

This milestone is complete only when `worth-signal` can prove:

- equivalent explicit branch basis inputs yield equivalent retained branch basis
  artifacts
- explicit fork basis prevents accidental active-branch inheritance
- selected-node and selected-aspect merge scope change candidate meaning before
  execution rather than after the fact
- scoped merge request, admission, skipped/no-op posture, denial, and
  unavailable posture lower through stable foundational vocabulary, canonical
  basis entries, and distinct locators
- unsupported or unsafe scoped merge posture fails typed and without side
  effects
- replay and restore preserve scoped merge proof and branch basis honesty
- signal strategy identity remains proof-bearing and helper-forging resistant
- later collaboration layers can consume one signal-owned compatibility witness
  instead of rediscovering branch/merge/strategy posture from raw host glue
- inspection/support surfaces expose admitted branch posture honestly and
  without synthetic shortcuts

## Sequencing Notes

This milestone belongs before Query `9.3.8` Phase 39 through 41 collaboration
closure because those phases need lower-authority branch, merge, and strategy
posture to already be hardened.

It also belongs alongside later sibling hardening specs in `worth-relational`
and `worth-runtime-bridge`, because Queryâ€™s collaboration-facing witness,
readmission, classification, and recovery seams must compose lower-authority
facts from all three runtimes without reopening local host glue.

Before implementation starts, this milestone should gain a Signal-side roadmap
home. The temporal/async roadmap is not the right parent for it.
