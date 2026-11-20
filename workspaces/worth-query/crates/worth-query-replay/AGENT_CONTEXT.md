# worth-query-replay

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `framework/query-audience`
- Domain noun: `replay`
- Crate root: `workspaces/worth-query/crates/worth-query-replay`
- Road 1 exemplar role: Query replay audience facade over `worth-query-execution`
- Deferred next homes:

- Public surface: facade-only
- Declared facade exports (glob routes remain explicit): `WorthQueryCertificationApplicationWork, WorthQueryCertificationCostObservation, WorthQueryCertificationCostRuntimeExt, WorthQueryCertificationCostScope, WorthQueryCertificationWorldHistory, WorthQueryCertificationWorldRetention`
- Owned internal modules: `none`
- Allowed in-tree dependency bands: `none`

Boundary guidance (the compiler and retained checks own enforcement):
- Framework Query audience facade (`replay`); legal consuming bands: cert.
- May depend only on its configured authority packages: `worth-query-execution`; must not depend on other audience facades.
- Leaf re-export surface only; guidance: cert-only certification cost evidence.

Skeleton fence:
- Framework audience facade: re-export-only; no seed-skeleton allowlist.
