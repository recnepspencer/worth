# Milestone 7E: Collaboration Merge Hardening

## Goal

Turn `worth-relational` merge from a capable execution engine into a
first-class collaboration substrate by retaining relational merge truth as
proof-bearing artifacts and lowering that truth into `worth-foundational`
native basis, canonical, locator, compatibility, readmission, and support
surfaces instead of growing a parallel relational-only collaboration grammar.

## Why This Milestone Exists

Milestones 7A through 7D establish that relational merge can reason about
history shape, canonical merge ontology, authoritative merge execution, and
deletion/topology execution. That is necessary, but it is not sufficient for a
production-grade collaboration surface.

Today the runtime already has strong merge internals, but too much of the
meaning still lives in:

- implicit history/head state instead of a retained branch-basis artifact
- thin branch-pair requests instead of explicit merge request meaning
- planner-only structs instead of retained merge proof
- execution summaries and diagnostic rows instead of collaboration-grade
  witnesses
- schema/correspondence/policy facts that are visible during planning but not
  preserved as first-class retained truth
- crate-local merge explanation surfaces that are not yet explicitly aligned to
  foundational-native compatibility, readmission, and support posture

That is the same failure mode that `worth-signal` had to harden away. The
engine works, but the collaboration substrate is still too
reconstruction-heavy. This milestone exists to close that gap before later
Query, support, and multi-crate collaboration work starts depending on
relational merge posture as real retained truth.

## Governing Summaries

- `MENTALITY.md`
  - Protects adversarial-first design and infrastructure-before-feature
    sequencing.
  - The strongest shaping constraint here is that the hard problem is not
    "make merge work" but "make retained merge truth survive replay, recovery,
    inspection, and readmission without planner archaeology."

- `arch_laws.md`
  - Protects proof-carrying phase boundaries, explicit boundary envelopes, and
    separation of authority from derivation.
  - The strongest shaping constraint here is that relational-native truth must
    lower into foundational-native shared boundary grammar rather than staying
    as crate-local bags that every later consumer reinterprets.

- `composition_laws.md`
  - Protects semantic compilation units and named orchestration over helper fog.
  - The strongest shaping constraint here is that this milestone must split
    branch basis, request meaning, proof retention, correspondence, schema,
    strategy, compatibility, and support into narrow responsibilities rather
    than adding more meaning to existing broad merge files.

- `domain_structure_laws.md`
  - Protects filesystem boundaries that preserve authority, truth source,
    lifecycle, and dependency direction.
  - The strongest shaping constraint here is that the tree must show the split
    between relational-native merge truth and foundational-native shared
    boundary lowering.

- `perf_laws.md`
  - Protects carrying proof forward, pre-resolving control-plane decisions, and
    making cost visible at the boundary where it is claimed.
  - The strongest shaping constraint here is that compatibility, support, and
    replay surfaces must consume retained artifacts in O(1) or retained-breadth
    cost, not reopen branch graphs, planner classifications, or broad merge
    state.

- `worth_relational_roadmap.md`
  - Protects the roadmap's explicit Milestone 7 promise that merge semantics
    become canonical, replayable, policy-governed, and diagnostics-rich before
    later product work depends on them.
  - The strongest shaping constraint here is sequencing: this milestone belongs
    after 7D because execution ontology now exists, and before Milestone 8/8.5
    because later scale and strategy work should not depend on a collaboration
    surface that still requires internal reconstruction.

## Adversarial Constraint

Any accepted relational merge, once published, must be reconstructable,
inspectable, replayable, and readmittable across restore, durability recovery,
and later support/query surfaces using retained artifacts alone. Equivalent
merge truth produced through different legitimate lanes must converge to
identical retained collaboration artifacts and identical foundational-lowered
shared boundary artifacts. Stale, mismatched, or incomplete retained posture
must fail closed before any consumer can mistake summary output for authority.

## Product Decision Lock

- This milestone is not new merge execution capability. It is retained merge
  collaboration hardening.
- This milestone does not replace Milestone 7B through 7D; it closes the gap
  between their internal ontology/execution work and a first-class product
  collaboration surface.
- `worth-relational` owns relational merge truth.
- `worth-foundational` owns the shared toolkit for canonical basis, locators,
  denial/deferred/stale/rebind grammar, compatibility posture, readmission
  posture, and support posture.
- `worth-proof` owns the machine-checked progression grammar for
  proof-carrying artifacts, authority witnesses, freshness-scoped basis,
  boundary-bridged readmission, and typed transition outcomes.
- If foundational already has an honest native surface, relational must lower
  into it instead of inventing a sibling grammar.
- If a boundary needs proof-bearing construction, typed readmission, or
  phase-typed denial/deferred/stale/rebind semantics, prefer `worth-proof`
  directly instead of crate-local approximations.
- Query-style recommendation or workflow advice is out of scope. This milestone
  stops at retained proof, compatibility, and support inspection.

## Phase 1: Relational Branch Basis Artifact

This phase freezes one thing: relational merge must stand on an explicit
retained branch-basis artifact rather than on ambient branch-head inspection.

### Relevant subsystems

- `history`
- `merge`
- `transactions`
- durability and replay retention

### Relevant APIs

- [crates/worth-relational/src/history/data/mod.rs](../../crates/worth-relational/src/history/data/mod.rs)
- [C:\Users\Esther\Documents\Programming\WORTH_workspace\worktree_3\crates\worth-relational\src\history\logic\access\ancestry.rs](../../crates/worth-relational/src/history/access/ancestry.rs)
- [crates/worth-relational/src/history/data/branch_creation.rs](../../crates/worth-relational/src/history/data/branch_creation.rs)

### Warnings

- Do not let "current head" remain the implicit merge basis contract.
- Do not encode basis only as `(source head, target head, merge base)` strings
  or counters.
- Do not let replay or support inspection rediscover branch basis by reopening
  live history state.

### Test requirements

- Adversarial parity test: equivalent retained branch basis built from live
  runtime, published merge outcome, and recovered durability state must carry
  identical basis digest and ordered-parent truth.
- Adversarial denial test: stale head, missing branch, or merge-base mismatch
  must produce typed basis denial before request lowering or support surfaces
  can publish a collaboration-looking artifact.

### Engineering decisions

- Introduce a retained `RelationalMergeBranchBasis` artifact family.
- The basis must carry source branch, target branch, selected head references,
  merge-base reference, and the selection rule used to establish the basis.
- Basis must be sealed and replay-retainable, and should be wrapped in a
  `worth-proof::Artifact` when it crosses collaboration-facing authority
  boundaries.
- Basis construction belongs at the history/merge boundary, not in diagnostics,
  support, or execution-summary helpers.

### Open questions

- Decision: branch basis is a reusable relational branch-basis artifact, not a
  merge-only shape.
- Decision: branch-fork provenance remains adjacent until a later milestone
  proves it is required to establish branch-basis identity honestly.

## Phase 2: Foundational Current-Basis Lowering

This phase freezes one thing: retained relational branch basis must lower into
foundational-native basis and trust-boundary/readmission vocabulary instead of
remaining a relational-only basis dialect above the authority boundary.

### Relevant subsystems

- `worth-foundational/transitions/basis`
- `worth-foundational/boundary_evidence`
- relational history and merge basis adapters

### Relevant APIs

- [crates/worth-foundational/src/transitions/basis/mod.rs](../../crates/worth-foundational/src/transitions/basis/mod.rs)
- [crates/worth-foundational/src/boundary_evidence_api/stronger_lane/readmission.rs](../../crates/worth-foundational/src/boundary_evidence_api/stronger_lane/readmission.rs)

### Warnings

- Do not design a separate relational readmission grammar if foundational basis
  and boundary evidence can express the posture honestly.
- Do not make support or replay reason over raw relational basis once a
  foundational basis artifact exists.
- Do not lower live state directly; lower the retained relational basis
  artifact.

### Test requirements

- Adversarial equivalence test: equivalent relational branch basis inputs must
- lower to identical foundational current-basis artifacts and boundary-bridged
  artifacts.
- Adversarial denial test: stale or mismatched relational basis must fail at
  foundational lowering with typed denial or stale posture before compatibility
  or support surfaces continue.

### Engineering decisions

- Add one explicit relational-to-foundational basis lowering seam.
- Support bridge/readmit progression through foundational-native artifacts.
- Treat this as a proof-bearing boundary, not a convenience conversion helper.
- Use `worth-proof` freshness/boundary progression surfaces for current,
  stale-readable, rebind-required, and boundary-bridged postures rather than
  crate-local posture enums.

### Open questions

- Decision: start by expressing merge basis through current foundational basis
  vocabulary and extend foundational only when relational truth cannot be
  lowered honestly.
- Decision: readmission authority remains foundational-native, with relational
  wrappers only for construction and access ergonomics.

## Phase 3: Explicit Relational Merge Request Vocabulary

This phase freezes one thing: relational merge requests must express admitted
meaning explicitly rather than only naming source branch, target branch, and a
single reconcile intent.

### Relevant subsystems

- `merge/request`
- `history`
- schema and lineage surfaces

### Relevant APIs

- [crates/worth-relational/src/merge/data/requests.rs](../../crates/worth-relational/src/merge/data/requests.rs)
- [crates/worth-relational/src/merge/data/execution.rs](../../crates/worth-relational/src/merge/data/execution.rs)

### Warnings

- Do not widen request meaning inside planning.
- Do not leave correspondence posture, schema posture, or topology posture as
  planner-local interpretation of a generic request.
- Do not overload one enum variant until it becomes a bag of optional fields.

### Test requirements

- Adversarial equivalence test: equivalent request meaning built through
  ordinary merge lanes and any specialist construction lanes must normalize to
  identical request digests and proof-bearing request families.
- Adversarial boundary test: malformed or unsupported request posture must be
  denied before history traversal, identity matching, or schema/policy
  resolution starts.

### Engineering decisions

- Replace or immediately lower the current thin `MergePlanningRequest` into a
  proof-bearing request family.
- The family must at minimum reserve explicit space for:
  - full-branch reconciliation
  - correspondence strictness or advisory posture
  - schema reconciliation posture
  - topology-sensitive intent
- The request boundary should admit more future behavior than current execution,
  but unsupported admitted meaning must fail with typed denial or unavailable
  rather than being silently flattened.
- The normalized request family should use sealed proof-carrying construction
  rather than public bags, even if it starts as relational-native before
  foundational lowering.

### Open questions

- Decision: do not admit record-scope or kind-scope merge requests in this
  milestone; only build the request family so they can be added later without
  reopening the boundary.
- Decision: request artifacts own caller intent and admitted request posture,
  while schema-derived admission owns actual schema compatibility and
  reconciliation classification.

## Phase 4: Foundational Merge Vocabulary Lowering

This phase freezes one thing: normalized relational merge request meaning must
lower into foundational-native merge scope, merge intent, and admission
vocabulary wherever the shared toolkit already has an honest equivalent.

### Relevant subsystems

- `worth-foundational/transitions/merges`
- relational merge request lowering

### Relevant APIs

- [crates/worth-foundational/src/transitions/merges/mod.rs](../../crates/worth-foundational/src/transitions/merges/mod.rs)

### Warnings

- Do not keep a parallel relational-only scope or denial grammar above this
  boundary if foundational can already express the same truth.
- Do not lie by lowering relational-specific meaning into an inexact
  foundational category; keep relational truth native when foundational is not
  yet expressive enough.
- Do not let planner code consume weaker pre-lowered request bags once this seam
  exists.

### Test requirements

- Adversarial equivalence test: equivalent normalized relational request
  meaning must lower to identical foundational scope family and admission
  vocabulary.
- Adversarial non-collapse test: distinct relational request meanings must stay
  distinct after foundational lowering rather than collapsing into the same
  scope or intent family.

### Engineering decisions

- Add a relational-to-foundational merge request lowering artifact.
- Foundational-native outcome families such as denied, deferred, stale, and
  rebind-required should become the shared outer grammar where honest.
- Keep relational-only semantics native until foundational can own them without
  flattening meaning.

### Open questions

- Decision: only distinctions required to avoid dishonest flattening should
  force foundational extension now.
- Decision: extend foundational early when a temporary relational-only shared
  boundary grammar would otherwise be introduced above this lowering seam.

## Phase 5: Retained Relational Merge Proof Packet

This phase freezes one thing: published merge truth must retain a proof packet
that survives beyond planning and execution rather than reducing to summaries,
digests, and counts.

### Relevant subsystems

- `merge/data/artifacts`
- `transactions/data/outcomes`
- merge replay and durable publication

### Relevant APIs

- [crates/worth-relational/src/merge/data/artifacts/planning_artifact_core.rs](../../crates/worth-relational/src/merge/data/artifacts/planning_artifact_core.rs)
- [crates/worth-relational/src/transactions/data/outcomes/plan_artifacts.rs](../../crates/worth-relational/src/transactions/data/outcomes/plan_artifacts.rs)
- [crates/worth-relational/src/merge/data/execution.rs](../../crates/worth-relational/src/merge/data/execution.rs)

### Warnings

- Do not treat `MergeExecutionSummary` as the collaboration artifact by
  accretion.
- Do not make replay reconstruct proof from diagnostic rows.
- Do not let retained proof depend on planner-private structs that cannot be
  serialized, sealed, and replayed honestly.

### Test requirements

- Adversarial replay-honesty test: preview, execution, durable publication, and
  replay or recovery must expose identical retained merge proof packet digests
  for equivalent merge truth.
- Adversarial fail-closed test: if retained proof is absent or inconsistent
  while summary output exists, replay and support surfaces must deny rather than
  synthesize the missing proof from planner residue.

### Engineering decisions

- Add a retained `RelationalMergeProofPacket` distinct from
  `MergePlanningArtifactCore` and `MergeExecutionSummary`.
- The packet must carry:
  - request meaning
  - branch basis
  - admitted affected scope or admitted merge surface
  - correspondence, schema, and strategy witness digests
  - execution digest and planning digest
  - typed denial or unavailable posture when execution did not admit
- Planning and execution may retain richer internal packets, but published
  collaboration truth must converge on this smaller, durable proof family.
- Where the packet crosses subsystem or replay boundaries, it should be carried
  as a `worth-proof::Artifact` with the narrowest honest phase and basis.

### Open questions

- Decision: the main retained proof packet stays compact and carries
  digest-backed links to subordinate witnesses rather than row-level payload by
  default.
- Decision: full retained proof should live as a replay-facing sibling artifact
  with canonical linkage unless envelope-local retention is required to preserve
  existing authoritative publication invariants honestly.

## Phase 6: Foundational Canonical Basis Lowering

This phase freezes one thing: retained relational merge proof must lower into
foundational-native canonical basis entries instead of each downstream consumer
assembling its own digest grammar.

### Relevant subsystems

- `worth-foundational/canonicalization`
- relational merge proof packet
- durability and replay

### Relevant APIs

- [crates/worth-foundational/src/transitions/basis/mod.rs](../../crates/worth-foundational/src/transitions/basis/mod.rs)
- [crates/worth-foundational/src/facade.rs](../../crates/worth-foundational/src/facade.rs)

### Warnings

- Do not keep relational-only digest strings as the final cross-crate basis for
  retained proof identity.
- Do not make support, compatibility, or diagnostics derive canonical basis
  independently.
- Do not lower unsupported meaning into fake canonical entries; extend
  foundational honestly first if needed.

### Test requirements

- Adversarial ordering test: equivalent retained merge proof produced through
  different lanes must lower to identical foundational canonical basis
  sequences.
- Adversarial family-distinction test: branch basis, request, correspondence,
  schema, strategy, and denial/unavailable posture must remain separately
  addressable in canonical basis output.

### Engineering decisions

- Build one canonical-basis lowering seam from relational proof families into
  foundational basis entries.
- Treat foundational canonical basis as the shared retained identity contract
  for later compatibility, support, and certification work.
- Use `worth-proof` proof sets and authority witnesses where canonical-basis
  readiness itself needs to be carried forward rather than inferred from
  successful construction.

### Open questions

- Decision: add foundational canonical basis locus kinds only where current
  foundational loci would flatten real relational distinctions.
- Decision: canonical basis lowering should live in a dedicated relational
  collaboration lowering boundary rather than being scattered across each
  witness.

## Phase 7: Correspondence As A First-Class Relational Witness

This phase freezes one thing: correspondence must stop being planner-local
evidence and become retained merge truth with explicit authority and denial.

### Relevant subsystems

- `merge/identity`
- `lineage`
- schema-declared correspondence
- replay and diagnostics

### Relevant APIs

- [crates/worth-relational/src/merge/data/identity.rs](../../crates/worth-relational/src/merge/data/identity.rs)
- [crates/worth-relational/src/merge/data/plans.rs](../../crates/worth-relational/src/merge/data/plans.rs)

### Warnings

- Do not let raw candidate lists remain the only retained correspondence truth.
- Do not collapse exact, reconciliable, ambiguous, missing-target, and
  schema-declared correspondence into one generic identity summary.
- Do not let advisory correspondence gain authority merely by being present in a
  replayed merge artifact.

### Test requirements

- Adversarial equivalence test: the same logical source-target correspondence
  discovered through storage identity, lineage identity, structural identity, or
  schema-declared correspondence must converge to the same retained witness only
  when the admitted authority basis truly matches.
- Adversarial denial test: ambiguous, non-unique, or stale correspondence must
  remain typed denial or unavailable with exact locus retention and no silent
  target duplication fallback.

### Engineering decisions

- Introduce a retained `RelationalMergeCorrespondenceWitness`.
- The witness must preserve:
  - authority basis
  - admitted source-target pairings
  - denied or unavailable correspondence posture
  - candidate-to-admission digest continuity
- Advisory lineage or schema hints may feed this witness, but they must not
  replace the witness.
- This witness should be sealed as a proof-bearing artifact, not a public struct
  that callers can synthesize without passing through correspondence admission.

### Open questions

- Decision: the correspondence witness retains concrete record-pair truth
  directly.
- Decision: promoted lineage correspondence and merge correspondence may share
  subordinate structural substrate, but remain distinct retained artifact
  families.

## Phase 8: Schema Reconciliation As A First-Class Relational Witness

This phase freezes one thing: schema reconciliation must become retained merge
truth rather than an execution-time side interpretation of schema declarations.

### Relevant subsystems

- schema continuity
- merge policy and merge lowering
- durability and replay
- diagnostics and publication

### Relevant APIs

- [crates/worth-relational/src/merge/data/policy.rs](../../crates/worth-relational/src/merge/data/policy.rs)
- [crates/worth-relational/src/merge/data/artifacts/planning_artifact_core.rs](../../crates/worth-relational/src/merge/data/artifacts/planning_artifact_core.rs)

### Warnings

- Do not collapse schema reconciliation into generic conflict classification.
- Do not let schema-declared correspondence remain visible only as summary
  counts.
- Do not force replay or support inspection to infer schema posture from
  execution digests or message strings.

### Test requirements

- Adversarial parity test: additive, narrowing, type-incompatible, and
  structural-incompatible schema reconciliation outcomes must retain identical
  witness truth across planning, execution, durable publication, and replay.
- Adversarial denial test: incompatible schema posture must remain typed and
  retained even when record-level merge truth is otherwise admissible; no
  consumer may flatten schema denial into generic merge failure.

### Engineering decisions

- Introduce a retained `RelationalSchemaReconciliationWitness`.
- The witness must carry:
  - source and target schema basis
  - reconciliation category
  - policy used or denial class
  - correspondence linkage where schema-declared correspondence influenced
    admitted record pairing
- This witness must compose with Milestone 5 schema reconciliation semantics
  rather than duplicating that authority in merge support helpers.
- Typed schema denial/unavailable posture should flow through
  `worth-proof::TransitionOutcome` families where the witness crosses into later
  compatibility or support phases.

### Open questions

- Decision: the schema reconciliation witness is owned under `merge/` because
  it exists as merge truth, even when it consumes schema-owned authority facts.
- Decision: descriptor detail should be retained by canonical digest unless the
  exact descriptor payload is required to localize denial or mismatch honestly.

## Phase 9: Strategy And Policy As A First-Class Relational Witness

This phase freezes one thing: merge policy, topology policy, and execution
strategy posture must be retained as a first-class witness rather than only as
summary output and scattered lowered-plan fields.

### Relevant subsystems

- merge policy
- merge lowering
- execution authority contract
- transactions outcome publication

### Relevant APIs

- [crates/worth-relational/src/merge/data/policy.rs](../../crates/worth-relational/src/merge/data/policy.rs)
- [crates/worth-relational/src/merge/data/artifacts/execution_authority_contract.rs](../../crates/worth-relational/src/merge/data/artifacts/execution_authority_contract.rs)
- [crates/worth-relational/src/transactions/data/outcomes/plan_artifacts.rs](../../crates/worth-relational/src/transactions/data/outcomes/plan_artifacts.rs)

### Warnings

- Do not let `MergeExecutionAuthorityContract` and policy summaries remain the
  de facto retained witness by coincidence.
- Do not collapse conflict-free policy, manual-resolution posture, rejection
  posture, deletion posture, and topology posture into one generic strategy
  digest.
- Do not rebuild strategy truth from lowered rows at every consumer.

### Test requirements

- Adversarial equivalence test: equivalent admitted strategy posture across
  ordinary and specialist merge lanes must converge to identical retained
  strategy witness digests.
- Adversarial differentiation test: when declared aspect policy, topology
  admission, or deletion policy actually differs, the retained strategy witness
  must differ in the correct sub-identity rather than only by a whole-artifact
  digest.

### Engineering decisions

- Introduce a retained `RelationalMergeStrategyWitness`.
- The witness must at minimum separate:
  - aspect merge policy posture
  - topology admission posture
  - deletion admission posture
  - execution authority contract posture
- Strategy witness construction must happen once after admission and lowering,
  then be retained across result, replay, and support surfaces.
- This witness should be sealed and authority-backed rather than publicly
  deserializable or WORTHable from scattered summary fields.

### Open questions

- Decision: relation-endpoint rewiring posture remains a sibling strategy
  category, not merely an unnamed subset of broad topology posture.
- Decision: the execution authority contract is referenced by digest from the
  strategy witness unless inline structure is required for honesty at a public
  boundary.

## Phase 10: Foundational Locator And Diagnostic Lowering

This phase freezes one thing: retained relational merge artifacts must lower
into foundational-native locators and diagnostic subjects instead of relying on
crate-local locator strings or support-specific key formats.

### Relevant subsystems

- `worth-foundational/diagnostics`
- `worth-foundational/locators`
- retained relational collaboration witnesses

### Relevant APIs

- [crates/worth-foundational/src/diagnostics/subjects.rs](../../crates/worth-foundational/src/diagnostics/subjects.rs)
- [crates/worth-foundational/src/facade.rs](../../crates/worth-foundational/src/facade.rs)

### Warnings

- Do not let every relational consumer invent its own locator naming scheme.
- Do not make compatibility and support rows depend on message text when
  foundational locators can carry the exact subject/locus identity.
- Do not collapse correspondence, schema, strategy, and branch-basis loci into
  one generic merge subject.

### Test requirements

- Adversarial parity test: equivalent relational collaboration witnesses must
  lower to identical foundational locators and diagnostic subjects across live,
  replayed, and recovered lanes.
- Adversarial distinction test: different witness families and different
  mismatch loci must stay separately addressable after foundational lowering.

### Engineering decisions

- Introduce foundational-locator lowering for each retained witness family.
- Reuse foundational diagnostic subjects and locators wherever honest.
- Extend foundational if required instead of growing relational-only locator
  dialects.
- Any locator-bearing support artifact that crosses boundaries should be wrapped
  in proof-bearing form rather than carried as untyped row bags.

### Open questions

- Decision: add foundational transition locator variants only for witness kinds
  that cannot be represented without semantic collapse.
- Decision: reuse existing foundational diagnostic subject families where honest
  and extend them only when relational witness identity would otherwise be
  obscured.

## Phase 11: Foundational Compatibility And Readmission

This phase freezes one thing: retained merge posture must admit one explicit
compatibility and readmission-preparation witness built on foundational-native
progression and boundary evidence rather than on a relational-only compatibility
grammar.

### Relevant subsystems

- retained relational collaboration witnesses
- `worth-foundational/boundary_evidence`
- `worth-foundational/profiles`
- replay and recovery

### Relevant APIs

- [crates/worth-foundational/src/boundary_evidence_api/stronger_lane/readmission.rs](../../crates/worth-foundational/src/boundary_evidence_api/stronger_lane/readmission.rs)
- [crates/worth-foundational/src/profiles/progression.rs](../../crates/worth-foundational/src/profiles/progression.rs)

### Warnings

- Do not make compatibility a report assembled on demand from raw retained
  fragments.
- Do not let readmission posture depend on ambient runtime inspection when the
  retained basis and merge proof already name the authoritative mismatch.
- Do not conflate missing retained proof, stale retained proof, and cross-basis
  mismatch.

### Test requirements

- Adversarial parity test: equivalent retained merge posture from result,
  durability replay, and recovered runtime must produce identical compatibility
  posture and readmission-preparation artifacts.
- Adversarial denial test: stale basis, missing proof, strategy mismatch, or
  cross-basis mismatch must yield typed foundational-native denial before
  support inspection or Query-facing code can publish compatibility-looking
  surfaces.

### Engineering decisions

- Build compatibility/readmission on foundational-native progression and support
  boundary evidence.
- Relational may add domain-specific fact inventory, but the outer grammar must
  reuse foundational-native denied/deferred/stale/rebind posture where honest.
- Use `worth-proof` directly for:
  - `AuthorityWitness`
  - `FreshnessScopedBasis`
  - `BoundaryBridged`
  - `readmit_with_authority(...)`
  - `TransitionOutcome`
  rather than inventing a relational-local readmission topology.

### Open questions

- Decision: start with current foundational boundary evidence and profile
  progression surfaces and extend them only if they cannot express merge
  collaboration posture honestly.
- Decision: compatibility retains only the lower-authority fact inventory needed
  for readmission/support decisions; richer relational truth remains in linked
  subordinate witnesses.

## Phase 12: Foundational Support Inspection

This phase freezes one thing: relational merge needs one support-grade
inspection witness that projects retained collaboration truth through
foundational-native support posture rather than through a crate-local support
ontology.

### Relevant subsystems

- inspection
- diagnostics
- `worth-foundational/profiles`
- retained relational collaboration witnesses

### Relevant APIs

- [crates/worth-foundational/src/profiles/mod.rs](../../crates/worth-foundational/src/profiles/mod.rs)
- [crates/worth-relational/src/merge/data/artifacts/inspection_artifact.rs](../../crates/worth-relational/src/merge/data/artifacts/inspection_artifact.rs)
- [crates/worth-relational/src/facade.rs](../../crates/worth-relational/src/facade.rs)

### Warnings

- Do not let the current lowered-plan inspection artifact become the support
  surface by accident.
- Do not flatten branch basis, correspondence, schema, strategy, and
  compatibility into one opaque summary blob.
- Do not let replay message text or diagnostic strings become authority.

### Test requirements

- Adversarial parity test: equivalent retained merge posture inspected from live
  result, replayed result, recovered durability state, and compatibility lane
  must produce identical support rows and identical foundational support
  posture.
- Adversarial absence test: missing retained proof in any required category must
  yield typed inspection absence; support rows must not be synthesized from
  summary-only output.

### Engineering decisions

- Introduce a support inspection witness that projects relational rows but
  lowers readiness/support posture into foundational-native support profile
  vocabulary.
- Keep row content relational-specific and support posture foundational-native.
- Where support inspection is retained or bridged across boundaries, prefer
  `worth-proof::Artifact` carriers over plain structs so the basis posture stays
  machine-visible.

### Open questions

- Decision: support inspection lives under `inspection/merge_support/`, not
  under `merge/support/`.
- Decision: only the major branch-basis, request/admission, correspondence,
  schema, strategy, and compatibility/support rows become stable public facade
  surfaces; formatting helpers remain internal.

## Phase 13: Replay, Recovery, And Certification Parity

This phase freezes one thing: every collaboration-facing merge artifact added by
this milestone must survive replay, durability recovery, and certification as
first-class canonical truth rather than best-effort reconstruction.

### Relevant subsystems

- replay
- durability recovery
- diagnostics/publication
- certification and hostile merge tests

### Relevant APIs

- [crates/worth-relational/src/merge/facade.rs](../../crates/worth-relational/src/merge/facade.rs)
- [crates/worth-relational/src/transactions/data/outcomes/plan_artifacts.rs](../../crates/worth-relational/src/transactions/data/outcomes/plan_artifacts.rs)

### Warnings

- Do not certify only execution success; certify retained collaboration truth.
- Do not let recovery reconstruct these surfaces from helper-local planner paths
  that production replay does not own.
- Do not allow any new artifact family to exist only in live runtime memory.

### Test requirements

- Adversarial replay-equivalence test: every new witness family from this
  milestone and every foundational-lowered shared boundary artifact must
  round-trip through durable publication and recovery with identical canonical
  digest.
- Adversarial hostile-drift test: mutate retained branch basis, correspondence
  witness, schema witness, strategy witness, or foundational-lowered
  compatibility/support artifact independently and prove the exact mismatch is
  localized rather than flattened into generic failure.
- Adversarial no-shortcut test: replay, recovery, compatibility, and support
  consumers must deny rather than reconstruct authority from planner residue,
  live branch state, replay text, summary-only output, or partially retained
  witness sets.

### Engineering decisions

- Extend the merge certification suite so these new retained witness families
  participate in:
  - commit/replay equivalence
  - durable recovery parity
  - merge history shape parity
  - correspondence hardening
- The certification program should prefer real merge-produced histories over
  synthetic retained artifact fixtures wherever feasible.
- Add compile-fail or mechanical boundary tests wherever a witness is supposed
  to be sealed against direct construction, deserialization, or out-of-order
  progression.

### Open questions

- Decision: promote existing hostile merge tests where they already certify real
  retained-proof properties, but add a dedicated collaboration-specific hostile
  suite for the new witness families and foundational-lowered surfaces.
- Decision: add an explicit new test-requirements entry for collaboration merge
  hardening rather than relying only on widened legacy names.

## Collaboration Certification Additions

The monolithic test requirements document remains the shared registry, but this
milestone imposes additional collaboration-hardening certification that must be
implemented either by tightening existing named tests or by adding explicit
companion requirements with the names below.

### Strengthen `Hostile commit/replay equivalence test`

This milestone requires that the commit or replay equivalence program certify
identical retained and foundational-lowered collaboration truth across live
runtime, published result, replay, and durability recovery.

It must verify identical:

- retained merge branch-basis artifact
- retained merge request artifact
- retained merge proof packet
- retained merge correspondence witness
- retained merge schema reconciliation witness
- retained merge strategy witness
- foundational-lowered current-basis artifact
- foundational canonical basis bundle
- foundational locator or diagnostic subject bundle
- foundational compatibility or readmission posture
- foundational support posture

It must not verify only whole-artifact digest equality. It must also verify:

- family-level presence or absence
- family-level denial, unavailable, stale, and rebind posture where applicable
- exact mismatch localization where one witness family is intentionally mutated
- no synthetic proof reconstruction from summary-only execution outputs

### Strengthen `Lineage/correspondence hardening test`

This milestone requires that the correspondence hardening program cover
merge-specific adversarial cases, not only lineage promotion in isolation.

It must include cases where:

- structural fingerprints match but lineage history differs
- schema-declared correspondence conflicts with lineage-based correspondence
- multiple plausible target candidates survive branch-local replacement chains
- independent branch-local rewrites resemble merge candidates but must remain
  advisory
- stale correspondence witness is replayed or recovered against a newer branch
  basis

It must verify:

- exact candidate-set retention
- exact admitted source-target pairing retention
- no advisory candidate silently upgrades during replay or recovery
- exact mismatch locus and authority basis localization
- compatibility and support surfaces reflect correspondence posture directly
  rather than flattening it into generic merge failure

### Strengthen `Merge-ready history shape test`

This milestone requires `Merge-ready history shape test` to move beyond
ordered-parent fixture safety into real collaboration-history certification.

It must prefer real merge-produced histories and verify:

- retained collaboration witnesses survive those histories canonically
- branch-basis parity across live runtime, replay, and durability recovery
- parent-order stability in foundational canonical basis and locator lowering
- compatibility and support surfaces over merge-produced histories
- typed failure localization if parent order, head lineage, or basis lineage is
  perturbed

Synthetic fixtures remain acceptable only for minimal negative topology cases
that cannot be exercised through real merge-produced histories without
obscuring the assertion.

### Strengthen `Durable recovery and schema mismatch test`

This milestone requires durable recovery certification to prove partial witness
or cross-family mismatch is fail-closed.

It must include cases where:

- merge proof packet exists but compatibility witness is missing
- relational branch basis exists but foundational-lowered current-basis posture
  is stale or mismatched
- schema reconciliation witness mismatches the retained merge proof packet
- strategy witness mismatches the retained merge proof packet
- replay history retains support-facing rows or text without the underlying
  retained witness family
- one artifact family is rebuilt from stale live state instead of retained
  publication

It must verify:

- exact typed denial family
- exact mismatch locus
- exact stale, rebind-required, or authority-revalidation posture
- no support-row synthesis from partial evidence
- no fallback to summary-only merge output

### Add `Retained collaboration merge proof test`

This milestone requires a new explicit certification requirement named
`Retained collaboration merge proof test`.

Its purpose is to prove that accepted merge collaboration truth is carried by
retained relational witnesses plus foundational-lowered shared-boundary
artifacts, and that later consumers use those artifacts instead of
reconstructing merge meaning from planner state or live-state shortcuts.

It must exercise:

- exact correspondence
- ambiguous or denied correspondence
- schema-driven reconciliation
- strategy or policy differentiation
- stale or readmitted basis posture
- replay and durable recovery
- support inspection after trust-boundary downgrade

It must verify at minimum:

- `merge_branch_basis_digest`
- `merge_request_digest`
- `merge_proof_digest`
- `merge_correspondence_digest`
- `merge_schema_reconciliation_digest`
- `merge_strategy_witness_digest`
- `merge_foundational_basis_digest`
- `merge_foundational_locator_digest`
- `merge_compatibility_digest`
- `merge_support_digest`
- `merge_posture_matrix`

It must also verify:

- all required witness families are present when the merge was admitted
- foundational-lowered basis, canonical, locator, compatibility, and support
  surfaces derive from retained truth only
- missing witness families fail closed
- equivalent live, replayed, and recovered lanes converge to identical
  retained-proof truth

### Add `Merge compatibility/readmission hardening test`

This milestone requires a new explicit certification requirement named
`Merge compatibility/readmission hardening test`.

Its purpose is to prove that compatibility and readmission are real
proof-bearing boundaries built on retained truth and foundational-native
progression, not convenience reports assembled on demand.

It must exercise:

- current basis
- stale-readable basis
- rebind-required basis
- boundary-bridged basis
- authority-backed readmission
- cross-basis mismatch
- witness-family mismatch

It must verify:

- exact foundational-native posture family
- readmission only succeeds with proper authority
- identical retained truth plus identical authority yields identical readmitted
  posture
- degraded posture never masquerades as current
- support posture narrows honestly with basis posture

### Add `Merge support inspection no-shortcut test`

This milestone requires a new explicit certification requirement named
`Merge support inspection no-shortcut test`.

Its purpose is to prove that support inspection depends on retained proof and
foundational-lowered compatibility or support artifacts, not on replay text,
planner residue, or summary-only execution output.

It must include cases where:

- one required witness family is removed at a time
- only execution summary exists
- only support rows or inspection rows exist
- replay detail text exists without retained proof
- foundational-lowered support artifacts are mismatched against retained
  relational truth

It must verify:

- typed inspection absence or denial
- no synthetic rows
- no fake readiness or support posture
- no compatibility reconstruction from raw schema state or live branch state
- no message-text authority

### Certification Style Requirements

All new or tightened collaboration-hardening tests for this milestone must obey
the following additional rules:

- they must not verify only by whole-bundle digest equality
- they must include at least one negative control where a single witness
  family, basis posture, or locus is intentionally perturbed
- denial, stale, unavailable, and rebind-required paths must assert no
  collaboration-looking artifact is published as though success occurred
- real merge-produced histories are preferred over synthetic retained-artifact
  fixtures whenever the assertion can be exercised honestly that way
- mechanical sealing or compile-fail checks must be added wherever a witness is
  supposed to be protected against direct construction, deserialization, or
  out-of-order progression

## Must Ship

- retained relational merge branch-basis artifact
- foundational-native lowering for current basis and trust-boundary readmission
- proof-bearing relational merge request family
- foundational-native merge vocabulary lowering where honest
- retained relational merge proof packet
- foundational-native canonical basis lowering
- retained relational correspondence witness
- retained relational schema reconciliation witness
- retained relational strategy/policy witness
- foundational-native compatibility/readmission surfaces fed by retained
  relational truth
- foundational-native support posture fed by retained relational truth
- replay/recovery retention and certification for all of the above

## Must Preserve

- single serialized authority for final truth commit
- canonical observability and replay
- explicit separation between relational domain truth and foundational shared
  boundary grammar
- explicit typed denial and unavailable posture instead of generic merge failure
- no host-side heuristic correspondence, schema, or policy logic becoming
  accidental authority
- no replay or support surface reopening broad planner or history internals when
  retained proof already exists

## Acceptance Requirements

This milestone is complete only when:

- the roadmap is updated so Milestone 7 explicitly includes this collaboration
  hardening slice before Milestone 8 and 8.5
- every new retained witness family added by this milestone is published,
  replayed, and recovered canonically
- foundational-lowered basis, canonical, locator, compatibility, readmission,
  and support surfaces are derived from retained relational truth rather than
  live-state reconstruction
- `worth-proof` is used directly at the boundaries that require:
  - authority witnesses
  - freshness-scoped basis
  - boundary-bridged readmission
  - typed denied/deferred/stale/rebind outcomes
  instead of relational-local substitutes
- `Hostile commit/replay equivalence test` remains satisfied for histories
  carrying the new retained collaboration witnesses
- `Durable recovery and schema mismatch test` remains satisfied for histories
  carrying the new retained collaboration witnesses
- `Merge-ready history shape test` remains satisfied for histories carrying the
  new retained collaboration witnesses
- `Lineage/correspondence hardening test` is widened or paired with an explicit
  merge-correspondence witness certification requirement
- an explicit collaboration-merge retained-proof certification requirement is
  added to
  [plans/WORTH-relational/test-requirements.md](./test-requirements.md)
  if the current requirements set does not already cover retained proof,
  foundational lowering, compatibility, and support inspection parity honestly
