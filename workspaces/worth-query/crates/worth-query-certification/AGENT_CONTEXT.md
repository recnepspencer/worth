# worth-query-certification

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `framework/query-certification`
- Domain noun: `certification`
- Crate root: `workspaces/worth-query/crates/worth-query-certification`
- Road 1 exemplar role: Cold Query compiler and hostile certification over `worth-query-host`, `worth-query-replay`
- Deferred next homes:

- Public surface: facade-only certification contracts
- Declared facade exports (glob routes remain explicit): `WorthQueryCertificationCounter, WorthQueryCertificationCounterSetDenial, WorthQueryCertificationCounters, WorthQueryCertificationDenialBoundary, WorthQueryCertificationDenialEvidence, WorthQueryCertificationFailure, WorthQueryCertificationHostileAttack, WorthQueryCertificationHostileCase, WorthQueryCertificationJourneyCheckpoint, WorthQueryCertificationObservation, WorthQueryCertificationObservationDenial, WorthQueryCertificationProvider, WorthQueryCertificationReport, WorthQueryCertificationScenario, WorthQueryCertificationScenarioDenial, WorthQueryCertificationScenarioKind, WorthQueryCertificationScenarioReport, WorthQueryCertificationSuite, WorthQueryCertificationSuiteDenial, WorthQueryHostileCertificationProvider, WorthQueryHostileCertificationReport, canonical_hostile_matrix, certify_hostile_provider, certify_provider_pair`
- Owned internal modules: `scenario, evidence, oracle`
- Allowed in-tree dependency bands: `none`

Boundary guidance (the compiler and retained checks own enforcement):
- Cold certification facade over `worth-query-host`, `worth-query-replay`; ordinary Query packages must not depend on it.
- Selected explicitly for compiler, replay, or hostile certification; absent from the ordinary workspace default members.
- Configured downstream certification owners: `worth-ui-certification`; all other consumers are denied.
- May expose provider-neutral scenario, evidence, and oracle contracts; must not expose pre-solved Query authority constructors or production runner state.

Skeleton fence:
- Framework certification leaf: cert-only semantic scenarios and hostile evidence; no product authority.
