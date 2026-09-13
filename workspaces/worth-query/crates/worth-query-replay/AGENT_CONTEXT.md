# worth-query-replay

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `framework/query-audience`
- Domain noun: `replay`
- Crate root: `workspaces/worth-query/crates/worth-query-replay`
- Road 1 exemplar role: Query replay audience facade over `worth-query`
- Deferred next homes:

- Public surface: facade-only
- Facade exports: `ScopedReplayBasis, WorthQueryCertificationApplicationWork, WorthQueryCertificationCostObservation, WorthQueryCertificationCostRuntimeExt, WorthQueryCertificationCostScope, WorthQueryCertificationReplayAdmissionDenial, WorthQueryCertificationReplayCapability, WorthQueryCertificationReplayCounters, WorthQueryCertificationReplayOutcome, WorthQueryCertificationReplayResult, WorthQueryCertificationReplayStop, WorthQueryCertificationWorldHistory, WorthQueryCertificationWorldRetention, WorthQueryHistoricalContext, WorthQueryHistoricalReplayAdmission, WorthQueryHistoricalReplayAdmissionDenial, WorthQueryInstalledHistoricalReplayPath, WorthQueryReplayBasisRelationship, WorthQueryReplayComparison, WorthQueryReplayDivergence, admit_installed_historical_replay_basis, issue_query_certification_replay_capability, replay_installed_workflow, replay_installed_workflow_historical`
- Owned internal modules: `none`
- Allowed in-tree dependency bands: `none`

Machine fences:
- Framework Query audience facade (`replay`); legal consuming bands: cert.
- May depend only on its configured authority packages: `worth-query`; must not depend on other audience facades.
- Leaf re-export surface only; guidance: cert-only reconstruction and replay.

Skeleton fence:
- Framework audience facade: re-export-only; no seed-skeleton allowlist.
