# Milestone 10 Engineering Spec: Retention, Compaction, And Reclamation

> **Status:** Draft
>
> **Roadmap parent:** [worth_store_roadmap.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_roadmap.md)
>
> **Vision parent:** [worth_store_vision.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/worth_store_vision.md)
>
> **Test requirements:** [test-requirements.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/test-requirements.md)
>
> **Prerequisite milestones:**
> - [milestone-4.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-4.md)
> - [milestone-5.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-5.md)
> - [milestone-5-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-5-closeout.md)
> - [milestone-6.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-6.md)
> - [milestone-6-closeout.md](/Users/Esther/Documents/Programming/worth_workspace/worth/plans/worth-store/milestone-6-closeout.md)
>
> **Concurrent milestones:**
> - `Milestone 8` (`Live-Query Substrate And Durable Sync Basis`)
>
> **Impacted later milestones:**
> - `Milestone 11` (`Background Maintenance Isolation And Scheduling Contracts`)
> - `Milestone 13` (`Tiering And Durable Working-Set Intelligence`)
> - `Milestone 14` (`Replication, Capsules, And Integrity Verification`)
> - `Milestone 20` (`Native Blob And Object Storage`)
>
> **Primary architectural driver:** make retention policy physically executable
> across snapshots, branch-delta layers, and Milestone 6 derived layout/block
> families without allowing compaction products, reclaim queues, or maintenance
> bookkeeping to become shadow authority

## Goal

Make retention policy, compaction, and reclaim first-class store programs that
rewrite or delete only what policy explicitly permits while preserving one exact
replayable retained truth lane from canonical authority.

## Why This Milestone Exists

Milestone 10 is not "add garbage collection later."

It is the milestone that decides whether `worth-store` can age, compact, and
reclaim durable state without quietly moving authority from canonical commits
into maintenance products, backend-local rewrite residue, or operator folklore.

Milestone 4 locked snapshots as derived restore substrates with explicit basis.

Milestone 5 locked branch-delta layering, rewrite lineage, and replay-parity
reads over the physical branch program.

Milestone 6 locked aspect-layout slices, structural blocks, and chunk models as
derived durable families with rebuild and control-lane parity.

Milestone 10 must now answer the next hard storage question:

- what retention policy is allowed to mean over canonical authority,
  branch-local history, snapshots, and Milestone 6 derived families
- what a compaction product is allowed to rewrite, summarize, or collapse
  without becoming a second history source
- when reclaim is allowed to physically delete snapshots, delta layers,
  structural blocks, or support artifacts
- how rebuild debt is tracked when policy intentionally permits reclaim of
  rebuildable derived families before they are needed again
- how concurrent live-query work in Milestone 8 continues to reason from stable
  basis and durable cursor surfaces while compaction and reclaim proceed

If this milestone is weak, later milestones inherit a false foundation:

- tiering will move data whose retention meaning is still ambiguous
- replication will ship compacted products that may not reconstruct the same
  retained truth as the original store
- blob retention will copy policy language that was never made exact for the
  structural store first
- background maintenance will schedule debt whose semantic safety is still
  undefined
- live-query continuation will race compaction and reclaim without a declared
  stable-basis contract

This milestone exists to make retention physically real before later platform
programs start assuming "of course old data can be compacted safely" means
anything mechanically precise.

## Hard Part

The hard part is not deleting old files.

The hard part is keeping one honest separation among six things naive storage
systems routinely collapse:

- retained authoritative truth that policy still promises must replay exactly
- authoritative truth that policy explicitly allows to disappear
- derived durable artifacts that may be rebuilt later from retained authority
- compaction products that summarize physical layout but never outrank the
  retained canonical basis they were derived from
- reclaim bookkeeping that proves deletion eligibility without becoming a hidden
  truth ledger
- live-query basis and cursor surfaces that continue to observe stable retained
  truth while Milestone 8 is developed concurrently

The design fails if:

- compaction output becomes the cheapest or only way to answer retained-history
  reads, turning a derived artifact into shadow authority
- retention policies name broad ideas like "old branches" without exact branch,
  frontier, artifact-family, and retained-range vocabulary
- reclaim deletes derived artifacts whose declared rebuild basis was already
  compacted away
- branch-delta rewrite, snapshot thinning, and structural-block reclaim each
  use different eligibility rules and drift away from one policy model
- live-query continuation has to infer whether a basis is still valid from
  backend-local maintenance residue instead of typed basis-survival rules

Milestone 10 therefore has to define one retention authority model, one
compaction-product non-authority rule, and one reclaim eligibility rule that
all later maintenance work inherits.

## Explicit Assumptions

- Milestone 1 authoritative artifact families remain the only semantic durable
  truth authority.
- Milestone 4 snapshots remain derived, basis-explicit restore artifacts and
  may be retained, compacted, or reclaimed only under explicit policy.
- Milestone 5 branch-delta layers, rewrite lineage, and authority replay
  control remain the retained-history substrate for branch-local truth.
- Milestone 6 aspect-layout slices, structural blocks, chunk models, and layout
  support lanes remain derived durable families with rebuildability and explicit
  non-authority boundaries.
- Milestone 8 is being built concurrently and owns stable-basis reads, durable
  cursor continuation, and basis-mismatch semantics; Milestone 10 must preserve
  those surfaces rather than redefining live-query meaning.
- `worth-relational` still owns commit, branch, lineage, schema, and replay
  semantics; retention may remove policy-expired authority, but it may not
  reinterpret surviving authority.
- authoritative artifact deletion is legal only when retention policy says the
  affected history is no longer promised as replayable truth.
- compaction products may accelerate retained-history reads or restore, but they
  must always remain derivable from the retained authoritative basis they claim
  to summarize.
- rebuild debt is an honest operational surface, not a hidden implementation
  detail; reclaiming a rebuildable family without publishing rebuild debt is a
  design defect.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects is solving the hostile retention problem
  before "cleanup later" becomes the real architecture. Milestone 10 therefore
  starts from accidental shadow authority and deletion ambiguity, not from disk
  pressure convenience.
- `arch_laws.md`
  The most important thing it protects here is authority-versus-derivation
  separation under proof-bearing phases. Compaction products, reclaim plans,
  retained-range manifests, and rebuild-debt summaries must stay structurally
  distinct from canonical truth and from each other.
- `perf_laws.md`
  The most important thing it protects is breadth honesty. Milestone 10 must
  name rewrite breadth, reclaim breadth, retained-range breadth, and rebuild
  debt as exact counters instead of hiding maintenance work behind one vague
  "compaction ran" signal.
- `domain_laws.md`
  The most important thing it protects is decomposition by reason-to-change.
  Retention policy definition, candidate planning, compaction publication,
  reclaim execution, rebuild debt tracking, and certification evidence must be
  separate subdomains rather than one maintenance module.
- `worth_store_vision.md`
  The most important thing it protects is that retention and reclamation are
  explicit product-visible policy layered over canonical authority. Milestone 10
  must therefore make compaction products rebuildable and must never let
  physical layout rewrites become meaning.
- `worth_store_roadmap.md`
  The most important thing it protects is sequencing. Milestone 10 belongs
  after snapshots, branch deltas, and Milestone 6 derived layout families are
  explicit, and it must stabilize rebuild and retention rules before tiering,
  replication, and blob lifecycle work.
- `test-requirements.md`
  The most important thing it protects is certification-grade parity. Milestone
  10 is not closed until `Retention/Compaction/Reclaim Parity Test` proves
  retained truth stays exact, reclaim only deletes policy-eligible artifacts,
  and compacted layouts do not change retained replay or restore conclusions.
- `milestone-4.md`
  The most important thing it protects is basis-explicit snapshot non-authority.
  Milestone 10 must carry that rule forward when thinning or reclaiming
  snapshots so restore acceleration never outranks retained canonical history.
- `milestone-5.md`
  The most important thing it protects is branch-delta replay parity and
  rewrite lineage. Milestone 10 must preserve replayable retained ranges across
  compaction instead of smuggling new history meaning into rewritten delta
  stacks.
- `milestone-5-closeout.md`
  The most important thing it protects is that branch rewrite and rebuild are
  already certified as cost-changing, not truth-changing. Milestone 10 should
  consume that certified substrate rather than reopening branch authority shape.
- `milestone-6.md`
  The most important thing it protects is that aspect-layout slices, structural
  blocks, and chunk exports are derived durable families with explicit rebuild
  and control parity. Milestone 10 must compact or reclaim those families under
  one policy model without turning them into retained-history authority.
- `milestone-6-closeout.md`
  The most important thing it protects is that published Milestone 6 lanes are
  now real derived families with certification evidence, while proof-only lanes
  remain explicit. Milestone 10 must not treat proof-only posture as reclaimable
  durable state or assume all live-query narrowing is already materialized.
- `worth_store_dependency_map.md`
  The most important thing it protects is the actual unlock shape: Milestone 10
  is unlocked by Milestones 4, 5, and 6; Milestone 8 can proceed concurrently;
  and later tiering, replication, and blob programs should wait for retention
  and rebuild rules to become honest first.

## Adversarial Constraint

Milestone 10 must survive this hostile condition:

> A store under branch pressure, snapshot pressure, repeated Milestone 6 layout
> publication, concurrent Milestone 8 live-query basis continuation, and mixed
> conservative and aggressive retention profiles must preserve one exact
> replayable retained truth lane, explicit basis-survival conclusions, and
> rebuildable derived artifacts even as compaction rewrites physical storage and
> reclaim deletes policy-expired artifacts.

## Product Decision Lock

- retention policy is always explicit over named artifact families, branch or
  frontier scopes, and retained-history windows; there is no ambient "keep
  enough history" mode
- surviving branch heads, retained snapshots, and any published stable basis
  imply ancestry closure over the authoritative commits required to replay them
- compaction products are always derived durable artifacts
- reclaim eligibility is always derived from retained authoritative basis plus
  declared family rebuild rules, never from storage pressure alone
- retained authoritative ranges remain answerable from canonical retained truth,
  even if a compacted acceleration path also exists
- basis survival for concurrent Milestone 8 live-query work is a typed
  retention conclusion, not a side effect inferred from missing files
- rebuild debt is part of the milestone surface and must be emitted whenever
  policy allows deletion of rebuildable families
- aggressive policies may ship as explicit `Debt` only after conservative
  policy paths prove parity and rebuild honesty

## Required Contracts And Counters

### Retention Authority Rule

Retention must decide survival against explicit authority scopes before any
physical rewrite or delete plan is admitted.

Required retention vocabulary:

- retained authoritative ranges
- policy-expired authoritative ranges
- rebuild-required derived families
- rebuild-optional derived families
- non-reclaimable live-query basis and cursor support families when their basis
  is still promised to survive

Rules:

- one retention decision must be explainable back to branch/frontier and
  artifact-family vocabulary
- reclaim planning may not discover truth eligibility from backend-local row
  shape
- if policy keeps an authoritative range, every claimed compaction product must
  declare that retained basis explicitly

### Retention Closure Rule

Retention may expire authority only after proving closure over every surviving
head and basis that still claims replayability.

Required retained-closure vocabulary:

- `RetainedHeadSet`
- `StableBasisSet`
- `RetentionClosureWitness`
- `PolicyExpiredAuthorityRange`

Minimum closure rules:

- if a branch head survives, the authoritative ancestry required to replay that
  head survives
- if a snapshot basis survives, the authoritative tail required by its declared
  restore contract survives
- if Milestone 8 publishes a stable basis as resumable, the basis commit and
  cursor continuation floor survive until an explicit basis-survival verdict says
  otherwise

Rules:

- "retain head metadata but drop the commits underneath it" is illegal
- a `PolicyExpiredAuthorityRange` may be constructed only from a
  `RetentionClosureWitness`, never from raw commit-age filtering
- closure must be defined in branch/frontier and basis vocabulary, not as
  backend-local segment membership

### Compaction Non-Authority Rule

Compaction may rewrite physical layout, not retained truth meaning.

Required posture:

- compaction products carry explicit retained-basis identity
- pre-compaction and post-compaction retained reads are mechanically comparable
- compacted physical summaries can be deleted and rebuilt from the retained
  basis they summarize

Rules:

- compacted output may not become the only surviving representation of
  retained truth
- no read path may silently depend on compaction output without exposing that
  it consumed a derived lane

### Compaction Cutover Rule

Compaction must publish, verify, and cut over atomically before reclaim is
allowed to consume superseded physical families.

Required cutover vocabulary:

- `CompactionPlan`
- `PublishedCompactionProduct`
- `CompactionCutoverWitness`
- `SupersededPhysicalFamily`

Minimum cutover phases:

- plan compaction against one retained basis
- publish the compacted product as a derived durable artifact
- verify retained-read parity and product integrity against that basis
- cut over through one typed witness
- only then admit superseded families for reclaim planning

Rules:

- reclaim may not race ahead of verified compaction publication
- crash between publish and cutover must leave the pre-cutover lane fully
  readable and the post-cutover lane explicitly unpublished
- crash after cutover but before reclaim must leave duplicate physical families
  legal, never ambiguous truth

### Reclaim Eligibility Rule

Reclaim must prove that deletion preserves all policy-promised retained truth
and all policy-promised rebuild inputs.

Minimum eligibility checks:

- authoritative range expiration under the declared retention policy
- no surviving retained branch or snapshot basis still references the target
- derived-family rebuild basis survives if the family is declared rebuildable
- concurrent basis-pinned reads or continuations do not require the target to
  remain live under the declared basis-survival rules

Rules:

- reclaim bookkeeping is proof, not authority
- if deletion legality cannot be proven from declared retained basis, the target
  is not reclaimable
- raw artifact ids or raw storage paths must not be sufficient to request
  reclaim; reclaim must consume typed eligibility witnesses

### Rebuild Debt Rule

If policy allows a rebuildable family to be reclaimed before demand returns,
the store must emit explicit rebuild debt and later clear it only through
successful rebuild.

Minimum debt surfaces:

- retained-range rebuild debt
- snapshot-family rebuild debt
- Milestone 6 layout/block rebuild debt
- live-query continuation fallback debt where concurrent Milestone 8 work cannot
  stay on the admitted narrow lane after reclaim

Rules:

- rebuild debt is not optional operator metadata
- debt must identify the family and retained basis it depends on

### First-Ship Policy Surface Rule

Milestone 10 must define a concrete conservative first-ship policy surface so
implementation cannot hide naive traps behind "aggressive policy later."

Required first-ship supported policies:

- retain current branch heads and their replay closure
- retain one named bounded history window per branch family
- retain snapshots explicitly pinned by policy
- reclaim Milestone 6 derived families only when their retained rebuild basis
  survives

Explicit out-of-scope debt for first ship:

- cross-branch global history thinning that requires merge-aware minimization
- policy-driven selective reclaim of Milestone 8 continuation support beyond
  explicit basis-survival verdicts
- density-adaptive or pressure-reactive policy switching

Rules:

- unsupported aggressive policies must fail typed or remain explicitly marked
  `Debt`
- first-ship conservative policy must be certifiable end to end before more
  aggressive modes are admitted

### Live-Query Concurrency Boundary Rule

Milestone 10 must preserve the vocabulary Milestone 8 needs while Milestone 8 is
built concurrently.

Required boundary:

- Milestone 10 owns retention, compaction, reclaim, and basis-survival
  publication
- Milestone 8 owns stable-basis reads, durable cursor continuation, and basis
  mismatch semantics
- compaction and reclaim may publish basis-survival conclusions, but they may
  not redefine what continuation means or invent a second cursor model

### Compile-Time Boundary Rule

The highest-risk retention boundaries must be compiler-enforced rather than left
as doc-only rules.

Required proof-bearing surfaces:

- `RetentionClosureWitness`
- `PolicyExpiredAuthorityRange`
- `CompactionCutoverWitness`
- `ReclaimEligibilityWitness`
- `BasisSurvivalVerdict`

Required compile-time posture:

- compaction execution may not consume raw branch ids, raw frontier ids, or raw
  artifact ids; it must consume a lowered `CompactionPlan`
- cutover may not be caller-synthesized from "publish succeeded" flags; it must
  consume verified parity and integrity proof types
- reclaim may not consume raw artifact ids; it must consume
  `ReclaimEligibilityWitness`
- stable-basis invalidation visible to Milestone 8 may not be represented as a
  nullable or best-effort file lookup; it must be a typed
  `BasisSurvivalVerdict`
- retained-history reads may not accept a `PublishedCompactionProduct` as though
  it were canonical authority

Required proof surface:

- compile-fail tests for raw-id reclaim attempts
- compile-fail tests for synthetic cutover witness construction
- compile-fail tests for direct read-path authority substitution from
  compaction products
- compile-fail tests for untyped basis-survival signaling across the Milestone 8
  boundary

### Performance-Shaping Types Rule

Milestone 10 must encode the dominant cost decisions into lowered types so the
executor consumes a pre-resolved maintenance path instead of rediscovering it.

Required performance-shaping surfaces:

- `RetentionClosureSummary`
- `RetentionCandidatePlan`
- `ConservativeRetentionPlan`
- `CompactionBackedRetentionPlan`
- `RebuildRequiredRetentionPlan`
- `RetainedReadPath`

Required posture:

- closure breadth is summarized once and carried forward
- policy resolution lowers to one concrete execution-plan family before any
  heavy execution begins
- retained reads expose whether they consumed canonical retained authority or a
  compacted derived lane

Rules:

- execution may not re-run policy resolution once a lowered plan exists
- downstream phases may not re-walk ancestry if an equivalent trusted
  `RetentionClosureSummary` already exists
- cheap-looking retained read APIs must not hide whether they traversed
  canonical retained authority or a compaction-derived path

### Lowered Work Unit Families Rule

Maintenance breadth must be partitioned by semantic family and locality before
execution.

Required lowered work unit families:

- `SnapshotCompactionUnit`
- `DeltaLayerCompactionUnit`
- `LayoutFamilyCompactionUnit`
- `AuthoritativeRangeReclaimUnit`
- `DerivedFamilyReclaimUnit`
- `RetainedRangeRebuildUnit`

Required posture:

- every work unit is tied to one retained basis and one artifact family
- family-local execution remains possible without constructing a global
  maintenance batch
- counters attach to work-unit families rather than to one rolled-up
  maintenance blob

Rules:

- snapshot, delta, and layout compaction may not be fused into one ambient work
  bucket by default
- reclaim and rebuild work units must remain distinct from compaction work units
- operators may orchestrate multiple units together later, but the architecture
  must preserve family-local execution as the primitive

### Read And Result Cost Surface Rule

Every retained read, compaction result, and reclaim result must expose its cost
posture and execution lane in the result envelope.

Required result surfaces:

- retained read path classification
- closure breadth summary or equivalent retained-basis summary
- compacted-family count and rewritten-range count
- reclaim deletion count and live-basis rejection count
- rebuild-debt delta for the operation

Rules:

- a consumer must be able to tell whether the operation used canonical retained
  authority, a compaction-derived lane, or an explicit fallback lane
- end-to-end cost claims are invalid unless the result envelope carries the
  structural counters that justify them
- result surfaces must distinguish zero-work reuse from real rewrite or delete
  execution

### Complexity-Status Surface Rule

Milestone 10 evidence must publish path-local complexity status rather than one
rolled-up maintenance verdict.

Minimum named paths:

- `retention_candidate_planning`
- `compaction_publication`
- `reclaim_execution`
- `retained_range_rebuild`

Rules:

- each path declares at least `Verified` or `Debt`
- any `Debt` path names the unresolved breadth, missing proof, or unsupported
  aggressive policy trigger

Minimum contracts:

- retention candidate planning cost is proportional to:
  - declared policy scopes inspected
  - retained authoritative ranges emitted
  - candidate artifact families evaluated
  - not total store bytes
- compaction publication cost is proportional to:
  - retained ranges rewritten
  - delta layers, snapshots, or structural blocks rewritten for those ranges
  - compacted product records emitted
- reclaim execution cost is proportional to:
  - reclaim candidates proven eligible
  - artifact families deleted
  - retained-basis validations required before delete
- retained-range rebuild cost is proportional to:
  - retained authoritative ranges replayed
  - derived families rebuilt from those ranges
  - not expired history already declared outside policy

Forbidden hidden work:

- whole-history replay to answer whether one narrow retained range survives
- reclaim-by-pressure without explicit policy scope evaluation
- silent live-query basis invalidation caused only by backend-local compaction
- implicit dependency on compaction products for retained truth reads

Minimum counters:

- `retention_policy_evaluation_count`
- `retained_authoritative_range_count`
- `expired_authoritative_range_count`
- `compaction_plan_count`
- `compacted_delta_layer_count`
- `compacted_snapshot_family_count`
- `compacted_layout_family_count`
- `compaction_cutover_count`
- `compaction_cutover_rejection_count`
- `reclaim_candidate_count`
- `reclaimed_authoritative_artifact_count`
- `reclaimed_derived_artifact_count`
- `reclaim_rejected_live_basis_count`
- `retention_closure_ancestor_count`
- `retention_closure_failure_count`
- `retained_range_rebuild_count`
- `rebuild_debt_count`
- `compaction_debt_count`
- `retention_truth_parity_failure_count`
- `retention_restore_parity_failure_count`
- `retention_artifact_rebuild_failure_count`

Required counter assertions:

- `retention_truth_parity_failure_count` remains zero for admitted conservative
  policy lanes
- `retention_restore_parity_failure_count` remains zero for retained-range
  restore lanes
- `retention_closure_failure_count` remains zero for admitted surviving-head and
  stable-basis lanes
- `reclaim_rejected_live_basis_count` increments when reclaim correctly refuses
  to delete still-required basis support during concurrent Milestone 8 work
- `rebuild_debt_count` and `compaction_debt_count` distinguish real deferred
  work from completed safe maintenance

## Phases

### Phase 1: Lock Retention Authority, Policy Scope, And Non-Authority Boundaries

Phase 1 defines what retention, compaction, and reclaim are allowed to mean
before any delete or rewrite work lands.

Required work:

- define retention policy vocabulary over authoritative ranges and derived
  families
- define first-ship conservative policy classes and explicit aggressive-policy
  debt markers
- define retained-range identity and basis-survival witness types
- define retained-head and stable-basis closure witness types
- define `RetentionClosureSummary` and lowered retention-plan families
- define compaction-product identity and rebuild-basis records
- define crash-safe compaction publish, verify, and cutover witness types
- define family-local lowered compaction, reclaim, and rebuild work units
- define reclaim eligibility proofs and rebuild-debt summaries
- define Milestone 8 concurrency boundaries for stable basis and cursor support
- define typed conservative versus aggressive policy classes

Exit condition:

- every retained or expired decision has one exact authority basis
- surviving heads and stable bases imply machine-checkable ancestry closure
- compaction and reclaim artifacts are structurally non-authoritative
- Milestone 8 can continue concurrently without inferring basis survival from
  ambient maintenance behavior

### Phase 2: Plan Retention Windows, Rebuild Debt, And Candidate Selection

Phase 2 makes policy evaluation and candidate selection machine-checkable.

Required work:

- implement policy evaluation over branches, history windows, snapshots, and
  Milestone 6 derived families
- implement retained-head and stable-basis closure planning before expiration
  candidates are emitted
- implement retained authoritative range planning
- lower policy-admitted candidates into concrete retention-plan families before
  execution begins
- implement compaction candidate planning and explicit reject reasons
- implement reclaim candidate planning with live-basis protection checks
- implement rebuild-debt publication for reclaimable derived families
- emit exact retention, candidate, and debt counters

Exit condition:

- retention planning can explain every candidate back to policy and retained
  basis
- raw age or pressure scans are not sufficient to emit expired authority ranges
- reclaimable derived families cannot bypass rebuild-debt publication
- live basis and cursor support remain explicitly protected when policy keeps
  their basis alive

### Phase 3: Publish Compaction Products As Derived Durable Artifacts

Phase 3 turns retention planning into real physical rewrite without changing
retained truth meaning.

Required work:

- implement compaction publication for admitted snapshot, delta, and Milestone 6
  derived-family lanes
- persist compacted-product basis records and diagnostics
- implement publish, parity-verify, and cutover as distinct typed phases
- execute compaction through family-local lowered work units rather than one
  fused maintenance path
- implement retained-range parity reads across pre- and post-compaction state
- expose typed compaction rejection, basis-mismatch, and integrity failures
- emit exact rewritten-range and compacted-family counters

Exit condition:

- compacted storage can answer retained reads without becoming the only source
  of retained truth
- compacted products are rebuildable from their declared retained basis
- cutover is crash-safe and reclaim-admitting only after verified publication
- compaction changes cost and footprint, not retained semantic outcome

### Phase 4: Reclaim Eligible Artifacts Without Losing Retained Truth

Phase 4 makes policy-authorized deletion honest instead of approximate.

Required work:

- implement reclaim execution for policy-expired authoritative ranges where
  allowed
- implement reclaim execution for rebuildable derived families with published
  rebuild debt
- require reclaim eligibility witnesses rather than raw artifact identifiers
- execute authoritative reclaim and derived-family reclaim as distinct lowered
  work-unit families
- implement post-reclaim retained-range verification and rebuild admission
- implement explicit live-basis rejection for still-needed support artifacts
- expose typed reclaim legality, dependency, and rebuild-basis failures
- emit exact reclaim, rejection, and rebuild counters

Exit condition:

- deleted artifacts are exactly those policy said could disappear
- retained authoritative truth remains replayable and restorable
- reclaimed derived families are either gone legally or rebuildable from the
  retained basis that survived
- crash between compaction cutover and reclaim leaves duplicate physical
  families possible, but never ambiguous retained truth

### Phase 5: Prove Retention, Compaction, And Reclaim Parity

Phase 5 turns maintenance into a certifiable store surface rather than an
operator superstition.

Required work:

- run the Milestone 10 named suite:
  `Retention/Compaction/Reclaim Parity Test`
- compare pre-compaction and post-compaction retained reads and replay results
- compare post-reclaim retained truth against a control lane that never deleted
  in-policy truth
- include crash points between compaction publish, cutover, and reclaim
- prove rebuildability for reclaimed derived families where policy says rebuild
  must remain possible
- emit machine-checkable truth, restore, artifact, and counter bundles

Exit condition:

- retained truth remains exact
- reclaim deletes only policy-eligible artifacts
- compaction products remain non-authoritative derived artifacts
- surviving heads and stable bases never lose replay closure through policy
  expiration
- Milestone 10 closeout evidence exists in machine-checkable form

## Must Ship

- explicit retention policy surfaces over authoritative ranges and derived
  artifact families
- concrete first-ship conservative policy classes with explicit aggressive-mode
  `Debt` boundaries
- retained-range identity and basis-survival witness types
- retained-head and stable-basis closure witnesses
- performance-shaping types for closure summaries, lowered retention plans, and
  retained-read path classification
- compaction products for admitted snapshot, branch-delta, and Milestone 6
  derived-family lanes
- crash-safe compaction publish/verify/cutover surfaces
- family-local lowered work units for compaction, reclaim, and rebuild
- reclaim eligibility proofs with live-basis protection
- rebuild-debt and compaction-debt evidence surfaces
- compile-fail boundary coverage for compaction cutover, reclaim admission, and
  basis-survival signaling
- typed retention, compaction, reclaim, and rebuild failures
- exact retention, rewrite, reclaim, and debt counters
- machine-checkable Milestone 10 certification output

## Must Preserve

- retained authoritative truth remains replayable from retained canonical basis
- compaction products never become a second authority source
- reclaim remains policy-driven, not pressure-driven
- surviving heads, retained snapshots, and stable bases keep their replay
  closure until policy explicitly says otherwise
- snapshot, branch-delta, and Milestone 6 derived-family rebuild rules remain
  explicit and basis-rooted
- concurrent Milestone 8 work keeps one stable-basis and durable-cursor model
- later tiering, replication, and blob lifecycle work inherit one retention
  policy model instead of inventing their own

## Acceptance Evidence

Milestone 10 is complete only when the store satisfies the named Milestone 10
suite:

- `Retention/Compaction/Reclaim Parity Test`

Required machine-checkable outputs:

- `truth_digest`
- `restore_digest`
- `artifact_digest`
- `counter_snapshot`

Milestone-specific proof obligations:

- retained truth remains exact before and after compaction
- retained restore results remain exact before and after compaction
- reclaimed derived artifacts are rebuildable where policy says they are
- reclaim never deletes policy-retained authority
- surviving heads and stable bases retain the authority closure needed for
  replay and continuation
- compacted products remain explicitly derived from retained basis
- crash between compaction publish, cutover, and reclaim never leaves an
  ambiguous retained-truth source
- concurrent live-query basis support is either preserved or rejected with typed
  basis-survival conclusions rather than silent drift

Milestone 10 is not closed by "disk usage went down" or "maintenance finished
eventually" tests.

## Architectural Notes

- The smart abstraction is not "garbage collection." The smart abstraction is
  one retained-range authority model plus one non-authoritative compaction and
  reclaim pipeline.
- Conservative retention policy is the proving ground. Aggressive retention
  variants may ship later as explicit `Debt`, but only after conservative parity
  is certified.
- Compaction and reclaim should remain separate subdomains even if one operator
  command eventually orchestrates them together.
- Milestone 10 should publish basis-survival conclusions that Milestone 8 can
  consume, but it must not absorb live-query continuation semantics.
- Milestone 11 should inherit named work classes and debt surfaces from this
  milestone instead of redefining what maintenance work exists.

## Sequencing Notes

This milestone belongs after Milestones 4, 5, and 6 because retention cannot be
made honest until snapshots, branch-delta layers, and derived layout/block
families all have explicit authority and rebuild boundaries.

- `Milestone 8` should proceed concurrently because live-query continuation
  needs the stable basis and cursor model from Milestone 7 plus the retention
  and basis-survival honesty defined here, but Milestone 10 must not wait for
  full live-query implementation to make retention policy exact.
- `Milestone 11` should follow this milestone because scheduling maintenance
  safely is a weaker problem until the maintenance semantics themselves are
  already exact.
- `Milestone 13`, `Milestone 14`, and `Milestone 20` should inherit this
  milestone's retained-range, rebuild, and reclaim rules rather than inventing
  new lifecycle semantics for tier movement, replication capsules, or blobs.
