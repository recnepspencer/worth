# WORTH Foundational Docs

This folder is the crate-facing feature documentation surface for
`worth-foundational`.

Start here if you need the big-picture orientation for crate boundaries and
chooser rules:

- [Foundational Orientation](./FOUNDATIONAL_README.md)

Read by capability area:

- [Aspect Contracts, Values, And Authoritative State](./aspect-contracts-values-and-authoritative-state/README.md)
- [Canonical Basis And Reproducible Identity](./canonical-basis-and-reproducible-identity/README.md)
- [Profiles And Policy Vocabulary](./profiles-and-policy-vocabulary/README.md)
- [Boundary Artifact Taxonomy And Materialization Contracts](./boundary-artifact-taxonomy-and-materialization-contracts/README.md)
- [Branching, Merging, And Commit Vocabulary](./branching-merging-and-commit-vocabulary/README.md)
- [Diagnostics And Explanation Ontology](./diagnostics-and-explanation-ontology/README.md)
- [Lineage, Provenance, Receipts, And Support Truth](./lineage-provenance-receipts-and-support-truth/README.md)
- [Performance, Layout, And Enforcement Vocabulary](./performance/README.md)
- [Scoped Merge And Cherry-Pick Vocabulary](./scoped-merge-adoption.md)
- [Physical Integrity And Offline Observation](../../../_docs/worth-store/physical-integrity-and-offline-verification.md): portable descriptive facts, never runtime/recovery/repair authority.

If you are working through the milestones in order, Milestone 7 is where the
crate stops talking only about artifacts, transitions, and diagnostics, and
starts giving you one shared language for:

- continuity claims
- provenance and freshness
- executed versus planned or blocked boundaries
- support-grade evidence and degraded recovery
- attachment, canonical participation, readmission, and readiness

Milestone 8 builds on that boundary vocabulary and gives you one shared way to
talk about:

- layout intent without storage-equivalence overclaim
- claim boundary, evidence strength, and work disclosure
- pre-execution admission versus executed counter-backed truth
- explicit report widening and support expansion
- proof-bearing certified/readmitted performance bundles when stronger claims
  are real
- grouped public lanes and performance readiness closure

Milestone 9 adds the shared scoped merge/cherry-pick vocabulary that adopting
runtime crates must lower into before implementing branch execution:

- full-branch, selected-node, and selected-aspect scope requests
- admitted scope evidence with direct-source or identity-corresponded basis
- selected no-op evidence distinct from skipped-out-of-scope candidates
- scoped denial and unavailable posture mapped through `worth-proof`
  progression categories
- canonical basis, transition locators, diagnostic explanations, and
  production readiness evidence for downstream runtime adoption
