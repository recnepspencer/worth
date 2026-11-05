# Ownership analysis: `domain_computation::primary_graph`

## Draft target stack (bottom to top)


### identity_keys_and_refusals

- `application_branch` (rank 0): the sole ordinary-branch name constant; uses below nothing; above nothing; used by application_runtime 3, conditional_operation 2, managed_bridge 1.
- `entity_key` (rank 1): caller-chosen key for a created or seeded entity; uses below nothing; above nothing; used by facade 2, application_attempt 2, typed_bootstrap 1.
- `principal_key` (rank 2): typed bootstrap identity of one principal entity; uses below nothing; above nothing; used by facade 2, bootstrap 1.
- `aggregate_identity` (rank 3): application aggregate names; reads still need view admission; uses below nothing; above nothing; used by invariant_projection 12, application_invariant 6, facade 3.
- `output_binding_identity` (rank 4): declared binding names that order lineage; uses below nothing; above nothing; used by output_lineage 6.
- `output_family_identity` (rank 5): declared output-family identity, the family ordering axis; uses below nothing; above nothing; used by output_lineage 3, application_output_demand 2, application_contribution 2.
- `admitted_capture` (rank 6): one original-request capture posture for opaque projections; uses below nothing; above nothing; used by causal_inspection 68, application_query 10, application_resource 7.
- `exact_basis_access` (rank 7): opens a snapshot from the exact current branch observation; uses below nothing; above nothing; used by provider 11, product_operation 3, application_query 2.
- `index_maintenance_budget` (rank 8): the ordinary commit's index maintenance budget; uses below nothing; above nothing; used by provider 4, index_refresh 2, index_currency 2.
- `request_local` (rank 9): state owned by one request's thread of control; uses below nothing; above nothing; used by invariant_projection 1, application_contribution 1.
- `request_allocation_control` (rank 10): attempt-local stop custody, never commit authority; uses below nothing; above nothing; used by application_attempt 3, provider 1, product_operation 1.
- `record_reservations` (rank 11): bounded host observation of native reservations; uses below nothing; above runtime_host 1; used by application_runtime 2, facade 1. **greedy disagrees** (greedy rank 72)
- `selected_read_indexes` (rank 12): selected-index currency retained at its admitted basis; uses below nothing; above nothing; used by application_query 4, product_operation 2, application_runtime 1.
- `read_observation` (rank 13): projection work and installed invariant read counters; uses below nothing; above nothing; used by invariant_projection 12, application_query 9, root_path_selection 5.
- `program_occurrence` (rank 14): which rostered program one occurrence runs under (identity), though it reads installation; uses below nothing; above application_installation 3, application_attempt 2, product_operation 1; used by application_attempt 13, application_installation 8, product_operation 7.
- `publication_request_denial` (rank 15): request stops carried by World's no-effect publication outcomes; uses below nothing; above nothing; used by nothing.
- `value_decode` (rank 16): one native projection's codec admission; uses below nothing; above nothing; used by application_query 23, application_attempt 10, invariant_projection 9.
- `content` (rank 17): field-bound completed content values and their door refusals; uses below value_decode 6; above application_entry 1, application_attempt 1; used by invariant_projection 3, handler 3, application_attempt 3. **greedy disagrees** (greedy rank 46)
- `entity_resolution_denial` (rank 18): why an entity could not be resolved; uses below nothing; above nothing; used by entity_resolution 20, invariant_projection 12, conditional_operation 10.
- `denial` (rank 19): why installing or publishing the primary graph runtime was refused; uses below nothing; above application_contribution 1; used by application_contribution 48, schema_layout 35, bootstrap 35.
- `initial_schema_denial` (rank 20): maps the initial schema installation refusal; uses below denial 2; above nothing; used by bootstrap 2.
- `resolution_denial` (rank 21): why a principal could not be resolved; uses below exact_basis_access 1; above provider 2, index_currency 1; used by resolution 22, observations 8, conditional_operation 8.
- `runtime_host` (rank 22): the runtime identity every module host extends (phase 2); uses below nothing; above root 2, managed_bridge 1, granular_invalidation 1; used by application_installation 12, application_output_demand 4, product_operation 3.

### schema_and_native_facts

- `schema_layout` (rank 23): the installed schema's layout: kinds, fields, indexes; uses below denial 35; above application_attempt 3, application_discovery 2; used by application_attempt 66, application_query 36, invariant_projection 17.
- `index_currency` (rank 24): probes the selected published index generation; uses below index_maintenance_budget 2, exact_basis_access 1; above root 2; used by application_discovery 5, resolution 4, application_query 3.
- `structural_walk` (rank 25): complete original Native truth, independent of its consumer; uses below nothing; above application_attempt 1; used by invariant_projection 17, application_query 9, source_facts 6.
- `derived_source` (rank 26): complete actual Native derived truth; uses below structural_walk 2; above application_attempt 1; used by source_facts 4, source_read_verification 2, root_path_selection 2.
- `source_facts` (rank 27): Native physical read facts shared by Query and operation selection; uses below structural_walk 6, derived_source 4, admitted_capture 2; above application_attempt 3, invariant_projection 2; used by application_query 100, invariant_projection 13, component_selection 13.
- `aggregate_observation` (rank 28): actual selected-root aggregate truth; uses below source_facts 2; above application_attempt 3; used by invariant_projection 23, application_query 12, component_selection 10.
- `root_path_selection` (rank 29): the one physical traversal engine for installed paths; uses below source_facts 9, read_observation 5, structural_walk 4; above nothing; used by application_query 13, invariant_projection 10, component_selection 2.
- `component_selection` (rank 30): a scoped answer's original Native bodies and visibility facts; uses below source_facts 13, aggregate_observation 10, schema_layout 3; above application_attempt 4, source_read_verification 1; used by invariant_projection 19, application_query 14, application_attempt 5.
- `source_read_verification` (rank 31): completed exact-Arc comparisons within one verification extent; uses below structural_walk 3, derived_source 2, component_selection 2; above application_attempt 3; used by application_attempt 2, output_reuse 1, component_selection 1.
- `source_read_equivalence` (rank 32): full immutable body equality within one interning pass; uses below structural_walk 3, component_selection 2, aggregate_observation 2; above nothing; used by application_query 4. **greedy disagrees** (greedy rank 67)
- `fact_verification_observation` (rank 33): retained-fact and computation handoff work; uses below nothing; above output_lineage 2, application_attempt 1; used by application_attempt 18, facade 12, application_output_demand 12.
- `aggregate_projection` (rank 34): every source a patch touched, for cached aggregate generations; uses below nothing; above application_attempt 1; used by invariant_projection 5, root 2.

### principals_and_resolution

- `authenticated_principal` (rank 35): opaque identity of one application principal; uses below nothing; above freshness 1; used by application_query 13, resolution 5, application_contribution 3.
- `observations` (rank 36): principal target and mapping observations; uses below resolution_denial 8, schema_layout 2; above nothing; used by resolution 5, freshness 5.
- `freshness` (rank 37): durable principal currentness evidence, never a live permit; uses below observations 5, resolution_denial 4, schema_layout 3; above application_attempt 3; used by resolution 4, application_attempt 2, authenticated_principal 1.
- `principal_currentness_capture` (rank 38): retains principal currentness under one admission; uses below runtime_host 1, authenticated_principal 1; above nothing; used by application_query 2, application_runtime 1, application_resource 1.
- `resolution` (rank 39): how identity lookups are verified while resolving a principal or entity; uses below resolution_denial 22, observations 5, authenticated_principal 5; above product_operation 5, provider 2, application_query 2; used by application_query 14, entity_resolution 6, causal_inspection 5.
- `entity_resolution` (rank 40): exact Relational truth that resolves one application entity; uses below entity_resolution_denial 20, resolution 6, schema_layout 3; above product_operation 5, application_query 4, application_contribution 2; used by application_query 29, application_attempt 10, causal_inspection 8.
- `source_disclosure` (rank 41): original source disclosure meaning; uses below entity_resolution 3; above application_attempt 3, application_runtime 2, application_query 2; used by causal_inspection 4, application_attempt 4, application_output_demand 2.

### graph_root_and_workflow_facts

- `root` (rank 42): the execution-owned primary logical graph of one installed schema; uses below schema_layout 5, aggregate_projection 2; above application_attempt 7, application_query 3, output_lineage 2; used by application_attempt 13, application_query 8, bootstrap 7.
- `granular_invalidation` (rank 43): installed granular invalidation and its delivery coverage; uses below root 1; above nothing; used by conditional_operation 16, facade 7, application_runtime 5.
- `managed_bridge` (rank 44): the installed application bridge source; uses below denial 2, schema_layout 1, root 1; above nothing; used by application_runtime 7, runtime_host 1.
- `expression` (rank 45): workflow condition expressions over installed query results; uses below nothing; above application_attempt 4, application_query 2; used by application_attempt 9, facade 1. **greedy disagrees** (greedy rank 20)

### attempt_and_commit

- `application_inbound` (rank 46): inbound occurrences and installed-transport completions (moved out of the runtime, proposal 02); uses below runtime_host 1; above provider 24, application_runtime 18, application_attempt 5; used by application_runtime 27, facade 17, provider 10.
- `application_attempt` (rank 47): one application attempt: read set, facts, effect program, compare-and-commit; uses below schema_layout 66, fact_verification_observation 18, denial 17; above provider 63, application_checkpoint 53, product_operation 39; used by provider 149, facade 118, invariant_projection 111.
- `live_delivery` (rank 48): live commit batches and their delivery source; uses below application_attempt 8, source_facts 6; above application_query 4; used by application_query 13, provider 9, application_discovery 2.
- `provider` (rank 49): the primary graph provider: Relational commit, decision facts, recovery; uses below application_attempt 149, schema_layout 12, exact_basis_access 11; above application_query 23, output_lineage 19, application_checkpoint 11; used by application_attempt 63, application_runtime 34, application_inbound 24.
- `application_checkpoint` (rank 50): opaque bytes of one complete application checkpoint; uses below application_attempt 44, fact_verification_observation 8, denial 6; above application_output_demand 17, output_lineage 4, bootstrap 4; used by application_attempt 53, output_lineage 33, bootstrap 14.
- `application_program` (rank 51): move-only owner evidence for one performed source publication; uses below application_attempt 1; above application_output_demand 2, application_installation 2, application_contribution 1; used by application_installation 27, facade 7, application_contribution 5. **greedy disagrees** (greedy rank 19)

### query

- `access_strategy` (rank 52): managed candidate execution for queries; uses below source_facts 2, application_attempt 1; above application_query 29; used by application_query 36, application_attempt 3, product_operation 1.
- `application_query` (rank 53): application query admission, execution, live leases and history; uses below source_facts 100, application_attempt 50, schema_layout 36; above product_operation 132, application_output_demand 20, output_reuse 8; used by application_contribution 152, facade 117, product_operation 100.
- `application_discovery` (rank 54): read-only projections of installed declaration meaning; uses below application_attempt 34, index_currency 5, live_delivery 2; above product_operation 10, application_installation 5, output_lineage 2; used by application_attempt 18, application_runtime 3, schema_layout 2.
- `application_invariant` (rank 55): domain algorithms for installed application invariants; uses below schema_layout 6, aggregate_identity 6, application_query 5; above nothing; used by invariant_installation 10, application_contribution 3, application_attempt 2.

### contribution_and_output_demand

- `output_reuse` (rank 56): retained output reuse selection and verification; uses below application_attempt 6, application_query 4, source_read_verification 1; above application_output_demand 6, output_lineage 5, invariant_projection 2; used by application_contribution 33, application_output_demand 26, application_query 8.
- `output_lineage` (rank 57): product-local semantic output correspondence; uses below application_attempt 93, application_checkpoint 33, application_query 16; above application_output_demand 48, invariant_projection 45, application_contribution 29; used by invariant_projection 98, application_output_demand 60, application_contribution 49.
- `invariant_projection` (rank 58): installation-minted authority for invariant projection; uses below application_attempt 111, output_lineage 98, application_query 24; above application_entry 19, application_contribution 17, operation_body 14; used by application_contribution 64, handler 48, output_lineage 45.
- `application_output_demand` (rank 59): the output demand registry and its progression; uses below application_attempt 61, output_lineage 60, application_query 47; above application_contribution 23, product_operation 15, conditional_operation 2; used by application_contribution 383, application_installation 55, output_lineage 48.
- `application_contribution` (rank 60): contributions: producers, computation, request advancement; uses below application_output_demand 383, application_query 152, invariant_projection 64; above product_operation 39, conditional_operation 18, application_entry 14; used by facade 81, application_installation 51, output_lineage 29.

### operation_execution

- `application_entry` (rank 61): the mutation entry contract and handler candidate building; uses below application_contribution 7, application_attempt 6, invariant_projection 3; above handler 10, product_operation 2, application_runtime 2; used by handler 30, operation_body 19, invariant_projection 19. **greedy disagrees** (greedy rank 40)
- `operation_body` (rank 62): execution of a complete installed operation body; uses below application_entry 19, application_attempt 18, invariant_projection 17; above handler 8; used by handler 42, invariant_projection 14, application_attempt 13.
- `handler` (rank 63): installed mutation handler registry and conditional definitions; uses below invariant_projection 48, operation_body 42, application_attempt 33; above product_operation 3, conditional_operation 1, bootstrap 1; used by application_entry 10, operation_body 8, invariant_projection 8.

### product_operations

- `product_activation` (rank 64): program activation inspection for a product branch; uses below program_occurrence 2, runtime_host 1, provider 1; above nothing; used by product_operation 11, application_runtime 3, application_installation 3. **greedy disagrees** (greedy rank 35)
- `product_operation` (rank 65): product-level operations: document history, adoption, materialization; uses below application_query 100, application_attempt 75, application_contribution 24; above application_runtime 11, application_installation 9, conditional_operation 6; used by application_query 132, facade 103, application_runtime 75.

### runtime_and_installation

- `typed_bootstrap` (rank 66): one entity or relation to seed at bootstrap; uses below denial 2, entity_key 1; above bootstrap 1; used by bootstrap 4, facade 2, bootstrap_publication 2. **greedy disagrees** (greedy rank 24)
- `bootstrap_publication` (rank 67): commits the bootstrap rows; uses below denial 4, provider 3, typed_bootstrap 2; above bootstrap 1; used by bootstrap 14.
- `bootstrap` (rank 68): the move-only installation phase of the primary graph; uses below denial 35, bootstrap_publication 14, application_checkpoint 14; above invariant_installation 3, application_installation 2, application_resource 1; used by application_installation 25, application_runtime 9, application_checkpoint 4.
- `invariant_installation` (rank 69): what a program-hosted installation carries into invariant lowering; uses below application_invariant 10, denial 9, schema_layout 5; above nothing; used by bootstrap 3, application_contribution 3, facade 2.
- `application_runtime` (rank 70): the purpose-scoped application runtime published from one graph; uses below application_query 94, product_operation 75, application_attempt 68; above conditional_operation 17, causal_inspection 6, application_resource 6; used by application_inbound 18, facade 12, product_operation 11.
- `application_resource` (rank 71): application resource custody over Native lifecycle and World sources; uses below application_query 9, admitted_capture 7, resolution 4; above nothing; used by application_runtime 6, application_contribution 3, conditional_operation 1.
- `conditional_operation` (rank 72): conditional operation installation and its runtime; uses below application_attempt 29, application_query 18, granular_invalidation 16; above nothing; used by facade 36, application_contribution 18, application_runtime 17.
- `application_invariant_preparation` (rank 73): prepares the graph with its exact invariant factories; uses below invariant_installation 1, denial 1, bootstrap 1; above nothing; used by nothing.
- `application_installation` (rank 74): complete construction of one contribution-composed application; uses below application_attempt 81, application_output_demand 55, application_contribution 51; above nothing; used by product_operation 9, application_attempt 7, application_discovery 5.

### surface

- `causal_inspection` (rank 75): diagnostic integration for receipt and certification callers; uses below admitted_capture 68, application_query 55, application_attempt 38; above nothing; used by application_runtime 6, application_query 1.
- `merge_unique_values` (rank 76): a merge writes unique values under the program law; uses below schema_layout 2, application_attempt 2, root 1; above nothing; used by facade 2.
- `index_refresh` (rank 77): executes a mutation and refreshes every committed branch it touched; uses below index_maintenance_budget 2, root 1, provider 1; above nothing; used by facade 2.
- `ordinary_read` (rank 78): ordinary read outcomes for the public contract; uses below invariant_projection 4, read_observation 1; above nothing; used by facade 4.
- `facade` (rank 79): public primary-graph contract aggregation; uses below application_attempt 118, application_query 117, product_operation 103; above nothing; used by nothing.

## Units referenced against the stack

- 352 units: 167 movable, 185 behaviour-coupled, 0 pinned; 1179 references
- 43 functions, constants and modules: 69 references

| unit | owner -> destination | refs | class | detail |
|---|---|---|---|---|
| `product_operation::context::WorthQuerySelectedProductOperation` | product_operation -> resolution | 79 | behaviour-coupled | 10 impl(s) need items above; blocked by 958: `with_checkpoint`, `with_resources`, `transition_required`, `validate_installed_program` … |
| `application_runtime::WorthQueryPrimaryGraphApplicationRuntime` | application_runtime -> runtime_host | 39 | behaviour-coupled | 10 impl(s) need items above; blocked by 1135: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_checkpoint::facts::payload::FactPayload` | application_checkpoint -> application_attempt | 31 | behaviour-coupled | 2 impl(s) need items above; blocked by 9: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `application_output_demand::denial::WorthQueryOutputDemandDenial` | application_output_demand -> application_attempt | 30 | behaviour-coupled | 3 impl(s) need items above; blocked by 14: `bridge_denial_kind`, `mismatch_or`, `canonical_wire_maximum`, `is_temporarily_deferred` … |
| `invariant_projection::consumed_output::ConsumedOutputEvidence` | invariant_projection -> application_attempt | 29 | behaviour-coupled | 2 impl(s) need items above; blocked by 673: `insert`, `lookup`, `with_checkpoint`, `with_resources` … |
| `product_operation::shared_query_basis::SharedSelectedProductOperation` | product_operation -> application_attempt | 25 | movable | carries 1 |
| `application_entry::mutation::HandlerExecutionDenial` | application_entry -> content | 22 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `native_decision_cause`, `native_index_decision_cause` |
| `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` | invariant_projection -> source_facts | 22 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` | application_output_demand -> application_attempt | 21 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `is_temporarily_deferred`, `equal_cause`, `relational_execution_kind` |
| `application_checkpoint::facts::payload::sequence::FactSequence` | application_checkpoint -> application_attempt | 18 | movable |  |
| `application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity` | application_output_demand -> application_checkpoint | 16 | movable | carries 2 |
| `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` | product_operation -> application_attempt | 16 | behaviour-coupled | 2 impl(s) need items above; blocked by 8: `provider`, `equal_cause`, `authorization_denial`, `denial` … |
| `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` | application_attempt -> structural_walk | 11 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `handler::decision_context_use::DecisionContextUse` | handler -> application_attempt | 11 | movable |  |
| `product_operation::document_history::WorthQueryDocumentHistory` | product_operation -> application_query | 11 | behaviour-coupled | 5 impl(s) need items above; blocked by 16: `prepare`, `reserve_arc`, `reserve_buffer`, `confirm_in_execution` … |
| `product_operation::document_history::denial::WorthQueryDocumentHistoryDenial` | product_operation -> application_query | 11 | movable | carries 2 |
| `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` | application_installation -> application_attempt | 10 | behaviour-coupled | 4 impl(s) need items above; blocked by 559: `with_checkpoint`, `with_resources`, `CHECKSUM_BYTES`, `HEADER_BYTES` … |
| `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` | application_query -> provider | 10 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `array_denial`, `storage`, `equal_cause` |
| `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` | application_query -> application_attempt | 10 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `handler::invariant::HandlerInterruption` | handler -> application_attempt | 10 | movable |  |
| `provider::WorthQueryPrimaryGraphProvider` | provider -> application_attempt | 10 | behaviour-coupled | 3 impl(s) need items above; blocked by 253: `WIRE_VERSION`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` | application_contribution -> application_attempt | 9 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `observe_retained_full_tree` |
| `application_contribution::producer::WorthQueryProducerInvariantRequirement` | application_contribution -> application_attempt | 9 | movable |  |
| `application_output_demand::registry::required_work::RequiredWorkMembership` | application_output_demand -> output_lineage | 9 | behaviour-coupled | 2 impl(s) need items above; blocked by 5: `bridge_denial_kind`, `mismatch_or`, `is_temporarily_deferred`, `bump_version` … |
| `handler::invariant::DecisionReader` | handler -> application_attempt | 9 | behaviour-coupled | 6 impl(s) need items above; blocked by 240: `WIRE_VERSION`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation` | provider -> application_inbound | 9 | movable | carries 5 |
| `application_discovery::workflow::admission::WorthQueryWorkflowReadDenial` | application_discovery -> application_attempt | 8 | movable | carries 1 |
| `application_installation::program::output_source::recovery::WorthQueryRecoveredProgramOutputSource` | application_installation -> provider | 8 | movable |  |
| `product_operation::conditional_definition::application_change::WorthQueryAdmittedApplicationConditionalDefinition` | product_operation -> application_attempt | 8 | movable | carries 2 |
| `product_operation::document_history::navigation::WorthQueryHistoryNavigationDenial` | product_operation -> application_query | 8 | movable | carries 4 |
| `product_operation::shared_query_basis::SelectedQueryBasisRetentionStop` | product_operation -> application_query | 8 | movable |  |
| `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` | application_contribution -> application_attempt | 7 | behaviour-coupled | 1 impl(s) need items above; blocked by 17: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `application_output_demand::settlement::WorthQueryOutputDemandSettlement` | application_output_demand -> application_attempt | 7 | behaviour-coupled | 2 impl(s) need items above; blocked by 542: `with_checkpoint`, `with_resources`, `CHECKSUM_BYTES`, `HEADER_BYTES` … |
| `application_output_demand::settlement::observation::OriginalSettledProducerObservation` | application_output_demand -> application_query | 7 | behaviour-coupled | 2 impl(s) need items above; blocked by 457: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `application_query::observed_source::WorthQueryObservedSource` | application_query -> expression | 7 | behaviour-coupled | 2 impl(s) need items above; blocked by 997: `insert`, `lookup`, `with_checkpoint`, `with_resources` … |
| `handler::candidate::CandidateWriter` | handler -> application_entry | 7 | behaviour-coupled | 1 impl(s) need items above; blocked by 108: `foreign_source`, `installation_denial`, `admit_unavailable_completion`, `succession_accepts` … |
| `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` | application_contribution -> application_attempt | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 39: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `application_output_demand::registry::prerequisite_claims::PreparedPrerequisiteClaims` | application_output_demand -> provider | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 603: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_query::derived_view::registry::predicate::preparation::PredicatePreparationRetirement` | application_query -> provider | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `array_denial`, `storage`, `equal_cause` |
| `application_query::native_access::ApplicationQueryNativeAccess` | application_query -> resolution | 6 | behaviour-coupled | 1 impl(s) need items above; blocked by 1031: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `conditional_operation::signal_decision_reentry::WorthQueryConditionalTruthBasis` | conditional_operation -> application_contribution | 6 | behaviour-coupled | 1 impl(s) need items above; blocked by 103: `native_decision_cause`, `foreign_source`, `installation_denial`, `admit_unavailable_completion` … |
| `invariant_projection::consumed_output::verification::ConsumedOutputVerification` | invariant_projection -> provider | 6 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `invariant_projection::operation_reader::decision_reads::expected_keys::ExpectedFactKeys` | invariant_projection -> application_attempt | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 24: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `operation_body::emission::BodyExternalEffectSelection` | operation_body -> application_attempt | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 5: `project`, `same_relation`, `same_relation_header`, `same_row` … |
| `output_lineage::invalidation::computation_changes::postcondition_carry::PostconditionCarry` | output_lineage -> application_attempt | 6 | behaviour-coupled | 1 impl(s) need items above; blocked by 11: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `product_operation::shared_query_basis::SharedQueryBasisScopedOps` | product_operation -> application_query | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 303: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `provider::inbound_completion::WorthQueryInboundCompletionPreparationDenial` | provider -> application_inbound | 6 | movable | carries 1 |
| `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenialStage` | application_attempt -> application_inbound | 5 | movable |  |
| `application_contribution::producer::WorthQueryWorkflowAssessmentPosture` | application_contribution -> application_attempt | 5 | movable |  |
| `application_contribution::producer::source_qualification::OutputSourceAddress` | application_contribution -> application_attempt | 5 | movable |  |
| `application_output_demand::registry::required_context::RequiredOutputDemandContext` | application_output_demand -> application_attempt | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 145: `WIRE_VERSION`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `application_query::managed_index::control::WorthQueryManagedIndexControl` | application_query -> access_strategy | 5 | movable | carries 3 |
| `application_query::managed_index::stop::WorthQueryManagedIndexStop` | application_query -> access_strategy | 5 | movable | carries 2 |
| `conditional_operation::installation::denial::WorthQueryConditionalRuntimeInstallationDenial` | conditional_operation -> application_contribution | 5 | movable | carries 1 |
| `invariant_installation::WorthQueryApplicationInvariantFactories` | invariant_installation -> application_contribution | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 4: `denial`, `is_mandatory_candidate_registration`, `lower_descriptor`, `registration` |
| `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerCutover` | product_operation -> provider | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `authorization_denial`, `denial`, `map_authorized_read_denial` |
| `provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::publication::committed_application::WorthQueryPrimaryGraphCommittedApplication` | provider -> application_attempt | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 12: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `provider::inbound_terminal_index::WorthQueryInboundTerminalIndexDenial` | provider -> application_inbound | 5 | movable |  |
| `application_attempt::fact::WorthQueryApplicationObservedFact` | application_attempt -> source_facts | 4 | behaviour-coupled | 3 impl(s) need items above; blocked by 1092: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_checkpoint::facts::payload::native::NativeFactPayloadBuilder` | application_checkpoint -> application_attempt | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 11: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `application_contribution::partitioned_computation::incremental::Comparator` | application_contribution -> application_attempt | 4 | movable |  |
| `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` | application_contribution -> output_lineage | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 8: `denial`, `field_denial`, `field_denial`, `PRIOR_IN_TEST` … |
| `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` | application_contribution -> application_attempt | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 699: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_contribution::producer::WorthQueryProducerCommitAuthority` | application_contribution -> application_output_demand | 4 | movable |  |
| `application_contribution::producer::demand::progression::required_wave::resolved::matched_predecessors::MatchedRequiredPredecessors` | application_contribution -> output_lineage | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 394: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` | application_installation -> application_output_demand | 4 | behaviour-coupled | 6 impl(s) need items above; blocked by 168: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `application_query::managed_index::source_rows::WorthQueryManagedIndexSourceSlot` | application_query -> access_strategy | 4 | movable |  |
| `application_query::projection::projected_tree::content_comparison::ProjectionContentComparisonFault` | application_query -> application_attempt | 4 | movable |  |
| `bootstrap::WorthQueryPrimaryGraphBootstrap` | bootstrap -> invariant_projection | 4 | behaviour-coupled | 4 impl(s) need items above; blocked by 42: `dispatch`, `denial`, `advancement_checkpoint_stop`, `execution_failure` … |
| `bootstrap::publication::WorthQueryPrimaryGraphPublication` | bootstrap -> application_checkpoint | 4 | movable |  |
| `conditional_operation::execution_provenance::classification::WorthQueryConditionalSignalDecision` | conditional_operation -> application_output_demand | 4 | movable |  |
| `conditional_operation::installation::clock_handle::WorthQueryConditionalClockHandle` | conditional_operation -> handler | 4 | movable | carries 1 |
| `conditional_operation::lifecycle::registry::WorthQueryConditionalOperationRegistry` | conditional_operation -> product_operation | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 11: `denial`, `CHUNK_BYTES`, `lock`, `lock` … |
| `invariant_projection::consumed_output::verification::EvidenceView` | invariant_projection -> output_lineage | 4 | movable | carries 1 |
| `invariant_projection::operation_projection_denial::WorthQueryOperationProjectionDenial` | invariant_projection -> application_attempt | 4 | movable | carries 3 |
| `invariant_projection::operation_reader::decision_reads::computation_inputs::ObservedComputationInputs` | invariant_projection -> application_attempt | 4 | movable |  |
| `operation_body::WorthQueryOperationBodyDecision` | operation_body -> application_attempt | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `project`, `same_relation`, `same_relation_header`, `same_row` … |
| `operation_body::WorthQueryOperationBodyDenial` | operation_body -> application_attempt | 4 | movable |  |
| `operation_body::requirement::value::PreparedRequirementOperand` | operation_body -> invariant_projection | 4 | movable |  |
| `output_lineage::invalidation::output_availability::ObservedSourcePublication` | output_lineage -> application_attempt | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 24: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `output_lineage::prepared_slot::PreparedOutputLineageSlot` | output_lineage -> provider | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 121: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `output_lineage::settlement_identity::RecordedSettlementIdentity` | output_lineage -> application_attempt | 4 | movable | carries 2 |
| `product_operation::application_basis::ApplicationBasisScopedOps` | product_operation -> application_query | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 303: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `product_operation::query::history_outputs::WorthQueryHistoryOutputReadDenial` | product_operation -> application_query | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 7: `bridge_denial_kind`, `mismatch_or`, `canonical_wire_maximum`, `is_temporarily_deferred` … |
| `provider::idempotency::WorthQueryProviderIdempotencyResolution` | provider -> application_attempt | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 12: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `application_attempt::denial::WorthQueryApplicationAttemptDenial` | application_attempt -> content | 3 | behaviour-coupled | 4 impl(s) need items above; blocked by 3: `relational_read_kind`, `equal_cause`, `query_read_execution_kind` |
| `application_contribution::producer::denial_value::WorthQueryProducerDenialValue` | application_contribution -> application_output_demand | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `mismatch_or` |
| `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` | application_contribution -> application_output_demand | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `denial`, `field_denial`, `field_denial` |
| `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` | application_contribution -> output_lineage | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 6: `denial`, `field_denial`, `field_denial`, `encode_declared` … |
| `application_discovery::workflow::indexes::WorkflowReadIndexes` | application_discovery -> schema_layout | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1110: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_entry::mutation::HandlerResult` | application_entry -> application_attempt | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 18: `bridge_denial_kind`, `mismatch_or`, `native_decision_cause`, `canonical_wire_maximum` … |
| `application_entry::mutation::execution::MutationHandlerExecutionDenial` | application_entry -> application_attempt | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `native_decision_cause`, `native_index_decision_cause` |
| `application_entry::mutation::failure::HandlerFailure` | application_entry -> application_contribution | 3 | behaviour-coupled | 3 impl(s) need items above; blocked by 1: `native_decision_cause` |
| `application_installation::program::supported_program::WorthQuerySupportedProgramRecord` | application_installation -> program_occurrence | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1142: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_output_demand::registry::WorthQueryOutputDemandRegistry` | application_output_demand -> application_attempt | 3 | behaviour-coupled | 5 impl(s) need items above; blocked by 139: `WIRE_VERSION`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture` | application_output_demand -> application_checkpoint | 3 | movable |  |
| `application_output_demand::registry::ready_backing::ReadyCompletion` | application_output_demand -> output_lineage | 3 | behaviour-coupled | 3 impl(s) need items above; blocked by 4: `denial`, `field_denial`, `field_denial`, `classify_bridge_signal` |
| `application_query::derived_view::registry::predicate::preparation::PreparedPredicatePublication` | application_query -> application_attempt | 3 | movable | carries 3 |
| `application_query::managed_index::source_structure::SourceTree` | application_query -> access_strategy | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 11: `PAGE_BITS`, `PAGE_SIZE`, `branch`, `build` … |
| `application_resource::installation::InstalledResourceRegistry` | application_resource -> application_runtime | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 4: `denial`, `CHUNK_BYTES`, `lock`, `lock` |
| `application_resource::installation::pending::PendingResourceRegistry` | application_resource -> application_contribution | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 4: `denial`, `CHUNK_BYTES`, `lock`, `lock` |
| `application_runtime::operation_control::attempt_registration::WorthQueryApplicationAttemptOperationControl` | application_runtime -> provider | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `arm`, `clear`, `reach`, `release` … |
| `output_lineage::invalidation::mark_state::FullVerificationReason` | output_lineage -> fact_verification_observation | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `product_operation::document_history::model::HistoryModelIdentity` | product_operation -> application_query | 3 | movable |  |
| `product_operation::document_history::navigation::selection::WorthQueryHistorySelection` | product_operation -> application_query | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::paging::WorthQueryHistoryPageDenial` | product_operation -> application_query | 3 | movable |  |
| `product_operation::security_basis::matching_head::SelectedPermissionSecurityStop` | product_operation -> application_query | 3 | movable |  |
| `provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial` | provider -> application_inbound | 3 | movable |  |
| `provider::mutation_work::WorthQueryPrimaryMutationWorkEvidence` | provider -> application_attempt | 3 | movable | carries 6 |
| `provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxBinding` | provider -> application_attempt | 3 | movable | carries 1 |
| `root::WorthQueryPrimaryGraph` | root -> runtime_host | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 355: `with_checkpoint`, `with_resources`, `checkpoint_names`, `affected_sources` … |
| `root::WorthQueryPrimaryGraphIntegrationHandle` | root -> runtime_host | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 776: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` | application_attempt -> program_occurrence | 2 | behaviour-coupled | 3 impl(s) need items above; blocked by 10: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` | application_attempt -> component_selection | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `relational_read_kind`, `heap_sort`, `union_visit`, `equal_cause` … |
| `application_attempt::workflow::definition::condition::CompiledWorkflowConditionExpression` | application_attempt -> expression | 2 | movable |  |
| `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` | application_attempt -> schema_layout | 2 | behaviour-coupled | 3 impl(s) need items above; blocked by 1110: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_checkpoint::header::CheckpointDecodeDenial` | application_checkpoint -> application_attempt | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `WIRE_VERSION`, `relational_execution_kind` |
| `application_checkpoint::metadata_funding::CheckpointMetadataFunding` | application_checkpoint -> application_attempt | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `WIRE_VERSION`, `relational_execution_kind` |
| `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` | application_contribution -> output_lineage | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 7: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` | application_contribution -> output_lineage | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 7: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `application_contribution::producer::WorthQueryApplicationOutputDemand` | application_contribution -> application_program | 2 | movable | carries 3 |
| `application_contribution::producer::WorthQueryProducerApplicability` | application_contribution -> application_output_demand | 2 | movable | carries 1 |
| `application_contribution::producer::WorthQueryProducerOutputFamily` | application_contribution -> invariant_projection | 2 | movable | carries 2 |
| `application_contribution::producer::demand::progression::required_wave::performed::certification::CurrentProof` | application_contribution -> application_output_demand | 2 | movable | carries 1 |
| `application_contribution::producer::demand::progression::required_wave::performed::certification::current_basis::CurrentBasis` | application_contribution -> application_output_demand | 2 | movable |  |
| `application_contribution::producer::input_reuse_contract::WorthQueryProducerInputReuseContract` | application_contribution -> application_attempt | 2 | movable | carries 1 |
| `application_discovery::workflow::view::WorthQueryWorkflowRecordedActor` | application_discovery -> application_attempt | 2 | movable |  |
| `application_entry::mutation::OperationHandler` | application_entry -> application_contribution | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 125: `execution_failure`, `native_decision_cause`, `foreign_source`, `installation_denial` … |
| `application_entry::mutation::decision_plan::WorthQueryDecisionPlan` | application_entry -> application_contribution | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `execution_failure`, `native_decision_cause` |
| `application_installation::program::program_owner::WorthQuerySelectedProgramOwnerDenial` | application_installation -> application_output_demand | 2 | movable | carries 1 |
| `application_installation::program::program_owner::contract::WorthQueryProgramOwner` | application_installation -> product_operation | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `commit_program_action`, `commit_program_action_retained` |
| `application_output_demand::host::OutputDemandHost` | application_output_demand -> application_attempt | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 541: `with_checkpoint`, `with_resources`, `CHECKSUM_BYTES`, `HEADER_BYTES` … |
| `application_output_demand::registry::accepted_checkpoint::AcceptedCheckpointFactSource` | application_output_demand -> application_checkpoint | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 568: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_output_demand::registry::required_work::ReplacedRequiredWorkHint` | application_output_demand -> output_lineage | 2 | movable | carries 2 |
| `application_query::derived_view::registry::ManagedDerivedViewRegistry` | application_query -> root | 2 | behaviour-coupled | 3 impl(s) need items above; blocked by 12: `MAXIMUM_REGISTERED_VIEWS`, `BTREE_MAXIMUM_NODE_COMPARISONS`, `storage`, `token_charge` … |
| `application_query::projection::projected_tree::content_storage::ContentScratch` | application_query -> access_strategy | 2 | movable | carries 1 |
| `application_runtime::operation_control::attempt_registration::source_publication::receipt::WorthQuerySourcePublicationImageForTest` | application_runtime -> provider | 2 | movable | carries 1 |
| `causal_inspection::application_observation::declared_query::WorthQueryApplicationQueryObservation` | causal_inspection -> application_query | 2 | movable |  |
| `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` | conditional_operation -> application_contribution | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 157: `execution_failure`, `native_decision_cause`, `foreign_source`, `installation_denial` … |
| `conditional_operation::signal_decision_reentry::selected_projections::WorthQuerySelectedSignalProjections` | conditional_operation -> application_contribution | 2 | movable | carries 2 |
| `handler::conditional_definition::ConditionalDefinitionAdmission` | handler -> application_attempt | 2 | movable | carries 3 |
| `handler::registry::PendingMutationHandlerRegistry` | handler -> application_contribution | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 135: `execution_failure`, `native_decision_cause`, `foreign_source`, `installation_denial` … |
| `invariant_projection::consumed_output::verification::capacity_stop::VerificationCapacityStop` | invariant_projection -> application_attempt | 2 | movable |  |
| `invariant_projection::operation_reader::computation_calls::sequence::shape::CallShape` | invariant_projection -> application_attempt | 2 | movable | carries 3 |
| `invariant_projection::operation_reader::computation_calls::sequence::shape::CallShapeTransition` | invariant_projection -> application_attempt | 2 | movable | carries 1 |
| `invariant_projection::output_availability_candidate::ObservedOutputCandidate` | invariant_projection -> output_lineage | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 8: `check`, `reserve`, `reads_at_or_after`, `fact_is_current` … |
| `invariant_projection::output_availability_candidate::ObservedOutputPosture` | invariant_projection -> application_attempt | 2 | movable |  |
| `operation_body::field_expectation::BodyFieldExpectation` | operation_body -> invariant_projection | 2 | movable |  |
| `operation_body::source::PreparedBodySource` | operation_body -> invariant_projection | 2 | movable |  |
| `output_lineage::WorthQueryApplicationOutputLineage` | output_lineage -> root | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 96: `with_checkpoint`, `with_resources`, `allocation_control`, `authority_execution` … |
| `output_lineage::invalidation::owner::SourceInvalidationOwner` | output_lineage -> root | 2 | behaviour-coupled | 3 impl(s) need items above; blocked by 58: `declared_member`, `WIRE_VERSION`, `array_storage_bytes`, `bytes` … |
| `output_lineage::invalidation::settlement::posting_continuation::PostingContinuation` | output_lineage -> provider | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 580: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `output_lineage::native_output_witness::PreparedNativeOutputWitness` | output_lineage -> provider | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 10: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `output_lineage::native_output_witness::SealedNativeOutputWitness` | output_lineage -> application_checkpoint | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `overflow`, `native_metadata` |
| `output_lineage::source_facts::ComparableSourceFacts` | output_lineage -> application_attempt | 2 | behaviour-coupled | 3 impl(s) need items above; blocked by 9: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `output_reuse::verification_extent::RetainedFactVerification` | output_reuse -> application_query | 2 | movable |  |
| `product_operation::context::ProductOperationContextScopedOps` | product_operation -> application_contribution | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 109: `native_decision_cause`, `foreign_source`, `installation_denial`, `admit_unavailable_completion` … |
| `product_operation::context::SelectedQueryOrigin` | product_operation -> application_attempt | 2 | movable |  |
| `product_operation::document_history::navigation::WorthQueryDocumentHistoryNavigation` | product_operation -> application_query | 2 | behaviour-coupled | 4 impl(s) need items above; blocked by 12: `prepare`, `reserve_arc`, `reserve_buffer`, `checkpoint` … |
| `product_operation::document_history::navigation::selection::WorthQueryHistoryNavigationCompletion` | product_operation -> application_query | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::navigation::target::WorthQueryHistoryNavigationIntent` | product_operation -> application_query | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::paging::WorthQueryHistoryPageCursor` | product_operation -> application_query | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::paging::WorthQueryHistoryPageLimits` | product_operation -> application_query | 2 | movable |  |
| `product_operation::document_history::preview::WorthQueryHistoryPreview` | product_operation -> application_query | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::step::WorthQueryHistoryNodeId` | product_operation -> application_query | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::step::WorthQueryHistoryStep` | product_operation -> application_query | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::generated_materialization::restoration::invariant_denial::WorthQueryGeneratedOutputInvariantAdmissionDenial` | product_operation -> application_output_demand | 2 | movable |  |
| `product_operation::observation_room::ObservationRoomScopedOps` | product_operation -> application_attempt | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 623: `with_checkpoint`, `with_resources`, `CHECKSUM_BYTES`, `HEADER_BYTES` … |
| `product_operation::security_basis::ProductOperationSecurityBasisScopedOps` | product_operation -> application_query | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 303: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `product_operation::security_basis::WorthQueryProductSecurityBasis` | product_operation -> application_query | 2 | movable | carries 1 |
| `product_operation::security_basis::matching_head::MatchingHeadScopedOps` | product_operation -> application_query | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 303: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::OwnEffectOnReads` | provider -> application_attempt | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `reads_source` |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedRebaseDenial` | provider -> application_attempt | 2 | movable |  |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase` | provider -> application_attempt | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 22: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `provider::branch_commit_coordination::WorthQueryApplicationBranchCommitCoordination` | provider -> application_attempt | 2 | movable |  |
| `provider::dispatch_outbox::WorthQueryDispatchOutboxBasis` | provider -> application_attempt | 2 | movable |  |
| `provider::host::ProviderHost` | provider -> application_inbound | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1006: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `provider::idempotency::WorthQueryProviderIdempotencyResolutionDenial` | provider -> application_attempt | 2 | movable |  |
| `provider::outstanding_dispatch::in_flight::OutstandingDispatchInFlightLease` | provider -> application_inbound | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `decrement`, `RETAINED_RECEIPT_BASIS_LIMIT` |
| `access_strategy::realized_footprint::CapturedManagedInfluence` | access_strategy -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `with_checkpoint`, `with_resources`, `equal_cause` |
| `access_strategy::realized_footprint::CheckedRealizedSource` | access_strategy -> application_attempt | 1 | movable |  |
| `access_strategy::realized_footprint::history::ManagedHistoryRelevanceDenial` | access_strategy -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `application_attempt::denial::kind::WorthQueryApplicationAttemptDenialKind` | application_attempt -> source_disclosure | 1 | movable |  |
| `application_attempt::fact::movement::Movement` | application_attempt -> source_facts | 1 | movable |  |
| `application_attempt::workflow::definition::compilation::reuse::WorkflowDefinitionCompilationReuse` | application_attempt -> root | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 11: `relational_read_kind`, `control_variant`, `decode_control`, `append_framed` … |
| `application_attempt::workflow::definition::compilation::reuse::counters::WorthQueryWorkflowCompilationReuseCounters` | application_attempt -> root | 1 | movable |  |
| `application_attempt::workflow::definition::condition::WorkflowConditionOperand` | application_attempt -> expression | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `OPERAND_FIELDS` |
| `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressKey` | application_attempt -> root | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `SHARD_COUNT` |
| `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressRetention` | application_attempt -> root | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 1010: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_attempt::workflow::instance::progression::retention::counters::WorthQueryWorkflowInstanceProgressCounters` | application_attempt -> root | 1 | movable |  |
| `application_attempt::workflow_transition_program::publication::RequiredWorkflowCondition` | application_attempt -> expression | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 4: `OPERAND_FIELDS`, `decode_draft`, `decode_operands`, `encode_draft` |
| `application_checkpoint::facts::fact_decode_denial::FactDecodeDenial` | application_checkpoint -> application_attempt | 1 | movable |  |
| `application_checkpoint::facts::payload::CodecPayloadReservation` | application_checkpoint -> application_attempt | 1 | movable | carries 1 |
| `application_checkpoint::facts::payload::patch::NativeFactPatchBuilder` | application_checkpoint -> provider | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 9: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `application_checkpoint::facts::stream::FactDecoder` | application_checkpoint -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `MAXIMUM_FACT_BYTES`, `MINIMUM_FACT_WIRE_BYTES`, `WIRE_VERSION`, `parse_quote` … |
| `application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial` | application_contribution -> denial | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `denial` |
| `application_contribution::partitioned_computation::incremental::deposit::ComputationDeposit` | application_contribution -> invariant_projection | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 7: `denial`, `field_denial`, `field_denial`, `observe_retained_full_tree` … |
| `application_contribution::partitioned_computation::incremental::prior_absence::Suppression` | application_contribution -> invariant_projection | 1 | movable |  |
| `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` | application_contribution -> invariant_projection | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `observe_retained_full_tree` |
| `application_contribution::producer::WorthQueryProducerLifecyclePosture` | application_contribution -> output_lineage | 1 | movable |  |
| `application_contribution::producer::denial_value::WorthQueryProducerDenialDecodeDenial` | application_contribution -> application_output_demand | 1 | movable | carries 1 |
| `application_contribution::producer::execution::prepared_plan::OwnerPlan` | application_contribution -> application_output_demand | 1 | movable | carries 10 |
| `application_contribution::producer::execution::unpublished::PendingProducerPublication` | application_contribution -> application_output_demand | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 214: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `application_contribution::producer::input_reuse_contract::WorthQueryDecisionContextDependencies` | application_contribution -> application_attempt | 1 | movable |  |
| `application_contribution::producer::source_qualification::QualifiedOutputSource` | application_contribution -> output_lineage | 1 | movable | carries 2 |
| `application_contribution::producer::source_qualification::SourceQualificationScopedOps` | application_contribution -> invariant_projection | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 339: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `application_discovery::workflow::WorkflowDefinitionCurrentness` | application_discovery -> application_attempt | 1 | movable |  |
| `application_discovery::workflow::history_view::WorthQueryWorkflowApprovalRead` | application_discovery -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `text` |
| `application_discovery::workflow::history_view::WorthQueryWorkflowHistoryAudit` | application_discovery -> application_attempt | 1 | movable | carries 1 |
| `application_discovery::workflow::history_view::WorthQueryWorkflowHistoryEntry` | application_discovery -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1: `text` |
| `application_discovery::workflow::view::WorthQueryWorkflowAudit` | application_discovery -> application_attempt | 1 | movable | carries 2 |
| `application_discovery::workflow::view::WorthQueryWorkflowObservedState` | application_discovery -> application_attempt | 1 | movable |  |
| `application_discovery::workflow::view::WorthQueryWorkflowTransitionActors` | application_discovery -> application_attempt | 1 | movable | carries 1 |
| `application_entry::host::ApplicationEntryHost` | application_entry -> application_contribution | 1 | movable |  |
| `application_entry::mutation::execution::preparation::PreparationScopedOps` | application_entry -> application_contribution | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 126: `execution_failure`, `build_handler_candidate`, `native_decision_cause`, `foreign_source` … |
| `application_installation::denial::WorthQueryInMemoryApplicationDenial` | application_installation -> bootstrap | 1 | movable | carries 2 |
| `application_installation::host::ApplicationInstallationHost` | application_installation -> application_runtime | 1 | movable |  |
| `application_installation::program::demand::WorthQueryAdmittedProgramOutput` | application_installation -> product_operation | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 55: `foreign_source`, `installation_denial`, `admit_unavailable_completion`, `succession_accepts` … |
| `application_installation::program::demand::WorthQuerySettledProgramOutput` | application_installation -> product_operation | 1 | movable | carries 1 |
| `application_installation::program::demand::root_completion::OriginalProgramRootCompletion` | application_installation -> product_operation | 1 | movable |  |
| `application_installation::program::output_source::recovery::WorthQueryUnpublishedProgramOutputSource` | application_installation -> application_program | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 568: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_installation::program::program_owner::WorthQuerySelectedProgramOwner` | application_installation -> application_runtime | 1 | movable | carries 1 |
| `application_installation::program::program_owner::selected_owner::SelectedOwnerOps` | application_installation -> application_runtime | 1 | movable | carries 3 |
| `application_installation::program::specialized_action::WorthQueryAdmittedProgramOperation` | application_installation -> product_operation | 1 | movable | carries 1 |
| `application_installation::program_admission::WorthQueryAdmittedProgramSupport` | application_installation -> bootstrap | 1 | movable |  |
| `application_invariant::refusal::WorthQueryInvariantRefusalDecodeDenial` | application_invariant -> application_attempt | 1 | movable |  |
| `application_output_demand::recovered_outputs::WorthQueryRecoveredOutputs` | application_output_demand -> application_checkpoint | 1 | movable | carries 4 |
| `application_output_demand::registry::WorthQueryAcceptedOutputAuthority` | application_output_demand -> output_lineage | 1 | movable | carries 5 |
| `application_output_demand::registry::accepted_checkpoint_identity::CheckpointOutputSlot` | application_output_demand -> application_checkpoint | 1 | movable |  |
| `application_output_demand::settlement::SettlementScopedOps` | application_output_demand -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 542: `with_checkpoint`, `with_resources`, `CHECKSUM_BYTES`, `HEADER_BYTES` … |
| `application_query::admission::funded_parameters::CapturedApplicationQueryParameters` | application_query -> source_disclosure | 1 | movable |  |
| `application_query::admitted_result::WorthQueryApplicationOutputDemandSource` | application_query -> expression | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1006: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `application_query::derived_view::publication::ViewPublicationBasis` | application_query -> provider | 1 | movable |  |
| `application_query::derived_view::registry::PreparedManagedViewPublication` | application_query -> provider | 1 | movable | carries 3 |
| `application_query::derived_view::registry::predicate::preparation::PredicatePreparationFactory` | application_query -> provider | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 3: `array_denial`, `storage`, `equal_cause` |
| `application_query::derived_view::registry::predicate::preparation::round::PredicateNoticeDisposition` | application_query -> live_delivery | 1 | movable |  |
| `application_query::derived_view::registry::predicate::preparation::round::PreparedPredicateNotices` | application_query -> live_delivery | 1 | movable | carries 2 |
| `application_query::host::ApplicationQueryHost` | application_query -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 623: `with_checkpoint`, `with_resources`, `CHECKSUM_BYTES`, `HEADER_BYTES` … |
| `application_query::managed_index::invariant_read::ReadPosture` | application_query -> access_strategy | 1 | movable |  |
| `application_query::managed_index::predicate::tree::WatchSlot` | application_query -> live_delivery | 1 | movable |  |
| `application_query::managed_index::realized_footprint::CheckedManagedFootprintCapture` | application_query -> access_strategy | 1 | movable |  |
| `application_query::managed_index::source::installation::ManagedSourceInstallation` | application_query -> root | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 4: `storage`, `storage`, `equal_cause`, `initialization_execution_denial` |
| `application_query::managed_index::source_rows::SourceImageView` | application_query -> access_strategy | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 563: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `application_query::observed_source::denial::WorthQuerySourceExpectationDenial` | application_query -> application_attempt | 1 | movable | carries 1 |
| `application_query::observed_source::query_context_binding::ObservedQueryExpressionContext` | application_query -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `project`, `same_relation`, `same_relation_header`, `same_row` … |
| `application_query::observed_source::result_set::WorthQueryObservedResultSet` | application_query -> application_attempt | 1 | movable | carries 4 |
| `application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisReleaseOutcome` | application_query -> application_attempt | 1 | movable |  |
| `application_query::resource_lifecycle::basis_registry::lease::WorthQueryApplicationBasisLease` | application_query -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1: `release` |
| `application_query::retained_read::WorthQueryApplicationReadObservation` | application_query -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 541: `with_checkpoint`, `with_resources`, `CHECKSUM_BYTES`, `HEADER_BYTES` … |
| `application_resource::host::ApplicationResourceHost` | application_resource -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 4: `denial`, `CHUNK_BYTES`, `lock`, `lock` |
| `application_resource::provider::contract::WorthQueryApplicationResourceProvider` | application_resource -> application_contribution | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `CHUNK_BYTES`, `lock` |
| `application_resource::request::RequestOps` | application_resource -> application_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 16: `prefer_inner`, `with_existing`, `prepare`, `denial` … |
| `application_resource::request::WorthQueryApplicationResourcesRequest` | application_resource -> application_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 16: `prefer_inner`, `with_existing`, `prepare`, `denial` … |
| `application_runtime::operation_control::attempt_registration::source_publication::receipt::WorthQuerySourcePublicationHeadForTest` | application_runtime -> output_lineage | 1 | movable |  |
| `bootstrap::WorthQueryPrincipalBootstrapRow` | bootstrap -> bootstrap_publication | 1 | movable |  |
| `causal_inspection::application_inspection::request::RequestOps` | causal_inspection -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 16: `validate_original_affinity`, `prepare`, `prepare_query`, `prepare` … |
| `causal_inspection::application_inspection::request::WorthQueryApplicationInspectionRequest` | causal_inspection -> application_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 16: `validate_original_affinity`, `prepare`, `prepare_query`, `prepare` … |
| `causal_inspection::application_inspection::request::source::SourceOps` | causal_inspection -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 14: `validate_original_affinity`, `prepare`, `original_native`, `installed` … |
| `causal_inspection::application_inspection::request::source::WorthQuerySourceInspectionRequest` | causal_inspection -> application_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 14: `validate_original_affinity`, `prepare`, `original_native`, `installed` … |
| `causal_inspection::host::CausalInspectionHost` | causal_inspection -> application_runtime | 1 | movable |  |
| `conditional_operation::host::ConditionalOperationHost` | conditional_operation -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 4: `denial`, `CHUNK_BYTES`, `lock`, `lock` |
| `conditional_operation::inspection::InspectionOps` | conditional_operation -> application_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 5: `denial`, `CHUNK_BYTES`, `lock`, `lock` … |
| `conditional_operation::inspection::WorthQueryConditionalRuntimeInspection` | conditional_operation -> application_runtime | 1 | movable |  |
| `conditional_operation::installation::denial::WorthQueryConditionalRuntimeInstallationDenialKind` | conditional_operation -> application_contribution | 1 | movable |  |
| `conditional_operation::installation::pending_operation::WorthQueryPendingConditionalOperation` | conditional_operation -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 10: `denial`, `CHUNK_BYTES`, `lock`, `lock` … |
| `conditional_operation::lifecycle_inventory::WorthQueryConditionalRuntimeLifecycleProbe` | conditional_operation -> application_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `live_count` |
| `conditional_operation::predicate_admission::providers::QueryConditionalComputeContext` | conditional_operation -> application_contribution | 1 | movable |  |
| `conditional_operation::predicate_observation::QueryTemporalPredicateProvider` | conditional_operation -> product_operation | 1 | movable | carries 1 |
| `conditional_operation::reinstallation::ReinstallationOps` | conditional_operation -> application_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 7: `denial`, `CHUNK_BYTES`, `lock`, `lock` … |
| `conditional_operation::reinstallation::WorthQueryConditionalRuntimeReinstallationReceipt` | conditional_operation -> application_runtime | 1 | movable |  |
| `freshness::WorthQueryPrincipalFreshnessEvidence` | freshness -> authenticated_principal | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1044: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `granular_invalidation::installation::WorthQueryGranularInvalidationInstallation` | granular_invalidation -> runtime_host | 1 | movable | carries 9 |
| `handler::host::HandlerHost` | handler -> application_attempt | 1 | movable | carries 2 |
| `handler::registry::RegistryScopedOps` | handler -> application_entry | 1 | movable | carries 5 |
| `handler::registry::operation_body::OperationBodyScopedOps` | handler -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 243: `WIRE_VERSION`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `index_currency::WorthQueryPrimaryIndexCurrencyDenial` | index_currency -> resolution_denial | 1 | movable |  |
| `invariant_installation::schema_resolver::WorthQueryApplicationInvariantSchemaResolver` | invariant_installation -> application_contribution | 1 | movable |  |
| `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` | invariant_projection -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 231: `WIRE_VERSION`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `invariant_projection::WorthQueryInvariantMutationTarget` | invariant_projection -> application_attempt | 1 | movable |  |
| `invariant_projection::consumed_output::pending_dependency::SelectedPendingConsumedOutput` | invariant_projection -> output_lineage | 1 | movable | carries 3 |
| `invariant_projection::consumed_output::publication_recovery::PublicationRecoveryStop` | invariant_projection -> output_lineage | 1 | movable | carries 2 |
| `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionSnapshot` | invariant_projection -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 57: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `invariant_projection::operation_reader::borrowed::BorrowedOperationProjection` | invariant_projection -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 57: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `invariant_projection::operation_reader::output_availability::denial::WorthQueryOutputAvailabilityDenial` | invariant_projection -> application_query | 1 | behaviour-coupled | 4 impl(s) need items above; blocked by 9: `bridge_denial_kind`, `mismatch_or`, `canonical_wire_maximum`, `is_temporarily_deferred` … |
| `invariant_projection::realized_scope::WorthQueryRealizedProjectionScope` | invariant_projection -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 10: `shape_bound`, `build`, `insert`, `leaf` … |
| `live_delivery::source::WorthQueryLiveAdmissionDenial` | live_delivery -> application_attempt | 1 | movable |  |
| `live_delivery::source::custody::WorthQueryLivePublicationReservation` | live_delivery -> application_attempt | 1 | movable | carries 9 |
| `managed_bridge::WorthQueryInstalledApplicationBridge` | managed_bridge -> runtime_host | 1 | movable |  |
| `operation_body::assessment::WorthQueryCurrentBodyRequirementAssessment` | operation_body -> invariant_projection | 1 | movable | carries 3 |
| `operation_body::handler::OperationBodyHandler` | operation_body -> application_contribution | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 117: `native_decision_cause`, `foreign_source`, `installation_denial`, `admit_unavailable_completion` … |
| `operation_body::retirement::PreparedRetirementBatch` | operation_body -> invariant_projection | 1 | movable |  |
| `operation_body::selection::PreparedBodySelection` | operation_body -> invariant_projection | 1 | movable |  |
| `operation_body::value::PreparedBodyValues` | operation_body -> application_attempt | 1 | movable | carries 1 |
| `output_lineage::computation_source::ComputationSourceEvidence` | output_lineage -> application_checkpoint | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 564: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `output_lineage::family_selection::checkpoint_priors::NativePriorCheckpointOutput` | output_lineage -> application_checkpoint | 1 | movable | carries 6 |
| `output_lineage::input_cutoff::stable_cutover::PublishedStableLineage` | output_lineage -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 673: `insert`, `lookup`, `with_checkpoint`, `with_resources` … |
| `output_lineage::invalidation::commit_touches::CommitTouchEntities` | output_lineage -> application_discovery | 1 | movable |  |
| `output_lineage::invalidation::commit_touches::CommitTouchInterest` | output_lineage -> application_discovery | 1 | movable | carries 1 |
| `output_lineage::invalidation::derived::SourceSettlementCurrentness` | output_lineage -> fact_verification_observation | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `output_lineage::invalidation::output_availability::publication_fence::SourcePublicationReadFence` | output_lineage -> provider | 1 | movable | carries 9 |
| `output_lineage::invalidation::query_coverage::QueryCoverageStop` | output_lineage -> application_query | 1 | movable | carries 3 |
| `output_lineage::invalidation::query_coverage::QueryFactCoverage` | output_lineage -> application_query | 1 | movable | carries 5 |
| `output_lineage::invalidation::settlement::posting_continuation::PostingRebaseBasis` | output_lineage -> provider | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 8: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `output_lineage::prepared_slot::recovery::PreparedLineageRecoveryMetadata` | output_lineage -> provider | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 119: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `output_lineage::recorded_source_identity::RecordedSourceIdentity` | output_lineage -> application_attempt | 1 | movable | carries 2 |
| `product_activation::host::ProductActivationHost` | product_activation -> application_attempt | 1 | movable |  |
| `product_activation::inspection::InspectionScopedOps` | product_activation -> application_query | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `inspect_selected_program` |
| `product_activation::inspection::WorthQuerySelectedProgramInspectionDenial` | product_activation -> application_discovery | 1 | movable |  |
| `product_operation::branches::BranchesOps` | product_operation -> application_contribution | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `check_live` |
| `product_operation::branches::owned_head::WorthQueryOwnedProductHead` | product_operation -> application_query | 1 | movable | carries 13 |
| `product_operation::conditional_definition::WorthQueryApplicationConditionalDefinitionAdmissionDenial` | product_operation -> handler | 1 | movable |  |
| `product_operation::conditional_definition::admission::AdmissionScopedOps` | product_operation -> handler | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 108: `foreign_source`, `installation_denial`, `admit_unavailable_completion`, `succession_accepts` … |
| `product_operation::context::ContextScopedOps` | product_operation -> application_query | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 303: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `product_operation::context::WorthQueryProductEntry` | product_operation -> application_output_demand | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 13: `denial`, `denial`, `CHUNK_BYTES`, `lock` … |
| `product_operation::correction_source::authored_identity::candidate::AuthoredCorrectionCandidateDenial` | product_operation -> application_attempt | 1 | movable |  |
| `product_operation::document_history::denial::WorthQueryDocumentHistoryAttachmentFailure` | product_operation -> application_query | 1 | movable | carries 15 |
| `product_operation::document_history::navigation::WorthQueryHistoryNavigationEligibilityDenial` | product_operation -> application_query | 1 | movable |  |
| `product_operation::document_history::navigation::output_authority::WorthQueryHistoryOutputAuthority` | product_operation -> application_query | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `confirm_in_execution`, `confirm_in_native` |
| `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryBorrowedReadStop` | product_operation -> application_query | 1 | movable | carries 5 |
| `product_operation::document_history::navigation::output_completion::WorthQueryCompletedHistoryOutputs` | product_operation -> application_query | 1 | movable | carries 5 |
| `product_operation::document_history::navigation::output_lookup::HistoryOutputLookup` | product_operation -> application_query | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::navigation::output_lookup::HistoryOutputLookupPhase` | product_operation -> application_query | 1 | movable |  |
| `product_operation::document_history::navigation::output_profile::WorthQueryHistoryOutputNavigationAdvance` | product_operation -> application_query | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 4: `confirm_in_execution`, `confirm_in_native`, `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::navigation::output_profile::WorthQueryHistoryOutputProfile` | product_operation -> application_query | 1 | movable | carries 2 |
| `product_operation::document_history::paging::WorthQueryHistoryPage` | product_operation -> application_query | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::redo::WorthQueryHistoryRedoChoices` | product_operation -> application_query | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::retention::HistoryRetentionChange` | product_operation -> application_query | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::retention::WorthQueryHistoryRetentionStatus` | product_operation -> application_query | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 2: `NEXT_DOCUMENT`, `mint_document_ordinal` |
| `product_operation::document_history::selection::HistorySelectionCell` | product_operation -> application_query | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 12: `prepare`, `reserve_arc`, `reserve_buffer`, `checkpoint` … |
| `product_operation::host::ProductOperationHost` | product_operation -> application_query | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 303: `dispatch`, `denial`, `field_denial`, `dependency_is_declared` … |
| `product_operation::program_adoption::authorization::ProgramAdoptionAdmission` | product_operation -> program_occurrence | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1141: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `product_operation::program_adoption::preparation::migration::WorthQueryProgramMigrationPreparationDenial` | product_operation -> application_attempt | 1 | movable |  |
| `product_operation::query::WorthQueryProductQueryControls` | product_operation -> application_query | 1 | movable |  |
| `product_operation::query::history_outputs::WorthQueryHistoryOutputControlDenial` | product_operation -> application_query | 1 | movable |  |
| `product_operation::security_basis::WorthQuerySelectedPermissionSecurityBasis` | product_operation -> application_query | 1 | movable | carries 2 |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::FactlessCurrentness` | provider -> application_attempt | 1 | movable |  |
| `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::diagnostic::RebaseVerificationReason` | provider -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `provider::application_decision_fact::application_facts::CompletedApplicationFacts` | provider -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 15: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `provider::application_decision_fact::application_table::ApplicationFactTable` | provider -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 14: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `provider::committed_dispatch_outbox::CommittedDispatchOutboxScopedOps` | provider -> application_inbound | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1006: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `provider::committed_dispatch_outbox::ProviderCommittedDispatchOutboxScopedOps` | provider -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 253: `WIRE_VERSION`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `provider::idempotency::WorthQueryProviderGuardedWorkflowOperationCustody` | provider -> application_attempt | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 12: `array_storage_bytes`, `bytes`, `deep_clone_bytes`, `join` … |
| `provider::inbound_completion::read::WorthQueryCanonicalCompletionRow` | provider -> application_inbound | 1 | movable | carries 1 |
| `provider::inbound_completion::read::WorthQueryInboundCompletionReadDenial` | provider -> application_inbound | 1 | movable |  |
| `provider::inbound_terminal_index::WorthQueryCanonicalInboundCompletion` | provider -> application_inbound | 1 | movable | carries 2 |
| `provider::outstanding_dispatch::OutstandingDispatchReservation` | provider -> application_attempt | 1 | movable | carries 1 |
| `provider::outstanding_dispatch::in_flight::WorthQueryOutstandingInFlightDenial` | provider -> application_attempt | 1 | movable |  |
| `provider::pending_application_publication::registry::WorthQueryApplicationPublicationRecoveryReservation` | provider -> application_attempt | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 178: `WIRE_VERSION`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxBindingDenial` | provider -> application_attempt | 1 | movable |  |
| `provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxReceiptSeal` | provider -> application_attempt | 1 | movable | carries 2 |
| `runtime_host::WorthQueryRuntimeIdentityHost` | runtime_host -> record_reservations | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1143: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `source_read_verification::SourceReadVerification` | source_read_verification -> component_selection | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1: `equal_cause` |

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
| `output_reuse::selection::read_denial` | 4 | 117 |
| `output_reuse::selection::retained_fact_denial` | 4 | 117 |
| `product_activation::inspection::inspect_selected_program` | 4 | 46 |
| `provider::relational_execution_denial::relational_execution_kind` | 3 | 25 |
| `application_attempt::observation::observe_field_value` | 2 | 43 |
| `application_attempt::workflow::instance::progression::retention::SHARD_COUNT` | 2 | 2 |
| `application_checkpoint::facts::WIRE_VERSION` | 2 | 3 |
| `application_checkpoint::facts::payload::chunk::deep_clone_bytes` | 2 | 32 |
| `application_discovery::workflow::history_view::text` | 2 | 2 |
| `causal_inspection::application_inspection::admission::preparation::prepare` | 2 | 2 |
| `causal_inspection::application_inspection::request::binding::installed` | 2 | 2 |
| `causal_inspection::application_inspection::request::funding::open` | 2 | 2 |
| `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` | 2 | 31 |
| `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` | 2 | 108 |
| `invariant_projection::consumed_output::verification::work_budget::reserve_pending` | 2 | 31 |
| `output_lineage::native_output_witness::overflow` | 2 | 4 |
| `output_lineage::prepared_slot::capacity::denial` | 2 | 6 |
| `output_reuse::selection::preflight_stop` | 2 | 31 |
| `product_operation::conditional_definition::admission::definition_request` | 2 | 80 |
| `provider::application_decision_fact::binding_failure` | 2 | 2 |
| `access_strategy::realized_footprint::comparison::with_checkpoint` | 1 | 1 |
| `access_strategy::realized_footprint::comparison::with_resources` | 1 | 1 |
| `application_attempt::compare_and_commit::commit_outcome::denial::execution::allocation_control` | 1 | 2 |
| `application_attempt::compare_and_commit::commit_outcome::denial::execution::authority_cause::authority_execution` | 1 | 2 |
| `application_attempt::compare_and_commit::commit_outcome::denial::execution::control_kind` | 1 | 2 |
| `application_attempt::compare_and_commit::commit_outcome::denial::execution::execution_kind` | 1 | 2 |
| `application_attempt::denial::request_read::relational_read_kind` | 1 | 3 |
| `application_attempt::fact::adjacency::observe_adjacency` | 1 | 79 |
| `application_attempt::fact::adjacency::observe_adjacency_with_work` | 1 | 4 |
| `application_attempt::fact::adjacency::remains_equal` | 1 | 4 |

### What holds the behaviour-coupled units (transitively)

Each non-type item above a destination that some coupled unit (directly or through a
coupled type it needs) depends on; inverting the top rows frees the most units.

| item | units held | their references |
|---|---|---|
| `application_query::managed_index::provider_failure::equal_cause` | 78 | 459 |
| `application_contribution::producer::denial_value::mismatch_or` | 70 | 376 |
| `application_contribution::conditional::output_readiness_contract::denial` | 69 | 338 |
| `product_operation::document_history::step::NEXT_DOCUMENT` | 69 | 320 |
| `product_operation::document_history::step::mint_document_ordinal` | 69 | 320 |
| `application_resource::provider::ingress::backing::CHUNK_BYTES` | 67 | 309 |
| `application_resource::provider::ingress::backing::lock` | 67 | 309 |
| `product_operation::document_history::current_head::confirm_in_execution` | 67 | 350 |
| `product_operation::document_history::current_head::confirm_in_native` | 67 | 350 |
| `application_contribution::conditional::output_readiness_contract::typed::field_denial` | 66 | 332 |
| `application_contribution::native_field_contract::field_denial` | 66 | 332 |
| `application_resource::installation::denial` | 66 | 308 |
| `application_resource::storage::lock` | 66 | 308 |
| `application_runtime::operation_control::attempt_registration::arm` | 65 | 323 |
| `application_runtime::operation_control::attempt_registration::clear` | 65 | 323 |
| `application_runtime::operation_control::attempt_registration::reach` | 65 | 323 |
| `application_runtime::operation_control::attempt_registration::source_publication::release` | 65 | 323 |
| `application_runtime::operation_control::attempt_registration::source_publication::wait` | 65 | 323 |
| `application_contribution::producer::denial_value::declaration::encode_declared` | 64 | 326 |
| `application_contribution::producer::registry::semantic_edition::posture_code` | 64 | 326 |

### Functions, constants and modules referenced against the stack

| item | kind | owner | refs | from |
|---|---|---|---|---|
| `provider::relational_execution_denial::relational_execution_kind` | fn | provider | 7 | application_attempt 4, application_inbound 1, resolution 1, resolution_denial 1 |
| `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` | fn | invariant_projection | 5 | application_query 1, output_lineage 1, provider 2, source_facts 1 |
| `application_contribution::execution_denial::publication::publication_execution_kind` | fn | application_contribution | 3 | application_query 1, entity_resolution 1, provider 1 |
| `conditional_operation::signal_decision_reentry::classify_bridge_signal` | fn | conditional_operation | 3 | application_contribution 2, application_output_demand 1 |
| `product_operation::document_history::current_head::confirm` | fn | product_operation | 3 | application_query 3 |
| `application_checkpoint::facts` | module | application_checkpoint | 2 | application_attempt 2 |
| `application_contribution::execution_denial::advancement_scope_stop` | fn | application_contribution | 2 | invariant_projection 1, output_lineage 1 |
| `application_contribution::execution_denial::scope::scope_stop_kind` | fn | application_contribution | 2 | application_query 1, entity_resolution 1 |
| `application_installation::program::workflow_runtime::authentication::workflow_approval_authentication_intent` | fn | application_installation | 2 | application_attempt 2 |
| `operation_body::requirement::value::field_value` | fn | operation_body | 2 | invariant_projection 2 |
| `output_lineage::prepared_slot::preparation::prepare` | fn | output_lineage | 2 | provider 2 |
| `output_reuse::selection::first_moved_fact_in` | fn | output_reuse | 2 | application_query 2 |
| `output_reuse::selection::retained_fact_denial` | fn | output_reuse | 2 | application_query 2 |
| `provider::relational_execution_denial::native_preparation_kind` | fn | provider | 2 | application_attempt 2 |
| `provider::relational_execution_denial::query_read_execution_kind` | fn | provider | 2 | application_attempt 1, resolution_denial 1 |
| `application_attempt::fact::consult_observation` | module | application_attempt | 1 | freshness 1 |
| `application_attempt::observation::observe_field_value` | fn | application_attempt | 1 | aggregate_projection 1 |
| `application_attempt::workflow::instance::progression::retention::default_progress_retention_shards` | fn | application_attempt | 1 | root 1 |
| `application_attempt::workflow::schema::indexes::register_indexes` | fn | application_attempt | 1 | root 1 |
| `application_attempt::workflow::schema::lower_workflow` | fn | application_attempt | 1 | schema_layout 1 |
| `application_checkpoint::facts::WIRE_VERSION` | const | application_checkpoint | 1 | application_attempt 1 |
| `application_contribution::execution_denial` | module | application_contribution | 1 | application_attempt 1 |
| `application_contribution::execution_denial::advancement_checkpoint_stop` | fn | application_contribution | 1 | application_query 1 |
| `application_contribution::partitioned_computation::incremental::published_state::observe_published` | fn | application_contribution | 1 | output_lineage 1 |
| `application_contribution::producer::demand::bridge_denial::bridge_denial_kind` | fn | application_contribution | 1 | application_output_demand 1 |
| `application_discovery::workflow::current_definition` | fn | application_discovery | 1 | application_attempt 1 |
| `application_entry::mutation::execution::candidate::build_handler_candidate` | fn | application_entry | 1 | application_attempt 1 |
| `application_invariant::refusal` | module | application_invariant | 1 | application_attempt 1 |
| `application_query::admission::funded_parameters::capture_parameters` | fn | application_query | 1 | source_disclosure 1 |
| `application_query::derived_view::change_summary::changes_from_summary` | fn | application_query | 1 | provider 1 |
| `conditional_operation::predicate_admission::output_readiness_installation::prepare_dependency_conditional_installation` | fn | conditional_operation | 1 | application_contribution 1 |
| `conditional_operation::publication::install_pending_bindings` | fn | conditional_operation | 1 | application_runtime 1 |
| `conditional_operation::publication::publication_denial` | fn | conditional_operation | 1 | application_runtime 1 |
| `conditional_operation::publication::require_complete_binding_inventory` | fn | conditional_operation | 1 | application_runtime 1 |
| `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` | fn | invariant_projection | 1 | application_query 1 |
| `invariant_projection::consumed_output::verification::work_budget::reserve_pending` | fn | invariant_projection | 1 | output_lineage 1 |
| `operation_body::requirement::evaluation::evaluate_requirements` | fn | operation_body | 1 | invariant_projection 1 |
| `output_lineage::input_cutoff::verification::cutoff_declines` | fn | output_lineage | 1 | output_reuse 1 |
| `output_lineage::invalidation::publication::register_completed` | fn | output_lineage | 1 | provider 1 |
| `output_reuse::selection::preflight_stop` | fn | output_reuse | 1 | application_query 1 |
| `output_reuse::verification_denial` | module | output_reuse | 1 | application_query 1 |
| `product_operation::generated_materialization::restoration::invariant_admission::admit_required_invariants` | fn | product_operation | 1 | provider 1 |
| `provider::relational_execution_denial::preflight::preflight_execution_cause` | fn | provider | 1 | resolution 1 |

## Ruling sheet: heaviest mutual pairs

### `application_output_demand` ↔ `application_contribution`

- references: application_output_demand -> application_contribution 23, application_contribution -> application_output_demand 383
- against the stack (application_output_demand -> application_contribution): 23 references; 6 movable units, 5 behaviour-coupled, 0 pinned, 1 functions/constants/modules
- crossing up: `WorthQueryProducerCommitAuthority` (movable, 4), `WorthQueryProducerDenialValue` (behaviour-coupled, 3), `ProducerOutputEvaluation` (behaviour-coupled, 3), `SealedComputationRetention` (behaviour-coupled, 2), `WorthQueryProducerApplicability` (movable, 2), `CurrentProof` (movable, 2), `CurrentBasis` (movable, 2), `ComputationPrior` (behaviour-coupled, 1), `WorthQueryProducerDenialDecodeDenial` (movable, 1), `OwnerPlan` (movable, 1), `PendingProducerPublication` (behaviour-coupled, 1), `bridge_denial_kind` (fn, 1)
- crossing down (top): `WorthQueryOutputDemandDenial` 91, `WorthQueryOutputDemandDenialKind` 69, `SelectedReadyReadmission` 21, `WorthQueryOutputDemandRegistry` 14, `WorthQueryOutputDemandSettlement` 14, `WorthQueryOutputDemandRecoveryPosture` 12, `WorthQueryOutputDemandKey` 11, `DemandAdmissionKind` 10
- option nest: move 21122 lines of application_output_demand under application_contribution; 120 references from other modules below application_contribution would then point up
- option move units: 6 units by script (12 references)
- option contract trait: a contract owned by application_output_demand, implemented in application_contribution, 6 items (11 references)
- **default**: move the 6 movable units by script; a contract owned by application_output_demand for the 6 items that stay behaviour-bound in application_contribution

### `application_query` ↔ `product_operation`

- references: application_query -> product_operation 132, product_operation -> application_query 100
- against the stack (application_query -> product_operation): 132 references; 20 movable units, 27 behaviour-coupled, 0 pinned, 1 functions/constants/modules
- crossing up: `WorthQuerySelectedProductOperation` (behaviour-coupled, 25), `WorthQueryDocumentHistory` (behaviour-coupled, 11), `WorthQueryDocumentHistoryDenial` (movable, 10), `WorthQueryHistoryNavigationDenial` (movable, 6), `ApplicationBasisScopedOps` (behaviour-coupled, 4), `WorthQueryHistoryOutputReadDenial` (behaviour-coupled, 4), `SelectedQueryBasisRetentionStop` (movable, 4), `HistoryModelIdentity` (movable, 3), `HistoryProducerInvocation` (behaviour-coupled, 3), `WorthQueryHistorySelection` (behaviour-coupled, 3), `WorthQueryHistoryPageDenial` (movable, 3), `SelectedPermissionSecurityStop` (movable, 3), `SharedQueryBasisScopedOps` (behaviour-coupled, 3), `SharedSelectedProductOperation` (movable, 3) …
- crossing down (top): `WorthQueryApplicationReadObservation` 18, `WorthQueryApplicationQueryAccessContext` 7, `WorthQueryApplicationQueryAdmissionDenial` 7, `WorthQueryApplicationQueryControls` 6, `WorthQueryApplicationBasisLease` 6, `AdmissionScopedOps` 6, `WorthQueryApplicationQueryAdmissionDenialKind` 4, `CompletedMetadataRead` 4
- option nest: move 56615 lines of application_query under product_operation; 344 references from other modules below product_operation would then point up
- option move units: 20 units by script (47 references)
- option contract trait: a contract owned by application_query, implemented in product_operation, 28 items (85 references)
- **default**: move the 20 movable units by script; a contract owned by application_query for the 28 items that stay behaviour-bound in product_operation

### `application_attempt` ↔ `provider`

- references: application_attempt -> provider 63, provider -> application_attempt 149
- against the stack (application_attempt -> provider): 63 references; 14 movable units, 12 behaviour-coupled, 0 pinned, 3 functions/constants/modules
- crossing up: `WorthQueryPrimaryGraphProvider` (behaviour-coupled, 10), `WorthQueryPrimaryGraphCommittedApplication` (behaviour-coupled, 5), `WorthQueryCommittedDispatchOutboxObservation` (movable, 4), `WorthQueryProviderIdempotencyResolution` (behaviour-coupled, 4), `WorthQueryPrimaryMutationWorkEvidence` (movable, 3), `WorthQueryCommittedDispatchOutboxBinding` (movable, 3), `OwnEffectOnReads` (behaviour-coupled, 2), `PreparedRebaseDenial` (movable, 2), `PreparedSourceFactRebase` (behaviour-coupled, 2), `WorthQueryApplicationBranchCommitCoordination` (movable, 2), `WorthQueryDispatchOutboxBasis` (movable, 2), `WorthQueryProviderIdempotencyResolutionDenial` (movable, 2), `WorthQueryInboundTerminalIndexDenial` (movable, 2), `FactlessCurrentness` (movable, 1) …
- crossing down (top): `WorthQueryApplicationIdempotencyBinding` 21, `WorthQueryApplicationObservedFact` 16, `WorthQueryPrimaryGraphApplicationAttempt` 16, `StorageControl` 14, `WorthQueryApplicationCommitOutcomeIdentity` 9, `StoreDenial` 9, `WorthQueryCommittedProductPublication` 7, `AdmittedAdjacencyEndpoints` 4
- option nest: move 86802 lines of application_attempt under provider; 53 references from other modules below provider would then point up
- option move units: 14 units by script (26 references)
- option contract trait: a contract owned by application_attempt, implemented in provider, 15 items (37 references)
- **default**: nest application_attempt under provider (86802 lines); its other users then go through provider

### `application_query` ↔ `application_contribution`

- references: application_query -> application_contribution 3, application_contribution -> application_query 152
- against the stack (application_query -> application_contribution): 3 references; 0 movable units, 0 behaviour-coupled, 0 pinned, 3 functions/constants/modules
- crossing up: `advancement_checkpoint_stop` (fn, 1), `publication_execution_kind` (fn, 1), `scope_stop_kind` (fn, 1)
- crossing down (top): `WorthQueryApplicationProjection` 25, `WorthQueryObservedSource` 25, `WorthQueryApplicationOutputDemandSource` 16, `WorthQueryApplicationReadObservation` 10, `ApplicationQueryNativeAccess` 7, `WorthQueryApplicationBasisSelectionIdentity` 7, `WorthQueryObservedSourceEpoch` 5, `WorthQueryApplicationOneShotDenialKind` 4
- option nest: move 56615 lines of application_query under application_contribution; 185 references from other modules below application_contribution would then point up
- option move units: 0 units by script (0 references)
- option contract trait: a contract owned by application_query, implemented in application_contribution, 3 items (3 references)
- **default**: move the 0 movable units by script; a contract owned by application_query for the 3 items that stay behaviour-bound in application_contribution

### `application_attempt` ↔ `invariant_projection`

- references: application_attempt -> invariant_projection 37, invariant_projection -> application_attempt 111
- against the stack (application_attempt -> invariant_projection): 37 references; 7 movable units, 7 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `ConsumedOutputEvidence` (behaviour-coupled, 7), `ExpectedFactKeys` (behaviour-coupled, 6), `ConsumedOutputVerificationStop` (behaviour-coupled, 5), `WorthQueryOperationProjectionDenial` (movable, 4), `ObservedComputationInputs` (movable, 4), `CallShape` (movable, 2), `CallShapeTransition` (movable, 2), `WorthQueryApplicationInvariantProjectionAuthority` (behaviour-coupled, 1), `WorthQueryInvariantMutationTarget` (movable, 1), `VerificationCapacityStop` (movable, 1), `WorthQueryApplicationOperationInvariantProjectionSnapshot` (behaviour-coupled, 1), `BorrowedOperationProjection` (behaviour-coupled, 1), `ObservedOutputPosture` (movable, 1), `WorthQueryRealizedProjectionScope` (behaviour-coupled, 1)
- crossing down (top): `WorthQueryApplicationFactKey` 17, `WorthQueryApplicationAttemptDenial` 11, `WorthQueryApplicationAttemptDenialKind` 9, `WorthQueryApplicationObservedFact` 8, `WorthQueryApplicationOutputCorrespondence` 6, `RetainedSourceFacts` 5, `WorthQueryApplicationSnapshotLease` 5, `WorthQueryApplicationAdjacencyDirection` 4
- option nest: move 86802 lines of application_attempt under invariant_projection; 431 references from other modules below invariant_projection would then point up
- option move units: 7 units by script (15 references)
- option contract trait: a contract owned by application_attempt, implemented in invariant_projection, 7 items (22 references)
- **default**: move the 7 movable units by script; a contract owned by application_attempt for the 7 items that stay behaviour-bound in invariant_projection

### `output_lineage` ↔ `invariant_projection`

- references: output_lineage -> invariant_projection 45, invariant_projection -> output_lineage 98
- against the stack (output_lineage -> invariant_projection): 45 references; 4 movable units, 4 behaviour-coupled, 0 pinned, 2 functions/constants/modules
- crossing up: `ConsumedOutputEvidence` (behaviour-coupled, 21), `ConsumedOutputVerificationStop` (behaviour-coupled, 8), `ConsumedOutputVerification` (behaviour-coupled, 5), `EvidenceView` (movable, 4), `ObservedOutputCandidate` (behaviour-coupled, 2), `SelectedPendingConsumedOutput` (movable, 1), `PublicationRecoveryStop` (movable, 1), `ObservedOutputPosture` (movable, 1), `map_admission_stop` (fn, 1), `reserve_pending` (fn, 1)
- crossing down (top): `RecordedSettlementIdentity` 15, `SourceInvalidationOwner` 13, `SealedNativeOutputWitness` 11, `FullVerificationReason` 8, `ComparableSourceFacts` 8, `SettlementRegistrationStop` 6, `SourceSettlementCurrentness` 5, `ConsumedOutputCurrentness` 3
- option nest: move 19006 lines of output_lineage under invariant_projection; 49 references from other modules below invariant_projection would then point up
- option move units: 4 units by script (7 references)
- option contract trait: a contract owned by output_lineage, implemented in invariant_projection, 6 items (38 references)
- **default**: move the 4 movable units by script; a contract owned by output_lineage for the 6 items that stay behaviour-bound in invariant_projection

### `application_attempt` ↔ `product_operation`

- references: application_attempt -> product_operation 39, product_operation -> application_attempt 75
- against the stack (application_attempt -> product_operation): 39 references; 5 movable units, 3 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQuerySelectedProductOperation` (behaviour-coupled, 18), `HistoryProducerInvocation` (behaviour-coupled, 9), `WorthQueryAdmittedApplicationConditionalDefinition` (movable, 7), `SelectedQueryOrigin` (movable, 1), `AuthoredCorrectionCandidateDenial` (movable, 1), `ObservationRoomScopedOps` (behaviour-coupled, 1), `WorthQueryProgramMigrationPreparationDenial` (movable, 1), `SharedSelectedProductOperation` (movable, 1)
- crossing down (top): `WorthQueryApplicationIdempotencyBinding` 16, `WorthQueryApplicationOutputCorrespondence` 8, `WorthQueryWorkflowDispositions` 5, `WorthQueryApplicationAttemptDenial` 4, `WorthQueryApplicationCommitReceipt` 4, `WorthQueryApplicationOutputProjectionDenial` 3, `WorthQueryApplicationObservedFact` 3, `OutputRoleUse` 3
- option nest: move 86802 lines of application_attempt under product_operation; 724 references from other modules below product_operation would then point up
- option move units: 5 units by script (11 references)
- option contract trait: a contract owned by application_attempt, implemented in product_operation, 3 items (28 references)
- **default**: move the 5 movable units by script; a contract owned by application_attempt for the 3 items that stay behaviour-bound in product_operation

### `output_lineage` ↔ `application_output_demand`

- references: output_lineage -> application_output_demand 48, application_output_demand -> output_lineage 60
- against the stack (output_lineage -> application_output_demand): 48 references; 4 movable units, 5 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryOutputDemandDenial` (behaviour-coupled, 14), `WorthQueryOutputDemandDenialKind` (behaviour-coupled, 11), `RequiredWorkMembership` (behaviour-coupled, 9), `WorthQueryAcceptedOutputCheckpointIdentity` (movable, 6), `ReadyCompletion` (behaviour-coupled, 3), `ReplacedRequiredWorkHint` (movable, 2), `WorthQueryAcceptedOutputAuthority` (movable, 1), `WorthQueryAcceptedOutputCheckpointPosture` (movable, 1), `WorthQueryOutputDemandSettlement` (behaviour-coupled, 1)
- crossing down (top): `RecordedSettlementIdentity` 22, `SourceInvalidationOwner` 11, `PublishedStableLineage` 7, `ComparableSourceFacts` 3, `RetainedOutputCurrentnessRead` 3, `SealedNativeOutputWitness` 3, `PreparedInputReuseKey` 2, `PriorComputationRecord` 2
- option nest: move 19006 lines of output_lineage under application_output_demand; 147 references from other modules below application_output_demand would then point up
- option move units: 4 units by script (10 references)
- option contract trait: a contract owned by output_lineage, implemented in application_output_demand, 5 items (38 references)
- **default**: move the 4 movable units by script; a contract owned by output_lineage for the 5 items that stay behaviour-bound in application_output_demand

### `application_attempt` ↔ `output_lineage`

- references: application_attempt -> output_lineage 13, output_lineage -> application_attempt 93
- against the stack (application_attempt -> output_lineage): 13 references; 2 movable units, 4 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `PostconditionCarry` (behaviour-coupled, 4), `ObservedSourcePublication` (behaviour-coupled, 3), `RecordedSettlementIdentity` (movable, 2), `ComparableSourceFacts` (behaviour-coupled, 2), `PublishedStableLineage` (behaviour-coupled, 1), `RecordedSourceIdentity` (movable, 1)
- crossing down (top): `WorthQueryApplicationObservedFact` 19, `WorthQueryApplicationOutputCorrespondence` 15, `StorageControl` 10, `CompletedDecisionReuseProof` 8, `CompletedHandlerFactBoundary` 7, `StoreDenial` 7, `WorthQuerySourceCurrentnessFailure` 6, `Movement` 5
- option nest: move 86802 lines of application_attempt under output_lineage; 338 references from other modules below output_lineage would then point up
- option move units: 2 units by script (3 references)
- option contract trait: a contract owned by application_attempt, implemented in output_lineage, 4 items (10 references)
- **default**: move the 2 movable units by script; a contract owned by application_attempt for the 4 items that stay behaviour-bound in output_lineage

### `application_attempt` ↔ `application_checkpoint`

- references: application_attempt -> application_checkpoint 53, application_checkpoint -> application_attempt 44
- against the stack (application_attempt -> application_checkpoint): 53 references; 3 movable units, 5 behaviour-coupled, 0 pinned, 2 functions/constants/modules
- crossing up: `FactPayload` (behaviour-coupled, 23), `FactSequence` (movable, 18), `NativeFactPayloadBuilder` (behaviour-coupled, 2), `CheckpointDecodeDenial` (behaviour-coupled, 2), `CheckpointMetadataFunding` (behaviour-coupled, 2), `FactDecodeDenial` (movable, 1), `CodecPayloadReservation` (movable, 1), `FactDecoder` (behaviour-coupled, 1), `facts` (module, 2), `WIRE_VERSION` (const, 1)
- crossing down (top): `StorageControl` 14, `WorthQueryApplicationObservedFact` 14, `StoreDenial` 14, `WorthQueryCheckpointOutputRole` 1, `AdmittedAdjacencyEndpoints` 1
- option nest: move 86802 lines of application_attempt under application_checkpoint; 202 references from other modules below application_checkpoint would then point up
- option move units: 3 units by script (20 references)
- option contract trait: a contract owned by application_attempt, implemented in application_checkpoint, 7 items (33 references)
- **default**: move the 3 movable units by script; a contract owned by application_attempt for the 7 items that stay behaviour-bound in application_checkpoint

## Estimate

- defaults for the pairs above: largest component 58, 798 references against the stack; 51 units by script, 69 items by hand, 1 nests
- defaults plus every movable unit moved: largest component 58, 530 references against the stack; 167 units by script, 233 items by hand
- and the five widest hub types (`WorthQueryProgramApplicationRuntime`, `WorthQueryPrimaryGraphBootstrap`, `WorthQuerySelectedProductOperation`, `WorthQueryPrimaryGraphProvider`, `WorthQueryPrimaryGraphIntegrationHandle`) behind contracts: largest component 57, 483 references against the stack

