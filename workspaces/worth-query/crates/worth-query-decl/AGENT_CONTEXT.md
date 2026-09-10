# worth-query-decl

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `framework/query-audience`
- Domain noun: `declaration`
- Crate root: `workspaces/worth-query/crates/worth-query-decl`
- Road 1 exemplar role: Query declaration audience facade over `worth-query-declaration`
- Deferred next homes:

- Public surface: facade-only
- Facade exports: `CanonicalQueryArtifact, application_aftermath, application_capability, application_query, application_schema, authentication, authoring, binding, branch, canonicalization, collection, diagnostics, identity, identity_authority, schema_view, typed, validation, view_declaration, worth_query_ability, worth_query_application_query, worth_query_application_schema, worth_query_aspect, worth_query_capability, worth_query_capability_context, worth_query_capability_context_entity_slot, worth_query_capability_provenance, worth_query_effect, worth_query_entity, worth_query_field, worth_query_operation, worth_query_operation_creates, worth_query_operation_deletes, worth_query_operation_emits, worth_query_operation_expects_fact, worth_query_operation_expects_version, worth_query_operation_links, worth_query_operation_reads, worth_query_operation_requires, worth_query_operation_unlinks, worth_query_operation_writes, worth_query_policy, worth_query_portable_type, worth_query_principal_binding, worth_query_relation, worth_query_unit`
- Owned internal modules: `none`
- Allowed in-tree dependency bands: `none`

Machine fences:
- Framework Query audience facade (`declaration`); legal consuming bands: entry, cert.
- May depend only on its configured authority packages: `worth-query-declaration`; must not depend on other audience facades.
- Leaf re-export surface only; guidance: declaration artifacts and handles.

Skeleton fence:
- Framework audience facade: re-export-only; no seed-skeleton allowlist.
