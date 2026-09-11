# worth-query-host

Generated from the machine-owned Road 1 boundary model. Do not edit by hand.
Canonical machine constitution: `tools/boundary-check/config/road1.toml`

- Constitutional class: `framework/query-audience`
- Domain noun: `host`
- Crate root: `workspaces/worth-query/crates/worth-query-host`
- Road 1 exemplar role: Query host audience facade over `worth-query-admission`, `worth-query-declaration`, `worth-query-execution`, `worth-query-installation`, `worth-query-publication`
- Deferred next homes:

- Public surface: facade-only
- Facade exports: `WorthQueryGraphObligationAdoptionDenial, WorthQueryGraphObligationAdoptionDenialKind, WorthQueryGraphObligationAdoptionProof, WorthQueryGraphObligationAdoptionRow, admission, convergence_epoch, declaration, domain, inspect_installed_graph_obligations, installed, primary_graph, product, provisional_aftermath, publication, runtime, worth_query_application_query, worth_query_application_schema, worth_query_aspect, worth_query_conditional_node, worth_query_effect, worth_query_entity, worth_query_field, worth_query_operation, worth_query_operation_emits, worth_query_operation_reads, worth_query_operation_writes, worth_query_portable_type, worth_query_principal_binding, worth_query_relation, worth_query_structured_value_binding`
- Owned internal modules: `none`
- Allowed in-tree dependency bands: `none`

Machine fences:
- Framework Query audience facade (`host`); legal consuming bands: entry, cert.
- May depend only on its configured authority packages: `worth-query-admission`, `worth-query-declaration`, `worth-query-execution`, `worth-query-installation`, `worth-query-publication`; must not depend on other audience facades.
- Leaf re-export surface only; guidance: admission, lowering, and execution.

Skeleton fence:
- Framework audience facade: re-export-only; no seed-skeleton allowlist.
