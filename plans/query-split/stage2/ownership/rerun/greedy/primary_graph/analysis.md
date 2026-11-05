# Ownership analysis: `domain_computation::primary_graph`

## Draft target stack (bottom to top)


### admitted_capture

- `admitted_capture` (rank 0): greedy rank 0; uses below nothing; above nothing; used by causal_inspection 68, application_query 10, application_resource 7.

### aggregate_identity

- `aggregate_identity` (rank 1): greedy rank 1; uses below nothing; above nothing; used by invariant_projection 12, application_invariant 6, facade 3.

### application_branch

- `application_branch` (rank 2): greedy rank 2; uses below nothing; above nothing; used by application_runtime 3, conditional_operation 2, managed_bridge 1.

### entity_key

- `entity_key` (rank 3): greedy rank 3; uses below nothing; above nothing; used by facade 2, application_attempt 2, typed_bootstrap 1.

### entity_resolution_denial

- `entity_resolution_denial` (rank 4): greedy rank 4; uses below nothing; above nothing; used by entity_resolution 20, invariant_projection 12, conditional_operation 10.

### exact_basis_access

- `exact_basis_access` (rank 5): greedy rank 5; uses below nothing; above nothing; used by provider 11, product_operation 3, application_query 2.

### index_maintenance_budget

- `index_maintenance_budget` (rank 6): greedy rank 6; uses below nothing; above nothing; used by provider 4, index_refresh 2, index_currency 2.

### output_binding_identity

- `output_binding_identity` (rank 7): greedy rank 7; uses below nothing; above nothing; used by output_lineage 6.

### output_family_identity

- `output_family_identity` (rank 8): greedy rank 8; uses below nothing; above nothing; used by output_lineage 3, application_output_demand 2, application_contribution 2.

### principal_key

- `principal_key` (rank 9): greedy rank 9; uses below nothing; above nothing; used by facade 2, bootstrap 1.

### publication_request_denial

- `publication_request_denial` (rank 10): greedy rank 10; uses below nothing; above nothing; used by nothing.

### read_observation

- `read_observation` (rank 11): greedy rank 11; uses below nothing; above nothing; used by invariant_projection 12, application_query 9, root_path_selection 5.

### request_allocation_control

- `request_allocation_control` (rank 12): greedy rank 12; uses below nothing; above nothing; used by application_attempt 3, provider 1, product_operation 1.

### request_local

- `request_local` (rank 13): greedy rank 13; uses below nothing; above nothing; used by invariant_projection 1, application_contribution 1.

### selected_read_indexes

- `selected_read_indexes` (rank 14): greedy rank 14; uses below nothing; above nothing; used by application_query 4, product_operation 2, application_runtime 1.

### value_decode

- `value_decode` (rank 15): greedy rank 15; uses below nothing; above nothing; used by application_query 23, application_attempt 10, invariant_projection 9.

### denial

- `denial` (rank 16): greedy rank 16; uses below nothing; above application_contribution 1; used by application_contribution 48, schema_layout 35, bootstrap 35.

### initial_schema_denial

- `initial_schema_denial` (rank 17): greedy rank 17; uses below denial 2; above nothing; used by bootstrap 2.

### aggregate_projection

- `aggregate_projection` (rank 18): greedy rank 18; uses below nothing; above application_attempt 1; used by invariant_projection 5, root 2.

### application_program

- `application_program` (rank 19): greedy rank 19; uses below nothing; above application_output_demand 2, application_installation 2, application_contribution 1; used by application_installation 27, facade 7, application_contribution 5.

### expression

- `expression` (rank 20): greedy rank 20; uses below nothing; above application_attempt 4, application_query 2; used by application_attempt 9, facade 1.

### fact_verification_observation

- `fact_verification_observation` (rank 21): greedy rank 21; uses below nothing; above output_lineage 2, application_attempt 1; used by application_attempt 18, facade 12, application_output_demand 12.

### structural_walk

- `structural_walk` (rank 22): greedy rank 22; uses below nothing; above application_attempt 1; used by invariant_projection 17, application_query 9, source_facts 6.

### derived_source

- `derived_source` (rank 23): greedy rank 23; uses below structural_walk 2; above application_attempt 1; used by source_facts 4, source_read_verification 2, root_path_selection 2.

### typed_bootstrap

- `typed_bootstrap` (rank 24): greedy rank 24; uses below denial 2, entity_key 1; above bootstrap 1; used by bootstrap 4, facade 2, bootstrap_publication 2.

### program_occurrence

- `program_occurrence` (rank 25): greedy rank 25; uses below nothing; above application_installation 3, application_attempt 2, product_operation 1; used by application_attempt 13, application_installation 8, product_operation 7.

### authenticated_principal

- `authenticated_principal` (rank 26): greedy rank 26; uses below nothing; above freshness 1; used by application_query 13, resolution 5, application_contribution 3.

### schema_layout

- `schema_layout` (rank 27): greedy rank 27; uses below denial 35; above application_attempt 3, application_discovery 2; used by application_attempt 66, application_query 36, invariant_projection 17.

### source_facts

- `source_facts` (rank 28): greedy rank 28; uses below structural_walk 6, derived_source 4, admitted_capture 2; above application_attempt 3, invariant_projection 2; used by application_query 100, invariant_projection 13, component_selection 13.

### root

- `root` (rank 29): greedy rank 29; uses below schema_layout 5, aggregate_projection 2; above application_attempt 7, application_query 3, output_lineage 2; used by application_attempt 13, application_query 8, bootstrap 7.

### managed_bridge

- `managed_bridge` (rank 30): greedy rank 30; uses below denial 2, schema_layout 1, root 1; above nothing; used by application_runtime 7, runtime_host 1.

### index_currency

- `index_currency` (rank 31): greedy rank 31; uses below root 2, index_maintenance_budget 2, exact_basis_access 1; above nothing; used by application_discovery 5, resolution 4, application_query 3.

### granular_invalidation

- `granular_invalidation` (rank 32): greedy rank 32; uses below root 1; above nothing; used by conditional_operation 16, facade 7, application_runtime 5.

### runtime_host

- `runtime_host` (rank 33): greedy rank 33; uses below root 2, managed_bridge 1, granular_invalidation 1; above application_runtime 1; used by application_installation 12, application_output_demand 4, product_operation 3.

### resolution_denial

- `resolution_denial` (rank 34): greedy rank 34; uses below index_currency 1, exact_basis_access 1; above provider 2; used by resolution 22, observations 8, conditional_operation 8.

### product_activation

- `product_activation` (rank 35): greedy rank 35; uses below program_occurrence 2, runtime_host 1; above provider 1, application_query 1; used by product_operation 11, application_runtime 3, application_installation 3.

### live_delivery

- `live_delivery` (rank 36): greedy rank 36; uses below source_facts 6; above application_attempt 8, application_query 4; used by application_query 13, provider 9, application_discovery 2.

### application_discovery

- `application_discovery` (rank 37): greedy rank 37; uses below index_currency 5, live_delivery 2, runtime_host 1; above application_attempt 34, product_operation 10, application_installation 5; used by application_attempt 18, application_runtime 3, schema_layout 2.

### observations

- `observations` (rank 38): greedy rank 38; uses below resolution_denial 8, schema_layout 2; above nothing; used by resolution 5, freshness 5.

### freshness

- `freshness` (rank 39): greedy rank 39; uses below observations 5, resolution_denial 4, schema_layout 3; above application_attempt 3; used by resolution 4, application_attempt 2, authenticated_principal 1.

### application_entry

- `application_entry` (rank 40): greedy rank 40; uses below runtime_host 1, request_allocation_control 1, read_observation 1; above handler 10, application_contribution 7, application_attempt 6; used by handler 30, operation_body 19, invariant_projection 19.

### application_checkpoint

- `application_checkpoint` (rank 41): greedy rank 41; uses below fact_verification_observation 8, denial 6, runtime_host 1; above application_attempt 44, application_output_demand 17, output_lineage 4; used by application_attempt 53, output_lineage 33, bootstrap 14.

### aggregate_observation

- `aggregate_observation` (rank 42): greedy rank 42; uses below source_facts 2; above application_attempt 3; used by invariant_projection 23, application_query 12, component_selection 10.

### source_read_verification

- `source_read_verification` (rank 43): greedy rank 43; uses below structural_walk 3, derived_source 2, aggregate_observation 1; above application_attempt 3, component_selection 2; used by application_attempt 2, output_reuse 1, component_selection 1.

### output_reuse

- `output_reuse` (rank 44): greedy rank 44; uses below source_read_verification 1, application_checkpoint 1; above application_output_demand 6, application_attempt 6, output_lineage 5; used by application_contribution 33, application_output_demand 26, application_query 8.

### entity_resolution

- `entity_resolution` (rank 45): greedy rank 45; uses below entity_resolution_denial 20, schema_layout 3, index_currency 2; above resolution 6, product_operation 5, application_query 4; used by application_query 29, application_attempt 10, causal_inspection 8.

### content

- `content` (rank 46): greedy rank 46; uses below value_decode 6, application_entry 1; above application_attempt 1; used by invariant_projection 3, handler 3, application_attempt 3.

### access_strategy

- `access_strategy` (rank 47): greedy rank 47; uses below source_facts 2; above application_query 29, application_attempt 1; used by application_query 36, application_attempt 3, product_operation 1.

### product_operation

- `product_operation` (rank 48): greedy rank 48; uses below product_activation 11, program_occurrence 7, fact_verification_observation 4; above application_query 100, application_attempt 75, application_contribution 24; used by application_query 132, facade 103, application_runtime 75.

### root_path_selection

- `root_path_selection` (rank 49): greedy rank 49; uses below source_facts 9, read_observation 5, structural_walk 4; above nothing; used by application_query 13, invariant_projection 10, component_selection 2.

### component_selection

- `component_selection` (rank 50): greedy rank 50; uses below source_facts 13, aggregate_observation 10, schema_layout 3; above application_attempt 4; used by invariant_projection 19, application_query 14, application_attempt 5.

### resolution

- `resolution` (rank 51): greedy rank 51; uses below resolution_denial 22, product_operation 5, observations 5; above provider 2, application_query 2; used by application_query 14, entity_resolution 6, causal_inspection 5.

### application_invariant

- `application_invariant` (rank 52): greedy rank 52; uses below schema_layout 6, aggregate_identity 6, read_observation 3; above application_query 5; used by invariant_installation 10, application_contribution 3, application_attempt 2.

### principal_currentness_capture

- `principal_currentness_capture` (rank 53): greedy rank 53; uses below runtime_host 1, authenticated_principal 1; above nothing; used by application_query 2, application_runtime 1, application_resource 1.

### invariant_installation

- `invariant_installation` (rank 54): greedy rank 54; uses below application_invariant 10, denial 9, schema_layout 5; above application_query 5, application_attempt 2; used by bootstrap 3, application_contribution 3, facade 2.

### bootstrap_publication

- `bootstrap_publication` (rank 55): greedy rank 55; uses below denial 4, typed_bootstrap 2, root 1; above provider 3, bootstrap 1, application_query 1; used by bootstrap 14.

### application_resource

- `application_resource` (rank 56): greedy rank 56; uses below admitted_capture 7, resolution 4, entity_resolution 4; above application_query 9, provider 3, application_attempt 1; used by application_runtime 6, application_contribution 3, conditional_operation 1.

### bootstrap

- `bootstrap` (rank 57): greedy rank 57; uses below denial 35, bootstrap_publication 14, application_checkpoint 14; above application_query 9, provider 5, handler 2; used by application_installation 25, application_runtime 9, application_checkpoint 4.

### source_disclosure

- `source_disclosure` (rank 58): greedy rank 58; uses below entity_resolution 3; above application_attempt 3, application_runtime 2, application_query 2; used by causal_inspection 4, application_attempt 4, application_output_demand 2.

### operation_body

- `operation_body` (rank 59): greedy rank 59; uses below application_entry 19, denial 8, runtime_host 1; above application_attempt 18, invariant_projection 17, handler 8; used by handler 42, invariant_projection 14, application_attempt 13.

### application_inbound

- `application_inbound` (rank 60): greedy rank 60; uses below runtime_host 1; above provider 24, application_runtime 18, application_attempt 5; used by application_runtime 27, facade 17, provider 10.

### application_attempt

- `application_attempt` (rank 61): greedy rank 61; uses below schema_layout 66, application_checkpoint 53, product_operation 39; above provider 63, invariant_projection 37, application_contribution 29; used by provider 149, facade 118, invariant_projection 111.

### conditional_operation

- `conditional_operation` (rank 62): greedy rank 62; uses below application_attempt 29, granular_invalidation 16, product_operation 11; above application_query 18, invariant_projection 13, application_contribution 6; used by facade 36, application_contribution 18, application_runtime 17.

### application_output_demand

- `application_output_demand` (rank 63): greedy rank 63; uses below application_attempt 61, output_reuse 26, product_operation 15; above output_lineage 60, application_query 47, application_contribution 23; used by application_contribution 383, application_installation 55, output_lineage 48.

### handler

- `handler` (rank 64): greedy rank 64; uses below operation_body 42, application_attempt 33, application_entry 30; above invariant_projection 48, application_contribution 5, application_query 2; used by application_entry 10, operation_body 8, invariant_projection 8.

### output_lineage

- `output_lineage` (rank 65): greedy rank 65; uses below application_attempt 93, application_output_demand 48, application_checkpoint 33; above invariant_projection 45, application_contribution 29, application_query 16; used by invariant_projection 98, application_output_demand 60, application_contribution 49.

### provider

- `provider` (rank 66): greedy rank 66; uses below application_attempt 149, output_lineage 19, schema_layout 12; above application_query 23, invariant_projection 10, application_installation 4; used by application_attempt 63, application_runtime 34, application_inbound 24.

### source_read_equivalence

- `source_read_equivalence` (rank 67): greedy rank 67; uses below structural_walk 3, component_selection 2, aggregate_observation 2; above nothing; used by application_query 4.

### causal_inspection

- `causal_inspection` (rank 68): greedy rank 68; uses below admitted_capture 68, application_attempt 38, provider 12; above application_query 55, application_contribution 5; used by application_runtime 6, application_query 1.

### application_query

- `application_query` (rank 69): greedy rank 69; uses below product_operation 132, source_facts 100, application_attempt 50; above invariant_projection 4, application_contribution 3, application_runtime 2; used by application_contribution 152, facade 117, product_operation 100.

### invariant_projection

- `invariant_projection` (rank 70): greedy rank 70; uses below application_attempt 111, output_lineage 98, application_query 24; above application_contribution 17; used by application_contribution 64, handler 48, output_lineage 45.

### application_installation

- `application_installation` (rank 71): greedy rank 71; uses below application_attempt 81, application_output_demand 55, product_operation 34; above application_contribution 51, application_runtime 4; used by product_operation 9, application_attempt 7, application_discovery 5.

### record_reservations

- `record_reservations` (rank 72): greedy rank 72; uses below runtime_host 1; above nothing; used by application_runtime 2, facade 1.

### application_runtime

- `application_runtime` (rank 73): greedy rank 73; uses below application_query 94, product_operation 75, application_attempt 68; above application_contribution 26; used by application_inbound 18, facade 12, product_operation 11.

### application_contribution

- `application_contribution` (rank 74): greedy rank 74; uses below application_output_demand 383, application_query 152, invariant_projection 64; above nothing; used by facade 81, application_installation 51, output_lineage 29.

### ordinary_read

- `ordinary_read` (rank 75): greedy rank 75; uses below invariant_projection 4, read_observation 1; above nothing; used by facade 4.

### merge_unique_values

- `merge_unique_values` (rank 76): greedy rank 76; uses below schema_layout 2, application_attempt 2, root 1; above nothing; used by facade 2.

### index_refresh

- `index_refresh` (rank 77): greedy rank 77; uses below index_maintenance_budget 2, root 1, provider 1; above nothing; used by facade 2.

### facade

- `facade` (rank 78): greedy rank 78; uses below application_attempt 118, application_query 117, product_operation 103; above nothing; used by nothing.

### application_invariant_preparation

- `application_invariant_preparation` (rank 79): greedy rank 79; uses below invariant_installation 1, denial 1, bootstrap 1; above nothing; used by nothing.

## Units referenced against the stack

- 391 units: 179 movable, 212 behaviour-coupled, 0 pinned; 1411 references
- 48 functions, constants and modules: 93 references

| unit | owner -> destination | refs | class | detail |
|---|---|---|---|---|
| `application_runtime::WorthQueryPrimaryGraphApplicationRuntime` | application_runtime -> runtime_host | 43 | behaviour-coupled | 9 impl(s) need items above; blocked by 1041: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_query::retained_read::WorthQueryApplicationReadObservation` | application_query -> application_discovery | 36 | behaviour-coupled | 1 impl(s) need items above; blocked by 642: `with_checkpoint`, `with_resources`, `checkpoint_names`, `allocation_control` … |
| `invariant_projection::consumed_output::ConsumedOutputEvidence` | invariant_projection -> application_attempt | 34 | behaviour-coupled | 2 impl(s) need items above; blocked by 262: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `output_lineage::settlement_identity::RecordedSettlementIdentity` | output_lineage -> output_reuse | 25 | movable | carries 2 |
| `application_attempt::fact::WorthQueryApplicationObservedFact` | application_attempt -> fact_verification_observation | 23 | behaviour-coupled | 4 impl(s) need items above; blocked by 1102: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` | invariant_projection -> source_facts | 23 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `invariant_projection::WorthQueryInvariantEntityIdentity` | invariant_projection -> operation_body | 20 | movable | carries 1 |
| `application_attempt::denial::WorthQueryApplicationAttemptDenial` | application_attempt -> application_discovery | 19 | behaviour-coupled | 4 impl(s) need items above; blocked by 3: `relational_read_kind`, `equal_cause`, `query_read_execution_kind` |
| `application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` | application_attempt -> product_operation | 16 | behaviour-coupled | 1 impl(s) need items above; blocked by 20: `relational_read_kind`, `append_identity_slot`, `append_scope_slot`, `encode_identity` … |
| `application_contribution::producer::WorthQueryApplicationOutputDemand` | application_contribution -> application_program | 16 | movable | carries 3 |
| `application_attempt::retained_decision_facts::control::StorageControl` | application_attempt -> application_checkpoint | 15 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `relational_read_kind`, `equal_cause`, `query_read_execution_kind` |
| `application_contribution::producer::WorthQueryApplicationProducerBinding` | application_contribution -> product_operation | 15 | behaviour-coupled | 1 impl(s) need items above; blocked by 4: `field_denial`, `encode_declared`, `mismatch_or`, `canonical_wire_maximum` |
| `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` | application_query -> output_reuse | 15 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `invariant_projection::WorthQueryInvariantMutationTarget` | invariant_projection -> operation_body | 15 | movable |  |
| `product_operation::context::WorthQuerySelectedProductOperation` | product_operation -> application_discovery | 15 | behaviour-coupled | 12 impl(s) need items above; blocked by 853: `with_checkpoint`, `with_resources`, `checkpoint_names`, `transition_required` … |
| `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` | application_attempt -> structural_walk | 14 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `application_attempt::retained_decision_facts::StoreDenial` | application_attempt -> application_checkpoint | 14 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `relational_read_kind`, `equal_cause`, `query_read_execution_kind` |
| `application_query::observed_source::WorthQueryObservedSource` | application_query -> expression | 13 | behaviour-coupled | 2 impl(s) need items above; blocked by 1105: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `output_lineage::invalidation::owner::SourceInvalidationOwner` | output_lineage -> root | 13 | behaviour-coupled | 3 impl(s) need items above; blocked by 984: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_contribution::producer::WorthQueryProducerOutputFamily` | application_contribution -> product_operation | 12 | movable | carries 2 |
| `application_output_demand::denial::WorthQueryOutputDemandDenial` | application_output_demand -> application_program | 12 | behaviour-coupled | 3 impl(s) need items above; blocked by 1086: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_query::observed_source::source_identity::identity_kind::WorthQueryRuntimeSourceIdentity` | application_query -> product_operation | 12 | movable |  |
| `application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity` | application_output_demand -> application_checkpoint | 11 | movable | carries 3 |
| `application_query::access_context::WorthQueryApplicationQueryAccessContext` | application_query -> product_operation | 11 | movable |  |
| `application_query::denial::WorthQueryApplicationQueryAdmissionDenial` | application_query -> product_operation | 11 | movable | carries 1 |
| `provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::publication::committed_application::WorthQueryPrimaryGraphCommittedApplication` | provider -> product_operation | 11 | behaviour-coupled | 2 impl(s) need items above; blocked by 315: `relational_read_kind`, `empty`, `pair`, `root` … |
| `resolution::WorthQueryPrincipalResolutionMode` | resolution -> application_discovery | 11 | movable |  |
| `application_contribution::producer::WorthQueryProducerInvariantRequirement` | application_contribution -> product_operation | 10 | movable |  |
| `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` | application_installation -> application_discovery | 10 | behaviour-coupled | 3 impl(s) need items above; blocked by 227: `with_checkpoint`, `with_resources`, `checkpoint_names`, `allocation_control` … |
| `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` | application_query -> provider | 10 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `array_denial`, `storage`, `equal_cause` |
| `application_query::projection::WorthQueryApplicationProjection` | application_query -> product_operation | 10 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `projection_denial`, `relation_cardinality_denial` |
| `application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisSelectionIdentity` | application_query -> product_operation | 10 | movable |  |
| `provider::WorthQueryPrimaryGraphProvider` | provider -> application_attempt | 10 | behaviour-coupled | 3 impl(s) need items above; blocked by 374: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation` | provider -> application_resource | 10 | movable | carries 5 |
| `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` | application_contribution -> application_attempt | 9 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `observe_retained_full_tree` |
| `application_contribution::producer::demand::progression::admission::AdmissionScopedOps` | application_contribution -> application_installation | 9 | behaviour-coupled | 2 impl(s) need items above; blocked by 29: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind` | application_query -> output_reuse | 9 | movable |  |
| `application_query::resource_lifecycle::basis_registry::lease::WorthQueryApplicationBasisLease` | application_query -> product_activation | 9 | behaviour-coupled | 2 impl(s) need items above; blocked by 1: `release` |
| `application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence` | application_attempt -> product_operation | 8 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `declared_member` |
| `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` | application_output_demand -> output_reuse | 8 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `is_temporarily_deferred`, `equal_cause`, `relational_execution_kind` |
| `application_output_demand::settlement::WorthQueryOutputDemandSettlement` | application_output_demand -> product_operation | 8 | behaviour-coupled | 2 impl(s) need items above; blocked by 12: `declared_member`, `denial`, `field_denial`, `field_denial` … |
| `application_query::controls::WorthQueryApplicationQueryControls` | application_query -> product_operation | 8 | behaviour-coupled | 3 impl(s) need items above; blocked by 1: `release` |
| `application_query::native_access::ApplicationQueryNativeAccess` | application_query -> entity_resolution | 8 | movable | carries 4 |
| `application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity` | application_query -> product_operation | 8 | movable |  |
| `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` | invariant_projection -> operation_body | 8 | behaviour-coupled | 8 impl(s) need items above; blocked by 366: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `output_lineage::input_cutoff::stable_cutover::PublishedStableLineage` | output_lineage -> application_attempt | 8 | behaviour-coupled | 2 impl(s) need items above; blocked by 263: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `application_attempt::denial::kind::WorthQueryApplicationAttemptDenialKind` | application_attempt -> product_operation | 7 | movable |  |
| `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` | application_contribution -> application_attempt | 7 | behaviour-coupled | 1 impl(s) need items above; blocked by 7: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `application_query::admission::AdmissionScopedOps` | application_query -> product_operation | 7 | behaviour-coupled | 2 impl(s) need items above; blocked by 817: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_query::one_shot::denial::WorthQueryApplicationOneShotDenialKind` | application_query -> output_reuse | 7 | movable | carries 1 |
| `application_query::projection::retained_projection::materialization::WorthQueryProjectionMaterializationRefusal` | application_query -> causal_inspection | 7 | movable |  |
| `handler::candidate::CandidateWriter` | handler -> application_entry | 7 | behaviour-coupled | 5 impl(s) need items above; blocked by 26: `relational_read_kind`, `aspect_conflict`, `external_payload_denial`, `invalid_payload_denial` … |
| `invariant_projection::consumed_output::verification::ConsumedOutputVerification` | invariant_projection -> application_output_demand | 7 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `invariant_projection::operation_projection_denial::WorthQueryOperationProjectionDenial` | invariant_projection -> application_entry | 7 | movable | carries 6 |
| `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenialStage` | application_attempt -> product_operation | 6 | movable |  |
| `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` | application_attempt -> schema_layout | 6 | behaviour-coupled | 3 impl(s) need items above; blocked by 1063: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_contribution::composition::WorthQueryApplicationContributionTuple` | application_contribution -> application_installation | 6 | behaviour-coupled | 1 impl(s) need items above; blocked by 63: `denial`, `validate_required_producer`, `denial`, `field_denial` … |
| `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` | application_contribution -> application_attempt | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 292: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` | application_contribution -> product_operation | 6 | behaviour-coupled | 4 impl(s) need items above; blocked by 177: `relational_read_kind`, `empty`, `pair`, `root` … |
| `application_query::derived_view::registry::predicate::preparation::PredicatePreparationRetirement` | application_query -> provider | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `array_denial`, `storage`, `equal_cause` |
| `application_query::host::ApplicationQueryHost` | application_query -> application_entry | 6 | behaviour-coupled | 1 impl(s) need items above; blocked by 909: `with_checkpoint`, `with_resources`, `checkpoint_names`, `transition_required` … |
| `application_query::managed_index::stop::WorthQueryManagedIndexStop` | application_query -> access_strategy | 6 | movable | carries 2 |
| `application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisIdentity` | application_query -> application_resource | 6 | movable | carries 1 |
| `handler::invariant::DecisionReader` | handler -> application_entry | 6 | behaviour-coupled | 6 impl(s) need items above; blocked by 298: `with_checkpoint`, `with_resources`, `checkpoint_names`, `allocation_control` … |
| `invariant_projection::operation_reader::decision_reads::expected_keys::ExpectedFactKeys` | invariant_projection -> application_attempt | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 22: `equal_cause`, `equal`, `checkpoint_source_identity`, `checkpoint_source_identity_admitted` … |
| `provider::inbound_completion::WorthQueryInboundCompletionPreparationDenial` | provider -> application_inbound | 6 | movable | carries 1 |
| `application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` | application_attempt -> application_program | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 1140: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_attempt::compare_and_commit::committed_publication::WorthQueryCommittedProductPublication` | application_attempt -> live_delivery | 5 | movable | carries 2 |
| `application_attempt::workflow::adoption::request::WorthQueryWorkflowDispositions` | application_attempt -> product_operation | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `definition_dispositions`, `instance_dispositions` |
| `application_contribution::producer::WorthQueryWorkflowAssessmentPosture` | application_contribution -> application_attempt | 5 | movable |  |
| `application_contribution::producer::demand::WorthQueryOutputDemandAdvance` | application_contribution -> application_installation | 5 | movable |  |
| `application_contribution::producer::source_qualification::OutputSourceAddress` | application_contribution -> application_attempt | 5 | movable |  |
| `application_installation::program::output_source::recovery::WorthQueryRecoveredProgramOutputSource` | application_installation -> application_program | 5 | movable | carries 4 |
| `application_output_demand::registry::required_context::RequiredOutputDemandContext` | application_output_demand -> application_attempt | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 109: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `application_query::graph_read_plan_binding::WorthQueryAdmittedApplicationQueryPlan` | application_query -> product_operation | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 799: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_query::managed_index::control::WorthQueryManagedIndexControl` | application_query -> access_strategy | 5 | movable | carries 3 |
| `application_query::observed_source::retained_dependencies::WorthQueryRequiredSetCurrentnessDenial` | application_query -> product_operation | 5 | movable | carries 6 |
| `application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies` | application_query -> product_operation | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 4: `is_temporarily_deferred`, `equal_cause`, `retain_joined_descriptors`, `relational_execution_kind` |
| `handler::decision_context_use::DecisionContextUse` | handler -> application_entry | 5 | movable |  |
| `invariant_projection::locked_reader::traversal_denial::WorthQueryInvariantProjectionTraversalDenial` | invariant_projection -> handler | 5 | movable | carries 1 |
| `invariant_projection::locked_reader::traversal_denial::WorthQueryInvariantProjectionTraversalDenialKind` | invariant_projection -> handler | 5 | movable |  |
| `output_lineage::native_output_witness::SealedNativeOutputWitness` | output_lineage -> application_checkpoint | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 6: `relational_read_kind`, `declared_member`, `equal_cause`, `overflow` … |
| `output_lineage::source_facts::ComparableSourceFacts` | output_lineage -> application_attempt | 5 | movable | carries 3 |
| `provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial` | provider -> application_resource | 5 | movable |  |
| `provider::inbound_terminal_index::WorthQueryInboundTerminalIndexDenial` | provider -> application_inbound | 5 | movable |  |
| `application_attempt::effect_program::output_correspondence::role_use::OutputRoleUse` | application_attempt -> product_operation | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `declared_member` |
| `application_attempt::fact::WorthQueryApplicationAdjacencyDirection` | application_attempt -> application_discovery | 4 | movable |  |
| `application_contribution::conditional::WorthQueryInstalledApplicationConditionalRegistry` | application_contribution -> conditional_operation | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 284: `denial`, `field_denial`, `dependency_is_declared`, `field_denial` … |
| `application_contribution::output_production::WorthQueryOutputProduction` | application_contribution -> application_runtime | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 51: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::partitioned_computation::incremental::Comparator` | application_contribution -> application_attempt | 4 | movable |  |
| `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` | application_contribution -> application_output_demand | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 127: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` | application_contribution -> application_attempt | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 292: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `application_contribution::producer::WorthQueryProducerCommitAuthority` | application_contribution -> application_output_demand | 4 | movable |  |
| `application_contribution::producer::demand::progression::admission::source_custody::SourceCustodyScopedOps` | application_contribution -> application_installation | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 9: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::producer::demand::progression::required_wave::resolved::matched_predecessors::MatchedRequiredPredecessors` | application_contribution -> output_lineage | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 9: `capacity_denial`, `equal_cause`, `check`, `reserve` … |
| `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` | application_contribution -> application_output_demand | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `denial`, `field_denial`, `field_denial` |
| `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` | application_installation -> product_operation | 4 | behaviour-coupled | 4 impl(s) need items above; blocked by 357: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `application_output_demand::registry::WorthQueryOutputDemandRegistry` | application_output_demand -> application_program | 4 | behaviour-coupled | 5 impl(s) need items above; blocked by 234: `with_checkpoint`, `with_resources`, `checkpoint_names`, `relational_read_kind` … |
| `application_query::document_history::metadata::CompletedMetadataRead` | application_query -> product_operation | 4 | movable | carries 6 |
| `application_query::managed_index::denial::WorthQueryManagedIndexDenial` | application_query -> product_operation | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 846: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_query::managed_index::source::installation::ManagedSourceInstallation` | application_query -> root | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 11: `storage`, `storage`, `equal_cause`, `admit_dependency` … |
| `application_query::managed_index::source_rows::WorthQueryManagedIndexSourceSlot` | application_query -> access_strategy | 4 | movable |  |
| `application_query::projection::projected_tree::content_comparison::ProjectionContentComparisonFault` | application_query -> access_strategy | 4 | movable |  |
| `bootstrap::publication::WorthQueryPrimaryGraphPublication` | bootstrap -> application_checkpoint | 4 | movable |  |
| `handler::invariant::HandlerInterruption` | handler -> application_entry | 4 | movable |  |
| `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` | invariant_projection -> operation_body | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 781: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `invariant_projection::consumed_output::verification::EvidenceView` | invariant_projection -> output_lineage | 4 | movable | carries 1 |
| `invariant_projection::operation_reader::decision_reads::computation_inputs::ObservedComputationInputs` | invariant_projection -> application_attempt | 4 | movable |  |
| `output_lineage::invalidation::commit_touches::CommitTouchInterest` | output_lineage -> application_discovery | 4 | movable | carries 1 |
| `output_lineage::invalidation::computation_changes::postcondition_carry::PostconditionCarry` | output_lineage -> application_attempt | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `equal_cause`, `visit` |
| `output_lineage::invalidation::mark_state::FullVerificationReason` | output_lineage -> fact_verification_observation | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `output_lineage::recorded_source_identity::RecordedSourceIdentity` | output_lineage -> product_operation | 4 | movable | carries 2 |
| `provider::branch_commit_coordination::WorthQueryApplicationBranchCommitCoordination` | provider -> product_operation | 4 | movable |  |
| `provider::idempotency::WorthQueryProviderIdempotencyResolution` | provider -> application_attempt | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `equal_cause`, `same_addresses`, `reads_source` |
| `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` | application_attempt -> program_occurrence | 3 | behaviour-coupled | 3 impl(s) need items above; blocked by 10: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `application_attempt::effect_program::output_correspondence::projection::WorthQueryApplicationOutputProjectionDenial` | application_attempt -> product_operation | 3 | movable |  |
| `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` | application_attempt -> freshness | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `relational_read_kind`, `heap_sort`, `union_visit`, `equal_cause` … |
| `application_contribution::producer::WorthQueryApplicationProducerProvider` | application_contribution -> product_operation | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `encode_declared`, `mismatch_or`, `canonical_wire_maximum` |
| `application_contribution::producer::demand::WorthQueryAdmittedOutputDemand` | application_contribution -> application_installation | 3 | behaviour-coupled | 5 impl(s) need items above; blocked by 29: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::producer::demand::progression::ProgressionScopedOps` | application_contribution -> application_installation | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 29: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::producer::demand::progression::source_recovery::SourceRecoveryScopedOps` | application_contribution -> application_installation | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 9: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::producer::denial_value::WorthQueryProducerDenialValue` | application_contribution -> application_output_demand | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `mismatch_or` |
| `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` | application_contribution -> output_lineage | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 6: `denial`, `field_denial`, `field_denial`, `encode_declared` … |
| `application_discovery::workflow::indexes::WorkflowReadIndexes` | application_discovery -> schema_layout | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1063: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_installation::program::supported_program::WorthQuerySupportedProgramRecord` | application_installation -> program_occurrence | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1093: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_output_demand::settlement::WorthQueryOutputSettlementPosture` | application_output_demand -> product_operation | 3 | movable |  |
| `application_query::admission::funded_parameters::CapturedApplicationQueryParameters` | application_query -> application_resource | 3 | movable |  |
| `application_query::admission::permission::fresh::FreshQueryPermissionStop` | application_query -> causal_inspection | 3 | movable | carries 2 |
| `application_query::admission::permission::fresh::access::AccessScopedOps` | application_query -> causal_inspection | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 63: `denial`, `inspect`, `storage`, `configured_in` … |
| `application_query::admitted_result::WorthQueryApplicationOutputDemandSource` | application_query -> expression | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 1137: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_query::derived_view::registry::ManagedDerivedViewRegistry` | application_query -> root | 3 | behaviour-coupled | 3 impl(s) need items above; blocked by 956: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_query::derived_view::registry::predicate::preparation::PreparedPredicatePublication` | application_query -> live_delivery | 3 | movable | carries 3 |
| `application_query::managed_index::configuration::ManagedIndexProviderCatalog` | application_query -> invariant_installation | 3 | behaviour-coupled | 4 impl(s) need items above; blocked by 836: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_query::managed_index::source::initialization::SourceInitializationIssuer` | application_query -> invariant_installation | 3 | movable | carries 3 |
| `application_query::managed_index::source::initialization::SourceInitializationRequest` | application_query -> bootstrap_publication | 3 | movable | carries 1 |
| `application_query::managed_index::source_structure::SourceTree` | application_query -> access_strategy | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 11: `PAGE_BITS`, `PAGE_SIZE`, `branch`, `build` … |
| `application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch` | application_query -> application_output_demand | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `equal`, `checkpoint_source_identity`, `admit_footprint_comparison` |
| `application_query::projection::projected_tree::materialization::WorthQueryMaterializedQueryField` | application_query -> causal_inspection | 3 | movable | carries 1 |
| `application_query::projection::projected_tree::materialization::WorthQueryMaterializedQueryRelation` | application_query -> causal_inspection | 3 | movable | carries 6 |
| `application_query::projection::projected_tree::materialization::WorthQueryMaterializedQueryRow` | application_query -> causal_inspection | 3 | movable | carries 5 |
| `application_query::projection::retained_projection::comparison::ProjectionComparisonDenial` | application_query -> causal_inspection | 3 | movable |  |
| `application_query::projection::retained_projection::materialization::WorthQueryMaterializedQueryProjection` | application_query -> causal_inspection | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `rows` |
| `application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisReleaseReceipt` | application_query -> application_resource | 3 | movable | carries 3 |
| `application_runtime::installation::input::ApplicationRuntimePublication` | application_runtime -> conditional_operation | 3 | movable | carries 2 |
| `application_runtime::operation_control::attempt_registration::WorthQueryApplicationAttemptOperationControl` | application_runtime -> provider | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `arm`, `clear`, `reach`, `release` … |
| `conditional_operation::installation::clock_handle::WorthQueryConditionalClockHandle` | conditional_operation -> product_operation | 3 | movable | carries 1 |
| `invariant_projection::consumed_output::verification::capacity_stop::VerificationCapacityStop` | invariant_projection -> application_attempt | 3 | movable |  |
| `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionSnapshot` | invariant_projection -> application_entry | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 83: `with_checkpoint`, `with_resources`, `checkpoint_names`, `allocation_control` … |
| `invariant_projection::pending_output_read::RequestedOutputRead` | invariant_projection -> application_output_demand | 3 | movable | carries 3 |
| `output_lineage::current_output::RetainedOutputCurrentnessRead` | output_lineage -> application_output_demand | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 30: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `output_lineage::invalidation::output_availability::ObservedSourcePublication` | output_lineage -> application_attempt | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 22: `equal_cause`, `equal`, `checkpoint_source_identity`, `checkpoint_source_identity_admitted` … |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::diagnostic::RebaseVerificationReason` | provider -> application_attempt | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `provider::mutation_work::WorthQueryPrimaryMutationWorkEvidence` | provider -> application_attempt | 3 | movable | carries 6 |
| `provider::resource_support::WorthQueryPrimaryGraphResourceSupport` | provider -> bootstrap | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `component_support` |
| `provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxBinding` | provider -> application_attempt | 3 | movable | carries 1 |
| `application_attempt::effect_program::model::WorthQueryApplicationEffectEntity` | application_attempt -> operation_body | 2 | movable |  |
| `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` | application_attempt -> application_entry | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 1017: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_attempt::effect_program::model::emission::WorthQueryApplicationEmission` | application_attempt -> live_delivery | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 1032: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_attempt::fact::movement::Movement` | application_attempt -> source_facts | 2 | movable |  |
| `application_attempt::host::ApplicationAttemptHost` | application_attempt -> product_operation | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 789: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_attempt::read_set::source_facts::SourceFactsScopedOps` | application_attempt -> application_entry | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 910: `with_checkpoint`, `with_resources`, `checkpoint_names`, `transition_required` … |
| `application_attempt::workflow::adoption::occurrence::WorthQueryWorkflowAdoptionInventory` | application_attempt -> product_operation | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `definition_dispositions`, `instance_dispositions` |
| `application_attempt::workflow::correction::PreparedWorkflowCorrection` | application_attempt -> product_operation | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 25: `relational_read_kind`, `declared_member`, `observe_field_value`, `actor_record_corruption` … |
| `application_attempt::workflow::definition::condition::CompiledWorkflowConditionExpression` | application_attempt -> expression | 2 | movable |  |
| `application_attempt::workflow_definition_program::publication::PublishedWorkflowDefinitionRef` | application_attempt -> application_discovery | 2 | movable |  |
| `application_attempt::workflow_instance_observation::read_only::ReadOnlyWorkflowHistory` | application_attempt -> application_discovery | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 1020: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_attempt::workflow_instance_program::discovery_intent::WorthQueryWorkflowInstanceDiscoveryIntent` | application_attempt -> application_discovery | 2 | movable | carries 3 |
| `application_contribution::computation::prepared_request::ComputedSlot` | application_contribution -> application_entry | 2 | movable |  |
| `application_contribution::computation::prepared_request::PreparedCompute` | application_contribution -> application_entry | 2 | movable | carries 7 |
| `application_contribution::conditional::WorthQueryApplicationConditionalBinding` | application_contribution -> handler | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 171: `denial`, `field_denial`, `dependency_is_declared`, `field_denial` … |
| `application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial` | application_contribution -> denial | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `denial` |
| `application_contribution::host::ApplicationContributionHost` | application_contribution -> application_installation | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 7: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` | application_contribution -> output_lineage | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 7: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` | application_contribution -> output_lineage | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 7: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `application_contribution::producer::WorthQueryProducerApplicability` | application_contribution -> application_output_demand | 2 | movable | carries 1 |
| `application_contribution::producer::demand::progression::admission::AdmissionOps` | application_contribution -> application_installation | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 30: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::producer::demand::progression::required_wave::performed::certification::CurrentProof` | application_contribution -> application_output_demand | 2 | movable | carries 1 |
| `application_contribution::producer::demand::progression::required_wave::performed::certification::current_basis::CurrentBasis` | application_contribution -> application_output_demand | 2 | movable |  |
| `application_contribution::producer::input_reuse_contract::WorthQueryProducerInputReuseContract` | application_contribution -> application_attempt | 2 | movable | carries 1 |
| `application_contribution::producer::output_evaluation::WorthQueryProducerEvaluationIneligibility` | application_contribution -> causal_inspection | 2 | movable |  |
| `application_contribution::producer::readiness::PendingOutputReadiness` | application_contribution -> conditional_operation | 2 | movable | carries 1 |
| `application_installation::program::program_owner::contract::WorthQueryProgramOwner` | application_installation -> product_operation | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 344: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `application_output_demand::completed_outputs::WorthQueryCompletedRequiredOutputs` | application_output_demand -> product_operation | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 316: `relational_read_kind`, `empty`, `pair`, `root` … |
| `application_output_demand::host::OutputDemandHost` | application_output_demand -> product_operation | 2 | movable | carries 1 |
| `application_output_demand::registry::accepted_checkpoint::AcceptedCheckpointFactSource` | application_output_demand -> application_checkpoint | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 978: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture` | application_output_demand -> application_checkpoint | 2 | movable |  |
| `application_output_demand::settlement::observation::OriginalSettledProducerObservation` | application_output_demand -> product_operation | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 798: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_query::basis::execution_custody::WorthQueryApplicationQueryBasisCustody` | application_query -> product_operation | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 1: `release` |
| `application_query::managed_index::source::initialization::SelectedSources` | application_query -> bootstrap | 2 | movable |  |
| `application_query::observed_source::query_context_binding::ObservedQueryExpressionContext` | application_query -> operation_body | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 10: `relational_read_kind`, `equal_cause`, `equal`, `project` … |
| `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` | application_query -> output_lineage | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `admit_footprint_comparison`, `same_footprint_membership` |
| `application_query::one_shot::WorthQueryApplicationOneShotResult` | application_query -> product_operation | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `inspect`, `inspect` |
| `application_query::one_shot::denial::WorthQueryApplicationOneShotDenial` | application_query -> product_operation | 2 | behaviour-coupled | 3 impl(s) need items above; blocked by 2: `authorization_denial`, `denial` |
| `application_query::output_source_observation::SourceQueryReaderAccess` | application_query -> application_entry | 2 | behaviour-coupled | 3 impl(s) need items above; blocked by 43: `checkpoint_names`, `allocation_control`, `authority_execution`, `control_kind` … |
| `application_query::projection::projected_tree::content_storage::ContentScratch` | application_query -> access_strategy | 2 | movable | carries 1 |
| `application_runtime::operation_control::attempt_registration::source_publication::receipt::WorthQuerySourcePublicationImageForTest` | application_runtime -> output_lineage | 2 | movable | carries 1 |
| `handler::conditional_definition::ConditionalDefinitionAdmission` | handler -> application_entry | 2 | movable | carries 6 |
| `handler::registry::PendingMutationHandlerRegistry` | handler -> bootstrap | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 811: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `invariant_projection::admission_denial::WorthQueryInvariantProjectionDenial` | invariant_projection -> conditional_operation | 2 | movable | carries 1 |
| `invariant_projection::admission_denial::WorthQueryInvariantProjectionDenialKind` | invariant_projection -> conditional_operation | 2 | movable |  |
| `invariant_projection::consumed_output::pending_dependency::SelectedPendingConsumedOutput` | invariant_projection -> application_output_demand | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `equal_cause`, `overflow`, `native_metadata` |
| `invariant_projection::operation_projection_denial::WorthQueryOperationProjectionDenialKind` | invariant_projection -> conditional_operation | 2 | movable | carries 1 |
| `invariant_projection::operation_reader::computation_calls::sequence::shape::CallShape` | invariant_projection -> application_attempt | 2 | movable | carries 3 |
| `invariant_projection::operation_reader::computation_calls::sequence::shape::CallShapeTransition` | invariant_projection -> application_attempt | 2 | movable | carries 1 |
| `invariant_projection::output_availability_candidate::ObservedOutputCandidate` | invariant_projection -> output_lineage | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 9: `equal_cause`, `check`, `reserve`, `reads_at_or_after` … |
| `invariant_projection::output_availability_candidate::ObservedOutputPosture` | invariant_projection -> application_attempt | 2 | movable |  |
| `output_lineage::WorthQueryApplicationOutputLineage` | output_lineage -> root | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 96: `with_checkpoint`, `with_resources`, `checkpoint_names`, `empty` … |
| `output_lineage::input_reuse_key::PreparedInputReuseKey` | output_lineage -> application_output_demand | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 8: `denial`, `field_denial`, `field_denial`, `encode_declared` … |
| `output_lineage::invalidation::derived::SourceSettlementCurrentness` | output_lineage -> fact_verification_observation | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `output_lineage::required_settlement::current_accepted::BoundCurrentAcceptedOutput` | output_lineage -> application_output_demand | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 23: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `output_lineage::retained_computation::PriorComputationRecord` | output_lineage -> application_output_demand | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 126: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `output_lineage::source_facts::RetainedSourceFacts` | output_lineage -> product_operation | 2 | movable | carries 7 |
| `provider::aftermath_causality::WorthQueryAftermathCausalityReadDenial` | provider -> product_operation | 2 | movable |  |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::FactlessCurrentness` | provider -> application_attempt | 2 | movable |  |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::OwnEffectOnReads` | provider -> application_attempt | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `reads_source` |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedRebaseDenial` | provider -> application_attempt | 2 | movable |  |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase` | provider -> application_attempt | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 11: `equal_cause`, `visit`, `same_addresses`, `prepare_decision_adjacency` … |
| `provider::branch_commit_coordination::WorthQueryApplicationBranchCommitLane` | provider -> application_checkpoint | 2 | movable | carries 4 |
| `provider::dispatch_outbox::WorthQueryDispatchOutboxBasis` | provider -> application_attempt | 2 | movable |  |
| `provider::host::ProviderHost` | provider -> application_inbound | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 781: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `provider::idempotency::WorthQueryProviderIdempotencyResolutionDenial` | provider -> application_attempt | 2 | movable |  |
| `provider::outstanding_dispatch::in_flight::OutstandingDispatchInFlightLease` | provider -> application_inbound | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `decrement`, `RETAINED_RECEIPT_BASIS_LIMIT` |
| `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` | provider -> product_operation | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 363: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `application_attempt::compare_and_commit::commit_outcome::WorthQueryApplicationCommitOutcome` | application_attempt -> product_operation | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 847: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_attempt::compare_and_commit::commit_receipt::committed_changes::WorthQueryApplicationCommittedChanges` | application_attempt -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 791: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_attempt::effect_program::model::emission::WorthQueryAdmittedApplicationEmissionBatch` | application_attempt -> live_delivery | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1032: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole` | application_attempt -> application_checkpoint | 1 | movable |  |
| `application_attempt::elevation_approval_outcome::WorthQueryApprovedElevation` | application_attempt -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 316: `relational_read_kind`, `empty`, `pair`, `root` … |
| `application_attempt::fact::movement::FactMovement` | application_attempt -> output_reuse | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 972: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_attempt::fact::source_read_authorization::SourceReadAuthorization` | application_attempt -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `application_attempt::idempotency_resolution::external_settlement::ExternalSettlementScopedOps` | application_attempt -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 789: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_attempt::provider_execution::entry::application_commit::ApplicationCommitOps` | application_attempt -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 336: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `application_attempt::provider_execution::entry::output_source::OutputSourceOps` | application_attempt -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 336: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `application_attempt::read_phase::WorthQueryProjectedApplicationMutation` | application_attempt -> application_entry | 1 | movable |  |
| `application_attempt::read_set::ReadSetScopedOps` | application_attempt -> application_entry | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 910: `with_checkpoint`, `with_resources`, `checkpoint_names`, `transition_required` … |
| `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` | application_attempt -> application_entry | 1 | behaviour-coupled | 4 impl(s) need items above; blocked by 762: `with_checkpoint`, `with_resources`, `checkpoint_names`, `transition_required` … |
| `application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` | application_attempt -> product_operation | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 2: `release_custody`, `release` |
| `application_attempt::workflow::actor::record::WorkflowAdmittedActor` | application_attempt -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 7: `relational_read_kind`, `declared_member`, `observe_field_value`, `actor_record_corruption` … |
| `application_attempt::workflow::adoption::choices::AdmittedWorkflowDispositions` | application_attempt -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `definition_dispositions`, `instance_dispositions` |
| `application_attempt::workflow::adoption::coverage::WorkflowVocabularyCoverageRegistry` | application_attempt -> product_operation | 1 | movable | carries 3 |
| `application_attempt::workflow::adoption::inventory::WorkflowAdoptionInventoryRequest` | application_attempt -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 6: `relational_read_kind`, `observe_field_value`, `actor_record_corruption`, `decode_actor_components` … |
| `application_attempt::workflow::adoption::truth::WorkflowAdoptionReadDenial` | application_attempt -> product_operation | 1 | movable |  |
| `application_attempt::workflow::definition::compilation::reuse::WorkflowDefinitionCompilationReuse` | application_attempt -> root | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 11: `relational_read_kind`, `control_variant`, `decode_control`, `append_framed` … |
| `application_attempt::workflow::definition::compilation::reuse::counters::WorthQueryWorkflowCompilationReuseCounters` | application_attempt -> root | 1 | movable |  |
| `application_attempt::workflow::definition::condition::WorkflowConditionOperand` | application_attempt -> expression | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `OPERAND_FIELDS` |
| `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressKey` | application_attempt -> root | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `SHARD_COUNT` |
| `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressRetention` | application_attempt -> root | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 1044: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_attempt::workflow::instance::progression::retention::counters::WorthQueryWorkflowInstanceProgressCounters` | application_attempt -> root | 1 | movable |  |
| `application_attempt::workflow_instance_observation::read_fields::ReadFields` | application_attempt -> application_discovery | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 4: `observe_adjacency`, `observe_field_value_borrowed`, `WORKFLOW_FACT_PROTOCOL_VERSION`, `text` |
| `application_attempt::workflow_instance_program::publication::PublishedWorkflowInstanceRef` | application_attempt -> application_discovery | 1 | movable |  |
| `application_attempt::workflow_proposal_program::publication::reference::PublishedWorkflowProposalRef` | application_attempt -> application_discovery | 1 | movable | carries 1 |
| `application_attempt::workflow_transition_program::actor_context::WorthQueryWorkflowActorContext` | application_attempt -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 849: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_attempt::workflow_transition_program::publication::RequiredWorkflowCondition` | application_attempt -> expression | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 4: `OPERAND_FIELDS`, `decode_draft`, `decode_operands`, `encode_draft` |
| `application_attempt::workflow_transition_program::publication::approval::WorkflowApprovalDecision` | application_attempt -> application_discovery | 1 | movable |  |
| `application_contribution::composition::WorthQueryConfiguredApplicationContributions` | application_contribution -> application_installation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 63: `denial`, `validate_required_producer`, `denial`, `field_denial` … |
| `application_contribution::computation::wave_dispatch::PreparedMember` | application_contribution -> application_entry | 1 | movable | carries 8 |
| `application_contribution::partitioned_computation::incremental::deposit::ComputationDeposit` | application_contribution -> invariant_projection | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 7: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `application_contribution::partitioned_computation::incremental::prior_absence::Suppression` | application_contribution -> invariant_projection | 1 | movable |  |
| `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` | application_contribution -> invariant_projection | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `observe_retained_full_tree` |
| `application_contribution::producer::WorthQueryProducerLifecyclePosture` | application_contribution -> output_lineage | 1 | movable |  |
| `application_contribution::producer::demand::WorthQuerySelectedApplicationProducer` | application_contribution -> application_runtime | 1 | movable | carries 2 |
| `application_contribution::producer::demand::progression::ProgressionOps` | application_contribution -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 25: `denial`, `field_denial`, `field_denial`, `denial` … |
| `application_contribution::producer::demand::progression::admission::dependent::DependentScopedOps` | application_contribution -> application_installation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 29: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::producer::demand::progression::admission::retained_observation::RetainedObservationOps` | application_contribution -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 25: `denial`, `field_denial`, `field_denial`, `denial` … |
| `application_contribution::producer::demand::progression::admission::source_custody::SourceCustodyOps` | application_contribution -> application_runtime | 1 | movable | carries 2 |
| `application_contribution::producer::demand::progression::selected_program::SelectedProgramOps` | application_contribution -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 26: `denial`, `field_denial`, `field_denial`, `denial` … |
| `application_contribution::producer::demand::progression::unavailable::UnavailableScopedOps` | application_contribution -> application_installation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 29: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::producer::demand::selection::SelectionOps` | application_contribution -> application_runtime | 1 | movable | carries 7 |
| `application_contribution::producer::denial_value::WorthQueryProducerDenialDecodeDenial` | application_contribution -> application_output_demand | 1 | movable | carries 1 |
| `application_contribution::producer::execution::prepared_plan::OwnerPlan` | application_contribution -> application_output_demand | 1 | movable | carries 10 |
| `application_contribution::producer::execution::unpublished::PendingProducerPublication` | application_contribution -> application_output_demand | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 125: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `application_contribution::producer::input_reuse_contract::WorthQueryDecisionContextDependencies` | application_contribution -> application_attempt | 1 | movable |  |
| `application_contribution::producer::output_evaluation::ComparedProducerOutput` | application_contribution -> causal_inspection | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `denial`, `field_denial`, `field_denial` |
| `application_contribution::producer::output_evaluation::ProducerOutputEvaluationState` | application_contribution -> causal_inspection | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `denial`, `field_denial`, `field_denial` |
| `application_contribution::producer::readiness::TypedPendingOutputReadiness` | application_contribution -> conditional_operation | 1 | movable |  |
| `application_contribution::producer::readiness::WorthQueryInstalledOutputReadinessRoutes` | application_contribution -> application_runtime | 1 | movable | carries 1 |
| `application_contribution::producer::scheduling::WorthQueryInstalledOutputProducerRoutes` | application_contribution -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 7: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_contribution::producer::source_qualification::QualifiedOutputSource` | application_contribution -> output_lineage | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 63: `denial`, `inspect`, `storage`, `configured_in` … |
| `application_contribution::producer::source_qualification::SourceQualificationScopedOps` | application_contribution -> invariant_projection | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 52: `denial`, `field_denial`, `field_denial`, `bridge_denial` … |
| `application_installation::denial::WorthQueryInMemoryApplicationDenial` | application_installation -> bootstrap | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `denial` |
| `application_installation::program::demand::WorthQueryAdmittedProgramOutput` | application_installation -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 316: `relational_read_kind`, `empty`, `pair`, `root` … |
| `application_installation::program::demand::WorthQuerySettledProgramOutput` | application_installation -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 12: `declared_member`, `denial`, `field_denial`, `field_denial` … |
| `application_installation::program::demand::root_completion::OriginalProgramRootCompletion` | application_installation -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 12: `declared_member`, `denial`, `field_denial`, `field_denial` … |
| `application_installation::program::output_source::recovery::WorthQueryUnpublishedProgramOutputSource` | application_installation -> application_program | 1 | movable | carries 13 |
| `application_installation::program::program_owner::WorthQuerySelectedProgramOwnerDenial` | application_installation -> application_output_demand | 1 | movable |  |
| `application_installation::program::specialized_action::WorthQueryAdmittedProgramOperation` | application_installation -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 342: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `application_installation::program_admission::WorthQueryAdmittedProgramSupport` | application_installation -> bootstrap | 1 | movable |  |
| `application_output_demand::original_outputs::WorthQueryCompletedOriginalReadOutputs` | application_output_demand -> product_operation | 1 | movable | carries 4 |
| `application_output_demand::original_outputs::WorthQueryOriginalReadOutputDenial` | application_output_demand -> product_operation | 1 | movable | carries 8 |
| `application_output_demand::recovered_outputs::WorthQueryRecoveredOutputs` | application_output_demand -> application_checkpoint | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1: `declared_member` |
| `application_output_demand::registry::accepted_checkpoint_identity::CheckpointOutputSlot` | application_output_demand -> application_checkpoint | 1 | movable |  |
| `application_output_demand::registry::required_work::wave_admission::PreparedRequiredInputs` | application_output_demand -> application_entry | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 22: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `application_output_demand::settlement::SettlementScopedOps` | application_output_demand -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 281: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `application_query::access_receipt::WorthQueryApplicationQueryAccessReceipt` | application_query -> product_operation | 1 | movable | carries 15 |
| `application_query::admission::correction_disclosure::CorrectionDisclosureScopedOps` | application_query -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 817: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_query::admission::permission::read_authorization::ReadAuthorizationScopedOps` | application_query -> causal_inspection | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 63: `denial`, `inspect`, `storage`, `configured_in` … |
| `application_query::admission::permission::selected_access::SelectedIssuedAccessRoot` | application_query -> causal_inspection | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `denial` |
| `application_query::batch::WorthQueryApplicationQueryBatchAdmission` | application_query -> product_operation | 1 | movable | carries 5 |
| `application_query::batch::WorthQueryApplicationQueryBatchResourceDenial` | application_query -> product_operation | 1 | movable |  |
| `application_query::derived_view::publication::ViewPublicationBasis` | application_query -> provider | 1 | movable |  |
| `application_query::derived_view::registry::PreparedManagedViewPublication` | application_query -> provider | 1 | movable | carries 3 |
| `application_query::derived_view::registry::predicate::preparation::PredicatePreparationFactory` | application_query -> provider | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 3: `array_denial`, `storage`, `equal_cause` |
| `application_query::derived_view::registry::predicate::preparation::round::PredicateNoticeDisposition` | application_query -> live_delivery | 1 | movable |  |
| `application_query::derived_view::registry::predicate::preparation::round::PreparedPredicateNotices` | application_query -> live_delivery | 1 | movable | carries 2 |
| `application_query::document_history::WorthQueryDocumentHistoryRead` | application_query -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 9: `prepare_governed_access`, `inspect`, `admit_metadata`, `authorization_denial` … |
| `application_query::document_history::output_intent::WorthQueryHistoryOutputIntent` | application_query -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `fund_comparison`, `scope_work`, `control_denial`, `validate_execution_lifetimes` … |
| `application_query::document_history::output_read::HistoryOutputControlPlan` | application_query -> product_operation | 1 | movable | carries 4 |
| `application_query::document_history::output_read::OutputReadScopedOps` | application_query -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 817: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_query::document_history::preview::admission::WorthQueryHistoryPreviewReadAdmission` | application_query -> product_operation | 1 | movable | carries 9 |
| `application_query::history::denial::WorthQueryApplicationHistoryDenial` | application_query -> product_operation | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 3: `authorization_denial`, `denial`, `map_authorized_read_denial` |
| `application_query::managed_index::denial::causes::WorthQueryManagedIndexDenialCause` | application_query -> application_output_demand | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `equal_cause`, `authorization_denial`, `denial` |
| `application_query::managed_index::invariant_read::ManagedSourceValue` | application_query -> application_invariant | 1 | movable |  |
| `application_query::managed_index::invariant_read::ReadPosture` | application_query -> access_strategy | 1 | movable |  |
| `application_query::managed_index::invariant_read::WorthQueryManagedIndexInvariantDenial` | application_query -> application_invariant | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `application_query::managed_index::predicate::tree::WatchSlot` | application_query -> live_delivery | 1 | movable |  |
| `application_query::managed_index::provider::WorthQueryManagedIndexProbeValue` | application_query -> application_invariant | 1 | movable |  |
| `application_query::managed_index::realized_footprint::CheckedManagedFootprintCapture` | application_query -> access_strategy | 1 | movable |  |
| `application_query::managed_index::source_rows::SourceImageView` | application_query -> access_strategy | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 954: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_query::observed_source::denial::WorthQuerySourceExpectationDenial` | application_query -> application_attempt | 1 | movable | carries 1 |
| `application_query::observed_source::prepared_expectation::BoundStableObservedSourceFacts` | application_query -> output_lineage | 1 | movable | carries 5 |
| `application_query::observed_source::result_set::WorthQueryObservedResultSet` | application_query -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `equal`, `admit_footprint_comparison` |
| `application_query::observed_source::source_identity::WorthQueryObservedSourceCoordinate` | application_query -> application_output_demand | 1 | movable | carries 1 |
| `application_query::one_shot::OneShotScopedOps` | application_query -> conditional_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 19: `denial`, `inspect`, `equal`, `checkpoint_source_identity` … |
| `application_query::one_shot::correction_disclosure::OneShotCorrectionDisclosureScopedOps` | application_query -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 822: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `application_query::output_source_observation::read_closure::SourceQueryReadClosure` | application_query -> output_lineage | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 63: `denial`, `inspect`, `storage`, `configured_in` … |
| `application_query::projection::denial::WorthQueryApplicationProjectionDenialKind` | application_query -> application_output_demand | 1 | movable |  |
| `application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisRegistrationDenial` | application_query -> product_operation | 1 | movable |  |
| `application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisRegistry` | application_query -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `acquire`, `record_one_saturating`, `release` |
| `application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisReleaseOutcome` | application_query -> application_attempt | 1 | movable |  |
| `application_runtime::operation_control::attempt_registration::source_publication::receipt::WorthQuerySourcePublicationHeadForTest` | application_runtime -> output_lineage | 1 | movable |  |
| `bootstrap::WorthQueryPrimaryGraphBootstrap` | bootstrap -> typed_bootstrap | 1 | behaviour-coupled | 4 impl(s) need items above; blocked by 1094: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `bootstrap::WorthQueryPrincipalBootstrapRow` | bootstrap -> bootstrap_publication | 1 | movable |  |
| `component_selection::ComponentSelection` | component_selection -> source_read_verification | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 977: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `component_selection::aggregates::ComponentAggregateBodies` | component_selection -> source_read_verification | 1 | movable |  |
| `conditional_operation::lifecycle::registry::WorthQueryConditionalOperationRegistry` | conditional_operation -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 16: `denial`, `is_temporarily_deferred`, `equal_cause`, `release` … |
| `conditional_operation::predicate_observation::QueryTemporalPredicateProvider` | conditional_operation -> product_operation | 1 | movable | carries 1 |
| `conditional_operation::signal_decision_reentry::WorthQueryConditionalTruthBasis` | conditional_operation -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `release` |
| `freshness::WorthQueryPrincipalFreshnessEvidence` | freshness -> authenticated_principal | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1093: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `handler::host::HandlerHost` | handler -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 490: `denial`, `field_denial`, `dependency_is_declared`, `field_denial` … |
| `handler::registry::RegistryScopedOps` | handler -> application_entry | 1 | movable | carries 6 |
| `handler::registry::operation_body::OperationBodyScopedOps` | handler -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 490: `denial`, `field_denial`, `dependency_is_declared`, `field_denial` … |
| `invariant_projection::component::result::WorthQueryInvariantProjectionComponent` | invariant_projection -> handler | 1 | movable | carries 2 |
| `invariant_projection::consumed_output::publication_recovery::PublicationRecoveryStop` | invariant_projection -> output_lineage | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `invariant_projection::graph_member::WorthQueryInvariantGraphMember` | invariant_projection -> handler | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 154: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `invariant_projection::operation_reader::borrowed::BorrowedOperationProjection` | invariant_projection -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 295: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `invariant_projection::operation_reader::current_output::WorthQueryCurrentOutputSelection` | invariant_projection -> handler | 1 | movable | carries 2 |
| `invariant_projection::operation_reader::decision_plan::prepared_selection::WorthQueryPreparedEntitySelection` | invariant_projection -> handler | 1 | movable |  |
| `invariant_projection::operation_reader::derived_relation::WorthQueryInvariantDerivedRelation` | invariant_projection -> handler | 1 | movable | carries 2 |
| `invariant_projection::operation_reader::derived_relation::member::WorthQueryInvariantDerivedMemberRelation` | invariant_projection -> handler | 1 | movable | carries 1 |
| `invariant_projection::operation_reader::derived_relation::member::WorthQueryInvariantDerivedWalk` | invariant_projection -> handler | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 154: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `invariant_projection::operation_reader::derived_relation::member::component::WorthQueryInvariantGraphComponent` | invariant_projection -> handler | 1 | movable | carries 1 |
| `invariant_projection::operation_reader::output_availability::WorthQueryObservedOutputAvailability` | invariant_projection -> handler | 1 | movable | carries 3 |
| `invariant_projection::operation_reader::output_availability::denial::WorthQueryOutputAvailabilityDenial` | invariant_projection -> application_query | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `meter`, `query_stop` |
| `invariant_projection::operation_reader::prepared_inputs::PreparedCurrentInputs` | invariant_projection -> application_output_demand | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 30: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `invariant_projection::operation_reader::prior_output::WorthQueryPriorOutputFamilyMember` | invariant_projection -> handler | 1 | movable | carries 2 |
| `invariant_projection::operation_reader::prior_output::WorthQueryPriorOutputRead` | invariant_projection -> handler | 1 | movable | carries 2 |
| `invariant_projection::orbit::result::WorthQueryInvariantProjectionOrbit` | invariant_projection -> handler | 1 | movable | carries 6 |
| `invariant_projection::realized_scope::WorthQueryRealizedProjectionScope` | invariant_projection -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 10: `shape_bound`, `build`, `insert`, `leaf` … |
| `invariant_projection::relation_identity::WorthQueryInvariantRelation` | invariant_projection -> handler | 1 | movable | carries 2 |
| `invariant_projection::walk::result::WorthQueryInvariantProjectionWalk` | invariant_projection -> handler | 1 | movable | carries 6 |
| `output_lineage::computation_source::ComputationSourceEvidence` | output_lineage -> application_checkpoint | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 749: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `output_lineage::family_selection::checkpoint_priors::NativePriorCheckpointOutput` | output_lineage -> application_checkpoint | 1 | movable | carries 7 |
| `output_lineage::invalidation::commit_touches::CommitTouchEntities` | output_lineage -> application_discovery | 1 | movable |  |
| `output_lineage::invalidation::commit_touches::TouchedCommits` | output_lineage -> conditional_operation | 1 | movable | carries 1 |
| `output_lineage::invalidation::derived::SettlementRegistrationStop` | output_lineage -> application_output_demand | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `output_lineage::native_output_witness::PreparedNativeOutputWitness` | output_lineage -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 358: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `output_lineage::semantic_source::SemanticSource` | output_lineage -> application_output_demand | 1 | movable |  |
| `product_operation::context::ProductOperationContextScopedOps` | product_operation -> application_entry | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 909: `with_checkpoint`, `with_resources`, `checkpoint_names`, `transition_required` … |
| `product_operation::program_adoption::authorization::ProgramAdoptionAdmission` | product_operation -> program_occurrence | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1092: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `product_operation::shared_query_basis::SharedSelectedProductOperation` | product_operation -> application_entry | 1 | movable | carries 1 |
| `provider::aftermath_causality::canonical_record::WorthQueryCanonicalCorrectionFact` | provider -> product_operation | 1 | movable |  |
| `provider::aftermath_causality::correction_history::WorthQueryActiveReversalProgress` | provider -> product_operation | 1 | movable | carries 2 |
| `provider::aftermath_causality::correction_history::WorthQueryCorrectionHistoryProgress` | provider -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 4: `allocation_denial`, `allocation_failed`, `overflow`, `slots` |
| `provider::aftermath_causality::correction_history::page_allocation::WorthQueryCorrectionHistoryPageAllocation` | provider -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 4: `allocation_denial`, `allocation_failed`, `overflow`, `slots` |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::failure::RebaseVerificationFailure` | provider -> output_lineage | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1: `equal_cause` |
| `provider::application_decision_fact::application_facts::CompletedApplicationFacts` | provider -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 6: `DISAGREEMENT`, `request`, `validate_locator`, `binding_failure` … |
| `provider::application_decision_fact::application_table::ApplicationFactTable` | provider -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 5: `DISAGREEMENT`, `request`, `validate_locator`, `binding_failure` … |
| `provider::committed_dispatch_outbox::CommittedDispatchOutboxScopedOps` | provider -> application_inbound | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 781: `transition_required`, `validate_installed_program`, `validate_target`, `allocation_control` … |
| `provider::committed_dispatch_outbox::ProviderCommittedDispatchOutboxScopedOps` | provider -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 374: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `provider::correction_idempotency::CorrectionIdempotencyObservation` | provider -> product_operation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 315: `relational_read_kind`, `empty`, `pair`, `root` … |
| `provider::correction_witness::CorrectionPerformedWitness` | provider -> product_operation | 1 | movable |  |
| `provider::correction_witness::CorrectionWitnessDenial` | provider -> product_operation | 1 | movable |  |
| `provider::idempotency::WorthQueryProviderGuardedWorkflowOperationCustody` | provider -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `equal_cause`, `same_addresses`, `reads_source` |
| `provider::inbound_completion::read::WorthQueryCanonicalCompletionRow` | provider -> application_inbound | 1 | movable | carries 1 |
| `provider::inbound_completion::read::WorthQueryInboundCompletionReadDenial` | provider -> application_inbound | 1 | movable |  |
| `provider::inbound_terminal_index::WorthQueryCanonicalInboundCompletion` | provider -> application_inbound | 1 | movable | carries 2 |
| `provider::outstanding_dispatch::OutstandingDispatchReservation` | provider -> application_attempt | 1 | movable | carries 1 |
| `provider::outstanding_dispatch::in_flight::WorthQueryOutstandingInFlightDenial` | provider -> application_attempt | 1 | movable |  |
| `provider::pending_application_publication::registry::WorthQueryApplicationPublicationRecoveryReservation` | provider -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 290: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxBindingDenial` | provider -> application_attempt | 1 | movable |  |
| `provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxReceiptSeal` | provider -> application_attempt | 1 | movable | carries 2 |

### Types whose impls live across many modules

A unit moves with all its inherent impls, so these types cannot move, or be split into
another crate, until their impls are regrouped or turned into owned extension traits.

| type | owner | impl blocks | impl lines | modules hosting impls |
|---|---|---|---|---|
| `product_operation::context::WorthQuerySelectedProductOperation` | product_operation | 65 | 6679 | 8 |
| `bootstrap::WorthQueryPrimaryGraphBootstrap` | bootstrap | 18 | 861 | 5 |
| `root::WorthQueryPrimaryGraphIntegrationHandle` | root | 13 | 484 | 4 |
| `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` | application_installation | 37 | 3055 | 3 |
| `provider::WorthQueryPrimaryGraphProvider` | provider | 64 | 2730 | 2 |
| `application_attempt::fact::WorthQueryApplicationObservedFact` | application_attempt | 14 | 1060 | 2 |
| `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` | invariant_projection | 13 | 800 | 2 |
| `root::WorthQueryPrimaryGraph` | root | 12 | 307 | 2 |
| `entity_resolution::resolved::WorthQueryApplicationEntityIdentity` | entity_resolution | 10 | 213 | 2 |
| `application_entry::mutation::failure::HandlerFailure` | application_entry | 7 | 68 | 2 |
| `application_checkpoint::DecodedApplicationCheckpoint` | application_checkpoint | 5 | 84 | 2 |
| `application_inbound::inbound_publication::custody::InstalledTransportPendingReason` | application_inbound | 2 | 48 | 1 |
| `product_operation::document_history::navigation::output_source::WorthQueryAdmittedHistoryProgramOutput` | product_operation | 1 | 29 | 1 |
| `product_operation::query::history_outputs::WorthQueryHistoryOutputReadDenial` | product_operation | 1 | 19 | 1 |

### Direct blockers

Non-type items above the destination that a coupled unit's own declaration or impls call.

| item | units held | their references |
|---|---|---|
| `application_contribution::producer::demand::progression::denial` | 9 | 25 |
| `application_attempt::fact::adjacency::observe_adjacency` | 3 | 17 |
| `application_attempt::read_set::denial` | 3 | 4 |
| `application_contribution::producer::demand::progression::admission::source_custody::validate_prepared_source_carrier` | 3 | 9 |
| `application_query::admission::governed_access::prepare_governed_access` | 3 | 23 |
| `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` | 3 | 37 |
| `application_attempt::fact::indexed_entity_selection::observe_indexed_entity_selection` | 2 | 16 |
| `application_attempt::observation::observe_field_value` | 2 | 66 |
| `application_attempt::workflow::actor::record::validate_actor_applicability` | 2 | 2 |
| `application_attempt::workflow::definition::compilation::reconstruction::reconstruct_compiled_definition` | 2 | 2 |
| `application_attempt::workflow::instance::facts::visit_instance_start_facts` | 2 | 3 |
| `application_attempt::workflow::instance::progression::retention::SHARD_COUNT` | 2 | 2 |
| `application_attempt::workflow::instance::start_provenance::admit` | 2 | 3 |
| `application_attempt::workflow::proposal::observation::captured::target` | 2 | 2 |
| `application_attempt::workflow::proposal::observation::observe_workflow_operation_input` | 2 | 2 |
| `application_attempt::workflow_instance_observation::recorded_actor::original_parent` | 2 | 2 |
| `application_attempt::workflow_instance_observation::settlement::observe_retained_transition` | 2 | 2 |
| `application_attempt::workflow_instance_program::succession::mismatch` | 2 | 3 |
| `application_attempt::workflow_transition_program::approval_decision::validation::validate_requirement_definition` | 2 | 2 |
| `application_attempt::workflow_transition_program::operation::mismatch` | 2 | 3 |
| `application_contribution::producer::demand::disclosure::validate_disclosure` | 2 | 4 |
| `application_query::component_result::inspect` | 2 | 5 |
| `application_query::execution_validation::validate_execution_lifetimes` | 2 | 2 |
| `application_query::execution_validation::validate_execution_plan` | 2 | 2 |
| `application_query::managed_index::source::initialization::initialization_execution_denial` | 2 | 5 |
| `application_query::observed_source::retained_dependencies::retain_joined_descriptors` | 2 | 20 |
| `application_query::one_shot::map_authorized_read_denial` | 2 | 2 |
| `application_query::orbit_result::inspect` | 2 | 5 |
| `invariant_projection::NEXT_INVARIANT_PROJECTION_AUTHORITY` | 2 | 5 |
| `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` | 2 | 49 |

### What holds the behaviour-coupled units (transitively)

Each non-type item above a destination that some coupled unit (directly or through a
coupled type it needs) depends on; inverting the top rows frees the most units.

| item | units held | their references |
|---|---|---|
| `application_query::managed_index::provider_failure::equal_cause` | 140 | 651 |
| `application_contribution::conditional::output_readiness_contract::denial` | 121 | 487 |
| `application_contribution::native_field_contract::field_denial` | 119 | 498 |
| `application_contribution::conditional::output_readiness_contract::typed::field_denial` | 118 | 483 |
| `application_contribution::producer::denial_value::mismatch_or` | 113 | 481 |
| `application_contribution::producer::denial_value::declaration::encode_declared` | 110 | 475 |
| `application_contribution::producer::registry::semantic_edition::posture_code` | 108 | 457 |
| `application_contribution::producer::demand::progression::required_wave::capacity_denial` | 101 | 436 |
| `application_query::observed_source::source_identity::material_storage::admit_footprint_comparison` | 100 | 429 |
| `application_query::one_shot::denial::authorization_denial` | 97 | 415 |
| `application_query::one_shot::denial::denial` | 97 | 415 |
| `application_query::resource_lifecycle::lifecycle_count::release` | 96 | 423 |
| `application_contribution::partitioned_computation::incremental::observed::observe_retained_full_tree` | 95 | 428 |
| `output_lineage::native_output_witness::overflow` | 95 | 429 |
| `output_lineage::native_output_witness::retirement::native_metadata` | 95 | 429 |
| `application_query::observed_source::source_identity::membership::same_footprint_membership` | 94 | 413 |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::resolution::reads_source` | 94 | 417 |
| `application_query::observed_source::source_identity::checkpoint_commitment::checkpoint_source_identity` | 93 | 416 |
| `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` | 93 | 410 |
| `provider::relational_execution_denial::relational_execution_kind` | 92 | 428 |

### Functions, constants and modules referenced against the stack

| item | kind | owner | refs | from |
|---|---|---|---|---|
| `provider::relational_execution_denial::relational_execution_kind` | fn | provider | 24 | application_attempt 4, application_checkpoint 2, application_inbound 1, application_output_demand 1, bootstrap 2, bootstrap_publication 2, conditional_operation 3, output_reuse 1, product_activation 1, product_operation 5, resolution 1, resolution_denial 1 |
| `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` | fn | invariant_projection | 5 | application_query 1, output_lineage 1, provider 2, source_facts 1 |
| `application_attempt::fact::adjacency::observe_adjacency` | fn | application_attempt | 4 | application_discovery 4 |
| `application_attempt::observation::predecode_admission::observe_field_value_borrowed` | fn | application_attempt | 3 | application_discovery 3 |
| `application_contribution::execution_denial::advancement_scope_stop` | fn | application_contribution | 3 | invariant_projection 1, output_lineage 1, product_operation 1 |
| `application_contribution::execution_denial::publication::publication_execution_kind` | fn | application_contribution | 3 | application_query 1, entity_resolution 1, provider 1 |
| `application_attempt::observation::observe_field_value` | fn | application_attempt | 2 | aggregate_projection 1, application_discovery 1 |
| `application_attempt::workflow_instance_observation::governed_record` | module | application_attempt | 2 | application_discovery 2 |
| `application_contribution::execution_denial::advancement_checkpoint_stop` | fn | application_contribution | 2 | application_entry 1, application_query 1 |
| `application_contribution::execution_denial::scope::scope_stop_kind` | fn | application_contribution | 2 | application_query 1, entity_resolution 1 |
| `application_installation::program::workflow_runtime::authentication::workflow_approval_authentication_intent` | fn | application_installation | 2 | application_attempt 2 |
| `application_query::admission::funded_parameters::capture_parameters` | fn | application_query | 2 | application_resource 1, source_disclosure 1 |
| `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` | fn | invariant_projection | 2 | application_output_demand 1, application_query 1 |
| `provider::relational_execution_denial::native_preparation_kind` | fn | provider | 2 | application_attempt 2 |
| `provider::relational_execution_denial::query_read_execution_kind` | fn | provider | 2 | application_attempt 1, resolution_denial 1 |
| `application_attempt::fact::consult_observation` | module | application_attempt | 1 | freshness 1 |
| `application_attempt::migration::prepare_program_migration_effects` | fn | application_attempt | 1 | product_operation 1 |
| `application_attempt::workflow::adoption::inventory::inventory_workflows` | fn | application_attempt | 1 | product_operation 1 |
| `application_attempt::workflow::adoption::staging::stage_workflow_dispositions` | fn | application_attempt | 1 | product_operation 1 |
| `application_attempt::workflow::correction::retain_current_continuation` | fn | application_attempt | 1 | product_operation 1 |
| `application_attempt::workflow::correction::unavailable` | fn | application_attempt | 1 | product_operation 1 |
| `application_attempt::workflow::instance::progression::retention::default_progress_retention_shards` | fn | application_attempt | 1 | root 1 |
| `application_attempt::workflow::schema::fact_custody::fact_custody_registration` | fn | application_attempt | 1 | invariant_installation 1 |
| `application_attempt::workflow::schema::indexes::register_indexes` | fn | application_attempt | 1 | root 1 |
| `application_attempt::workflow::schema::lower_workflow` | fn | application_attempt | 1 | schema_layout 1 |
| `application_attempt::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION` | const | application_attempt | 1 | application_discovery 1 |
| `application_attempt::workflow::schema::version::WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION` | const | application_attempt | 1 | application_discovery 1 |
| `application_attempt::workflow_instance_observation::audit::observe` | fn | application_attempt | 1 | application_discovery 1 |
| `application_attempt::workflow_instance_observation::history_projection` | module | application_attempt | 1 | application_discovery 1 |
| `application_attempt::workflow_instance_observation::subject::observe_subject` | fn | application_attempt | 1 | application_discovery 1 |
| `application_contribution::computation::wave_dispatch::dispatch` | fn | application_contribution | 1 | application_entry 1 |
| `application_contribution::execution_denial` | module | application_contribution | 1 | application_attempt 1 |
| `application_contribution::partitioned_computation::incremental::published_state::observe_published` | fn | application_contribution | 1 | output_lineage 1 |
| `application_contribution::producer::demand::bridge_denial::bridge_denial_kind` | fn | application_contribution | 1 | application_output_demand 1 |
| `application_contribution::producer::readiness::install_output_readiness_routes` | fn | application_contribution | 1 | application_runtime 1 |
| `application_query::derived_view::change_summary::changes_from_summary` | fn | application_query | 1 | provider 1 |
| `application_query::managed_index::invariant_read` | module | application_query | 1 | application_invariant 1 |
| `application_query::managed_index::source::initialization::initialization_execution_denial` | fn | application_query | 1 | bootstrap 1 |
| `application_query::projection::retained_projection::comparison::compare_original_projections` | fn | application_query | 1 | causal_inspection 1 |
| `application_query::projection::retained_projection::materialization::materialize_original_projection` | fn | application_query | 1 | causal_inspection 1 |
| `application_runtime::installation::publish_application_runtime_with_conditionals` | fn | application_runtime | 1 | conditional_operation 1 |
| `invariant_projection::consumed_output::verification::work_budget::reserve_pending` | fn | invariant_projection | 1 | output_lineage 1 |
| `invariant_projection::operation_reader::managed_index::native_index_decision_cause` | fn | invariant_projection | 1 | application_entry 1 |
| `output_lineage::input_cutoff::verification::cutoff_declines` | fn | output_lineage | 1 | output_reuse 1 |
| `output_lineage::invalidation::publication::register_republished` | fn | output_lineage | 1 | product_operation 1 |
| `provider::invariant_execution_failure::map_validation_failure` | fn | provider | 1 | bootstrap_publication 1 |
| `provider::relational_execution_denial` | module | provider | 1 | product_operation 1 |
| `provider::relational_execution_denial::preflight::preflight_execution_cause` | fn | provider | 1 | resolution 1 |

## Ruling sheet: heaviest mutual pairs

### `application_output_demand` ↔ `application_contribution`

- references: application_output_demand -> application_contribution 23, application_contribution -> application_output_demand 383
- against the stack (application_output_demand -> application_contribution): 23 references; 6 movable units, 5 behaviour-coupled, 0 pinned, 1 functions/constants/modules
- crossing up: `WorthQueryProducerCommitAuthority` (movable, 4), `WorthQueryProducerDenialValue` (behaviour-coupled, 3), `ProducerOutputEvaluation` (behaviour-coupled, 3), `SealedComputationRetention` (behaviour-coupled, 2), `WorthQueryProducerApplicability` (movable, 2), `CurrentProof` (movable, 2), `CurrentBasis` (movable, 2), `ComputationPrior` (behaviour-coupled, 1), `WorthQueryProducerDenialDecodeDenial` (movable, 1), `OwnerPlan` (movable, 1), `PendingProducerPublication` (behaviour-coupled, 1), `bridge_denial_kind` (fn, 1)
- crossing down (top): `WorthQueryOutputDemandDenial` 91, `WorthQueryOutputDemandDenialKind` 69, `SelectedReadyReadmission` 21, `WorthQueryOutputDemandRegistry` 14, `WorthQueryOutputDemandSettlement` 14, `WorthQueryOutputDemandRecoveryPosture` 12, `WorthQueryOutputDemandKey` 11, `DemandAdmissionKind` 10
- option nest: move 21122 lines of application_output_demand under application_contribution; 218 references from other modules below application_contribution would then point up
- option move units: 6 units by script (12 references)
- option contract trait: a contract owned by application_output_demand, implemented in application_contribution, 6 items (11 references)
- **default**: move the 6 movable units by script; a contract owned by application_output_demand for the 6 items that stay behaviour-bound in application_contribution

### `product_operation` ↔ `application_query`

- references: product_operation -> application_query 100, application_query -> product_operation 132
- against the stack (product_operation -> application_query): 100 references; 15 movable units, 22 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryApplicationReadObservation` (behaviour-coupled, 18), `WorthQueryApplicationQueryAccessContext` (movable, 7), `WorthQueryApplicationQueryAdmissionDenial` (movable, 7), `AdmissionScopedOps` (behaviour-coupled, 6), `WorthQueryApplicationQueryControls` (behaviour-coupled, 6), `WorthQueryApplicationBasisLease` (behaviour-coupled, 6), `WorthQueryApplicationQueryAdmissionDenialKind` (movable, 4), `CompletedMetadataRead` (movable, 4), `WorthQueryAdmittedApplicationQueryPlan` (behaviour-coupled, 4), `WorthQueryCheckpointSourceIdentity` (movable, 3), `WorthQueryRuntimeSourceIdentity` (movable, 3), `WorthQueryApplicationQueryBasisCustody` (behaviour-coupled, 2), `WorthQueryManagedIndexDenial` (behaviour-coupled, 2), `WorthQueryObservedSource` (behaviour-coupled, 2) …
- crossing down (top): `WorthQuerySelectedProductOperation` 25, `WorthQueryDocumentHistory` 11, `WorthQueryDocumentHistoryDenial` 10, `WorthQueryHistoryNavigationDenial` 6, `ApplicationBasisScopedOps` 4, `SelectedQueryBasisRetentionStop` 4, `WorthQueryHistoryOutputReadDenial` 4, `WorthQueryHistoryPageDenial` 3
- option nest: move 25495 lines of product_operation under application_query; 102 references from other modules below application_query would then point up
- option move units: 15 units by script (38 references)
- option contract trait: a contract owned by product_operation, implemented in application_query, 22 items (62 references)
- **default**: move the 15 movable units by script; a contract owned by product_operation for the 22 items that stay behaviour-bound in application_query

### `application_attempt` ↔ `provider`

- references: application_attempt -> provider 63, provider -> application_attempt 149
- against the stack (application_attempt -> provider): 63 references; 14 movable units, 12 behaviour-coupled, 0 pinned, 3 functions/constants/modules
- crossing up: `WorthQueryPrimaryGraphProvider` (behaviour-coupled, 10), `WorthQueryPrimaryGraphCommittedApplication` (behaviour-coupled, 5), `WorthQueryCommittedDispatchOutboxObservation` (movable, 4), `WorthQueryProviderIdempotencyResolution` (behaviour-coupled, 4), `WorthQueryPrimaryMutationWorkEvidence` (movable, 3), `WorthQueryCommittedDispatchOutboxBinding` (movable, 3), `OwnEffectOnReads` (behaviour-coupled, 2), `PreparedRebaseDenial` (movable, 2), `PreparedSourceFactRebase` (behaviour-coupled, 2), `WorthQueryApplicationBranchCommitCoordination` (movable, 2), `WorthQueryDispatchOutboxBasis` (movable, 2), `WorthQueryProviderIdempotencyResolutionDenial` (movable, 2), `WorthQueryInboundTerminalIndexDenial` (movable, 2), `FactlessCurrentness` (movable, 1) …
- crossing down (top): `WorthQueryApplicationIdempotencyBinding` 21, `WorthQueryPrimaryGraphApplicationAttempt` 16, `WorthQueryApplicationObservedFact` 16, `StorageControl` 14, `WorthQueryApplicationCommitOutcomeIdentity` 9, `StoreDenial` 9, `WorthQueryCommittedProductPublication` 7, `AdmittedAdjacencyEndpoints` 4
- option nest: move 86802 lines of application_attempt under provider; 457 references from other modules below provider would then point up
- option move units: 14 units by script (26 references)
- option contract trait: a contract owned by application_attempt, implemented in provider, 15 items (37 references)
- **default**: move the 14 movable units by script; a contract owned by application_attempt for the 15 items that stay behaviour-bound in provider

### `application_query` ↔ `application_contribution`

- references: application_query -> application_contribution 3, application_contribution -> application_query 152
- against the stack (application_query -> application_contribution): 3 references; 0 movable units, 0 behaviour-coupled, 0 pinned, 3 functions/constants/modules
- crossing up: `advancement_checkpoint_stop` (fn, 1), `publication_execution_kind` (fn, 1), `scope_stop_kind` (fn, 1)
- crossing down (top): `WorthQueryApplicationProjection` 25, `WorthQueryObservedSource` 25, `WorthQueryApplicationOutputDemandSource` 16, `WorthQueryApplicationReadObservation` 10, `WorthQueryApplicationBasisSelectionIdentity` 7, `ApplicationQueryNativeAccess` 7, `WorthQueryObservedSourceEpoch` 5, `WorthQueryApplicationOneShotDenialKind` 4
- option nest: move 56615 lines of application_query under application_contribution; 508 references from other modules below application_contribution would then point up
- option move units: 0 units by script (0 references)
- option contract trait: a contract owned by application_query, implemented in application_contribution, 3 items (3 references)
- **default**: move the 0 movable units by script; a contract owned by application_query for the 3 items that stay behaviour-bound in application_contribution

### `application_attempt` ↔ `invariant_projection`

- references: application_attempt -> invariant_projection 37, invariant_projection -> application_attempt 111
- against the stack (application_attempt -> invariant_projection): 37 references; 7 movable units, 7 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `ConsumedOutputEvidence` (behaviour-coupled, 7), `ExpectedFactKeys` (behaviour-coupled, 6), `ConsumedOutputVerificationStop` (behaviour-coupled, 5), `WorthQueryOperationProjectionDenial` (movable, 4), `ObservedComputationInputs` (movable, 4), `CallShape` (movable, 2), `CallShapeTransition` (movable, 2), `WorthQueryApplicationInvariantProjectionAuthority` (behaviour-coupled, 1), `WorthQueryInvariantMutationTarget` (movable, 1), `VerificationCapacityStop` (movable, 1), `WorthQueryApplicationOperationInvariantProjectionSnapshot` (behaviour-coupled, 1), `BorrowedOperationProjection` (behaviour-coupled, 1), `ObservedOutputPosture` (movable, 1), `WorthQueryRealizedProjectionScope` (behaviour-coupled, 1)
- crossing down (top): `WorthQueryApplicationFactKey` 17, `WorthQueryApplicationAttemptDenial` 11, `WorthQueryApplicationAttemptDenialKind` 9, `WorthQueryApplicationObservedFact` 8, `WorthQueryApplicationOutputCorrespondence` 6, `RetainedSourceFacts` 5, `WorthQueryApplicationSnapshotLease` 5, `ComputationFactAttribution` 4
- option nest: move 86802 lines of application_attempt under invariant_projection; 694 references from other modules below invariant_projection would then point up
- option move units: 7 units by script (15 references)
- option contract trait: a contract owned by application_attempt, implemented in invariant_projection, 7 items (22 references)
- **default**: move the 7 movable units by script; a contract owned by application_attempt for the 7 items that stay behaviour-bound in invariant_projection

### `output_lineage` ↔ `invariant_projection`

- references: output_lineage -> invariant_projection 45, invariant_projection -> output_lineage 98
- against the stack (output_lineage -> invariant_projection): 45 references; 2 movable units, 6 behaviour-coupled, 0 pinned, 2 functions/constants/modules
- crossing up: `ConsumedOutputEvidence` (behaviour-coupled, 21), `ConsumedOutputVerificationStop` (behaviour-coupled, 8), `ConsumedOutputVerification` (behaviour-coupled, 5), `EvidenceView` (movable, 4), `ObservedOutputCandidate` (behaviour-coupled, 2), `SelectedPendingConsumedOutput` (behaviour-coupled, 1), `PublicationRecoveryStop` (behaviour-coupled, 1), `ObservedOutputPosture` (movable, 1), `map_admission_stop` (fn, 1), `reserve_pending` (fn, 1)
- crossing down (top): `RecordedSettlementIdentity` 15, `SourceInvalidationOwner` 13, `SealedNativeOutputWitness` 11, `ComparableSourceFacts` 8, `FullVerificationReason` 8, `SettlementRegistrationStop` 6, `SourceSettlementCurrentness` 5, `WorthQueryPriorOutputDenial` 3
- option nest: move 19006 lines of output_lineage under invariant_projection; 120 references from other modules below invariant_projection would then point up
- option move units: 2 units by script (5 references)
- option contract trait: a contract owned by output_lineage, implemented in invariant_projection, 8 items (40 references)
- **default**: move the 2 movable units by script; a contract owned by output_lineage for the 8 items that stay behaviour-bound in invariant_projection

### `product_operation` ↔ `application_attempt`

- references: product_operation -> application_attempt 75, application_attempt -> product_operation 39
- against the stack (product_operation -> application_attempt): 75 references; 5 movable units, 25 behaviour-coupled, 0 pinned, 5 functions/constants/modules
- crossing up: `WorthQueryApplicationIdempotencyBinding` (behaviour-coupled, 16), `WorthQueryApplicationOutputCorrespondence` (behaviour-coupled, 8), `WorthQueryWorkflowDispositions` (behaviour-coupled, 5), `WorthQueryApplicationCommitReceipt` (behaviour-coupled, 4), `WorthQueryApplicationAttemptDenial` (behaviour-coupled, 4), `WorthQueryApplicationOutputProjectionDenial` (movable, 3), `OutputRoleUse` (behaviour-coupled, 3), `WorthQueryApplicationObservedFact` (behaviour-coupled, 3), `WorthQueryWorkflowAdoptionInventory` (behaviour-coupled, 2), `PreparedWorkflowCorrection` (behaviour-coupled, 2), `WorthQueryApplicationCommitOutcome` (behaviour-coupled, 1), `WorthQueryApplicationCommitDenial` (behaviour-coupled, 1), `WorthQueryApplicationCommitDenialStage` (movable, 1), `WorthQueryApplicationCommittedChanges` (behaviour-coupled, 1) …
- crossing down (top): `WorthQuerySelectedProductOperation` 18, `HistoryProducerInvocation` 9, `WorthQueryAdmittedApplicationConditionalDefinition` 7, `AuthoredCorrectionCandidateDenial` 1, `SelectedQueryOrigin` 1, `ObservationRoomScopedOps` 1, `WorthQueryProgramMigrationPreparationDenial` 1, `SharedSelectedProductOperation` 1
- option nest: move 25495 lines of product_operation under application_attempt; 24 references from other modules below application_attempt would then point up
- option move units: 5 units by script (7 references)
- option contract trait: a contract owned by product_operation, implemented in application_attempt, 30 items (68 references)
- **default**: nest product_operation under application_attempt (25495 lines); its other users then go through application_attempt

### `application_output_demand` ↔ `output_lineage`

- references: application_output_demand -> output_lineage 60, output_lineage -> application_output_demand 48
- against the stack (application_output_demand -> output_lineage): 60 references; 4 movable units, 10 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `RecordedSettlementIdentity` (movable, 22), `SourceInvalidationOwner` (behaviour-coupled, 11), `PublishedStableLineage` (behaviour-coupled, 7), `RetainedOutputCurrentnessRead` (behaviour-coupled, 3), `SealedNativeOutputWitness` (behaviour-coupled, 3), `ComparableSourceFacts` (movable, 3), `PreparedInputReuseKey` (behaviour-coupled, 2), `BoundCurrentAcceptedOutput` (behaviour-coupled, 2), `PriorComputationRecord` (behaviour-coupled, 2), `SettlementRegistrationStop` (behaviour-coupled, 1), `SourceSettlementCurrentness` (behaviour-coupled, 1), `FullVerificationReason` (behaviour-coupled, 1), `RecordedSourceIdentity` (movable, 1), `SemanticSource` (movable, 1)
- crossing down (top): `WorthQueryOutputDemandDenial` 14, `WorthQueryOutputDemandDenialKind` 11, `RequiredWorkMembership` 9, `WorthQueryAcceptedOutputCheckpointIdentity` 6, `ReadyCompletion` 3, `ReplacedRequiredWorkHint` 2, `WorthQueryAcceptedOutputCheckpointPosture` 1, `WorthQueryOutputDemandSettlement` 1
- option nest: move 21122 lines of application_output_demand under output_lineage; 67 references from other modules below output_lineage would then point up
- option move units: 4 units by script (27 references)
- option contract trait: a contract owned by application_output_demand, implemented in output_lineage, 10 items (33 references)
- **default**: move the 4 movable units by script; a contract owned by application_output_demand for the 10 items that stay behaviour-bound in output_lineage

### `application_attempt` ↔ `output_lineage`

- references: application_attempt -> output_lineage 13, output_lineage -> application_attempt 93
- against the stack (application_attempt -> output_lineage): 13 references; 3 movable units, 3 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `PostconditionCarry` (behaviour-coupled, 4), `ObservedSourcePublication` (behaviour-coupled, 3), `RecordedSettlementIdentity` (movable, 2), `ComparableSourceFacts` (movable, 2), `PublishedStableLineage` (behaviour-coupled, 1), `RecordedSourceIdentity` (movable, 1)
- crossing down (top): `WorthQueryApplicationObservedFact` 19, `WorthQueryApplicationOutputCorrespondence` 15, `StorageControl` 10, `CompletedDecisionReuseProof` 8, `CompletedHandlerFactBoundary` 7, `StoreDenial` 7, `WorthQuerySourceCurrentnessFailure` 6, `Movement` 5
- option nest: move 86802 lines of application_attempt under output_lineage; 364 references from other modules below output_lineage would then point up
- option move units: 3 units by script (5 references)
- option contract trait: a contract owned by application_attempt, implemented in output_lineage, 3 items (8 references)
- **default**: move the 3 movable units by script; a contract owned by application_attempt for the 3 items that stay behaviour-bound in output_lineage

### `application_checkpoint` ↔ `application_attempt`

- references: application_checkpoint -> application_attempt 44, application_attempt -> application_checkpoint 53
- against the stack (application_checkpoint -> application_attempt): 44 references; 1 movable units, 4 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryApplicationObservedFact` (behaviour-coupled, 14), `StoreDenial` (behaviour-coupled, 14), `StorageControl` (behaviour-coupled, 14), `WorthQueryCheckpointOutputRole` (movable, 1), `AdmittedAdjacencyEndpoints` (behaviour-coupled, 1)
- crossing down (top): `FactPayload` 23, `FactSequence` 18, `CheckpointMetadataFunding` 2, `facts` 2, `CheckpointDecodeDenial` 2, `NativeFactPayloadBuilder` 2, `FactDecodeDenial` 1, `CodecPayloadReservation` 1
- option nest: move 4169 lines of application_checkpoint under application_attempt; 18 references from other modules below application_attempt would then point up
- option move units: 1 units by script (1 references)
- option contract trait: a contract owned by application_checkpoint, implemented in application_attempt, 4 items (43 references)
- **default**: move the 1 movable units by script; a contract owned by application_checkpoint for the 4 items that stay behaviour-bound in application_attempt

## Estimate

- defaults for the pairs above: largest component 58, 1043 references against the stack; 52 units by script, 78 items by hand, 1 nests
- defaults plus every movable unit moved: largest component 58, 618 references against the stack; 179 units by script, 263 items by hand
- and the five widest hub types (`WorthQueryProgramApplicationRuntime`, `WorthQueryPrimaryGraphBootstrap`, `WorthQuerySelectedProductOperation`, `WorthQueryPrimaryGraphProvider`, `WorthQueryPrimaryGraphIntegrationHandle`) behind contracts: largest component 58, 598 references against the stack

