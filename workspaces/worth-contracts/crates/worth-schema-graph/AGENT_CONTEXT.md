# worth-schema-graph

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `worth/schema`
- Domain noun: `graph`
- Crate root: `workspaces/worth-contracts/crates/worth-schema-graph`
- Road 1 exemplar role: No exemplar route assigned yet.
- Deferred next homes:

- Public surface: facade-only
- Facade exports: `CarryingArtifactIdentity, DurableReferenceKind, GraphPromotionIdentityBasis, PromotionRequest, SubelementKey, lower_graph_promotion_identity_basis`
- Owned internal modules: `promotion`
- Allowed in-tree dependency bands: `none`

Machine fences:
- Must not depend on worthy-* crates.
- Must not depend on Query engine `worth-query` directly; consume only through configured audience facades.
- No Query audience facade is legal for this band; derived and other ordinary bands have no Query audience in this milestone.
- Must not depend on Query audience facade `worth-query-decl` (allowed bands: entry, cert).
- Must not depend on Query audience facade `worth-query-host` (allowed bands: entry, cert).
- Must not depend on Query audience facade `worth-query-replay` (allowed bands: cert).
- Must not depend on replay surface families such as certification replay [worth-cert-replay, worthy-cert-replay; cert domains: replay, reconstruction].

Skeleton fence:
- Seed skeleton is machine-fenced by boundary-check; undeclared files and mixed-class modules are denied.
