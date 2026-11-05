# Hub map: `WorthQueryPrimaryGraphApplicationRuntime`

- impl blocks: 270; methods: 606
- impl blocks by module: application_query 67, application_contribution 33, product_operation 29, application_attempt 27, dc::authorization 20, application_runtime::inbound_occurrence 16, application_runtime::certification_controls 14, application_output_demand 9, application_runtime::inbound_publication 6, provider 5, dc::application_aftermath 4, application_entry 3, application_runtime 3, dc::product_publication 3, application_checkpoint 2, application_runtime::advancement_opening 2, application_runtime::certification_cost 2, application_runtime::conditional_cleanup 2, application_runtime::operation_control 2, causal_inspection 2

## Fields, by the modules whose methods touch them

| field | owner | methods | modules (methods touching) |
|---|---|---|---|
| `runtime.product_runtime` | shared | 100 | application_query 22, application_attempt 13, application_contribution 12, dc::authorization 10, product_operation 10, application_runtime::certification_controls 9 |
| `runtime.authority_identity()` | shared | 62 | application_contribution 22, application_query 15, dc::application_aftermath 5, application_attempt 5, application_runtime::inbound_occurrence 4, dc::authorization 2 |
| `installed_schema` | shared | 45 | application_query 16, application_contribution 8, application_attempt 7, dc::application_aftermath 3, application_entry 2, application_output_demand 2 |
| `runtime.output_demands` | application_contribution | 43 | application_contribution 25, application_output_demand 13, application_runtime::certification_controls 3, application_checkpoint 1, product_operation 1 |
| `runtime.primary_graph()` | shared | 42 | application_query 15, dc::authorization 9, application_runtime::certification_controls 8, application_attempt 3, application_contribution 2, application_runtime::derived_relation 2 |
| `runtime.primary_provider.graph` | shared | 26 | application_contribution 7, product_operation 4, application_runtime::certification_cost 3, application_attempt 2, application_checkpoint 2, application_query 2 |
| `runtime.product_runtime.owner` | shared | 24 | application_runtime::inbound_occurrence 4, product_operation 4, application_runtime::inbound_publication 3, dc::product_publication 3, application_runtime::certification_cost 2, application_runtime::operation_control 2 |
| `runtime.inbound_custody` | application_runtime::inbound_occurrence | 16 | application_runtime::inbound_occurrence 15, application_runtime::external_dispatch_attempt 1 |
| `runtime.authorization` | dc::authorization | 13 | dc::authorization 13 |
| `runtime.bridge` | shared | 12 | application_runtime::operation_control 4, conditional_operation 3, application_contribution 2, application_runtime::certification_controls 1, application_runtime::conditional_cleanup 1, product_activation 1 |
| `installed_producers` | application_contribution | 10 | application_contribution 7, product_operation 2, application_runtime 1 |
| `runtime.installed_operations` | application_attempt | 10 | application_attempt 5, application_entry 2, provider 2, product_operation 1 |
| `runtime.product_runtime.admit_product_occurrence()` | shared | 9 | application_runtime::inbound_publication 3, product_operation 2, application_checkpoint 1, application_runtime::certification_controls 1, application_runtime::inbound_occurrence 1, provider 1 |
| `runtime.authorization_clock` | shared | 8 | dc::application_aftermath 3, application_runtime::inbound_occurrence 3, dc::authorization 1, application_runtime::external_dispatch_attempt 1 |
| `runtime.primary_provider.lookup_completed_inbound()` | application_runtime::inbound_occurrence | 8 | application_runtime::inbound_occurrence 4, application_runtime::inbound_publication 2, application_attempt 1, application_runtime::external_dispatch_attempt 1 |
| `runtime.primary_provider.application_attempt_operation_control` | application_runtime::operation_control | 7 | application_runtime::operation_control 5, application_contribution 2 |
| `runtime.primary_provider.unpublished_idempotency_disposition()` | shared | 7 | application_runtime::inbound_publication 3, dc::product_publication 2, application_runtime::inbound_occurrence 1, product_operation 1 |
| `runtime.retain_primary_graph_integration_handle()` | application_runtime::certification_controls | 7 | application_runtime::certification_controls 7 |
| `program_support` | application_attempt | 6 | application_attempt 3, application_runtime 3 |
| `runtime.next_output_producer_attempt` | shared | 6 | application_contribution 2, application_runtime::certification_cost 2, application_output_demand 1, application_runtime::certification_controls 1 |
| `conditional_operations` | conditional_operation | 5 | conditional_operation 4, application_runtime::conditional_cleanup 1 |
| `managed_indexes` | application_query | 5 | application_query 5 |
| `runtime.inbound_verifiers` | application_runtime::inbound_occurrence | 5 | application_runtime::inbound_occurrence 5 |
| `runtime.primary_provider.application_branch_commit_lane_for_occurrence()` | application_runtime::inbound_publication | 5 | application_runtime::inbound_publication 3, application_checkpoint 1, application_runtime::inbound_occurrence 1 |
| `runtime.product_runtime.activations` | application_runtime::operation_control | 5 | application_runtime::operation_control 3, product_activation 1, product_operation 1 |
| `runtime.transport_completion_custody` | application_runtime::inbound_publication | 5 | application_runtime::inbound_publication 5 |
| `runtime.external_effect_transport` | application_attempt | 4 | application_attempt 4 |
| `runtime.primary_provider.clear_inbound_completion_publication_pending()` | application_runtime::inbound_publication | 4 | application_runtime::inbound_publication 3, application_runtime::inbound_occurrence 1 |
| `runtime.product_runtime.source` | application_runtime::inbound_publication | 4 | application_runtime::inbound_publication 2, application_runtime::external_dispatch_attempt 1, application_runtime::inbound_occurrence 1 |
| `runtime.output_producer_routes` | application_contribution | 3 | application_contribution 3 |
| `runtime.output_readiness_routes` | application_contribution | 3 | application_contribution 3 |
| `runtime.primary_provider.application_attempt_work_for_test()` | application_runtime::certification_cost | 3 | application_runtime::certification_cost 2, application_runtime::certification_controls 1 |
| `runtime.primary_provider.application_branch_commit_lane()` | application_attempt | 3 | application_attempt 2, provider 1 |
| `runtime.primary_provider.live_delivery` | application_query | 3 | application_query 3 |
| `runtime.product_runtime.operation_control_for_test()` | application_runtime::operation_control | 3 | application_runtime::operation_control 3 |
| `runtime.product_runtime.root_identity()` | product_operation | 3 | product_operation 2, application_contribution 1 |
| `runtime.product_runtime.with_product_observation()` | application_query | 3 | application_query 2, product_operation 1 |
| `runtime.recovered_outputs` | application_contribution | 3 | application_contribution 2, application_checkpoint 1 |
| `runtime.relational_branch_identity` | application_query | 3 | application_query 2, application_runtime::certification_controls 1 |
| `installed_resources` | application_runtime::conditional_cleanup | 2 | application_runtime::conditional_cleanup 1, conditional_operation 1 |
| `mutation_handlers` | handler | 2 | handler 2 |
| `program_required_operations` | application_runtime | 2 | application_runtime 2 |
| `runtime.application_readiness_schema_token` | application_query | 2 | application_query 2 |
| `runtime.installed_operations()` | dc::application_aftermath | 2 | dc::application_aftermath 2 |
| `runtime.installed_packages()` | application_query | 2 | application_query 1, conditional_operation 1 |
| `runtime.primary_graph_authority()` | application_query | 2 | application_query 2 |
| `runtime.primary_provider.correction_commit_lane()` | application_attempt | 2 | application_attempt 1, provider 1 |
| `runtime.primary_provider.mark_inbound_completion_publication_pending()` | application_runtime::inbound_publication | 2 | application_runtime::inbound_publication 2 |
| `runtime.primary_provider.publication_fault_control` | application_runtime::certification_controls | 2 | application_runtime::certification_controls 2 |
| `runtime.product_runtime.default_occurrence` | application_checkpoint | 2 | application_checkpoint 1, application_runtime::operation_control 1 |
| `runtime.product_runtime.integration_admit_product_branch()` | application_runtime::certification_controls | 2 | application_runtime::certification_controls 1, application_runtime::certification_cost 1 |
| `runtime.product_runtime.recovered_root_authority` | application_contribution | 2 | application_contribution 2 |
| `runtime.product_runtime.reserve_owner_cleanup_for_application()` | application_runtime::operation_control | 2 | application_runtime::operation_control 1, dc::product_publication 1 |
| `runtime.product_runtime.with_funded_advancement()` | application_runtime::advancement_opening | 2 | application_runtime::advancement_opening 2 |
| `runtime.recovery_handles` | dc::application_aftermath | 2 | dc::application_aftermath 2 |
| `runtime.validate_commit_program()` | application_attempt | 2 | application_attempt 2 |
| `runtime.validate_source_admission()` | application_query | 2 | application_query 2 |
| `installed_conditionals` | shared | 1 | application_runtime 1 |
| `mutation_projection` | shared | 1 | application_entry 1 |
| `output_teardown` | shared | 1 | application_runtime::conditional_cleanup 1 |
| `post_conditional_teardown` | shared | 1 | application_runtime::conditional_cleanup 1 |
| `program_required_bindings` | shared | 1 | application_runtime 1 |
| `runtime` | shared | 1 | application_runtime::conditional_cleanup 1 |
| `runtime.application_query_basis_observer()` | shared | 1 | application_runtime 1 |
| `runtime.authentication_is_expired()` | shared | 1 | application_runtime 1 |
| `runtime.basis_leases` | shared | 1 | product_operation 1 |
| `runtime.capability_plan_compilation_evidence()` | shared | 1 | application_runtime 1 |
| `runtime.checkpoint_restore_work()` | shared | 1 | application_runtime 1 |
| `runtime.close_live_delivery()` | shared | 1 | application_runtime 1 |
| `runtime.execution` | shared | 1 | application_runtime::conditional_cleanup 1 |
| `runtime.granular_invalidation` | shared | 1 | conditional_operation 1 |
| `runtime.granular_invalidation_installation()` | shared | 1 | application_runtime 1 |
| `runtime.graph_work_provider_identity()` | shared | 1 | application_runtime 1 |
| `runtime.graph_work_resource_support()` | shared | 1 | application_runtime 1 |
| `runtime.graph_work_resource_support_ref()` | shared | 1 | application_runtime 1 |
| `runtime.inbound_maintenance_turn` | shared | 1 | application_runtime::inbound_occurrence 1 |
| `runtime.next_application_mutation_partition` | shared | 1 | application_runtime::mutation_partition 1 |
| `runtime.next_external_dispatch_attempt` | shared | 1 | application_runtime::external_dispatch_attempt 1 |
| `runtime.output_demand_resource_profile()` | shared | 1 | application_contribution 1 |
| `runtime.primary_provider` | shared | 1 | provider 1 |
| `runtime.primary_provider.abandon_inbound_reconstruction()` | shared | 1 | application_runtime::inbound_occurrence 1 |
| `runtime.primary_provider.active_application_attempt_count_for_test()` | shared | 1 | application_runtime::operation_control 1 |
| `runtime.primary_provider.active_live_consumer_count_for_test()` | shared | 1 | application_runtime::operation_control 1 |
| `runtime.primary_provider.advance_inbound_reconstruction()` | shared | 1 | application_runtime::inbound_occurrence 1 |
| `runtime.primary_provider.application_attempt_resource_count()` | shared | 1 | application_runtime::certification_controls 1 |
| `runtime.primary_provider.begin_external_dispatch_in_flight()` | shared | 1 | application_attempt 1 |
| `runtime.primary_provider.begin_managed_unpublished_recovery()` | shared | 1 | provider 1 |
| `runtime.primary_provider.cancel_after_next_inspection_materialization_for_test()` | shared | 1 | application_runtime::certification_controls 1 |
| `runtime.primary_provider.cancel_after_next_output_comparison_for_test()` | shared | 1 | application_runtime::certification_controls 1 |
| `runtime.primary_provider.committed_dispatch_outbox()` | shared | 1 | provider 1 |
| `runtime.primary_provider.committed_dispatch_outbox_for_binding()` | shared | 1 | application_attempt 1 |
| `runtime.primary_provider.committed_dispatch_outbox_for_correlation()` | shared | 1 | provider 1 |
| `runtime.primary_provider.completed_evidence_retained()` | shared | 1 | provider 1 |
| `runtime.primary_provider.completion_row_from_commit_patch()` | shared | 1 | application_runtime::inbound_occurrence 1 |
| `runtime.primary_provider.correction_commit_lanes()` | shared | 1 | application_attempt 1 |
| `runtime.primary_provider.delay_next_output_readiness_delivery_for_test()` | shared | 1 | application_runtime::certification_controls 1 |
| `runtime.primary_provider.fail_next_output_readiness_evaluation_for_test()` | shared | 1 | application_runtime::certification_controls 1 |
| `runtime.primary_provider.fail_next_post_commit_snapshot_for_test()` | shared | 1 | application_runtime::certification_controls 1 |
| `runtime.primary_provider.inbound_reconstruction_cursor()` | shared | 1 | application_runtime::inbound_occurrence 1 |
| `runtime.primary_provider.managed_cleanup_control` | shared | 1 | application_runtime::certification_controls 1 |
| `runtime.primary_provider.observe_branch_commit_contention_for_test()` | shared | 1 | application_runtime::operation_control 1 |
| `runtime.primary_provider.observe_external_dispatch_admission()` | shared | 1 | application_runtime::external_dispatch_attempt 1 |
| `runtime.primary_provider.output_comparison_completed()` | shared | 1 | application_contribution 1 |
| `runtime.primary_provider.outstanding_dispatch_count_for_operation()` | shared | 1 | application_runtime::inbound_occurrence 1 |
| `runtime.primary_provider.prepare_inbound_completion_candidate()` | shared | 1 | application_runtime::inbound_publication 1 |
| `runtime.primary_provider.prepare_installed_transport_completion_candidate()` | shared | 1 | application_runtime::inbound_publication 1 |
| `runtime.primary_provider.press_next_readiness_with_world_snapshots_for_test()` | shared | 1 | application_runtime::certification_controls 1 |
| `runtime.primary_provider.press_next_ready_read_with_world_snapshots_for_test()` | shared | 1 | application_runtime::certification_controls 1 |
| `runtime.primary_provider.release_installed_transport_dispatch_provenance()` | shared | 1 | application_runtime::inbound_publication 1 |
| `runtime.primary_provider.release_terminal_dispatch_provenance()` | shared | 1 | application_runtime::inbound_occurrence 1 |
| `runtime.primary_provider.resolve_guarded_workflow_operation_custody()` | shared | 1 | application_attempt 1 |
| `runtime.primary_provider.resolve_idempotency_binding_at_product()` | shared | 1 | application_attempt 1 |
| `runtime.primary_provider.retained_application_commit_basis()` | shared | 1 | application_query 1 |
| `runtime.primary_provider.take_delayed_output_readiness_delivery()` | shared | 1 | application_contribution 1 |
| `runtime.primary_provider.take_failed_output_readiness_evaluation()` | shared | 1 | application_contribution 1 |
| `runtime.primary_provider.take_readiness_snapshot_pressure()` | shared | 1 | application_contribution 1 |
| `runtime.primary_provider.verify_completed_inbound_for_cleanup()` | shared | 1 | application_runtime::inbound_occurrence 1 |
| `runtime.product_runtime.admit_product_branch()` | shared | 1 | product_operation 1 |
| `runtime.product_runtime.advancement_owner()` | shared | 1 | application_runtime::advancement_opening 1 |
| `runtime.product_runtime.advancement_source_instance()` | shared | 1 | application_runtime::advancement_opening 1 |
| `runtime.product_runtime.clone()` | shared | 1 | dc::product_publication 1 |
| `runtime.product_runtime.enter_named_read()` | shared | 1 | product_operation 1 |
| `runtime.product_runtime.lease_from_observation()` | shared | 1 | application_query 1 |
| `runtime.product_runtime.owner_cleanup` | shared | 1 | application_runtime::operation_control 1 |
| `runtime.product_runtime.product_branches()` | shared | 1 | product_operation 1 |
| `runtime.product_runtime.publication_binding()` | shared | 1 | product_operation 1 |
| `runtime.product_runtime.reserve_owner_cleanup_for_creation()` | shared | 1 | dc::product_publication 1 |
| `runtime.product_runtime.root_product_branch()` | shared | 1 | product_operation 1 |
| `runtime.product_runtime.with_advancement()` | shared | 1 | application_runtime::advancement_opening 1 |
| `runtime.product_runtime.with_request_advancement()` | shared | 1 | application_runtime::advancement_opening 1 |
| `runtime.publication()` | shared | 1 | application_runtime 1 |
| `runtime.recovered_program_source_delivery` | shared | 1 | provider 1 |
| `runtime.retain_installed_packages()` | shared | 1 | conditional_operation 1 |
| `runtime.validate_active_capability_currentness()` | shared | 1 | dc::authorization 1 |
| `runtime.workflow_compilation_reuse_counters()` | shared | 1 | application_runtime 1 |
| `runtime.workflow_instance_progress_counters()` | shared | 1 | application_runtime 1 |
| `workflow_guarded_operations` | shared | 1 | application_runtime 1 |
| `workflow_teardown` | shared | 1 | application_runtime::conditional_cleanup 1 |

## Candidate subsystems (heaviest first)

- **application_contribution**: 54 methods, 64 owned-field accesses; fields `installed_producers`, `runtime.output_demands`, `runtime.output_producer_routes`, `runtime.output_readiness_routes`, `runtime.product_runtime.recovered_root_authority`, `runtime.recovered_outputs`
- **application_runtime::inbound_occurrence**: 21 methods, 26 owned-field accesses; fields `runtime.inbound_custody`, `runtime.inbound_verifiers`, `runtime.primary_provider.lookup_completed_inbound()`
- **application_attempt**: 19 methods, 25 owned-field accesses; fields `program_support`, `runtime.external_effect_transport`, `runtime.installed_operations`, `runtime.primary_provider.application_branch_commit_lane()`, `runtime.primary_provider.correction_commit_lane()`, `runtime.validate_commit_program()`
- **application_query**: 20 methods, 21 owned-field accesses; fields `managed_indexes`, `runtime.application_readiness_schema_token`, `runtime.installed_packages()`, `runtime.primary_graph_authority()`, `runtime.primary_provider.live_delivery`, `runtime.product_runtime.with_product_observation()`, `runtime.relational_branch_identity`, `runtime.validate_source_admission()`
- **application_runtime::inbound_publication**: 10 methods, 18 owned-field accesses; fields `runtime.primary_provider.application_branch_commit_lane_for_occurrence()`, `runtime.primary_provider.clear_inbound_completion_publication_pending()`, `runtime.primary_provider.mark_inbound_completion_publication_pending()`, `runtime.product_runtime.source`, `runtime.transport_completion_custody`
- **application_runtime::operation_control**: 15 methods, 15 owned-field accesses; fields `runtime.primary_provider.application_attempt_operation_control`, `runtime.product_runtime.activations`, `runtime.product_runtime.operation_control_for_test()`, `runtime.product_runtime.reserve_owner_cleanup_for_application()`
- **dc::authorization**: 13 methods, 13 owned-field accesses; fields `runtime.authorization`
- **application_runtime::certification_controls**: 10 methods, 10 owned-field accesses; fields `runtime.primary_provider.publication_fault_control`, `runtime.product_runtime.integration_admit_product_branch()`, `runtime.retain_primary_graph_integration_handle()`
- **conditional_operation**: 5 methods, 5 owned-field accesses; fields `conditional_operations`
- **dc::application_aftermath**: 4 methods, 4 owned-field accesses; fields `runtime.installed_operations()`, `runtime.recovery_handles`
- **application_runtime::certification_cost**: 3 methods, 3 owned-field accesses; fields `runtime.primary_provider.application_attempt_work_for_test()`
- **application_runtime**: 2 methods, 2 owned-field accesses; fields `program_required_operations`

## Cross-subsystem calls (would become narrow traits)

- application_runtime::inbound_publication -> application_runtime::inbound_occurrence: `reserve_installed_transport_publication` (2)
- application_runtime::inbound_publication -> application_runtime::inbound_occurrence: `resolve_installed_transport_completion_binding` (2)
- application_attempt -> application_runtime::inbound_occurrence: `refuse_completed_external_effect` (1)
- application_attempt -> handler: `mutation_handler_for_attempt` (1)
- application_contribution -> application_runtime::certification_controls: `hold_world_snapshot_pressure_for_test` (1)
- application_contribution -> application_runtime::operation_control: `own_publication_is_selected` (1)
- application_query -> application_contribution: `admit_with_observation_room` (1)
- application_runtime::inbound_occurrence -> application_runtime::inbound_publication: `pending_transport_maintenance_count` (1)
- application_runtime::inbound_occurrence -> application_runtime::inbound_publication: `progress_unpublished_inbound_occurrence` (1)
- application_runtime::inbound_occurrence -> application_runtime::inbound_publication: `publish_inbound_completion` (1)
- application_runtime::inbound_occurrence -> application_runtime::inbound_publication: `resume_installed_transport_completion` (1)
- application_runtime::inbound_occurrence -> application_runtime::inbound_publication: `transport_maintenance_candidates` (1)
- application_runtime::inbound_publication -> application_runtime::inbound_occurrence: `inbound_cost_for_operation` (1)
- application_runtime::inbound_publication -> application_runtime::inbound_occurrence: `refuse_completed_external_effect` (1)
- application_runtime::inbound_publication -> application_runtime::inbound_occurrence: `release_retained_inbound_terminal` (1)

## Fields used outside their subsystem

- application_contribution uses product_operation's `runtime.product_runtime.root_identity()` (3 methods)
- application_runtime uses application_attempt's `program_support` (2 methods)
- application_runtime::inbound_publication uses application_runtime::inbound_occurrence's `runtime.primary_provider.lookup_completed_inbound()` (2 methods)
- conditional_operation uses application_runtime::conditional_cleanup's `installed_resources` (2 methods)
- application_checkpoint uses application_runtime::operation_control's `runtime.product_runtime.operation_control_for_test()` (1 methods)
- application_contribution uses application_checkpoint's `runtime.product_runtime.default_occurrence` (1 methods)
- application_contribution uses application_runtime::inbound_publication's `runtime.primary_provider.application_branch_commit_lane_for_occurrence()` (1 methods)
- application_contribution uses application_runtime::operation_control's `runtime.product_runtime.activations` (1 methods)
- application_runtime::certification_cost uses application_runtime::certification_controls's `runtime.product_runtime.integration_admit_product_branch()` (1 methods)
- application_runtime::inbound_occurrence uses application_runtime::inbound_publication's `runtime.product_runtime.source` (1 methods)
- application_runtime::inbound_publication uses application_runtime::inbound_occurrence's `runtime.inbound_custody` (1 methods)
- conditional_operation uses application_query's `runtime.installed_packages()` (1 methods)
