# worth-signals-wasm Engineering Docs

This folder holds the engineering-side material for `worth-signals-wasm`:
specs, milestone plans, closeouts, and the crate roadmap.

Product-facing package docs live separately in:

- [crates/worth-signal-wasm/docs](../../crates/worth-signal-wasm/docs)

## Core Docs

- [web_runtime_spec.md](web_runtime_spec.md)
  Broad product and architecture spec for the web runtime.
- [wasm_product_roadmap.md](wasm_product_roadmap.md)
  The crate roadmap, including composition, graph lifecycle, forms/resources,
  and aspect-capacity work.

## Milestone And Design Docs

- [host_callback_computed_spec.md](host_callback_computed_spec.md)
- [composition-api-plan.md](composition-api-plan.md)
- [controller_scope_and_graph_lifecycle_plan.md](controller_scope_and_graph_lifecycle_plan.md)
- [opaque_identity_and_ergonomic_authoring_plan.md](opaque_identity_and_ergonomic_authoring_plan.md)
- [api_surface_plan.md](api_surface_plan.md)
- [api_surface_dx_plan.md](api_surface_dx_plan.md)
- [resource_response_lens_contracts_plan.md](resource_response_lens_contracts_plan.md)
  Branch-native resource effects and response-lens topology lowering.
- [branch_merge_materialization_foundation_plan.md](branch_merge_materialization_foundation_plan.md)
  Branch merge materialization strategies, aspect policy exposure, executable
  merge intent, explicit fork basis, and proof-bearing conflict resolution.
- [branch_merge_resolution_and_materialization_plan.md](branch_merge_resolution_and_materialization_plan.md)
  TypeScript local-truth authority, aspect-level branch merge, manual
  resolution, Signal projection, and the standalone/platform boundary.
- [resource_mutation_response_reconciliation_plan.md](resource_mutation_response_reconciliation_plan.md)
  Mutation response reconciliation, granular detail lenses, create/update/remove
  response lanes, identity migration, placement, deletion, and multi-family
  write-result convergence.
- [host_capability_spec.md](host_capability_spec.md)
- [host_capability_closeout.md](host_capability_closeout.md)
- [react_adapter_spec.md](react_adapter_spec.md)
- [worker_runtime_placement_plan.md](worker_runtime_placement_plan.md)
- [worker_runtime_placement_closeout.md](worker_runtime_placement_closeout.md)
- [worker_runtime_product_entrypoint_correction_plan.md](worker_runtime_product_entrypoint_correction_plan.md)
- [worker_runtime_test_requirements.md](worker_runtime_test_requirements.md)
- [router_navigation_projection_plan.md](router_navigation_projection_plan.md)
- [router_test_requirements.md](router_test_requirements.md)

## Reading Order

1. [web_runtime_spec.md](web_runtime_spec.md)
2. [host_callback_computed_spec.md](host_callback_computed_spec.md)
3. [composition-api-plan.md](composition-api-plan.md)
4. [controller_scope_and_graph_lifecycle_plan.md](controller_scope_and_graph_lifecycle_plan.md)
5. [opaque_identity_and_ergonomic_authoring_plan.md](opaque_identity_and_ergonomic_authoring_plan.md)
6. [api_surface_plan.md](api_surface_plan.md)
7. [wasm_product_roadmap.md](wasm_product_roadmap.md)
8. [api_surface_dx_plan.md](api_surface_dx_plan.md)
9. [resource_response_lens_contracts_plan.md](resource_response_lens_contracts_plan.md)
10. [branch_merge_materialization_foundation_plan.md](branch_merge_materialization_foundation_plan.md)
11. [branch_merge_resolution_and_materialization_plan.md](branch_merge_resolution_and_materialization_plan.md)
12. [resource_mutation_response_reconciliation_plan.md](resource_mutation_response_reconciliation_plan.md)
13. [worker_runtime_placement_plan.md](worker_runtime_placement_plan.md)
14. [worker_runtime_test_requirements.md](worker_runtime_test_requirements.md)
15. [router_navigation_projection_plan.md](router_navigation_projection_plan.md)
16. [router_test_requirements.md](router_test_requirements.md)
17. [worker_runtime_placement_closeout.md](worker_runtime_placement_closeout.md)
18. [worker_runtime_product_entrypoint_correction_plan.md](worker_runtime_product_entrypoint_correction_plan.md)
