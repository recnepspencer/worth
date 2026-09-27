# Milestone 5: Branching, Merging, And Commit Vocabulary

## Goal

Define the shared language for branch-local intent, merge admission and
verdicts, committed authority transitions, and commit receipts so WORTH crates
can describe the same authority-changing event one way everywhere without
inventing incompatible transaction, preview, merge, or history folklore.

## Governing Document Summaries

### `MENTALITY.md`

Protects adversarial-constraint-first engineering and mechanical enforcement.
The shaping constraint is that Milestone 5 must solve authority-transition
honesty first: branch-local work, merge decisions, and committed authority must
become typed, self-describing, and proof-bearing before later diagnostics,
lineage, provenance, and migration work tries to attach to them.

### `arch_laws.md`

Protects authority-versus-derivation separation, self-describing envelopes,
proof-bearing progression, and one canonical artifact per authority boundary.
The shaping constraint is that branch-local candidates, merge verdicts,
committed authority, and commit receipts must be mechanically distinct and must
compose through explicit transition artifacts rather than ambient flags.

### `composition_laws.md`

Protects responsibility-shaped files and named semantic steps. The shaping
constraint is that branch identity, merge verdict law, committed authority
artifacts, commit receipts, basis lowering, and readiness evidence must live in
separate responsibility homes rather than one broad transitions dump.

### `domain_structure_laws.md`

Protects structure as responsibility topology rather than convenience filing.
The shaping constraint is that branch-local state, merge evaluation, committed
authority, receipt issuance, and replay/readmission boundaries must be
independently locatable and testable.

### `perf_laws.md`

Protects cost-honest boundaries, planning-before-execution, explicit locality,
and path separation between authority and descriptive richness. The shaping
constraint is that merge planning, merge execution, branch comparison, and
receipt issuance must expose their real breadth and basis explicitly rather than
looking like cheap state labels.

### `worth_foundational_vision.md`

Protects the thesis that `worth-foundational` owns shared truth vocabulary for
authority transitions while preserving crate-local execution and storage. The
shaping constraint is that Milestone 5 must standardize branch/merge/commit
boundary meaning without forcing one transaction engine, journal model, or
merge runtime.

### `worth_foundational_roadmap.md`

Protects the sequencing rule that branch/merge/commit vocabulary follows
profiles and boundary artifacts, and precedes diagnostics, lineage/provenance,
and migration closure. The shaping constraint is that Milestone 5 must replace
Milestone 4's reserved-authority-transition fail-closed placeholder with real
shared ontology and typed proof obligations.

### `test-requirements.md`

Protects standalone proof before adopting-crate migration. The shaping
constraint is that branch-local candidate separation, merge-verdict topology,
commit-receipt issuance, canonical basis parity, reduced-richness preservation,
and replay/blind-consumer interpretation all need local hostile proof before
adopting runtimes are allowed to depend on them. For Milestone 5 specifically,
that proof must also attack ambient basis choice, hidden strategy influence,
thin receipts, generic transition-result bags, and cheap convenience helpers
that try to bypass inspectable planning or proof-bearing authority seams.

### `milestone-4.md`

Protects shared boundary artifact categories, explicit materialization seams,
bundle legality, canonical basis participation, and the new Phase 4.5
proof-bearing current-basis lane. The shaping constraint is that Milestone 5
must consume those surfaces rather than rebuilding artifact, receipt, or
current-basis law locally.

### `milestone-4-closeout.md`

Protects the fact that boundary artifact taxonomy, materialization law,
proof-bearing authoritative/current-basis lanes, planned-work descriptive room,
and production-test readiness are already implemented and locally certified.
The shaping constraint is that Milestone 5 must replace only the reserved
authority-transition placeholder and must reuse the existing artifact and
current-basis lanes instead of reopening them.

## Existing Runtime Patterns

This milestone is intentionally shaped by what already worked well in the
runtime crates.

### `worth-runtime-bridge`

What to keep:

- speculation is a session-shaped workflow, not an ambient mode
- compare-to-main is explicit and basis-bearing
- discard is positive zero-residue evidence, not hopeful absence
- promote is an explicit authority crossing
- no-op versus committed outcome remains first-class
- route/writeback results often honestly emit summary + artifact + receipt

What to prune:

- too much crate-local dialect around preview/promotion/writeback naming
- too many authority-transition meanings hidden behind bridge-local family or
  adapter taxonomy that later consumers would have to relearn

### `worth-signal`

What to keep:

- transactions are all-or-nothing authority transitions
- `history()` is the public front door for branch/snapshot/replay/history work
- replay versus lineage are explicitly different questions
- merge follows a guided `plan()` then `execute()` flow

What to prune:

- runtime-specific surface sprawl around history and diagnostics names should
  not become foundational vocabulary wholesale
- foundational should standardize the transition nouns, not Signal's whole
  runtime API story

### `worth-relational`

What to keep:

- structural summary exists before execution and records topology/touched scope
- merge planning and merge execution are distinct
- merge outcomes distinguish admitted, blocked, rejected, manual, and resolved
  shapes rather than flattening to success/failure
- lineage candidate promotion is explicit and can publish metadata-only commits
- canonical commit envelope carries parentage, patch, diagnostics, lineage, and
  merge basis in one authority artifact
- strategy identity, family, version, and ownership are durable enough to
  survive canonical commit envelopes and replay
- custom merge policy and strategy-bearing conflict semantics are explicit
  rather than ambient hook behavior

What to prune:

- the ontology is currently too relational-local and too wide to ask every
  adopting crate to reuse directly
- foundational should capture the shared transition boundary law, not all of
  relational's internal merge-policy or lineage-runtime detail

### `worth-store`

What to keep:

- canonical commit envelopes reject noncanonical parentage and mismatched branch
  context
- historical identity and lineage support artifacts are explicit support
  surfaces, not ambient queries
- digest and basis evidence live with the authority artifact

What to prune:

- store-local canonicalization mechanics should not become the only foundational
  story for authority transitions
- foundational should own the shared vocabulary, while store continues owning
  durable canonicalization/storage mechanics

### `worth-query`

What to keep:

- support matrices and digests make posture and coverage explicit
- support/receipt/report surfaces are basis-bearing and consumer-readable

What to prune:

- query's subscription-specific support classes should not leak into transition
  vocabulary
- foundational should borrow the explicit posture/report discipline, not the
  domain-specific matrix layout

## Why This Milestone Exists

Milestones 1 through 4 established:

- aspect-native truth vocabulary
- canonical basis and digest-honest comparison law
- profile meaning and reduced-richness elision law
- boundary artifact categories, materialization seams, and proof-bearing
  current-basis boundary lanes

They did not yet answer the next authority question:

- what is branch-local candidate state versus staged branch state
- what is a merge candidate versus a merge verdict
- what is a committed authority artifact versus a commit receipt
- what parentage, merge basis, and committed deltas must a consumer see
- which of those surfaces are descriptive, which are authoritative, and which
  are proof-bearing stronger claims

Milestone 4 intentionally fail-closed this space through reserved
authority-transition denials. Milestone 5 exists to replace that placeholder
with one shared authority-transition language so later diagnostics, lineage,
provenance, and migrations stop inventing local transaction or history folklore
again.

This milestone also exists to stop extensible merge and commit strategies from
becoming runtime-private truth. Once a runtime supports intent-bearing commits,
custom merge policy, or pluggable commit semantics, those strategy-bearing
authority transitions must still materialize into one shared boundary language.
Otherwise replay, certification, provenance, and later runtime rebuilds will
all rediscover the same strategy ontology separately.

## Adversarial Constraint

Several WORTH crates with different preview/session models, merge planners,
journal layouts, durability strategies, and extensible merge/commit strategy
surfaces must be able to describe the same branch-local candidate, merge
verdict, strategy-bearing committed authority transition, and commit receipt
with one canonical meaning everywhere, while preserving explicit parentage,
basis, deltas, no-op-versus-commit classification, strategy identity,
replay-safe interpretation, and reduced-richness elision of only optional
forensic detail.

This milestone fails if:

- branch-local candidate state can masquerade as committed authority
- merge verdicts collapse into success/failure
- no-op and committed authority outcomes are not distinguishable
- commit receipts can be minted from intent or candidates instead of completed
  authority transitions
- strategy-bearing commits or merge decisions are flattened into generic commit
  evidence with no durable strategy identity, family, version, or ownership
  surface
- multi-parent or nontrivial merge ancestry is silently collapsed into one
  parent plus ad hoc metadata
- consumers need producer-private state to understand branch basis, merge
  parentage, conflict loci, committed deltas, or transition-shaping strategy
- reduced-richness profiles remove or alter authority-bearing meaning
- foundational rebuilds Milestone 2 canonicalization or Milestone 4 current-
  basis/readiness law locally
- foundational standardizes one branch graph, transaction engine, or merge
  runtime instead of shared boundary meaning

## Dependencies On Earlier Milestones

Milestone 5 is downstream of earlier foundational work and must reuse it
explicitly.

### Milestone 2: Canonicalization

Milestone 2 remains the owner of canonical basis, digest slots, comparison
readiness, and current-basis law. Milestone 5 may add branch/merge/commit
domains, entry kinds, and basis builders, but it must not invent a second
canonicalization dialect for transitions.

Use Milestone 2 for:

- canonical basis preparation for branch, merge, committed-authority, and
  commit-receipt surfaces
- digest-honest parentage, merge-basis, and committed-delta identity
- current-basis readmission and trust-boundary strengthening where exposed

### Milestone 3: Profiles

Milestone 3 remains the owner of profile families, attachment law, materialized
meaning, reduced-richness planning, and certification posture. Milestone 5 must
consume those profile surfaces rather than reinterpreting them.

Use Milestone 3 for:

- support/certification posture attached to transition artifacts
- reduced-richness suppression of optional forensic branch/merge detail
- target-aware profile legality for receipts, reports, support surfaces, and
  current-basis authority artifacts

### Milestone 4: Boundary Artifacts

Milestone 4 remains the owner of `Summary`, `Report`, `Artifact`, `Receipt`,
bundle law, materialization seams, and the proof-bearing current-basis boundary
lane. Milestone 5 must land inside those categories rather than reopening them.

Use Milestone 4 for:

- commit receipts as real `Receipt` boundary artifacts rather than local commit
  bags
- merge verdict, branch candidate, and committed-authority descriptive/support
  surfaces as real boundary artifacts
- stronger authority/current-basis progression and readmission where transition
  artifacts make those claims

## WORTH-Proof Dependency Boundary

Milestone 5 uses `worth-proof` for stronger authority-transition and receipt
claims, but not for plain branch/merge/commit nouns.

The mandatory Milestone 5 proof lane is:

- use `worth-proof::TransitionOutcome` for merge-plan admission and merge-
  verdict non-success topology
- use `worth-proof::Artifact` as the proof-bearing carrier for committed-
  authority, receipt-bearing, current-basis, and readiness surfaces
- use `AuthorityWitness::from_authority_marker(...)` and
  `Proof::from_authority_witness(...)` for authority-scoped strengthening
- use `Artifact::with_proofs_and_current_basis(...)` as the default stronger
  constructor when a transition surface claims both proof-bearing authority and
  current-basis meaning
- use `Artifact::with_current_basis(...)` only for stronger current-basis
  surfaces that intentionally do not carry an additional proof set
- use `.bridge_trust_boundary()`, `.readmit_with_authority(...)`, and
  `.rebind_with_authority(...)` for trust-boundary weakening and
  re-strengthening where Milestone 5 exposes those lanes

Milestone 5 does not standardize `worth-proof::Recipe` as its default proof
carrier. `Recipe` remains out of scope here unless a later milestone
explicitly chooses staged recipe progression for a specific transition family.

`worth-proof` is mandatory for:

- merge admission and merge verdict non-success topology through
  `TransitionOutcome`
- committed-authority transition artifacts that claim a real authority crossing
- commit receipts that attest a completed authority transition
- trust-boundary bridge/readmission for committed-authority or receipt-bearing
  current-basis artifacts
- production-test readiness artifacts for Milestone 5

`worth-proof` is forbidden for:

- plain branch ids, merge ids, commit ids, parentage lists, and delta loci
- plain merge candidate or merge verdict vocabulary
- plain strategy identity, strategy family, strategy ownership, and strategy
  version vocabulary
- plain structural summaries or conflict-classification nouns
- plain branch-local staged/candidate descriptive surfaces

The operating rule is:

`worth-foundational` defines what branch/merge/commit boundary meaning is.
`worth-proof` proves when a transition crossed authority or still carries a
strong current-basis claim.

The implementation is not allowed to choose its own proof lane here. The spec
choice is already made:

- `TransitionOutcome` is the mandatory merge-admission outcome carrier
- `Artifact` is the mandatory proof-bearing authority/receipt/current-basis
  carrier
- `Recipe` is not the Milestone 5 default and must not be introduced ad hoc as
  an alternate transition substrate

## Practical Type Targets

The implementation may choose better names, but these responsibilities must
exist concretely somewhere:

```rust
pub struct FoundationalBranchId { /* private fields */ }
pub struct FoundationalMergeId { /* private fields */ }
pub struct FoundationalCommitId { /* private fields */ }

pub struct FoundationalBranchCandidateForkBasis { /* private fields */ }
pub struct FoundationalBranchCandidateId { /* private fields */ }

pub enum FoundationalBranchLocalStateKind {
    Candidate,
    Staged,
}

pub struct FoundationalBranchCandidateArtifact<T> { /* private fields */ }
pub struct FoundationalStagedBranchArtifact<T> { /* private fields */ }

pub enum FoundationalMergeIntent {
    ReconcileIntoTarget,
}

pub struct FoundationalMergeCandidate<T> { /* private fields */ }

pub struct FoundationalTransitionStrategyId { /* private fields */ }
pub struct FoundationalTransitionStrategyFamily { /* private fields */ }
pub struct FoundationalTransitionStrategySemanticName { /* private fields */ }
pub struct FoundationalTransitionStrategyVersion { /* private fields */ }
pub struct FoundationalTransitionStrategyDescriptorDigest { /* private fields */ }
pub struct FoundationalTransitionStrategyContractBasis { /* private fields */ }

pub enum FoundationalTransitionStrategyOwnershipClass {
    RuntimeBuiltIn,
    CustomRegistered,
    CompatibilityLowered,
}

pub struct FoundationalTransitionStrategyIdentity { /* private fields */ }

pub struct FoundationalTransitionBasisIdentity { /* private fields */ }
pub struct FoundationalTransitionBasisFamily { /* private fields */ }
pub struct FoundationalTransitionBasisVersion { /* private fields */ }

pub struct FoundationalBranchCandidateObservationBasis { /* private fields */ }
pub struct FoundationalBranchCandidateForkObservationBasis { /* private fields */ }
pub struct FoundationalBranchCandidateComparisonBasis { /* private fields */ }
pub struct FoundationalMergeBaseSelectionBasis { /* private fields */ }
pub struct FoundationalStrategyBasis { /* private fields */ }

pub struct FoundationalTransitionCorrespondenceBasis { /* private fields */ }
pub struct FoundationalTransitionRemapBasis { /* private fields */ }

pub enum FoundationalMergeVerdictKind {
    Accepted,
    Advisory,
    Conflict,
    Denied,
    Superseded,
    StaleBasis,
}

pub struct FoundationalMergeConflictLocus { /* private fields */ }
pub struct FoundationalCommittedDeltaLocus { /* private fields */ }

pub struct FoundationalMergeVerdict { /* private fields */ }
pub struct FoundationalMergeStructuralSummary { /* private fields */ }
pub struct FoundationalMergeDecisionRow { /* private fields */ }

pub enum FoundationalBranchBasisDriftKind {
    TargetAdvanced,
    SourceAdvanced,
    MergeBasisInvalidated,
    ParentBasisUnavailable,
}

pub struct FoundationalBranchBasisDrift { /* private fields */ }

pub struct FoundationalCommittedAuthorityArtifact<T> { /* private fields */ }
pub struct FoundationalCommitParentBasis { /* private fields */ }
pub struct FoundationalCommitParentage { /* private fields */ }
pub struct FoundationalMergeAncestryBasis { /* private fields */ }
pub struct FoundationalCommitDeltaSummary { /* private fields */ }

pub enum FoundationalAuthorityTransitionClass {
    NoOp,
    Commit,
    MetadataOnlyCommit,
    PromotionCommit,
    ReplayRevalidatedCommit,
}

pub enum FoundationalNoOpCause {
    AlreadyConverged,
    BasisEquivalent,
    StrategySuppressed,
    ChangeDenied,
    ReplayEquivalent,
}

pub struct FoundationalCommitReceiptArtifact { /* private fields */ }
pub struct FoundationalCommitReceiptIdentity { /* private fields */ }
pub struct FoundationalBranchDiscardReceipt { /* private fields */ }

pub struct FoundationalTransitionProvenanceRow { /* private fields */ }

pub struct FoundationalTransitionBundle<Primary> { /* private fields */ }

pub enum FoundationalAuthorityTransitionOutcomeKind {
    NoOp,
    Committed,
}

pub enum FoundationalAuthorityTransitionDenial {
    BranchCandidateNotAdmitted,
    MergeVerdictNotCommitEligible,
    ReceiptRequiresCommittedAuthority,
    ParentBasisMissing,
    CurrentBasisReadmissionRequired,
}

pub struct FoundationalTransitionLocator { /* private fields */ }
pub struct FoundationalTransitionCanonicalBasisReady<T> { /* private fields */ }
```

These sketches imply concrete constraints:

- branch ids, merge ids, and commit ids remain distinct even if their storage
  is identical
- branch-local candidate and staged state remain distinct from committed
  authority and commit receipts
- merge candidates and merge verdicts remain distinct surfaces rather than one
  mutable merge record
- extensible merge and commit strategies remain first-class transition-shaping
  evidence rather than private runtime implementation detail
- strategy identity should be explicit enough to preserve semantic name, family,
  version, and ownership without dragging in runtime registries
- strategy-bearing transitions should carry deterministic descriptor/contract
  identity strongly enough for replay, certification, and migration parity to
  tell whether "the same strategy" actually means the same thing
- transition-shaping basis should be explicit enough to preserve basis identity,
  basis family, basis version, observation basis, comparison basis, merge-base
  selection basis, and strategy basis where they materially shape truth
- correspondence and remap basis should be first-class wherever transition
  meaning depends on nontrivial matching rather than obvious identity
- verdict kinds distinguish accepted, advisory, conflict, denied, superseded,
  and stale-basis outcomes explicitly
- stale-basis meaning should be structured enough to distinguish target
  advancement, source advancement, invalidated merge basis, and missing parent
  basis rather than one opaque stale flag
- parentage, merge basis, and committed deltas remain typed and consumer-
  visible rather than hidden in free-form metadata
- parentage must support canonical ordered multi-parent ancestry rather than
  assuming unary commits with optional merge metadata
- authority transition class should be explicit enough to distinguish no-op,
  ordinary commit, metadata-only commit, promotion commit, and replay-
  revalidated commit where those distinctions are real
- no-op meaning should be explicit enough to distinguish already-converged,
  basis-equivalent, strategy-suppressed, denied-to-change, and replay-
  equivalent outcomes
- transition provenance should be row-bearing and typed rather than prose-only
  explanation
- transition provenance should include strategy-bearing evidence wherever a
  strategy materially shaped merge or commit meaning
- transition provenance should include basis-bearing and correspondence-bearing
  evidence wherever those materially shaped merge or commit meaning
- if strategy, basis, correspondence, or remap semantics materially shaped a
  verdict, committed authority artifact, or receipt, the relevant evidence must
  be structurally present rather than optional ambient context
- one authority transition may honestly emit a committed-authority artifact,
  commit receipt, merge verdict report, and summary together, but that bundle
  must remain typed and category-honest rather than becoming a local result bag
- branch discard and non-authoritative closeout should have explicit typed room
  where a runtime needs positive zero-residue evidence
- no-op versus committed authority transition remains explicit
- stronger committed-authority and receipt-bearing claims reuse Milestone 4's
  artifact/current-basis lanes plus `worth-proof`
- branch/merge/commit basis preparation reuses Milestone 2 canonicalization
- receipt issuance is derived from committed-authority artifacts rather than
  local booleans or post-hoc strings

## Naive Traps To Avoid

- Do not model branch-local candidate, staged branch state, merge verdict, and
  committed authority as one struct plus a lifecycle enum.
- Do not flatten merge outcomes into success/failure or accepted/rejected. Real
  runtimes need advisory, conflict, superseded, and stale-basis meaning too.
- Do not let receipts mean both "candidate prepared" and "authority crossed."
- Do not let committed authority or receipts be reconstructed from branch-local
  materialization without proof-bearing authority evidence.
- Do not let parentage, merge basis, or committed deltas hide inside producer-
  private strings or untyped metadata blobs.
- Do not treat every stale condition as the same thing. "stale" without drift
  structure is not enough for replay, support, or safe retry decisions.
- Do not let transition bundles become arbitrary vectors of surfaces. If bundle
  membership legality can be known structurally, encode it.
- Do not issue thin receipts that require asking the producer what actually
  happened. A blind consumer should be able to tell whether the boundary was a
  no-op, metadata-only commit, ordinary commit, or promotion commit.
- Do not rebuild canonicalization or current-basis law locally. Reuse
  Milestones 2 and 4 instead.
- Do not let reduced-richness profiles erase the branch/merge detail that is
  required to interpret the committed authority outcome itself.
- Do not standardize one branch graph, transaction journal, conflict engine, or
  lineage runtime here. Standardize only the boundary meaning.

## Phases

These phases are implementation order, not topic buckets and not a buffet.

An engineer should be able to start at Phase 1 and move downward without
inventing a private sequencing model. The phase list is meant to function like
a build order for the milestone. Each phase leaves behind a concrete
responsibility home, a minimum honest public surface, and a proof boundary that
the next phase is allowed to consume. If a later phase depends on a concept
that the earlier phase has not yet made concrete, the implementation is out of
order.

The working rule for this milestone is:

1. Finish the nouns and identity of the current phase first.
2. Expose only the minimum honest API needed to use those nouns.
3. Prove the separation and deny paths of that phase locally.
4. Freeze that phase as the dependency floor for the next one.
5. Only then start the next layer that consumes it.

That means the engineer should experience the milestone as a chain of handoffs,
not as six parallel workstreams:

- Phase 1 leaves behind branch-local truth that is clearly non-authoritative.
- Phase 2 consumes that branch-local truth and turns it into merge planning and
  admitted merge meaning.
- Phase 3 consumes admitted merge meaning and creates the first real
  proof-bearing authority-transition lane.
- Phase 4 consumes committed authority and turns it into receipt-bearing,
  bundle-capable, blind-consumer boundary artifacts.
- Phase 5 consumes those transition artifacts and lowers them through
  canonicalization, locators, profiles, and stronger current-basis reuse.
- Phase 6 freezes the exact closure surface and refuses any unstated
  assumptions.

If an engineer cannot describe "what Phase N produces that Phase N+1 is now
allowed to rely on," then the phase plan is still too abstract and the
implementation should not advance.

Phase progression gates:

| Phase | Gate before next phase |
| --- | --- |
| Phase 1 | Branch-local identity and candidate/staged separation exist before merge or commit APIs exist. |
| Phase 2 | Merge candidate and merge verdict law exist before committed-authority artifacts or receipts exist. |
| Phase 3 | Committed-authority transition law exists before receipt issuance or replay/readmission strengthening exists. |
| Phase 4 | Commit receipts and canonical transition envelopes exist before profile-richness and locator integration closes. |
| Phase 5 | Canonical basis, locator, and profile attachment law exist before readiness closes. |
| Phase 6 | Production-test readiness evidence exists before diagnostics, provenance, or adopting runtimes consume Milestone 5 as closed authority-transition vocabulary. |

### Phase 1: Define Branch-Local Identity And Candidate Separation

Purpose:

Freeze the branch-local nouns and make candidate/staged state mechanically
distinct from committed authority before any merge or receipt API exists.

Engineer order:

Start by building the smallest branch-local world that later phases can trust.
Do not create merge, commit, or receipt vocabulary yet. The engineer should
finish this phase with a branch-local surface that can represent candidate and
staged work, plus the basis context that shaped that work, while still making
it impossible to confuse any of that with authority.

Practical implementation order:

1. Create the branch-local identity home and define branch ids, fork basis, and
   candidate ids.
2. Define separate branch-local candidate and staged branch surfaces.
3. Add the first deny paths for branch-local surfaces pretending to be
   committed authority.
4. Export only those nouns through the facade.
5. Write blind-consumer and compile-fail tests before moving on.

What should exist at the end of the phase:

- a caller can name branch-local candidate or staged branch state without
  seeing merge verdict or receipt APIs yet
- branch-local state is obviously descriptive/non-authoritative
- the facade exposes branch-local meaning, not a half-built transition stack
- branch-local observation and comparison basis already have explicit room so
  later branching semantics do not smuggle them through local runtime context

Must ship:

- typed branch identity vocabulary
- typed branch fork-basis vocabulary
- typed branch observation-basis, fork-observation-basis, and branch-
  comparison-basis vocabulary
- typed branch-local candidate and staged branch surfaces
- compile-time or strongly typed separation from committed-authority and
  receipt-bearing APIs

Must preserve:

- branch-local state remains non-authoritative
- branch identity remains distinct from commit identity
- staged branch state does not silently acquire receipt or commit semantics
- branch-local observation basis remains explicit rather than ambient runtime
  context

Acceptance evidence:

- hostile tests proving branch-local candidate and staged branch state are not
  substitutable with committed-authority surfaces
- hostile tests proving branch-local observation and comparison basis remain
  explicit and blind-consumer interpretable
- misuse-pressure tests proving no convenience branch-local helper can smuggle
  committed-authority or receipt meaning into candidate/staged surfaces
- blind-consumer tests proving branch ids and fork basis remain interpretable
  without producer-private runtime state
- compile-fail tests proving callers cannot pass branch-local artifacts where
  committed-authority APIs are required

### Phase 2: Define Merge Candidate And Verdict Law

Purpose:

Create the shared merge language before any committed-authority or receipt
surface tries to consume it.

Engineer order:

Once branch-local work is closed and obviously non-authoritative, consume it to
build the merge layer. This is the first phase where the implementation may
talk about reconciliation between branches, but it is still not allowed to mint
committed authority. The engineer should finish this phase with a merge plan
and a merge verdict surface that fully explain merge meaning without any
receipt, commit, or replay machinery.

Practical implementation order:

1. Create the merge home and define merge intent plus merge candidate shape.
2. Add merge structural summary and typed conflict/basis loci.
3. Define the verdict kinds and make merge admission return
   `worth-proof::TransitionOutcome` rather than a local `Result` or boolean
   verdict gate.
4. Encode verdict legality and impossible-shape denials inside that typed
   `TransitionOutcome` lane so denied, deferred, stale, rebind-required, and
   failed outcomes remain structurally visible if the merge flow needs them.
5. Prove that merge candidates and merge verdicts remain distinct.

What should exist at the end of the phase:

- merge planning and merge verdicts are explicitly separate surfaces
- conflict/advisory/stale-basis meaning is typed rather than inferred
- later committed-authority work can consume a verdict rather than re-deciding
  merge meaning
- stale-basis meaning is specific enough to support replay, retry, and support
  interpretation without producer-private drift folklore
- strategy-bearing merge semantics are explicit enough that built-in versus
  custom-registered merge behavior survives as shared boundary meaning
- merge-base selection, correspondence basis, and remap basis are explicit
  enough that complex matching or geometry-style basis work can shape merge
  meaning without becoming local folklore

Must ship:

- typed merge intent vocabulary
- typed merge candidate artifact
- merge admission carried by `worth-proof::TransitionOutcome`
- typed transition strategy identity, family, semantic-name, version, and
  ownership vocabulary
- typed transition strategy descriptor digest and contract-basis vocabulary
- typed transition basis identity, family, and version vocabulary
- typed merge structural summary
- typed conflict loci and merge basis vocabulary
- typed merge-base selection basis
- typed transition correspondence basis and remap basis
- typed branch-basis drift vocabulary for stale or invalidated merge meaning
- typed merge verdict with distinct accepted, advisory, conflict, denied,
  superseded, and stale-basis outcomes

Must preserve:

- merge candidates remain descriptive/planning surfaces
- merge verdicts remain distinct from committed authority and receipts
- merge admission must not collapse back to `Result<T, E>` where stale,
  deferred, denied, or rebind-required topology still matters
- verdict kinds preserve real correctness differences rather than collapsing
  into booleans
- stale-basis meaning remains structurally analyzable rather than one vague
  stale status
- built-in and extensible/custom strategy-bearing merge semantics remain visible
  where they materially shaped the verdict
- strategy-bearing merge semantics remain deterministically identifiable enough
  for replay/certification rather than relying on local registry coincidence
- merge-base selection, correspondence, and remap basis remain explicit where
  complex matching rather than obvious identity shaped the merge

Acceptance evidence:

- hostile tests proving merge verdict topology preserves accepted, advisory,
  conflict, denied, superseded, and stale-basis distinctions
- hostile tests proving stale-basis drift kinds preserve target-advanced,
  source-advanced, invalidated-basis, and missing-parent differences
- hostile tests proving strategy-bearing merge verdicts preserve strategy
  identity and ownership instead of collapsing into generic merge outcomes
- hostile tests proving strategy-bearing merge verdicts preserve descriptor
  digest / contract-basis identity rather than only friendly names
- hostile tests proving complex basis-bearing merge verdicts preserve basis
  identity/family/version plus correspondence/remap basis instead of collapsing
  to plain merge conflict labels
- proof-lane tests proving merge admission uses `TransitionOutcome` rather than
  local `Result` or status-flag wrappers
- misuse-pressure tests proving merge planning cannot hide ambient basis choice
  or hidden strategy influence behind cheap convenience entrypoints
- compile-fail or typed-boundary tests proving merge candidates cannot satisfy
  committed-authority or receipt APIs
- blind-consumer tests proving conflict loci and merge basis are interpretable
  without producer-private state

### Phase 3: Define Committed-Authority Transition Law

Purpose:

Introduce the real authority transition boundary and reuse `worth-proof` there
instead of later pretending receipts can prove authority retroactively.

Engineer order:

Only after merge meaning is fully closed should the engineer introduce real
authority transition law. This phase is where branch-local and merge surfaces
stop being merely descriptive and one explicit lane becomes proof-bearing and
authority-changing. The engineer should finish this phase with exactly one real
committed-authority lane and no ambiguity about no-op versus committed
authority.

Practical implementation order:

1. Create the committed-authority home after merge verdicts already exist.
2. Define parent basis and committed-delta summary vocabulary.
3. Add authority-transition outcome kinds including explicit `NoOp` versus
   `Committed`.
4. Reuse `worth-proof` plus Milestone 4 current-basis law for stronger
   committed-authority admission, specifically through
   `AuthorityWitness::from_authority_marker(...)`,
   `Proof::from_authority_witness(...)`, and
   `Artifact::with_proofs_and_current_basis(...)`.
5. Prove that branch-local and merge-verdict surfaces still cannot satisfy the
   stronger lane.

What should exist at the end of the phase:

- there is one real committed-authority transition surface
- no-op versus committed authority is explicit
- stronger authority claims are proof-bearing instead of ambient
- the transition class is explicit enough that metadata-only and promotion-style
  commits do not collapse back into generic commit
- strategy-bearing commit semantics are explicit enough that later intent-style
  commits do not need a second foundational dialect
- no-op causes are explicit enough that convergence, suppression, denial, and
  replay-equivalence meaning do not collapse into one generic no-op

Must ship:

- typed committed-authority artifact
- typed parent-basis vocabulary
- typed canonical ordered parentage vocabulary
- typed merge ancestry basis vocabulary
- typed committed-delta summary vocabulary
- typed authority-transition class vocabulary
- typed no-op cause vocabulary
- typed strategy-bearing committed-authority evidence where a strategy shaped
  the transition
- explicit authority-transition outcome kinds
- proof-bearing committed-authority admission using `worth-proof::Artifact`
- authority-scoped strengthening through
  `AuthorityWitness::from_authority_marker(...)` and
  `Proof::from_authority_witness(...)`
- stronger committed-authority construction through
  `Artifact::with_proofs_and_current_basis(...)`

Must preserve:

- committed authority remains stronger than branch-local or merge-verdict
  meaning
- no-op and committed transitions remain distinct
- metadata-only, promotion, and replay-revalidated transition classes remain
  visible where they matter
- no-op causality remains explicit where later support, replay, or intent
  systems need to distinguish "already true" from "suppressed" or "denied"
- parentage and deltas remain typed and consumer-visible
- parentage remains canonical for unary and multi-parent authority transitions
- strategy-bearing commits remain strategy-identifiable across replay,
  canonicalization, and support surfaces

Acceptance evidence:

- hostile tests proving merge verdicts cannot directly satisfy
  committed-authority APIs
- hostile tests proving distinct authority-transition classes do not collapse
  into one generic committed state
- hostile tests proving strategy-bearing committed transitions preserve strategy
  identity and ownership across authority crossing
- hostile tests proving unary and multi-parent committed transitions preserve
  canonical ordered parentage and merge ancestry meaning
- hostile tests proving no-op causes remain distinct across already-converged,
  basis-equivalent, strategy-suppressed, denied-to-change, and replay-
  equivalent outcomes
- misuse-pressure tests proving committed-authority helpers cannot silently
  bypass proof-bearing authority crossing or hide strategy/basis evidence that
  materially shaped the transition
- proof-lane tests proving committed-authority admission uses
  `AuthorityWitness::from_authority_marker(...)`,
  `Proof::from_authority_witness(...)`, and
  `Artifact::with_proofs_and_current_basis(...)` rather than local pseudo-
  proof wrappers
- blind-consumer tests proving parent basis and committed deltas are
  interpretable without producer-private state

### Phase 4: Define Commit Receipts And Transition Boundary Artifacts

Purpose:

Issue commit receipts only after real committed-authority law exists, and land
them inside the Milestone 4 boundary-artifact language instead of inventing a
second receipt ontology.

Engineer order:

Once the committed-authority lane exists, the engineer may build the public
boundary artifacts that describe it. This phase should not rediscover commit
meaning; it should materialize already-proved authority transitions into
receipts, reports, summaries, and bundles that blind consumers can understand.
The engineer should finish this phase with receipt issuance that depends on
committed authority and with no public path that mints receipts from intent or
verdict alone.

Practical implementation order:

1. Create the receipt home after committed-authority artifacts already exist.
2. Define receipt identity and receipt issuance from committed-authority
   artifacts.
3. Encode the minimum self-describing receipt payload: branch id, commit id,
   parent basis, committed deltas, and authority-transition outcome.
4. Reuse Milestone 4 receipt/artifact categories and bundle law rather than
   inventing new envelope shapes, and keep receipt-bearing stronger claims on
   the `worth-proof::Artifact` lane rather than flattening them into plain
   boundary records.
5. Prove that candidates and merge verdicts still cannot mint receipts.

What should exist at the end of the phase:

- commit receipts are real receipt-bearing boundary artifacts
- receipt issuance depends on committed authority rather than branch-local
  intent
- authority transition meaning is self-describing for blind consumers
- real transition outcomes can emit coordinated receipt/report/summary surfaces
  without local result bags
- strategy-bearing committed transitions can issue receipts that still preserve
  strategy identity and ownership meaning
- branch discard and non-authoritative closeout have explicit typed room where
  a runtime needs positive zero-authority-residue evidence

Must ship:

- typed commit receipt artifact
- typed commit receipt identity
- typed branch-discard or non-authoritative closeout receipt vocabulary
- typed transition provenance-row surface
- typed transition bundle surface for coordinated authority artifacts, receipts,
  verdict reports, and summaries
- typed receipt carriage for strategy-bearing transition evidence where present
- receipt issuance path from committed-authority artifacts
- receipt-bearing stronger claims carried by `worth-proof::Artifact`
- typed denial vocabulary for impossible receipt issuance
- transition-boundary artifact surfaces that compose with Milestone 4 category
  and bundle law

Must preserve:

- receipts attest completed authority transitions only
- receipt meaning remains distinct from merge verdict or branch-local state
- receipt issuance reuses Milestone 4 artifact law instead of rebuilding it
- transition bundles remain typed and legality-checked instead of becoming
  arbitrary surface collections
- transition provenance remains structured and machine-readable rather than
  prose-only explanation
- receipt-bearing strategy evidence remains descriptive of the completed
  transition rather than a second strategy runtime surface
- discard/closeout evidence remains explicitly non-authoritative rather than
  being mistaken for commit receipt evidence
- receipts cannot omit strategy/basis/correspondence evidence when those
  surfaces materially shaped the committed transition they attest

Acceptance evidence:

- hostile tests proving merge candidates, merge verdicts, and staged branch
  surfaces cannot satisfy commit-receipt APIs
- bundle-legality tests proving duplicate or incoherent transition bundle
  members are rejected
- provenance-row tests proving source branch, target branch, parent basis,
  merge basis, transition class, and issuance cause remain blind-consumer
  interpretable
- receipt tests proving strategy-bearing commits preserve strategy identity and
  ownership through receipt issuance where present
- receipt tests proving material strategy/basis/correspondence evidence cannot
  be silently absent when it shaped the attested transition
- proof-lane tests proving receipt issuance stays on the committed-authority
  `worth-proof::Artifact` lane rather than minting plain boundary records that
  merely look authoritative
- misuse-pressure tests proving thin receipts, generic transition result bags,
  and discard/closeout evidence cannot masquerade as real receipt-bearing
  authority evidence
- hostile tests proving discard/closeout receipts cannot satisfy committed
  authority or commit-receipt APIs
- blind-consumer tests proving receipt consumers can recover branch id, commit
  id, parent basis, and committed delta meaning without producer-private state
- compile-fail or typed-boundary tests proving callers cannot mint receipts
  from plain boundary artifacts

### Phase 5: Define Canonical Basis, Locator, And Profile Integration

Purpose:

Make transition surfaces digest-honest, replay-safe, and profile-aware without
rebuilding canonicalization or profile law locally.

Engineer order:

Only after the transition surfaces are already real should the engineer lower
them through the earlier foundational lanes. This phase is not allowed to
invent fresh transition meaning; it must take the branch, merge, committed, and
receipt surfaces from Phases 1 through 4 and make them digest-honest,
locator-addressable, profile-aware, and current-basis-compatible. The engineer
should finish this phase with semantically identical transition artifacts
canonicalizing the same way and with reduced-richness affecting only optional
descriptive detail.

Practical implementation order:

1. Extend the canonical-basis grammar with branch/merge/commit domains and
   entry kinds through the Milestone 2 lane.
2. Define typed transition locators for branch loci, merge-conflict loci,
   parentage loci, and committed-delta loci.
3. Add profile attachment and reduced-richness behavior to the new transition
   artifacts through the Milestone 3 lane.
4. Reuse Milestone 4 current-basis/readmission surfaces where stronger
   current-basis transition artifacts are exposed, specifically
   `Artifact::with_current_basis(...)`, `.bridge_trust_boundary()`,
   `.readmit_with_authority(...)`, and `.rebind_with_authority(...)`.
5. Prove parity across independent producers and honest reduced-richness
   elision.

What should exist at the end of the phase:

- branch/merge/commit artifacts are canonical-basis-ready
- consumers can point at exact transition loci with typed locators
- support/certification/reduced-richness posture is attached through the real
  profile system
- receipts have an explicit evidence floor that remains preserved under
  canonicalization and reduced-richness pressure
- strategy-bearing transition evidence participates canonically rather than as
  opaque provenance text
- basis-bearing and correspondence-bearing transition evidence participates
  canonically rather than as local support prose

Must ship:

- canonical basis participation for branch-local, merge, committed-authority,
  and commit-receipt surfaces
- canonical basis participation for strategy identity/family/version/ownership
  where strategy-bearing transitions exist
- canonical basis participation for basis identity/family/version, observation
  basis, comparison basis, merge-base selection basis, correspondence basis,
  and remap basis where those shaped the transition
- typed transition locator vocabulary
- profile attachment points and reduced-richness law for transition artifacts
- stronger current-basis/readmission reuse where transition surfaces expose it
- explicit reuse of `Artifact::with_current_basis(...)`,
  `.bridge_trust_boundary()`, `.readmit_with_authority(...)`, and
  `.rebind_with_authority(...)` where stronger transition validity is exposed
- explicit receipt evidence floor:
  commit id, branch id, parent basis, authority transition class, committed
  delta evidence or explicit no-op evidence, and current-basis/readmission
  posture where relevant

Must preserve:

- Milestone 2 remains the owner of canonicalization law
- Milestone 3 remains the owner of profile and elision law
- reduced-richness profiles may remove optional forensic detail only
- canonical identity excludes incidental producer layout or ordering
- thin receipts remain impossible; blind-consumer receipt interpretation must
  survive without producer-private lookups
- strategy-bearing transition identity remains canonical and replay-safe rather
  than local provenance theater
- basis-bearing and correspondence-bearing transition identity remains canonical
  and replay-safe rather than local support folklore
- canonicalization must preserve ordered multi-parent ancestry and not treat it
  as incidental metadata

Acceptance evidence:

- canonical-basis parity tests across independent transition producers
- strategy-bearing canonical-basis parity tests across independent producers
- basis-bearing and correspondence-bearing canonical-basis parity tests across
  independent producers
- parentage parity tests proving unary and multi-parent transitions canonicalize
  deterministically across independent producers
- locator tests for branch loci, merge-conflict loci, parentage loci, and
  committed-delta loci
- receipt evidence-floor tests proving every receipt carries the minimum blind-
  consumer authority evidence
- hostile reduced-richness tests proving optional branch/merge detail can be
  elided without changing authoritative commit outcome meaning
- misuse-pressure tests proving profile-aware canonical transition surfaces
  cannot hide basis choice, strategy influence, or evidence-floor weakening
  behind materialization or lowering shortcuts
- proof-lane tests proving current-basis/readmission reuse
  `Artifact::with_current_basis(...)`, `.bridge_trust_boundary()`,
  `.readmit_with_authority(...)`, and `.rebind_with_authority(...)` from the
  Milestone 4 lane rather than rebuilding them locally

### Phase 6: Certify Production-Test Readiness

Purpose:

Close Milestone 5 with a proof-bearing readiness artifact that later
diagnostics, lineage/provenance, and adopting runtimes can depend on.

Engineer order:

Treat this as the final engineering phase, not documentation cleanup. The work
here is to freeze what truly shipped, what the proof lane actually is, which
canonical/profile integrations are real, and which assumptions later milestones
are allowed to rely on. The engineer should finish this phase with a readiness
artifact that can be read as the exact contract between Milestone 5 and every
later milestone or adopting runtime.

Practical implementation order:

1. Inventory the exact surfaces that actually shipped from Phases 1 through 5.
2. Map each certified surface to runtime tests, compile-fail boundaries, and
   blind-consumer evidence.
3. Record runtime assumptions, non-assumptions, and residual debt while the
   implementation details are still concrete.
4. Refuse closure for any phase surface that cannot be tied to real evidence.
5. Freeze the readiness artifact as the only thing later milestones may assume.

What should exist at the end of the phase:

- a later engineer can tell exactly what Milestone 6, 7, and adopting runtimes
  may rely on
- the chosen proof lane and canonical-basis lane are named explicitly
- the milestone closes as a proved authority-transition boundary, not as "the
  transition code seems fine"

Must ship:

- a Milestone 5 production-test readiness artifact or report
- certified-surface inventory for branch-local separation, merge verdict law,
  committed-authority transitions, commit receipts, canonical basis/locator
  integration, and profile-richness behavior
- hostile-pressure inventory for authority separation, merge-topology honesty,
  no-op-versus-commit classification, receipt issuance, replay interpretation,
  and reduced-richness preservation
- exact `worth-proof` appendix naming the mandatory Milestone 5 APIs:
  `TransitionOutcome`,
  `AuthorityWitness::from_authority_marker(...)`,
  `Proof::from_authority_witness(...)`,
  `Artifact::with_proofs_and_current_basis(...)`,
  `Artifact::with_current_basis(...)`,
  `.bridge_trust_boundary()`,
  `.readmit_with_authority(...)`, and
  `.rebind_with_authority(...)`
- compile-fail boundary inventory
- runtime assumptions, non-assumptions, and residual debt

Must preserve:

- later milestones and adopting crates may assume only what the readiness
  artifact names
- local doubles remain semantic fixtures, not generic branch/merge engines
- later milestones still own diagnostics ontology, provenance ontology, and
  full lineage/support semantics beyond the transition surfaces closed here

Acceptance evidence:

- readiness tests proving every certified surface has hostile evidence,
  compile-fail coverage, and blind-consumer interpretation where required
- readiness tests proving every stronger authority/receipt lane names its real
  `worth-proof` APIs and current-basis reuse points
- readiness tests proving ambient basis choice, hidden strategy influence, thin
  receipts, generic transition-result bags, and cheap convenience bypasses were
  all attacked explicitly rather than left to implied coverage
- exact inventory tests for runtime non-assumptions and residual debt
- topology review proving Milestone 5 tests live in responsibility-owned homes

## Must Ship

- typed branch identity, branch fork-basis, and branch-local candidate/staged
  vocabulary
- typed branch observation-basis, fork-observation-basis, and branch-
  comparison-basis vocabulary
- typed merge intent, merge candidate, merge structural summary, merge basis,
  and merge-conflict locus vocabulary
- typed branch-basis drift vocabulary
- typed transition strategy identity, family, semantic-name, version, and
  ownership vocabulary
- typed transition strategy descriptor digest and contract-basis vocabulary
- typed transition basis identity, family, and version vocabulary
- typed merge verdict vocabulary with accepted, advisory, conflict, denied,
  superseded, and stale-basis distinctions
- merge admission carried by `worth-proof::TransitionOutcome`
- typed merge-base selection basis plus typed correspondence/remap basis
- typed committed-authority transition artifact, parent basis, and committed-
  delta summary vocabulary
- typed canonical ordered parentage and merge ancestry basis vocabulary
- typed authority-transition class vocabulary
- typed no-op cause vocabulary
- typed strategy-bearing committed-authority evidence
- explicit `NoOp` versus `Committed` authority-transition outcome kinds
- proof-bearing committed-authority admission using `worth-proof::Artifact`
- authority-scoped strengthening through
  `AuthorityWitness::from_authority_marker(...)` and
  `Proof::from_authority_witness(...)`
- stronger committed-authority construction through
  `Artifact::with_proofs_and_current_basis(...)`
- typed commit receipt artifact and receipt identity
- typed branch-discard or non-authoritative closeout receipt vocabulary
- typed transition provenance-row surface
- typed transition bundle surface for coordinated authority/report/receipt
  emission
- typed receipt carriage for strategy-bearing transition evidence where present
- receipt issuance from committed-authority artifacts rather than candidates or
  plain descriptive surfaces
- canonical basis participation for branch-local, merge, committed-authority,
  and receipt surfaces using the Milestone 2 lane
- typed transition locators for branch, merge-conflict, parentage, and
  committed-delta loci
- profile attachment and reduced-richness behavior for transition artifacts
  using the Milestone 3 lane
- current-basis/readmission reuse for stronger transition artifacts using the
  Milestone 4 lane where exposed
- explicit reuse of `Artifact::with_current_basis(...)`,
  `.bridge_trust_boundary()`, `.readmit_with_authority(...)`, and
  `.rebind_with_authority(...)` where stronger transition validity is exposed
- explicit receipt evidence floor for blind-consumer authority interpretation
- production-test readiness artifact for Milestone 5

## Must Preserve

- branch-local candidate and staged state remain distinct from committed
  authority
- merge candidates, merge verdicts, committed authority, and commit receipts
  remain structurally distinct
- merge outcomes preserve accepted, advisory, conflict, denied, superseded, and
  stale-basis differences
- stale-basis meaning preserves distinct drift causes where recovery or replay
  behavior differs
- no-op and committed authority transitions remain distinct
- authority transition classes remain visible where metadata-only, promotion,
  or replay-revalidated meaning matters
- built-in and extensible/custom strategy-bearing transition semantics remain
  visible where they materially shaped merge or commit meaning
- strategy-bearing transition semantics remain deterministically identifiable
  enough for replay/certification, not just human-readable by name
- observation/comparison/merge-base/correspondence/remap basis remain first-
  class where they materially shaped transition truth
- no-op causality remains explicit where later runtimes need to distinguish
  convergence, suppression, denial, and replay-equivalence
- receipts attest completed authority transitions only
- parentage, merge basis, and committed deltas remain typed and self-
  describing
- multi-parent ancestry remains canonical and first-class rather than optional
  metadata
- transition provenance remains structured and machine-readable
- coordinated transition emission remains typed and legality-checked rather than
  arbitrary collections
- discard and non-authoritative closeout remain explicitly non-authoritative
- Milestone 2 remains the owner of canonicalization law
- Milestone 3 remains the owner of profile and reduced-richness law
- Milestone 4 remains the owner of boundary-artifact categories, current-basis
  strengthening, and receipt/artifact materialization law
- foundational does not standardize one branch graph, transaction journal,
  conflict engine, or durability runtime

## Acceptance Evidence

- branch-local separation hostility tests
- merge-verdict topology tests proving accepted, advisory, conflict, denied,
  superseded, and stale-basis remain distinct
- drift-structure tests proving stale-basis causes remain distinct
- strategy-bearing merge and commit tests proving strategy identity, family,
  version, and ownership remain visible and canonical where they shaped the
  transition
- strategy descriptor/contract-basis tests proving replay-grade extensibility
  identity is stronger than a friendly strategy name
- basis-bearing tests proving observation/comparison/merge-base/correspondence/
  remap basis remain visible and canonical where they shaped the transition
- proof-lane tests proving merge admission uses `TransitionOutcome` and
  committed-authority / receipt / current-basis lanes use the named
  `worth-proof::Artifact` APIs rather than local pseudo-proof wrappers
- compile-fail tests proving branch-local candidate, staged branch state, merge
  candidate, and merge verdict cannot satisfy committed-authority or receipt
  APIs
- transition-class tests proving metadata-only, promotion, no-op, and ordinary
  commit outcomes remain distinguishable
- bundle-legality tests proving coordinated transition emission rejects
  duplicate or incoherent members
- provenance-row tests proving transition explanation is structured and blind-
  consumer interpretable
- misuse-pressure tests proving ambient basis choice, hidden strategy
  influence, thin receipts, generic transition-result bags, and cheap
  convenience bypasses are all rejected or made structurally visible
- blind-consumer tests proving branch basis, merge basis, parentage, conflict
  loci, and committed deltas are interpretable without producer-private state
- canonical-basis parity tests across independent transition producers
- parentage parity tests proving unary and multi-parent ancestry canonicalize
  deterministically across independent producers
- locator tests for branch loci, merge-conflict loci, parentage loci, and
  committed-delta loci
- receipt evidence-floor tests proving thin receipts are impossible
- discard/closeout hostility tests proving non-authoritative closeout evidence
  cannot satisfy committed-authority or commit-receipt APIs
- hostile reduced-richness tests proving optional forensic branch/merge detail
  can be elided without changing authoritative commit outcome meaning
- readiness artifact tests covering certified surfaces, hostile pressures,
  compile-fail boundaries, `worth-proof` appendix, assumptions,
  non-assumptions, and debt

## Architectural Notes

The implementation should preserve distinct transition responsibility homes. A
likely shape is:

```text
crates/worth-foundational/src/
  transitions/
    branches/
    merges/
    commits/
    receipts/
    basis/
    readiness/
```

Public exports should remain facade-controlled. The root may exist, but it must
not become an unnamed bucket where branch ids, merge verdicts, committed
authority, receipts, basis lowering, and readiness reporting collapse into one
file.

The likely structural split is:

- `branches/` or equivalent owns branch ids, fork basis, candidate identity,
  and branch-local candidate/staged vocabulary
- `merges/` or equivalent owns merge intent, merge candidates, structural
  summaries, verdict kinds, and conflict loci
- `commits/` or equivalent owns committed-authority artifacts, parent basis,
  delta summaries, and authority-transition outcomes
- `receipts/` or equivalent owns commit receipt issuance and receipt identity
- `basis/` or equivalent owns canonical basis lowering and typed locators for
  transition artifacts
- `readiness/` or equivalent owns the milestone closeout artifact

Transition outputs should also preserve the split:

- branch-local and merge surfaces are descriptive/planning surfaces until they
  cross the committed-authority lane
- committed-authority and receipt-bearing surfaces are stronger claims and must
  use `worth-proof`
- profile decisions attach as derivative boundary evidence, not as authority
- later diagnostics, lineage, and provenance attachments remain named hooks
  here, while their deeper ontologies remain later milestone work

## Desired DX End State

Milestone 5 should not finish as "ids, enums, and receipts." It should finish
as a layered transition-authoring surface where the common path reads like
branch-local intent, merge admission, committed authority, and receipt issuance,
while deeper paths expose basis, drift, breadth, and provenance explicitly.

The finished developer experience should follow these rules:

- branch-local path: opening candidate or staged branch work should feel scoped
  and obviously non-authoritative
- merge path: callers should plan first, then inspect structure, then admit a
  real verdict
- authority path: committing should look visibly stronger than planning or
  describing
- receipt path: receipts should be issued from committed authority, not
  hand-assembled
- support path: provenance rows, drift, loci, and basis should be readable
  before or alongside materialized boundary artifacts
- bundle path: coordinated authority/report/receipt emission should be typed
  rather than result-bag folklore

The finished code should also have an explicit surface grammar instead of a
grab-bag of constructors:

- noun entrypoints: `transitions::branch_candidate()`, `transitions::merge()`,
  `committed.issue_receipt()`, `committed.emit_transition_bundle()`, and
  `candidate.discard_with_zero_residue_proof()` should be the normal front
  doors because they name semantic intent directly
- object-spec inputs where the whole semantic shape is known at once: branch
  ids, strategy descriptors, basis identities, parentage records, and receipt
  evidence floors should be defined as complete objects rather than accumulated
  through long setter chains
- builder progression where ordered proof or phase progression matters:
  candidate creation, merge planning, merge admission, authority commitment,
  bundle emission, and current-basis/readmission strengthening should read like
  ordered transitions where only valid next steps are in autocomplete
- inspectable plan surfaces before execution: merge planning and any authority-
  crossing preparation should expose cost, touched scope, basis, drift,
  locality, decision rows, and strategy influence before the caller crosses the
  stronger lane
- obviously expensive verbs: `plan()`, `admit()`, `commit_with(...)`,
  `issue_receipt()`, `materialize()`, `bridge_trust_boundary()`, and
  `readmit_with(...)` should look expensive and boundary-bearing; none of them
  should masquerade as field access or cheap conversion
- no cheap flattening conversions: candidate, verdict, committed-authority,
  receipt, discard, and bundle surfaces should not quietly deref or coerce into
  plain payloads, plain ids, or generic bags

The implementation should converge toward a layered authoring model:

- friendly lane: semantic entrypoints that read like branch-local intent, merge
  planning, authority crossing, and receipt issuance
- accountability lane: inspectable plans, summaries, drift reports, decision
  rows, and provenance rows that explain what the stronger lane will or did do
- proof lane: explicitly stronger `worth-proof`-bearing committed-authority,
  current-basis, receipt, and readiness surfaces
- support lane: summaries, reports, and bundles that let blind consumers
  interpret transitions without producer-private state

That means the same operation should normally have both a readable authoring
surface and a lower inspectable surface. The common path should not require the
caller to manually assemble parentage, strategy evidence, receipt envelopes, or
bundle members, but the lowered path must still make those things visible and
auditable before or after the boundary crossing that actually needs them.

The intended finished module shape should feel approximately like this:

```rust
use worth_foundational::transitions::{
    self,
    basis::TransitionBasisIdentity,
    branches::{BranchId, BranchForkBasis},
    commits::AuthorityTransitionClass,
    merges::MergeVerdictKind,
    receipts::CommitReceiptIdentity,
    strategies::TransitionStrategyIdentity,
};
```

The intended branch-local path should separate branch identity, fork basis,
observation basis, and staged content clearly enough that the call site itself
teaches "this is not authority yet":

```rust
let candidate = transitions::branch_candidate()
    .on_branch(feature_branch)
    .from_fork_basis(branch_fork_basis)
    .from_current_basis(main_basis)
    .under_observation_basis(observation_basis)
    .stage(changes)?;

candidate.branch_id();
candidate.branch_local_state_kind();
candidate.fork_basis();
candidate.observation_basis();
candidate.transition_basis_identity();
```

If the caller needs a fuller branch-local intent surface, that should still
read like intent rather than ad hoc mutation:

```rust
let staged = transitions::branch_candidate()
    .on_branch(feature_branch)
    .from_fork_basis(branch_fork_basis)
    .from_current_basis(main_basis)
    .under_observation_basis(observation_basis)
    .stage(changes)?
    .staged()?;

staged.branch_id();
staged.fork_basis();
staged.transition_basis_family();
staged.transition_basis_version();
```

The intended branch-local path should look like:

```rust
let candidate = transitions::branch_candidate()
    .on_branch(feature_branch)
    .from_current_basis(main_basis)
    .under_observation_basis(observation_basis)
    .stage(changes)?;
```

The intended merge path should expose explicit planning before verdict
admission:

```rust
let plan = transitions::merge()
    .source(feature_branch)
    .target(main_branch)
    .plan()?;

plan.structural_summary();
plan.merge_basis();
plan.merge_base_selection_basis();
plan.correspondence_basis();
plan.remap_basis();
plan.conflict_loci();
plan.branch_basis_drift();
plan.touched_scope();
plan.parallel_admission();
plan.strategy_identity();
plan.strategy_descriptor_digest();
plan.transition_basis_identity();
plan.decision_rows();
plan.cost();
plan.explain();
```

The intended merge-verdict path should read like admitted transition meaning,
not mutable merge state:

```rust
let verdict = transitions::merge()
    .source(feature_branch)
    .target(main_branch)
    .plan()?
    .admit()?;

verdict.kind();
verdict.merge_basis();
verdict.merge_base_selection_basis();
verdict.correspondence_basis();
verdict.remap_basis();
verdict.conflict_loci();
verdict.branch_basis_drift();
verdict.strategy_identity();
verdict.strategy_descriptor_digest();
verdict.transition_basis_family();
verdict.transition_basis_version();
verdict.parentage_expectation();
verdict.explain();
```

The intended authority path should be visibly stronger than merge admission:

```rust
let committed = verdict.commit_with(authority_witness)?;

committed.transition_class();
committed.noop_cause();
committed.parent_basis();
committed.parentage();
committed.merge_ancestry_basis();
committed.committed_delta_summary();
committed.strategy_identity();
committed.strategy_descriptor_digest();
committed.strategy_contract_basis();
committed.transition_basis_identity();
committed.proofs();
committed.explain();
```

The intended receipt path should be derived from committed authority:

```rust
let receipt = committed.issue_receipt()?;

receipt.commit_id();
receipt.branch_id();
receipt.parent_basis();
receipt.parentage();
receipt.transition_class();
receipt.noop_cause();
receipt.delta_evidence();
receipt.strategy_identity();
receipt.strategy_descriptor_digest();
receipt.transition_basis_identity();
receipt.receipt_identity();
receipt.proofs();
receipt.explain();
```

The intended bundle path should give real coordinated room without arbitrary
surface bags:

```rust
let emitted = committed
    .emit_transition_bundle()
    .with_summary()
    .with_merge_report()
    .with_receipt()
    .materialize()?;

let authority = emitted.primary();
let summary = emitted.summary();
let report = emitted.merge_report();
let receipt = emitted.receipt();

emitted.transition_class();
emitted.strategy_identity();
emitted.transition_basis_identity();
emitted.materialization_cost();
```

The intended provenance/support path should be row-bearing and typed:

```rust
for row in receipt.transition_provenance_rows() {
    row.source_branch();
    row.target_branch();
    row.parent_basis();
    row.merge_basis();
    row.transition_class();
    row.strategy_identity();
    row.strategy_ownership();
    row.observation_basis();
    row.correspondence_basis();
    row.remap_basis();
    row.issuance_cause();
    row.transition_locator();
    row.receipt_identity();
}
```

The intended strategy-bearing path should make extensibility look first-class
without turning `worth-foundational` into a strategy runtime:

```rust
let verdict = transitions::merge()
    .source(feature_branch)
    .target(main_branch)
    .under_strategy(transition_strategy)
    .plan()?
    .admit()?;

verdict.strategy_identity();
verdict.strategy_family();
verdict.strategy_version();
verdict.strategy_ownership();
verdict.strategy_descriptor_digest();
verdict.strategy_contract_basis();
verdict.strategy_basis();
```

The intended complex-basis path should make basis-bearing matching and remap
reasoning feel native rather than bolted on later. This matters for things like
geometry-kernel-heavy query runtimes where correspondence itself shapes truth:

```rust
let verdict = transitions::merge()
    .source(feature_branch)
    .target(main_branch)
    .under_strategy(transition_strategy)
    .under_comparison_basis(comparison_basis)
    .under_correspondence_basis(correspondence_basis)
    .under_remap_basis(remap_basis)
    .plan()?
    .admit()?;

verdict.transition_basis_identity();
verdict.transition_basis_family();
verdict.transition_basis_version();
verdict.correspondence_basis();
verdict.remap_basis();
verdict.conflict_loci();
```

The intended non-authoritative closeout path should be explicit too:

```rust
let discard = candidate.discard_with_zero_residue_proof()?;

discard.branch_id();
discard.fork_basis();
discard.closeout_cause();
discard.non_authoritative_residue_report();
discard.transition_basis_identity();
discard.explain();
```

The intended replay/readmission path should also remain visibly stronger than
plain observation:

```rust
let bridged = committed.bridge_trust_boundary();
let readmitted = bridged.readmit_with(readmission_authority)?;

readmitted.current_basis();
readmitted.transition_basis_identity();
readmitted.proofs();
readmitted.explain();
```

The implementation should make several bad call-site shapes impossible or at
least obviously discouraged:

- no `commit(branch, changes, true, false)` style positional authority calls
- no `Receipt::new(...)` public constructor from plain ids and metadata
- no generic `TransitionResult { status, artifact, receipt }` bags
- no hidden branch drift or strategy influence that only appears in logs
- no ambient basis choice where the caller cannot tell which comparison,
  correspondence, or remap basis shaped the verdict
- no cheap `as_payload()` or `Deref` path that erases candidate, verdict,
  committed-authority, or receipt boundaries
- no convenience helpers that silently perform planning, authority crossing,
  receipt issuance, and bundle materialization in one uninspectable step

The intended stale-basis path should be explicit enough for retry/replay
reasoning:

```rust
match verdict.branch_basis_drift() {
    Some(drift) if drift.is_target_advanced() => retry_against_new_target_basis(),
    Some(drift) if drift.is_source_advanced() => rebuild_source_candidate(),
    Some(drift) if drift.is_merge_basis_invalidated() => recompute_merge_basis(),
    Some(drift) if drift.is_parent_basis_unavailable() => request_readmission(),
    None => continue_flow(),
}
```

These examples are not cosmetic. They describe the actual DX target the
implementation should converge toward:

- callers express branch-local intent, merge planning, committed authority, and
  receipt issuance by semantic name
- merge breadth, basis, drift, and decision rows are inspectable before
  authority crossing
- authority-bearing lanes are visibly stronger than planning or descriptive
  lanes
- receipts are issued from committed authority rather than assembled from local
  metadata
- coordinated transition emission is typed rather than bag-shaped
- strategy-bearing transitions preserve strategy identity and ownership without
  leaking runtime registry mechanics into foundational APIs
- strategy-bearing transitions preserve deterministic descriptor/contract
  identity strong enough for replay and certification
- observation/comparison/merge-base/correspondence/remap basis preserve the
  truth-shaping basis story without leaking runtime matching engines into
  foundational APIs
- unary and multi-parent ancestry remain equally first-class at the boundary
- no-op causality and non-authoritative closeout remain first-class instead of
  dissolving into vague status strings
- provenance and stale-basis structure are machine-readable instead of prose-
  only

If the final call sites do not feel approximately like these examples, the
milestone has likely left convergence and ergonomics value on the table.

## Sequencing Notes

Milestone 5 belongs immediately after boundary artifacts because it is the first
milestone that needs real authority-transition language to land inside those
categories:

- Milestone 4 created the category/materialization/current-basis lanes but
  intentionally fail-closed real authority transitions
- Milestone 5 replaces that fail-closed placeholder with real branch/merge/
  commit ontology
- Milestone 6 diagnostics need transition nouns before they can explain merge
  and commit outcomes honestly
- Milestone 7 lineage/provenance/receipt work needs typed commit parentage,
  deltas, and receipts before it can attach deeper history or provenance law
- Milestone 11 migrations need this shared transition vocabulary before runtime
  crates can retire local transaction/preview/commit dialects

This milestone must remain after Milestone 4 because it consumes boundary
artifact categories, receipts, and current-basis strengthening. It must also
remain before diagnostics and provenance, because those later milestones need
shared transition nouns to attach to.

## Explicit Non-Goals

- one transaction engine or one speculative session runtime
- one merge policy engine or one conflict-resolution runtime
- one strategy registry, hook executor, plugin loader, or callback runtime
- one correspondence engine, remap engine, or geometry kernel implementation
- one durability journal or one branch graph layout
- full diagnostics ontology
- full provenance or lineage ontology
- migration closure for adopting runtimes
- replacing Milestone 2 canonicalization or Milestone 4 boundary-artifact law

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes. It replaces the fail-closed transition placeholder with a
  real shared authority-transition language that later milestones and adopting
  runtimes actually need.
- Is the adversarial constraint precise and load-bearing? Yes. It attacks
  candidate-versus-commit confusion, flattened merge outcomes, receipt
  overclaiming, producer-private history interpretation, and truth-changing
  reduced-richness behavior.
- Does the milestone preserve crate authority boundaries? Yes.
  `worth-foundational` owns shared transition meaning and proof-composed
  boundary language; domain crates keep execution, conflict policy, journal
  layout, and durability mechanics.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes. Closure requires authority-separation hostility, merge-topology proof,
  proof-lane proof, canonical-basis parity, reduced-richness hostility, and
  readiness evidence.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes. The phases map directly to branches, merges, commits, receipts,
  basis/locator integration, and readiness.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes. It depends on canonicalization, profiles, and boundary artifacts, and it
  is a prerequisite for diagnostics, provenance, and real runtime migrations.
