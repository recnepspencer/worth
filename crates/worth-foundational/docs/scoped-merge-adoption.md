# Scoped Merge And Cherry-Pick Vocabulary

## What This Feature Is

`worth-foundational` now provides the shared vocabulary for scoped merge and
cherry-pick boundaries.

Use this surface when an adopting runtime crate needs to describe:

- full-branch merge
- selected-node merge
- selected-aspect merge
- selected-but-unchanged no-op evidence
- skipped-out-of-scope candidate evidence
- invalid selected-scope denial
- unsupported or unavailable selected-scope posture
- scoped merge canonical basis, locators, diagnostics, and readiness evidence

This is vocabulary and proof shape, not merge execution. `worth-foundational`
does not mutate branch graphs, materialize cherry-picks, or resolve conflicts
for a runtime.

## Why You Use It

Scoped merge is exactly the kind of feature that can drift if every crate names
it locally. One crate might call a selected aspect a "partial merge", another
might call it "pick fields", and a third might hide it as a post-filter over a
full merge plan.

This surface makes the boundary shared:

- selection is requested before admission
- full-branch scope is explicit, not an omitted option
- selected nodes and selected aspects are distinct families
- no-op selected work is not confused with skipped work
- unavailable runtime capability is not confused with invalid user input
- diagnostics and canonical basis can point at the same selected scope loci

Adopting crates keep their optimized storage and merge planners. They lower
their local facts into this vocabulary before exposing runtime execution.

## Stable Entry Points

Use these public facade surfaces:

- `FoundationalMergeScope::full_branch()`
- `FoundationalMergeScope::selected_nodes(...)`
- `FoundationalMergeScope::selected_aspects(...)`
- `FoundationalSelectedNodeLocus::new(...)`
- `FoundationalSelectedAspectLocus::new(...)`
- `FoundationalSelectedAspectRequestEntry::new(...)`
- `foundational_merge(...).with_scope(...).plan()`
- `FoundationalAdmittedMergeScopeEvidence::new(...)`
- `FoundationalSelectedScopeNoOpEvidence::new(...)`
- `FoundationalSkippedOutOfScopeEvidence::new(...)`
- `FoundationalScopedMergeDenialEvidence::new(...)`
- `FoundationalScopedMergeUnavailablePosture::new(...)`
- `prepare_merge_scope_for_canonical_basis(...)`
- `prepare_admitted_merge_scope_for_canonical_basis(...)`
- `prepare_scoped_merge_denial_for_canonical_basis(...)`
- `prepare_scoped_merge_unavailable_for_canonical_basis(...)`
- `prepare_scoped_merge_diagnostic_explanation(...)`
- `foundational_transition_milestone9_scoped_merge_readiness_report()`
- `certify_foundational_transition_milestone9_scoped_merge_production_test_readiness()`
- `require_foundational_transition_milestone9_scoped_merge_production_test_readiness(...)`

## Core Mental Model

A scoped merge has three separate phases:

1. Request scope.
2. Admit or decline the scope.
3. Execute runtime-specific merge work outside `worth-foundational`.

The request says what the caller asked for. The admitted evidence says which
selected loci actually participated, which selected loci were no-ops, and how
many broader candidates were skipped. Denial and unavailable posture explain why
the request did not become admitted scope evidence.

`worth-proof::TransitionOutcome` remains the progression owner:

- invalid selected scope becomes `Denied`
- unsupported selected-scope capability becomes `Deferred`
- stale retained proof becomes `Stale`
- missing correspondence/basis becomes `RebindRequired`
- machinery failure becomes `Failed`

Do not invent a second top-level proof outcome for scoped merge.

## How It Executes

It does not execute in this crate.

`worth-foundational` gives adopting crates the boundary artifacts they need
before execution:

- request scope with canonical selected loci
- admitted scope evidence with breadth counters
- denial or unavailable posture with typed reason
- canonical basis participation for reproducible identity
- transition locators for selected scope diagnostics and receipts
- diagnostic explanation attachment for request, admission, denial, no-op,
  skipped, and unavailable outcomes

After that, `worth-signal`, `worth-signal-wasm`, or another adopting runtime
owns branch graph mutation, selected materialization, conflict resolution, and
storage traversal.

## Small Example

Use selected-aspect scope when the runtime wants to merge only specific aspects
of a node.

```rust
use worth_foundational::{
    FoundationalMergeScope, FoundationalSelectedAspectLocus,
    FoundationalSelectedAspectRequestEntry, FoundationalSelectedNodeLocus,
};

let invoice = FoundationalSelectedNodeLocus::new("invoice")?;
let amount = FoundationalSelectedAspectLocus::new("amount")?;
let currency = FoundationalSelectedAspectLocus::new("currency")?;

let scope = FoundationalMergeScope::selected_aspects([
    FoundationalSelectedAspectRequestEntry::new(invoice.clone(), amount),
    FoundationalSelectedAspectRequestEntry::new(invoice, currency),
])?;

assert_eq!(scope.requested_locus_count(), 2);
```

Construction canonicalizes selected loci and rejects empty or duplicate
selection. Raw strings are not accepted by APIs that require typed selected
scope loci.

## Real Example

An adopting runtime can lower a cherry-pick into one selected-aspect request,
then admit only the loci it can prove, while preserving no-op and skipped
evidence.

```rust
use worth_foundational::{
    CanonicalDigestId, FoundationalAdmittedMergeScopeEvidence,
    FoundationalScopeAdmissionBasis, FoundationalScopeBreadthSummary,
    FoundationalSelectedScopeLocus, FoundationalSelectedScopeNoOpCause,
    FoundationalSelectedScopeNoOpEvidence, FoundationalSkippedOutOfScopeEvidence,
};

let scope = selected_invoice_scope()?;
let skipped_digest = CanonicalDigestId::new([8; 32]);

let admitted_scope = FoundationalAdmittedMergeScopeEvidence::new(
    source_branch,
    target_branch,
    scope,
    FoundationalScopeAdmissionBasis::IdentityCorresponded,
    [],
    [selected_aspect("invoice", "amount")?],
    [FoundationalSelectedScopeNoOpEvidence::new(
        FoundationalSelectedScopeLocus::Aspect(selected_aspect("invoice", "currency")?),
        FoundationalSelectedScopeNoOpCause::UnchangedSourceTruth,
    )],
    FoundationalSkippedOutOfScopeEvidence::new(3, Some(skipped_digest)),
    conflict_check_width,
)?;

let breadth: FoundationalScopeBreadthSummary = *admitted_scope.breadth();
assert_eq!(breadth.admitted_locus_count(), 1);
assert_eq!(breadth.no_op_locus_count(), 1);
assert_eq!(breadth.skipped_candidate_count(), 3);
```

The admitted scope artifact is the shared explanation. The runtime may have a
very different internal journal, graph index, or aspect store, but consumers do
not need to inspect that private state to understand the boundary.

## How It Relates To Other Features

Scoped merge extends the existing transition vocabulary.

- `FoundationalMergeCandidate` carries the requested scope.
- `FoundationalMergeVerdict` carries admitted scope evidence on successful
  admission.
- Canonical basis functions make request, admitted, denial, and unavailable
  scope reproducible across construction paths.
- Transition locators can point at whole merge scope, selected-node scope, or
  selected-aspect scope.
- Diagnostics consume scoped merge artifacts through the existing diagnostic
  explanation surface.
- Production readiness uses `worth-proof` artifacts with a
  `worth-foundational.milestone-9.scoped-merge` basis.

Legacy broad merge candidates lower to explicit full-branch scope for
compatibility, so old broad merge meaning does not become "scope absent".

## Inspection And Debugging

For diagnostics, call `prepare_scoped_merge_diagnostic_explanation(...)` with a
`FoundationalScopedMergeDiagnosticInput`.

Supported diagnostic inputs are:

- `ScopeRequest`
- `AdmittedScope`
- `DeniedScope`
- `UnavailableScope`

Diagnostic materialization preserves the existing richness contract:

- operational-minimal can keep compact request/admission rows
- standard can include skipped, no-op, and admitted-locus summaries
- forensic can include retained selected no-op locus provenance

Use readiness to inspect the certified boundary:

```rust
use worth_foundational::{
    certify_foundational_transition_milestone9_scoped_merge_production_test_readiness,
    require_foundational_transition_milestone9_scoped_merge_production_test_readiness,
};

let readiness =
    certify_foundational_transition_milestone9_scoped_merge_production_test_readiness();
let report =
    require_foundational_transition_milestone9_scoped_merge_production_test_readiness(&readiness);

assert!(report.passes_readiness_checklist());
assert_eq!(
    report.scope().milestone(),
    "worth-foundational.milestone-9.scoped-merge",
);
```

The readiness report names the certified surfaces, hostile pressures,
compile-fail boundaries, residual debt, and `worth-proof` API evidence that
adopting crates should treat as the shared source of truth.

## Anti-Patterns

Do not:

- implement cherry-pick as a filter over an already-planned full-branch merge
- use raw strings or producer-private ids where typed selected loci are required
- treat selected no-op work as skipped-out-of-scope work
- treat unsupported runtime capability as invalid user input
- invent a local selected-node or selected-aspect ontology in an adopting crate
- emit runtime execution artifacts before lowering into foundational scope
  request/admission/denial/unavailable vocabulary
- imply that `worth-foundational` executes scoped merge or mutates branch graphs

## Current Limits

This crate intentionally does not provide:

- runtime branch graph mutation
- native or wasm merge execution
- cherry-pick materialization
- conflict-resolution UI
- storage-specific node or aspect lookup
- runtime-specific merge strategy registries

Those are adopting-runtime responsibilities. The shared contract is that they
must lower their selected merge meaning into this vocabulary first.

## Related Docs

- [Branching, Merging, And Commit Vocabulary](./branching-merging-and-commit-vocabulary/README.md)
- [Transition Production Readiness](./branching-merging-and-commit-vocabulary/transition-production-readiness.md)
- [Transition Canonical Basis, Locators, And Current-Basis](./branching-merging-and-commit-vocabulary/transition-canonical-basis-locators-and-current-basis.md)
- [Diagnostics And Explanation Ontology](./diagnostics-and-explanation-ontology/README.md)
