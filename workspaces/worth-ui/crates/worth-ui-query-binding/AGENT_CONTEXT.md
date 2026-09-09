# worth-ui-query-binding

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `worth/ui`
- Domain noun: `query-binding`
- Crate root: `workspaces/worth-ui/crates/worth-ui-query-binding`
- Road 1 exemplar role: WORTH UI workspace-owned implementation surface.
- Deferred next homes:

- Public surface: workspace-owned; package targets remain the explicit export or composition owners
- Facade exports: `none`
- Owned internal modules: `application_binding, application_item_key_tests, certification, collection_delivery, collection_projection_binding_tests, collection_projection_refresh_tests, collection_text_projection_tests, compatibility, declaration, domain_marker, domain_package, entry, inspection, installed_domain, installed_operations_tests, live_resource, native_aspect_contracts, operation_live, operation_live_tests, prerequisites, presentation_async, product_projection, product_projection_tests, projection_binding, projection_compatibility_tests, projection_consumption, projection_contract_tests, projection_invalidation, projection_observation, reporting_projection, scalar_projection_async_fixture, scalar_projection_drift_tests, scalar_projection_lifecycle_tests, scalar_text_progression_tests, scalar_text_projection_fixture, scalar_text_projection_tests, snapshot_derivation_denial_tests, snapshot_progression_tests, snapshot_refresh_isolation_tests, succession_tests`
- Allowed in-tree dependency bands: `WORTH UI manifest-declared dependencies`

Machine fences:
- Must not depend on worthy-* crates.
- Replay dependencies are admitted only for configured certification packages: worth-ui-certification.
- Production dependencies on the direct Query engine remain confined by the configured Worth UI Query edge; certification-only test dependencies are outside that production fence.

Skeleton fence:
- No Road 1 seed skeleton applies; WORTH UI topology is workspace-owned and mechanically discovered.
