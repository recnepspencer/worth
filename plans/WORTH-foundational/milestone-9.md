# Milestone 9: Scoped Merge Selection And Cherry-Pick Vocabulary

## Goal

Extend `worth-foundational` so branch merge scope, selected-node merge,
selected-aspect merge, cherry-pick posture, skipped-out-of-scope evidence, and
scope-denial topology have one shared transition vocabulary before adopting
crates add scoped merge execution.

This milestone does not make `worth-foundational` a merge runtime. It defines
the shared boundary language that real runtimes, especially `worth-signal`, can
lower into when they plan and execute scoped merges.

The target outcome is:

- a scoped merge request has canonical shared meaning
- full-branch merge remains the explicit default scope
- selected-node and selected-aspect merge are distinct scope families
- cherry-pick is represented as native merge scope, not as a consumer-side
  filter over already-planned records
- no-op selected scope, skipped-out-of-scope candidates, unavailable scope
  materialization, and denied scope admission are named artifacts
- adopting crates can use foundational scope vocabulary without abandoning
  their local optimized merge candidate storage

## Why This Milestone Exists

Milestone 5 shipped the shared branch/merge/commit transition language:
branch-local candidates, staged branches, merge candidates, merge verdicts,
committed authority, receipts, transition basis, locators, and readiness.

That surface is strong, but it intentionally stops at broad merge topology:

- `FoundationalMergeStructuralSummary` counts source, target, touched, and
  conflict-check breadth.
- `FoundationalMergeConflictLocus` names conflicts.
- `FoundationalMergeCandidate` carries target branch, intent, strategy,
  merge-basis, correspondence, and remap evidence.

It does not yet define what a selected merge scope means. That omission is now
load-bearing because `worth-signal-wasm` needs selected-node and selected-aspect
merge for honest branch materialization. If `worth-signal` invents local scoped
merge vocabulary first, WORTH repeats the exact drift `worth-foundational`
exists to prevent.

## Governing Source Summaries

- `MENTALITY.md`
  Protects hard-problem-first engineering. Scoped merge must become shared
  language before demos or wasm surfaces normalize local cherry-pick folklore.
- `arch_laws.md`
  Protects lowered plans and proof-bearing phase progression. Scope selection
  must be planned before merge admission and execution, not rediscovered by
  materializers.
- `composition_laws.md`
  Protects semantic compilation units. Scope request vocabulary, admitted
  scope, skipped scope, no-op scope, denial topology, canonicalization, and
  readiness evidence must remain separate homes.
- `domain_structure_laws.md`
  Protects authority boundaries. Foundational owns scope meaning; adopting
  crates own runtime-specific scope planning, execution, and storage layout.
- `perf_laws.md`
  Protects cost honesty. Scope width, selected width, skipped width, conflict
  width, and materialization breadth must be visible rather than hidden behind
  cheap-looking cherry-pick APIs.
- `worth_foundational_vision.md`
  Protects shared meaning without shared runtime representation. This milestone
  standardizes branch/merge selection meaning, not one merge planner.
- `worth_foundational_roadmap.md`
  Protects the rule that branching, merging, and commits have one shared
  authority-transition language.
- `test-requirements.md`
  Protects local hostile proof before adopting-crate migration. Scoped merge
  vocabulary must be certified with synthetic producers before `worth-signal`
  depends on it.
- `milestone-5-closeout.md`
  Protects the existing merge candidate and verdict lane. This milestone must
  extend that lane without reopening committed authority, receipt, or proof
  progression law.

## Adversarial Constraint

Two WORTH crates with different local merge planners must be able to describe
the same scoped merge boundary in shared foundational vocabulary:

- one selects an entire source branch journal
- one cherry-picks a selected source node
- one cherry-picks selected aspects on one node
- one maps selected source aspects through identity correspondence to a target
  node with a different local id
- one selects a node that changed nothing
- one skips unselected candidate records
- one denies an unknown, ambiguous, deleted, declaration-rejected,
  unsupported, or non-adoptable selected scope

If equivalent scoped merge meaning can materialize as different foundational
categories, or if a consumer must inspect producer-private runtime state to
understand what was selected, skipped, denied, or admitted, this milestone has
failed.

## Product Decision Lock

- Full-branch merge is a named scope, not the absence of scope meaning.
- Cherry-pick is selected merge scope. It is not filtering, hiding, or deleting
  records after a broader merge was planned.
- Selected nodes and selected aspects are different scope families.
- Selected-aspect scope must preserve whether selection is by source node, by
  target-corresponded node, or unavailable because correspondence is ambiguous.
- Selected-but-unchanged scope is a first-class no-op outcome.
- Skipped-out-of-scope evidence is compact by default but must be countable and
  digestible.
- Foundational scope vocabulary must stay descriptive and boundary-facing.
  Adopting crates may keep local graph ids, journals, aspect sets, and policy
  registries internally.
- Scope admission and scope non-success must use the existing `worth-proof`
  progression categories. This milestone must not invent a parallel proof
  outcome enum for scoped merge.

## Architectural Model

### Scope Request Vocabulary

This milestone should introduce shared request vocabulary equivalent to:

- `FoundationalMergeScope::FullBranch`
- `FoundationalMergeScope::SelectedNodes`
- `FoundationalMergeScope::SelectedAspects`

The exact Rust names may change, but the semantic split may not.

Selected-node scope must carry selected node locators or identity handles that
are boundary-safe. Selected-aspect scope must carry selected node identity plus
selected aspect locators or aspect keys.

### Admitted Scope Vocabulary

An admitted scope artifact must answer:

- what scope family was requested
- what source branch and target branch the scope belongs to
- which selected nodes were admitted
- which selected aspects were admitted
- whether admission used direct source-node identity or identity
  correspondence
- which selected entries were no-ops
- what compact evidence explains skipped-out-of-scope candidates

### Scope Denial Vocabulary

Scope denial must distinguish:

- unknown selected node
- unknown selected aspect
- selected node not present in source scope
- selected node deleted before admission
- selected target correspondence ambiguous
- selected target correspondence rejected by declaration
- selected node non-adoptable
- selected aspect unsupported by the selected node or strategy
- scope family rejected by the adopting declaration

### Canonical Basis And Locator Participation

Scoped merge vocabulary must participate in the existing transition canonical
basis lane. Ordering must be deterministic across construction paths:

- selected nodes canonicalize by locator identity
- selected aspects canonicalize by node locator, then aspect locator
- skipped-out-of-scope evidence canonicalizes by scope family and count/digest
- denial evidence canonicalizes by denial kind and selected locus

### Relationship To Adopting Crates

`worth-signal` should later lower:

- `BranchMergeRequest` scope into foundational scope request vocabulary
- planned candidates into admitted/skipped scope evidence
- identity correspondence into selected-aspect correspondence basis
- aspect decision plans into admitted selected-aspect evidence
- scope failures into foundational scope denials

`worth-signal-wasm` should consume that lowered meaning. It must not create a
parallel scoped merge ontology in TypeScript.

### WORTH-Proof Usage Boundary

This milestone extends `worth-foundational`, but the actual crate already uses
`worth-proof` for transition progression:

- merge admission returns `worth_proof::TransitionOutcome` through
  `FoundationalMergeAdmissionOutcome`
- committed authority and commit receipts use `worth_proof::Artifact`,
  `AuthorityWitness`, `Proof`, and current-basis freshness wrappers
- current-basis trust boundaries use `bridge_trust_boundary` and
  `readmit_with_authority`

Milestone 9 must follow that existing boundary:

- plain scope vocabulary such as full-branch, selected-node, selected-aspect,
  skipped, no-op, and scoped denial records belongs in `worth-foundational`
- scope construction denials may remain ordinary construction errors because
  they happen before a merge candidate exists
- scope admission must lower through the existing
  `FoundationalMergeAdmissionOutcome` / `worth_proof::TransitionOutcome` lane
  or a directly equivalent Foundational outcome alias
- admitted scoped merge evidence that becomes commit-eligible must be carried
  by the existing merge verdict, committed authority, receipt, and current-basis
  artifact families rather than by a new proof carrier
- trust-boundary restoration for scoped merge artifacts must use the existing
  current-basis readmission pattern rather than ad hoc revalidation flags

`worth-proof::TransitionOutcome` currently has these categories:

- `Success`
- `Denied`
- `Deferred`
- `Stale`
- `RebindRequired`
- `Failed`

Therefore `scope unavailable` is not a new top-level proof outcome. It is a
Foundational domain posture that must be carried by the correct proof category:

- invalid selected scope is `Denied`
- capability not available yet is `Deferred`
- stale or insufficient correspondence/basis is `Stale` or `RebindRequired`
- machinery failure is `Failed`

If a later implementation discovers that these categories cannot honestly carry
scoped merge unavailability, the required first step is a `worth-proof`
milestone to extend progression law before `worth-foundational` claims a new
proof outcome shape.

## Phases

### Phase 1: Full-Branch Scope Vocabulary

Purpose:

- make the current default merge shape explicit before adding narrower scopes

This phase must ship:

- public full-branch scope request type through the `worth-foundational` facade
- full-branch scope family identity and stable semantic name
- full-branch scope summary row that can preserve current
  `FoundationalMergeStructuralSummary` meaning
- compile-fail coverage proving a raw absence of scope cannot satisfy APIs that
  require an explicit merge scope

Phase 1 gate:

- existing broad merge candidates can carry explicit full-branch scope without
  changing their current merge verdict semantics.

### Phase 2: Selected Node Locus Vocabulary

Purpose:

- define the address vocabulary for selecting whole nodes without defining
  selected-aspect behavior yet

This phase must ship:

- selected-node locus wrapper
- node-locus construction denial for empty or malformed boundary identity
- canonical ordering law for selected node loci
- compile-fail coverage proving raw strings/integers cannot satisfy selected
  node locus APIs

Phase 2 gate:

- selected-node requests can be represented as typed loci independent of any
  merge candidate or runtime planner.

### Phase 3: Selected Aspect Locus Vocabulary

Purpose:

- define the address vocabulary for selecting aspects on a selected node

This phase must ship:

- selected-aspect locus wrapper
- selected-aspect request entry combining selected node locus and aspect locus
- canonical ordering law by node locus, then aspect locus
- construction denial for empty selected-aspect request entries
- compile-fail coverage proving selected nodes and selected aspects are not
  substitutable categories

Phase 3 gate:

- selected-aspect requests can be represented without relying on producer
  metadata, payload fields, or stringly aspect names.

### Phase 4: Scope Request Builder

Purpose:

- assemble full-branch, selected-node, and selected-aspect requests through one
  validated scope request surface

This phase must ship:

- `FoundationalMergeScope` or equivalent public request enum
- constructors for full-branch, selected-node, and selected-aspect scopes
- empty selected-node and empty selected-aspect request denial
- duplicate selected-node and duplicate selected-aspect denial or canonical
  de-duplication with explicit proof
- producer-diversity tests showing two construction orders produce the same
  request meaning

Phase 4 gate:

- scoped merge request meaning is canonical before it is attached to merge
  candidates.

### Phase 5: Merge Candidate Scope Attachment

Purpose:

- attach explicit scope request proof to the existing merge candidate lane

This phase must ship:

- builder method equivalent to `with_scope(...)`
- `FoundationalMergeCandidate` accessor for requested scope
- construction denial for missing scope, unless compatibility explicitly
  lowers legacy candidates to full-branch scope
- canonical behavior for legacy/full-branch compatibility
- tests proving scope request is not smuggled through payload metadata

Phase 5 gate:

- every new scoped merge candidate carries request scope as first-class
  transition evidence.

### Phase 6: Admitted Scope Evidence

Purpose:

- describe which selected loci actually participated in the merge candidate

This phase must ship:

- admitted scope artifact
- admitted selected-node evidence
- admitted selected-aspect evidence
- direct-source-node versus identity-corresponded admission basis
- scope breadth summary with requested, admitted, and conflict-check widths
- merge candidate/verdict accessors for admitted scope evidence
- admission through `FoundationalMergeAdmissionOutcome` so success, denial,
  deferred, stale, rebind-required, and failed scoped outcomes preserve the
  existing `worth-proof` progression categories

Phase 6 gate:

- a consumer can tell which selected nodes or aspects participated without
  reading producer-private state, and successful scoped admission travels
  through the same merge verdict lane as existing full-branch admission.

### Phase 7: Selected No-Op Evidence

Purpose:

- make selected-but-unchanged work visible without treating it as skipped or
  denied

This phase must ship:

- selected no-op artifact
- no-op cause vocabulary for unchanged source truth and equivalent target truth
- no-op breadth counter
- merge verdict accessors for selected no-op evidence
- tests proving no-op selected loci remain distinct from skipped-out-of-scope
  and denied loci

Phase 7 gate:

- selected work that changes nothing remains explainable and digestible.

### Phase 8: Skipped-Out-Of-Scope Evidence

Purpose:

- explain what a scoped merge intentionally left outside the merge

This phase must ship:

- compact skipped-out-of-scope evidence
- skipped candidate count
- optional skipped digest when richer evidence is retained
- skipped breadth counter
- tests proving skipped evidence is stable across producer ordering and does
  not require materializing every skipped record by default

Phase 8 gate:

- scoped merge can explain excluded candidates without hiding broad scans or
  forcing rich replay materialization.

### Phase 9: Scope Denial Topology

Purpose:

- make invalid selected scopes deny as scoped-merge failures rather than
  generic policy failures

This phase must ship:

- typed scope denial variants for unknown node, unknown aspect, source-missing
  node, deleted selected node, ambiguous correspondence, declaration-rejected
  correspondence, non-adoptable selected node, unsupported selected aspect, and
  declaration-rejected scope family
- denied selected-locus evidence
- admission APIs that preserve no-side-effect denial semantics
- hostile tests for every denial family

Phase 9 gate:

- invalid scoped merge requests cannot collapse into ordinary merge denials
  without preserving scope-specific evidence.

### Phase 10: Scope Unavailable Posture

Purpose:

- distinguish unsupported runtime capability from invalid user scope

This phase must ship:

- typed scope-unavailable posture
- unavailable reason vocabulary for runtime-does-not-support-selected-nodes,
  runtime-does-not-support-selected-aspects, materializer-unavailable,
  identity-correspondence-unavailable, and retained-proof-unavailable
- mapping from each unavailable reason to the correct
  `worth-proof::TransitionOutcome` category: `Deferred`, `Stale`,
  `RebindRequired`, or `Failed`
- tests proving unavailable posture is not substitutable for invalid-scope
  `Denied`, selected no-op, or skipped evidence

Phase 10 gate:

- adopting crates can honestly say "this scope family is not supported here"
  without pretending the request was invalid and without inventing a proof
  outcome category outside `worth-proof`.

### Phase 11: Canonical Basis Participation

Purpose:

- make scoped merge evidence reproducible across construction paths

This phase must ship:

- canonical basis entries for scope request
- canonical basis entries for admitted scope
- canonical basis entries for selected no-op evidence
- canonical basis entries for skipped-out-of-scope evidence
- canonical basis entries for denial and unavailable posture
- ordering-hostility tests for selected node and selected aspect inputs

Phase 11 gate:

- scoped merge artifacts produce stable canonical basis entries without
  producer-private state.

### Phase 12: Transition Locator Participation

Purpose:

- let diagnostics, provenance, and receipts point at selected scope loci

This phase must ship:

- transition locator variants for selected node scope
- transition locator variants for selected aspect scope
- locator-to-canonical-basis lowering
- tests proving merge-conflict locators, selected-node locators, and
  selected-aspect locators remain distinct

Phase 12 gate:

- scoped merge evidence can be addressed by shared locator vocabulary.

### Phase 13: Diagnostics And Explanation Attachment

Purpose:

- attach scoped merge evidence to the Milestone 6 explanation ontology without
  recreating transition meaning

This phase must ship:

- diagnostic/explanation attachment compatibility for scope request, admitted
  scope, skipped scope, no-op scope, denial, and unavailable posture
- reduced-richness behavior that can keep compact scope facts while eliding
  rich selected-locus detail
- tests proving diagnostics consume scoped merge artifacts rather than
  reconstructing scope meaning locally

Phase 13 gate:

- scoped merge can be explained through existing diagnostic surfaces without
  becoming a second transition ontology.

Current implementation evidence:

- `prepare_scoped_merge_diagnostic_explanation(...)` lowers scope requests,
  admitted scope evidence, denial evidence, and unavailable posture into the
  existing `FoundationalDiagnosticExplanationInput` surface instead of
  materializing a transition-specific diagnostic artifact.
- scoped explanations emit compact required rows for request/admission/
  denial/unavailable outcomes, standard rows for skipped/admitted selected
  loci, and forensic rows for no-op/provenance details so
  `OperationalMinimal`, `Standard`, and `Forensic` richness profiles preserve
  the existing Milestone 6 elision contract.
- `FoundationalMergeScopeLocator` adds request-level transition locator
  vocabulary for full-branch, selected-node, and selected-aspect scope
  diagnostics; selected node/aspect locators remain exact locus locators for
  provenance and boundary attachment.
- `scoped_merge_diagnostics` certification proves compact elision, skipped
  and no-op explanation, denial/unavailable provenance rows, canonical locator
  basis, separator-safe diagnostic key fragments, and selected-locus boundary
  compatibility.

### Phase 14: Production Readiness And Adoption Contract

Purpose:

- close the foundational milestone and state exactly how adopting crates should
  consume it

This phase must ship:

- production-test readiness rows naming certified scope surfaces
- hostile-pressure inventory
- compile-fail boundary inventory
- residual debt inventory
- crate-facing docs that tell `worth-signal` and `worth-signal-wasm` to lower
  scoped merge/cherry-pick into this vocabulary before exposing runtime
  execution
- readiness evidence naming the exact `worth-proof` surfaces used by scoped
  merge progression: `TransitionOutcome`, committed authority artifacts,
  receipt artifacts, current-basis artifacts, trust-boundary bridging, and
  readmission

Phase 14 gate:

- scoped merge vocabulary is locally certified enough that adopting-crate work
  is an integration exercise, not a discovery phase.

Current implementation evidence:

- `foundational_transition_milestone9_scoped_merge_readiness_report()` exposes
  a scoped-merge-specific production readiness report beside the existing
  Milestone 5 transition readiness report, preserving older exact inventories
  while certifying the new scoped merge surface.
- `certify_foundational_transition_milestone9_scoped_merge_production_test_readiness()`
  emits a proof-bearing readiness artifact with the
  `worth-foundational.milestone-9.scoped-merge` basis.
- scoped readiness rows name request vocabulary, admitted evidence,
  denial/unavailable topology, canonical/locator/diagnostic participation, and
  downstream adoption contract surfaces.
- hostile pressure, compile-fail, residual-debt, phase-gate, and
  `worth-proof` API evidence all point at concrete source, docs, certification,
  and trybuild files.
- `docs/scoped-merge-adoption.md` teaches adopting crates to lower runtime
  scoped merge/cherry-pick into this vocabulary before exposing runtime
  execution.

## Required Named Proof Families

### The Scoped Merge Request Vocabulary Test

Proves full-branch, selected-node, and selected-aspect requests are distinct and
canonical.

Pass condition:

- emit scope-family digest, selected-locus digest, construction-denial digest,
  and compile-boundary proof.

### The Scoped Merge Admission Evidence Test

Proves admitted scope, selected no-op, and skipped-out-of-scope evidence survive
hostile producer diversity.

Pass condition:

- emit admitted scope digest, no-op digest, skipped-scope digest, scope breadth
  envelope, and producer-diversity parity proof.

### The Scoped Merge Denial Topology Test

Proves likely invalid scope requests do not collapse into generic failure.

Pass condition:

- emit denial-kind digest, denied-locus digest, unavailable-posture digest, and
  no-authority-crossing proof.

### The Scoped Merge Canonical Basis Test

Proves scoped merge evidence has stable canonical ordering and locator
participation.

Pass condition:

- emit canonical-basis digest, locator digest, ordering-hostility digest, and
  consumer-blindness proof.

## Must Ship

- foundational scoped merge request vocabulary
- selected-node and selected-aspect locus vocabulary
- admitted scope evidence
- selected no-op evidence
- skipped-out-of-scope evidence
- scope denial and scope-unavailable topology
- scope breadth counters
- merge candidate/verdict integration
- canonical basis and locator participation
- compile-fail and hostile runtime-independent certification

## Must Preserve

- `worth-foundational` remains shared vocabulary, not a merge executor
- `worth-proof` remains the proof progression owner
- existing Milestone 5 branch, merge, commit, receipt, and current-basis APIs
  remain compatible
- adopting crates keep local optimized merge planners and storage layouts
- unsupported scope families stay explicit unavailable posture rather than
  generic policy denial

## Acceptance Evidence

This milestone is complete only when:

- scoped merge request vocabulary is public and facade-curated
- selected-node and selected-aspect scope are distinct categories
- merge candidates/verdicts can carry admitted scope evidence
- skipped-out-of-scope and selected no-op evidence are digestible
- scope denials are typed and non-generic
- canonical basis entries are stable across independent construction paths
- compile-fail tests prevent category substitution
- docs teach downstream crates to lower into scoped merge vocabulary before
  implementing runtime-specific cherry-pick behavior

## Sequencing Notes

This milestone belongs after Milestone 8 because performance/layout vocabulary
helps name scope breadth honestly, and because Milestones 5 through 7 already
established the transition, diagnostics, lineage, and receipt surfaces scoped
merge evidence must extend.

It belongs before any `worth-signal` or `worth-signal-wasm` scoped merge work.
Those crates may own execution and materialization, but they should not invent
the shared language for selected-node and selected-aspect merge.

## Self-Check

- Does the milestone solve a real structural problem?
  Yes. It prevents scoped merge/cherry-pick from becoming crate-local dialect.
- Is the adversarial constraint precise and load-bearing?
  Yes. It attacks full, selected, no-op, skipped, denied, and unavailable scope
  cases across independent producers.
- Does the milestone preserve crate authority boundaries?
  Yes. Foundational owns shared meaning; adopting crates own runtime planning
  and execution.
- Does the milestone define proof obligations?
  Yes. It names vocabulary, admission, denial, canonicalization, locator, and
  compile-fail proof families.
- Could a competent engineer map this spec into honest types, modules, and
  tests?
  Yes. The phases map directly onto `transitions/merges`, `locators`,
  `canonicalization`, diagnostics attachment, and readiness evidence.
- Does the milestone belong in this roadmap sequence?
  Yes. It extends transition vocabulary before adopting runtime migrations use
  scoped merge as source-of-truth language.
