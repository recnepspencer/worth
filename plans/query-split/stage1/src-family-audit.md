# In-source test family audit

Library: `workspaces/worth-query/crates/worth-query-execution/src/lib.rs`

| Decision | Families | Tests | Lines |
|---|---:|---:|---:|
| public-surface | 23 | 45 | 1260 |
| runtime-root-public | 4 | 11 | 205 |
| runtime-root-private | 107 | 1457 | 163557 |
| private-logic | 302 | 732 | 31680 |
| admitted | 12 | 21 | 2696 |
| support | 57 | 0 | 5213 |

| Family | Decision | Tests | Lines | Roots | Private | Hooks |
|---|---|---:|---:|---:|---:|---:|
| `domain_computation::primary_graph::tests` | runtime-root-private | 571 | 73873 | 6 | 77 | 58 |
| `domain_computation::managed_run::tests` | runtime-root-private | 184 | 20076 | 5 | 3 | 7 |
| `domain_computation::convergence_epoch::tests` | runtime-root-private | 69 | 7596 | 3 | 0 | 5 |
| `domain_computation::primary_graph::application_output_demand::registry::tests` | runtime-root-private | 105 | 7532 | 2 | 76 | 24 |
| `domain_computation::primary_graph::output_lineage::ownership_oracle` | runtime-root-private | 26 | 6658 | 2 | 6 | 3 |
| `domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests` | runtime-root-private | 47 | 5381 | 3 | 28 | 15 |
| `domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests` | runtime-root-private | 31 | 5120 | 3 | 53 | 15 |
| `domain_computation::primary_graph::output_lineage::tests` | runtime-root-private | 32 | 2906 | 2 | 40 | 14 |
| `domain_computation::primary_graph::application_query::continuation::tests` | runtime-root-private | 14 | 1333 | 1 | 0 | 8 |
| `domain_computation::primary_graph::workflow::definition::compilation::reconstruction::start_provenance_tests` | runtime-root-private | 5 | 1192 | 3 | 17 | 4 |
| `domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruct_parallel::read_ceiling_tests` | runtime-root-private | 16 | 1177 | 3 | 8 | 7 |
| `domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests` | runtime-root-private | 4 | 1131 | 2 | 16 | 1 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::tests` | runtime-root-private | 9 | 1039 | 1 | 16 | 6 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::stops::leased_requests` | runtime-root-private | 14 | 1037 | 5 | 10 | 2 |
| `domain_computation::primary_graph::invariant_projection::aggregate::tests` | runtime-root-private | 9 | 1034 | 4 | 1 | 8 |
| `domain_computation::primary_graph::provider::application_touch_admission::tests` | runtime-root-private | 11 | 936 | 3 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::retained_decision_facts::tests` | runtime-root-private | 14 | 899 | 1 | 8 | 3 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::tests` | runtime-root-private | 5 | 893 | 2 | 14 | 7 |
| `domain_computation::primary_graph::application_contribution::request_execution::advancement::tests` | runtime-root-private | 14 | 874 | 5 | 6 | 4 |
| `domain_computation::application_aftermath::aftermath_schema_fixture` | support | 0 | 859 | 1 | 0 | 0 |
| `domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::tests` | runtime-root-private | 15 | 856 | 2 | 8 | 1 |
| `domain_computation::primary_graph::application_checkpoint::tests` | private-logic | 23 | 779 | 0 | 22 | 1 |
| `domain_computation::primary_graph::application_contribution::computation::wave_dispatch::tests` | runtime-root-private | 11 | 770 | 1 | 7 | 1 |
| `domain_computation::primary_graph::output_lineage::native_output_witness::checkpoint::tests` | runtime-root-private | 2 | 761 | 3 | 7 | 3 |
| `domain_computation::primary_graph::application_query::live::lease::lifecycle::lifecycle_tests` | runtime-root-private | 6 | 680 | 1 | 0 | 9 |
| `domain_computation::primary_graph::application_contribution::producer::registry::tests` | runtime-root-private | 16 | 626 | 1 | 13 | 3 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progressed::association_tests` | runtime-root-private | 10 | 620 | 2 | 6 | 6 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::affinity_tests` | runtime-root-private | 6 | 612 | 1 | 8 | 5 |
| `domain_computation::primary_graph::application_attempt::workflow_instance_observation::history::operation_outputs::tests` | runtime-root-private | 3 | 611 | 3 | 11 | 3 |
| `domain_computation::operation_binding::authority::tests` | runtime-root-private | 5 | 582 | 1 | 5 | 8 |
| `domain_computation::primary_graph::provider::decision_facts::comparison::tests` | runtime-root-private | 2 | 576 | 2 | 8 | 5 |
| `domain_computation::primary_graph::application_contribution::conditional::output_readiness_contract::typed::tests` | runtime-root-private | 5 | 553 | 1 | 2 | 1 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::preimage_retention::selection::tests` | runtime-root-private | 11 | 544 | 1 | 7 | 4 |
| `domain_computation::primary_graph::application_query::projection::projected_tree::content_comparison::tests` | private-logic | 11 | 538 | 0 | 9 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::canonical_apply::tests` | private-logic | 7 | 536 | 0 | 4 | 3 |
| `domain_computation::primary_graph::application_contribution::contracts::tests` | runtime-root-private | 7 | 514 | 1 | 3 | 1 |
| `domain_computation::primary_graph::application_query::read_execution::dispatch_witness` | support | 0 | 493 | 1 | 0 | 0 |
| `domain_computation::primary_graph::application_query::live::lease::delivery_tests` | runtime-root-private | 3 | 491 | 1 | 0 | 4 |
| `domain_computation::provider_session::graph_provider::tests` | private-logic | 10 | 477 | 0 | 2 | 1 |
| `domain_computation::application_aftermath::external_effect::tests` | runtime-root-private | 11 | 476 | 1 | 5 | 5 |
| `domain_computation::primary_graph::provider::committed_dispatch_outbox::tests` | runtime-root-private | 7 | 463 | 1 | 7 | 5 |
| `domain_computation::primary_graph::application_installation::program::speculation::counted_read_stops` | runtime-root-private | 8 | 454 | 1 | 0 | 1 |
| `domain_computation::primary_graph::application_attempt::fact::cancellation_tests` | runtime-root-private | 5 | 431 | 4 | 7 | 2 |
| `domain_computation::primary_graph::application_query::managed_index::nodes::pages::tests` | admitted | 4 | 423 | 2 | 8 | 0 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::structural::tests` | runtime-root-private | 2 | 415 | 3 | 7 | 4 |
| `domain_computation::artifact_owner::authority_affinity_tests` | runtime-root-private | 1 | 397 | 2 | 1 | 0 |
| `domain_computation::primary_graph::invariant_projection::component::tests` | runtime-root-private | 3 | 397 | 1 | 0 | 4 |
| `domain_computation::primary_graph::application_query::governance_affinity_tests` | runtime-root-private | 2 | 386 | 1 | 0 | 6 |
| `domain_computation::primary_graph::product_operation::generated_materialization::restoration::republication::own_write` | private-logic | 1 | 386 | 0 | 5 | 1 |
| `domain_computation::primary_graph::application_attempt::read_set::source_facts::indexed_selection` | runtime-root-private | 2 | 382 | 1 | 2 | 2 |
| `domain_computation::primary_graph::application_attempt::dependency_identity::tests` | private-logic | 11 | 380 | 0 | 4 | 3 |
| `domain_computation::primary_graph::provider::committed_dispatch_outbox::corruption_tests` | runtime-root-private | 4 | 378 | 1 | 10 | 3 |
| `domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::safe_retry_affinity_tests` | private-logic | 5 | 377 | 0 | 0 | 6 |
| `test_read_stops` | support | 0 | 362 | 2 | 0 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_compare_denial::tests` | runtime-root-private | 7 | 361 | 1 | 5 | 2 |
| `domain_computation::primary_graph::application_query::read_execution::interruption::tests` | runtime-root-private | 5 | 358 | 1 | 2 | 4 |
| `domain_computation::primary_graph::output_lineage::own_write_fixture` | runtime-root-private | 2 | 355 | 1 | 11 | 8 |
| `domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests` | private-logic | 4 | 347 | 0 | 1 | 5 |
| `domain_computation::primary_graph::application_attempt::read_set::computation_routing::tests` | runtime-root-private | 7 | 346 | 2 | 9 | 2 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::admission::rich::tests` | admitted | 1 | 346 | 0 | 2 | 2 |
| `domain_computation::authorization::retained_capability_request::portable::tests` | runtime-root-private | 4 | 344 | 2 | 9 | 1 |
| `domain_computation::primary_graph::application_checkpoint::facts::payload::patch::tests` | private-logic | 4 | 344 | 0 | 9 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_binding::unique_value_tests` | private-logic | 13 | 343 | 0 | 10 | 1 |
| `domain_computation::primary_graph::application_query::managed_index::nodes::pages::edit::tests` | admitted | 4 | 338 | 2 | 2 | 0 |
| `domain_computation::primary_graph::workflow::instance::progression::retention::tests` | private-logic | 9 | 337 | 0 | 9 | 1 |
| `domain_computation::primary_graph::application_contribution::execution_denial::tests` | private-logic | 5 | 331 | 0 | 5 | 1 |
| `domain_computation::primary_graph::application_query::observed_source::source_identity::tests` | private-logic | 9 | 318 | 0 | 3 | 7 |
| `domain_computation::primary_graph::invariant_projection::walk::tests` | runtime-root-private | 3 | 316 | 2 | 0 | 2 |
| `domain_computation::primary_graph::application_contribution::computation::tests` | private-logic | 5 | 315 | 0 | 1 | 1 |
| `domain_computation::primary_graph::bootstrap_publication::execution_refusals` | runtime-root-private | 3 | 314 | 4 | 1 | 0 |
| `domain_computation::primary_graph::provider::invariant_execution_failure::tests` | private-logic | 6 | 314 | 0 | 6 | 1 |
| `domain_computation::primary_graph::output_lineage::input_cutoff::verification::tests` | private-logic | 2 | 306 | 0 | 24 | 5 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::touched_records::tests` | runtime-root-private | 1 | 298 | 1 | 2 | 3 |
| `domain_computation::primary_graph::schema_layout::aggregate_identity::tests` | private-logic | 3 | 295 | 0 | 2 | 0 |
| `domain_computation::primary_graph::schema_layout::registry_lowering::relation_integrity::tests` | private-logic | 9 | 289 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::independent_axis_tests` | private-logic | 1 | 287 | 0 | 12 | 3 |
| `domain_computation::primary_graph::application_contribution::producer::demand::scheduling_progression::tests` | runtime-root-private | 3 | 285 | 4 | 5 | 2 |
| `domain_computation::primary_graph::application_attempt::effect_program::outbox_persistence_tests` | private-logic | 3 | 283 | 0 | 6 | 1 |
| `domain_computation::primary_graph::application_contribution::producer::execution::outcome::tests` | runtime-root-private | 5 | 279 | 1 | 5 | 0 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::admission::publication_tests` | admitted | 1 | 278 | 2 | 8 | 2 |
| `domain_computation::primary_graph::output_lineage::invalidation::settlement::postings::tests` | private-logic | 6 | 277 | 0 | 13 | 0 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::stop::owner::capture_stage_tests` | private-logic | 0 | 275 | 0 | 9 | 0 |
| `domain_computation::primary_graph::application_checkpoint::encode::tests` | private-logic | 3 | 274 | 0 | 8 | 0 |
| `domain_computation::provider_session::tests` | runtime-root-private | 2 | 273 | 1 | 0 | 1 |
| `domain_computation::primary_graph::application_checkpoint::facts::tests` | private-logic | 7 | 267 | 0 | 7 | 1 |
| `domain_computation::primary_graph::application_query::projection::retained_projection::tests` | private-logic | 2 | 266 | 0 | 2 | 3 |
| `domain_computation::primary_graph::workflow::definition::compilation::reuse::tests` | private-logic | 6 | 262 | 0 | 14 | 0 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::stops::tests` | private-logic | 5 | 256 | 0 | 5 | 0 |
| `domain_computation::application_aftermath::recovery_progression::authority_tests` | private-logic | 8 | 255 | 0 | 6 | 7 |
| `domain_computation::primary_graph::application_attempt::read_set::structural_walk::tests` | private-logic | 3 | 252 | 0 | 6 | 4 |
| `domain_computation::primary_graph::application_invariant::typed_view::derived_relation::native_source_shapes` | runtime-root-private | 1 | 252 | 2 | 0 | 1 |
| `domain_computation::primary_graph::application_output_demand::settlement::observation::court` | support | 0 | 247 | 0 | 3 | 1 |
| `domain_computation::primary_graph::output_lineage::invalidation::fact_keys::tests` | private-logic | 5 | 245 | 0 | 6 | 0 |
| `domain_computation::primary_graph::application_attempt::read_set::aggregate::tests` | private-logic | 3 | 243 | 0 | 6 | 4 |
| `domain_computation::primary_graph::causal_inspection::application_observation::declared_query::tests` | private-logic | 2 | 243 | 0 | 0 | 3 |
| `domain_computation::primary_graph::handler::registry::computations::tests` | private-logic | 5 | 243 | 0 | 3 | 0 |
| `domain_computation::primary_graph::conditional_operation::lifecycle::tests` | runtime-root-private | 3 | 238 | 2 | 10 | 0 |
| `domain_computation::primary_graph::conditional_operation::application_operation_reentry::sealed_commit::denial::tests` | private-logic | 6 | 234 | 0 | 8 | 0 |
| `domain_computation::primary_graph::entity_resolution::tests` | private-logic | 4 | 232 | 0 | 1 | 6 |
| `domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::editing_tests` | private-logic | 2 | 227 | 0 | 7 | 2 |
| `domain_computation::primary_graph::application_query::derived_view::registry::managed_source::fork::tests` | admitted | 1 | 226 | 2 | 0 | 0 |
| `domain_computation::primary_graph::bootstrap_publication::commit_refusals` | runtime-root-private | 3 | 226 | 1 | 3 | 0 |
| `domain_computation::primary_graph::application_query::managed_index::source_structure::cold::tests` | runtime-root-private | 5 | 224 | 2 | 2 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::execution::denial::source_readmission::tests` | runtime-root-private | 6 | 221 | 1 | 13 | 0 |
| `domain_computation::primary_graph::application_attempt::fact::workflow_history_basis::tests` | runtime-root-private | 1 | 220 | 2 | 6 | 0 |
| `domain_computation::primary_graph::workflow::instance::progression::tests` | private-logic | 8 | 220 | 0 | 5 | 2 |
| `domain_computation::primary_graph::application_attempt::fact::workflow_instance_capacity::tests` | runtime-root-private | 1 | 218 | 2 | 5 | 0 |
| `domain_computation::primary_graph::application_attempt::dependency_identity::content_tree::oracle` | private-logic | 2 | 217 | 0 | 9 | 0 |
| `domain_computation::primary_graph::application_query::live::lease::reservation_race_tests` | private-logic | 2 | 217 | 0 | 0 | 5 |
| `domain_computation::primary_graph::application_attempt::elevation_lifecycle_facts::tests` | private-logic | 4 | 216 | 0 | 6 | 0 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::scope_replacement` | admitted | 1 | 214 | 0 | 0 | 3 |
| `domain_computation::primary_graph::invariant_projection::traversal::tests` | runtime-root-private | 3 | 213 | 2 | 0 | 3 |
| `domain_computation::primary_graph::application_contribution::request_execution::test_placement::certification_tests` | runtime-root-private | 5 | 207 | 1 | 1 | 2 |
| `domain_computation::primary_graph::application_contribution::computation::worker_custody::tests` | runtime-root-private | 2 | 206 | 1 | 5 | 2 |
| `domain_computation::primary_graph::application_query::managed_index::source_structure::ancestry::tests` | admitted | 3 | 205 | 2 | 2 | 0 |
| `domain_computation::primary_graph::workflow::instance::progression::scaling` | private-logic | 4 | 205 | 0 | 4 | 1 |
| `domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::redispatch_denial_tests` | public-surface | 3 | 204 | 0 | 0 | 0 |
| `domain_computation::evidence_identity::tests` | private-logic | 10 | 202 | 0 | 0 | 1 |
| `domain_computation::primary_graph::application_invariant::typed_view::derived_relation::native_audience` | runtime-root-private | 1 | 201 | 2 | 0 | 1 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::stop::projection_tests` | private-logic | 9 | 200 | 0 | 6 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::certification::acknowledgement_basis` | private-logic | 2 | 193 | 0 | 6 | 1 |
| `domain_computation::primary_graph::workflow::adoption::legality::tests` | private-logic | 9 | 192 | 0 | 10 | 1 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::comparison_scope_tests` | runtime-root-private | 1 | 191 | 1 | 7 | 4 |
| `domain_computation::primary_graph::application_attempt::read_set::completed_facts::tests` | private-logic | 3 | 189 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::denial_value::tests` | public-surface | 7 | 189 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_query::live::lease::sibling_delivery_tests` | runtime-root-private | 1 | 189 | 1 | 0 | 5 |
| `domain_computation::primary_graph::product_operation::correction_source::authored_identity::candidate_tests` | private-logic | 3 | 188 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::own_effect_assertions` | support | 0 | 187 | 2 | 5 | 0 |
| `domain_computation::primary_graph::application_checkpoint::facts::payload::tests` | private-logic | 4 | 185 | 0 | 6 | 1 |
| `domain_computation::primary_graph::application_checkpoint::facts::payload::concatenation::tests` | private-logic | 4 | 180 | 0 | 5 | 0 |
| `domain_computation::primary_graph::application_query::derived_view::publication::tests` | private-logic | 4 | 180 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_query::managed_index::predicate::tree::tests` | private-logic | 3 | 180 | 0 | 3 | 0 |
| `domain_computation::managed_run::workflow_yield::corruption_tests` | private-logic | 2 | 177 | 0 | 0 | 2 |
| `domain_computation::primary_graph::application_query::derived_view::change_summary::tests` | private-logic | 4 | 175 | 0 | 4 | 0 |
| `domain_computation::managed_run::step_contract_admission::tests` | private-logic | 1 | 172 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::read_set::source_facts::merging` | private-logic | 6 | 172 | 0 | 4 | 0 |
| `domain_computation::primary_graph::application_query::observed_source::fact_conversion::tests` | runtime-root-private | 3 | 172 | 2 | 4 | 0 |
| `domain_computation::primary_graph::provider::session_commit::outbox_commit::tests` | runtime-root-private | 2 | 172 | 1 | 7 | 1 |
| `domain_computation::primary_graph::application_query::managed_index::invocation::tests` | runtime-root-private | 2 | 169 | 2 | 4 | 0 |
| `domain_computation::primary_graph::output_lineage::invalidation::query_coverage::tests` | private-logic | 1 | 168 | 0 | 5 | 3 |
| `domain_computation::execution_runtime::product_world::owner_cleanup::registry::retirement_tests` | admitted | 2 | 167 | 3 | 6 | 2 |
| `domain_computation::primary_graph::invariant_installation::schema_resolver::tests` | admitted | 1 | 164 | 1 | 2 | 0 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::stop::owner::tests` | private-logic | 11 | 161 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::tests` | private-logic | 3 | 160 | 0 | 0 | 4 |
| `domain_computation::primary_graph::workflow::definition::compilation::plan::resume::tests` | private-logic | 1 | 160 | 0 | 13 | 2 |
| `domain_computation::primary_graph::application_query::observed_source::source_identity::membership::tests` | private-logic | 5 | 157 | 0 | 8 | 2 |
| `process_deadline` | support | 0 | 157 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::lifecycle::closed_retirement::walk_bounds` | private-logic | 7 | 155 | 0 | 6 | 1 |
| `domain_computation::primary_graph::application_attempt::idempotency::tests` | private-logic | 6 | 154 | 0 | 3 | 1 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::stop::owner::stage_tests` | private-logic | 2 | 154 | 0 | 8 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_binding::effect_accumulator::compound_patch_tests` | private-logic | 2 | 152 | 0 | 2 | 0 |
| `domain_computation::primary_graph::conditional_operation::authorization_sources::tests` | private-logic | 2 | 152 | 0 | 1 | 3 |
| `domain_computation::primary_graph::output_lineage::retained_computation::tests` | runtime-root-private | 1 | 152 | 1 | 7 | 7 |
| `domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruct_parallel::denial_tests` | private-logic | 2 | 151 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::execution::source_readmission::ready::tests` | private-logic | 3 | 150 | 0 | 4 | 0 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::admission::materialization::tests` | admitted | 1 | 150 | 1 | 5 | 1 |
| `domain_computation::primary_graph::provider::aftermath_causality::correction_history::page_allocation::tests` | private-logic | 2 | 150 | 0 | 2 | 2 |
| `domain_computation::primary_graph::application_attempt::workflow_transition_program::operation::tests` | private-logic | 1 | 148 | 0 | 4 | 2 |
| `domain_computation::authorization::capability_observation::grant_selection::denial_tests` | private-logic | 2 | 146 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_invariant::typed_view::derived_relation::native_snapshot` | runtime-root-private | 1 | 146 | 2 | 0 | 1 |
| `domain_computation::primary_graph::workflow::definition::compilation::publication_binding::tests` | private-logic | 3 | 146 | 0 | 9 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::reinspection::tests` | private-logic | 4 | 145 | 0 | 7 | 0 |
| `domain_computation::primary_graph::schema_layout::derived_relation::tests` | private-logic | 1 | 145 | 0 | 0 | 1 |
| `domain_computation::primary_graph::workflow::assessment::tests` | private-logic | 2 | 143 | 0 | 7 | 1 |
| `domain_computation::authorization::retained_capability_request::material::tests` | private-logic | 3 | 142 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_attempt::dependency_identity::content_tree::postcondition::tests` | private-logic | 2 | 142 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::idempotency::recorded_intent_tests` | private-logic | 6 | 141 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_query::basis::truth_view_admission` | support | 0 | 140 | 2 | 0 | 3 |
| `domain_computation::primary_graph::provider::committed_dispatch_outbox::owner_test_support` | support | 0 | 140 | 2 | 6 | 3 |
| `domain_computation::primary_graph::workflow::proposal::identity::tests` | private-logic | 3 | 139 | 0 | 1 | 1 |
| `domain_computation::execution_runtime::tests` | runtime-root-public | 8 | 137 | 1 | 0 | 0 |
| `domain_computation::primary_graph::product_operation::correction_source::restored_support::eligibility::retirement` | runtime-root-private | 1 | 137 | 2 | 5 | 3 |
| `domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::denial::tests` | private-logic | 3 | 135 | 0 | 5 | 0 |
| `domain_computation::primary_graph::application_query::one_shot::denial::capacity_tests` | runtime-root-private | 2 | 134 | 2 | 4 | 0 |
| `domain_computation::primary_graph::application_query::read_execution::tree_materialization::source_footprint::tests` | runtime-root-private | 1 | 133 | 2 | 6 | 0 |
| `domain_computation::primary_graph::invariant_projection::locked_reader::record_reads::tests` | runtime-root-private | 2 | 131 | 2 | 0 | 3 |
| `domain_computation::primary_graph::provider::invariant_execution::receipt_closure::tests` | private-logic | 5 | 131 | 0 | 4 | 0 |
| `domain_computation::application_aftermath::recovery_progression::binding_axis::tests` | private-logic | 8 | 129 | 0 | 0 | 6 |
| `domain_computation::primary_graph::application_attempt::denial::native_query_cause` | runtime-root-private | 2 | 129 | 1 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::principal_comparison_tests` | private-logic | 1 | 129 | 0 | 7 | 4 |
| `domain_computation::artifact_owner::native_access::projection_sink::tests` | public-surface | 3 | 126 | 0 | 0 | 0 |
| `domain_computation::managed_run::semantic_basis::tests` | private-logic | 4 | 126 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::drive::member_refusal::tests` | private-logic | 3 | 124 | 0 | 6 | 1 |
| `domain_computation::application_aftermath::recovery_handle::tests` | private-logic | 4 | 123 | 0 | 3 | 3 |
| `domain_computation::primary_graph::application_query::resource_lifecycle::basis_registry::maintenance_refusal_cleanup` | runtime-root-private | 3 | 123 | 1 | 3 | 1 |
| `domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy` | support | 0 | 122 | 3 | 3 | 1 |
| `domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tests` | private-logic | 2 | 121 | 0 | 9 | 2 |
| `domain_computation::provider_session::graph_provider::bounded_step::provider_execution_release_tests` | private-logic | 1 | 120 | 0 | 1 | 0 |
| `domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::retained_activation_read_tests` | support | 0 | 119 | 3 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_binding::effect_accumulator::ordering_tests` | private-logic | 2 | 119 | 0 | 3 | 0 |
| `domain_computation::authorization::decision_facts::authorization::durable_dependencies::cancellation_tests` | support | 0 | 118 | 3 | 3 | 0 |
| `domain_computation::primary_graph::bootstrap::preparation::tests` | private-logic | 3 | 118 | 0 | 1 | 0 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::stop::owner::extended_tests` | private-logic | 6 | 118 | 0 | 12 | 0 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::adjacency::tests` | runtime-root-private | 1 | 118 | 1 | 5 | 5 |
| `domain_computation::primary_graph::application_attempt::effect_program::external_payload_tests` | private-logic | 4 | 117 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::post_commit::product_retirement::tests` | private-logic | 2 | 117 | 0 | 1 | 4 |
| `domain_computation::primary_graph::output_lineage::invalidation::full_verification::root_source::tests` | runtime-root-private | 1 | 116 | 1 | 5 | 3 |
| `domain_computation::primary_graph::provider::committed_dispatch_outbox::test_support` | support | 0 | 116 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_binding::effect_lowering::optional_field_patch::tests` | private-logic | 2 | 114 | 0 | 4 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_binding::unobserved_write_tests` | private-logic | 1 | 114 | 0 | 1 | 1 |
| `domain_computation::primary_graph::application_attempt::provider_execution::program_occurrence_gate::tests` | private-logic | 1 | 114 | 0 | 2 | 2 |
| `domain_computation::primary_graph::output_lineage::prepared_slot::recovery::tests` | private-logic | 1 | 114 | 0 | 11 | 5 |
| `domain_computation::primary_graph::provider::relational_execution_denial::tests` | private-logic | 3 | 113 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::progression::required_fresh::admission_court` | support | 0 | 112 | 2 | 6 | 0 |
| `domain_computation::primary_graph::live_delivery::source::count_tests` | private-logic | 2 | 112 | 0 | 3 | 3 |
| `domain_computation::primary_graph::application_checkpoint::facts::indexed_selection::tests` | private-logic | 4 | 111 | 0 | 9 | 1 |
| `domain_computation::primary_graph::bootstrap_publication::index_request_tests` | runtime-root-private | 1 | 111 | 3 | 1 | 0 |
| `domain_computation::primary_graph::product_operation::generated_materialization::restoration::invariant_admission::tests` | private-logic | 3 | 111 | 0 | 3 | 0 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::undecided_tests` | private-logic | 1 | 111 | 0 | 3 | 3 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::overlay_conflict_tests` | private-logic | 1 | 110 | 0 | 7 | 4 |
| `domain_computation::primary_graph::application_attempt::read_set::computation_routing::reattribution_tests` | private-logic | 3 | 110 | 0 | 5 | 0 |
| `domain_computation::primary_graph::invariant_projection::operation_reader::decision_reads::tests` | runtime-root-private | 3 | 109 | 1 | 6 | 0 |
| `domain_computation::primary_graph::provider::correction_witness::tests` | runtime-root-private | 1 | 109 | 2 | 1 | 1 |
| `domain_computation::primary_graph::workflow::definition::compilation::reconstruction::node::tests` | private-logic | 2 | 109 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::checkpoint_delivery::stopped_delivery` | runtime-root-private | 1 | 107 | 1 | 12 | 5 |
| `domain_computation::primary_graph::root::index_installation::tests` | private-logic | 2 | 107 | 0 | 2 | 1 |
| `domain_computation::authorization::capability_lowering::accumulator::tests` | private-logic | 1 | 105 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_binding::effect_lowering::observed_fact_index::tests` | private-logic | 3 | 105 | 0 | 5 | 0 |
| `domain_computation::primary_graph::application_query::observed_source::retained_dependencies::storage_court` | support | 0 | 105 | 2 | 2 | 0 |
| `domain_computation::primary_graph::conditional_operation::clock_observation::tests` | private-logic | 2 | 105 | 0 | 2 | 1 |
| `domain_computation::primary_graph::application_attempt::fact::consult_observation` | support | 0 | 104 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::read_set::computation_routing::test_mutation` | support | 0 | 104 | 0 | 7 | 1 |
| `domain_computation::primary_graph::output_lineage::registry_fixture` | support | 0 | 104 | 0 | 11 | 5 |
| `domain_computation::primary_graph::causal_inspection::materialization::contract::tests` | private-logic | 2 | 103 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_attempt::fact::source_merge::tests` | private-logic | 2 | 102 | 0 | 2 | 0 |
| `domain_computation::provider_session::graph_obligation::branch_affinity::tests` | runtime-root-private | 1 | 102 | 1 | 1 | 0 |
| `domain_computation::primary_graph::application_query::derived_view::registry::index_directory::tests` | admitted | 1 | 101 | 2 | 3 | 3 |
| `domain_computation::primary_graph::application_query::live::lease::foreign_phase_tests` | private-logic | 1 | 101 | 0 | 0 | 3 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::currentness::tests` | private-logic | 1 | 100 | 0 | 4 | 1 |
| `domain_computation::primary_graph::invariant_projection::operation_reader::current_output::resolution::tests` | private-logic | 2 | 99 | 0 | 2 | 1 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::native_reference_invariants::tests` | private-logic | 3 | 98 | 0 | 5 | 1 |
| `domain_computation::primary_graph::application_query::observed_source::retained_parameters::tests` | private-logic | 1 | 97 | 0 | 1 | 2 |
| `domain_computation::primary_graph::access_strategy::realized_footprint::comparison::tests` | private-logic | 4 | 96 | 0 | 3 | 0 |
| `domain_computation::primary_graph::workflow::definition::condition::tests` | private-logic | 3 | 96 | 0 | 5 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::required_work::wave_admission::tests` | private-logic | 2 | 95 | 0 | 5 | 1 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::stop::owner::lifecycle_tests` | private-logic | 3 | 94 | 0 | 2 | 0 |
| `domain_computation::primary_graph::product_operation::document_history::authority_lifetime::tests` | private-logic | 4 | 93 | 0 | 3 | 0 |
| `domain_computation::primary_graph::workflow::schema::fact_custody::tests` | private-logic | 1 | 93 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::authority_cause::tests` | private-logic | 1 | 92 | 0 | 0 | 3 |
| `domain_computation::primary_graph::conditional_operation::lifecycle::commit_watch::tests` | runtime-root-private | 1 | 92 | 2 | 1 | 0 |
| `domain_computation::primary_graph::output_lineage::prepared_slot::cancellation_custody` | private-logic | 1 | 92 | 0 | 8 | 2 |
| `domain_computation::provider_session::protocol::declared_closure::application::tests` | private-logic | 2 | 92 | 0 | 5 | 1 |
| `domain_computation::execution_runtime::product_world::owner_cleanup::registry::canonical_retirement` | runtime-root-private | 2 | 89 | 1 | 4 | 3 |
| `domain_computation::primary_graph::application_attempt::provider_binding::effect_lowering::tests` | private-logic | 1 | 89 | 0 | 8 | 0 |
| `domain_computation::primary_graph::application_checkpoint::facts::payload::copied::tests` | private-logic | 1 | 89 | 0 | 6 | 0 |
| `domain_computation::primary_graph::application_output_demand::recovered_outputs::tests` | runtime-root-private | 2 | 88 | 2 | 6 | 0 |
| `domain_computation::authorization::application_disclosure::influence_validation::tests` | private-logic | 3 | 86 | 0 | 3 | 1 |
| `domain_computation::primary_graph::application_query::managed_index::denial::tests` | private-logic | 2 | 86 | 0 | 4 | 0 |
| `domain_computation::primary_graph::conditional_operation::application_operation_reentry::denial::tests` | private-logic | 3 | 86 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::overlay_conflict_tests` | support | 0 | 85 | 1 | 2 | 2 |
| `domain_computation::primary_graph::product_activation::retention_tests` | private-logic | 2 | 85 | 0 | 2 | 5 |
| `domain_computation::managed_run::readmission::direct_lane::preflight::preparation::progression::tests` | private-logic | 1 | 84 | 0 | 2 | 2 |
| `domain_computation::primary_graph::admitted_capture::retained_stage::tests` | admitted | 1 | 84 | 2 | 2 | 0 |
| `domain_computation::primary_graph::conditional_operation::application_operation_reentry::tests` | private-logic | 2 | 83 | 0 | 6 | 0 |
| `domain_computation::primary_graph::application_attempt::elevation_lifecycle_emission::tests` | private-logic | 1 | 82 | 0 | 4 | 0 |
| `domain_computation::primary_graph::application_attempt::workflow_deadline::tests` | private-logic | 3 | 82 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::workflow_transition_program::assessment::evidence_budget::tests` | private-logic | 1 | 82 | 0 | 4 | 1 |
| `domain_computation::primary_graph::application_query::observed_source::source_identity::checkpoint_commitment::tests` | private-logic | 1 | 82 | 0 | 5 | 6 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::tests` | support | 0 | 82 | 0 | 0 | 2 |
| `domain_computation::authorization::capability_lowering::semantic_model_tests` | private-logic | 1 | 80 | 0 | 0 | 1 |
| `domain_computation::primary_graph::application_contribution::partitioned_computation::tests` | private-logic | 3 | 80 | 0 | 2 | 1 |
| `domain_computation::primary_graph::application_invariant::typed_view::field_reads::tests` | private-logic | 2 | 80 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_binding::effect_accumulator::expected_steps::tests` | private-logic | 4 | 79 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_query::managed_index::provider_failure::tests` | private-logic | 2 | 78 | 0 | 2 | 0 |
| `domain_computation::primary_graph::expression::tests` | private-logic | 3 | 78 | 0 | 3 | 0 |
| `domain_computation::artifact_owner::provider_resource_tests` | private-logic | 1 | 77 | 0 | 1 | 0 |
| `domain_computation::primary_graph::product_operation::generated_materialization::restoration::failure::tests` | runtime-root-private | 1 | 77 | 1 | 1 | 0 |
| `domain_computation::provider_session::protocol::declared_closure_tests` | private-logic | 3 | 76 | 0 | 3 | 2 |
| `domain_computation::primary_graph::entity_resolution::interruption_control` | support | 0 | 75 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::workflow_instance_observation::completed_source::output_match::tests` | private-logic | 1 | 74 | 0 | 4 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::execution::denial::content_identity::tests` | runtime-root-private | 1 | 74 | 1 | 2 | 0 |
| `domain_computation::primary_graph::application_checkpoint::facts::payload::performed_fields::tests` | private-logic | 1 | 73 | 0 | 4 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::completion_custody` | private-logic | 1 | 73 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_query::admission_preparation::tests` | private-logic | 1 | 73 | 0 | 0 | 1 |
| `domain_computation::primary_graph::provider::committed_dispatch_outbox::restoration_tests` | private-logic | 3 | 73 | 0 | 1 | 1 |
| `domain_computation::primary_graph::application_attempt::workflow_transition_program::publication::authentication::tests` | private-logic | 1 | 72 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_query::basis::historical_authority` | support | 0 | 72 | 0 | 1 | 0 |
| `domain_computation::primary_graph::invariant_projection::consumed_output::equivalence::tests` | private-logic | 3 | 72 | 0 | 7 | 0 |
| `domain_computation::primary_graph::application_attempt::workflow_instance_observation::settlement::dependency::aspect_presence::tests` | private-logic | 1 | 71 | 0 | 6 | 1 |
| `domain_computation::primary_graph::application_contribution::producer::demand::selection::retained_basis_tests` | private-logic | 1 | 71 | 0 | 1 | 1 |
| `domain_computation::primary_graph::operation_body::input::tests` | private-logic | 1 | 71 | 0 | 2 | 0 |
| `domain_computation::primary_graph::value_decode::state::tests` | runtime-root-private | 2 | 71 | 2 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::composite_dispatch_tests` | private-logic | 1 | 70 | 0 | 0 | 4 |
| `domain_computation::primary_graph::invariant_projection::locked_reader::predecode_admission::decoding::tests` | private-logic | 2 | 70 | 0 | 1 | 0 |
| `domain_computation::primary_graph::product_operation::query::retained::tests` | private-logic | 2 | 70 | 0 | 1 | 1 |
| `domain_computation::primary_graph::provider::inbound_completion::preparation_denial::tests` | runtime-root-private | 1 | 70 | 1 | 3 | 0 |
| `domain_computation::primary_graph::operation_body::assessment::tests` | private-logic | 1 | 69 | 0 | 1 | 1 |
| `domain_computation::primary_graph::aggregate_projection::tests` | private-logic | 2 | 68 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::progressed::cleanup_failure::tests` | private-logic | 2 | 68 | 0 | 3 | 1 |
| `domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::arrival_tests` | private-logic | 2 | 68 | 0 | 2 | 0 |
| `domain_computation::primary_graph::output_lineage::required_publication::head::tests` | private-logic | 3 | 68 | 0 | 2 | 0 |
| `domain_computation::primary_graph::workflow::assessment::dependency::tests` | private-logic | 1 | 68 | 0 | 2 | 2 |
| `domain_computation::application_aftermath::recovery_progression::resolve_tests` | private-logic | 2 | 67 | 0 | 0 | 5 |
| `domain_computation::primary_graph::application_contribution::producer::execution::outcome::invariant_refusal_tests` | private-logic | 2 | 67 | 0 | 3 | 0 |
| `domain_computation::primary_graph::output_lineage::input_cutoff::stable_publication::fact_spans::tests` | private-logic | 1 | 67 | 0 | 5 | 0 |
| `domain_computation::primary_graph::workflow::instance::progression::replay::tests` | private-logic | 2 | 66 | 0 | 2 | 0 |
| `domain_computation::execution_runtime::product_world::read_lock_probe` | support | 0 | 65 | 2 | 0 | 2 |
| `domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::checkpoint::tests` | private-logic | 1 | 65 | 0 | 3 | 0 |
| `domain_computation::application_aftermath::external_effect::transport::tests` | private-logic | 1 | 64 | 0 | 2 | 0 |
| `domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::request_read_stops_tests` | support | 0 | 64 | 2 | 3 | 0 |
| `domain_computation::primary_graph::application_runtime::operation_control::attempt_registration::caller_pass::tests` | private-logic | 3 | 64 | 0 | 1 | 0 |
| `domain_computation::primary_graph::fact_verification_observation::input_cutoff::tests` | runtime-root-private | 2 | 64 | 1 | 4 | 0 |
| `domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::forgery_doors` | support | 0 | 63 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_invariant::program_selection::tests` | private-logic | 6 | 63 | 0 | 3 | 0 |
| `domain_computation::primary_graph::output_lineage::invalidation::canonical_binding_order` | private-logic | 1 | 63 | 0 | 3 | 1 |
| `domain_computation::provider_session::graph_obligation::owner_execution::pin_control` | support | 0 | 63 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::fact::indexed_entity_selection::denials_tests` | runtime-root-private | 1 | 62 | 1 | 1 | 3 |
| `domain_computation::primary_graph::application_contribution::producer::execution::outcome::domain_denial_tests` | private-logic | 2 | 62 | 0 | 2 | 0 |
| `domain_computation::runtime_time::time_basis::tests` | private-logic | 2 | 62 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_attempt::dependency_identity::output_postcondition::owned_program_facts_tests` | private-logic | 1 | 61 | 0 | 5 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::post_commit::custody_reader::tests` | private-logic | 1 | 61 | 0 | 1 | 0 |
| `domain_computation::primary_graph::invariant_installation::program_activation_work_reserve::tests` | private-logic | 5 | 61 | 0 | 2 | 0 |
| `domain_computation::primary_graph::invariant_projection::consumed_output::test_support` | support | 0 | 60 | 0 | 5 | 1 |
| `domain_computation::primary_graph::invariant_projection::consumed_output::verification::work_budget::tests` | private-logic | 3 | 60 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::read_set::decision_reuse::tests` | private-logic | 1 | 59 | 0 | 3 | 3 |
| `domain_computation::primary_graph::application_attempt::read_set::unique_write_dependencies::tests` | runtime-root-private | 1 | 59 | 1 | 12 | 0 |
| `domain_computation::primary_graph::application_query::basis::admission_denial::owner_unavailable_tests` | private-logic | 3 | 59 | 0 | 1 | 2 |
| `domain_computation::primary_graph::output_lineage::required_settlement::current_accepted::own_write_recovery` | private-logic | 2 | 59 | 0 | 6 | 2 |
| `domain_computation::primary_graph::provider::idempotency::snapshot_resolution::counted_read_stops` | private-logic | 1 | 59 | 0 | 6 | 3 |
| `domain_computation::primary_graph::provider::invariant_execution::relational_validation::tests` | private-logic | 1 | 59 | 0 | 2 | 0 |
| `domain_computation::primary_graph::schema_layout::tests` | private-logic | 1 | 59 | 0 | 0 | 1 |
| `domain_computation::managed_run::test_phase_custody` | support | 0 | 57 | 0 | 0 | 1 |
| `domain_computation::primary_graph::workflow::operation_output::context::tests` | private-logic | 2 | 57 | 0 | 1 | 0 |
| `domain_computation::artifact_owner::authority_match::tests` | private-logic | 1 | 56 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::idempotency::workflow_definition::tests` | public-surface | 3 | 56 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_entry::mutation::failure::tests` | public-surface | 1 | 56 | 0 | 0 | 0 |
| `domain_computation::primary_graph::resolution::selected_product::admitted_tests` | private-logic | 1 | 56 | 0 | 0 | 3 |
| `domain_computation::primary_graph::application_contribution::producer::demand::disclosure::producer_input::transport_court` | support | 0 | 55 | 0 | 4 | 0 |
| `domain_computation::primary_graph::application_query::projection::projected_tree::content::tests` | private-logic | 2 | 55 | 0 | 1 | 0 |
| `domain_computation::invalidation_equivalence_control::scope_tests` | public-surface | 3 | 54 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::prerequisite_claims::custody_note_steps` | support | 0 | 54 | 0 | 4 | 0 |
| `domain_computation::primary_graph::root::test_inspection` | support | 0 | 54 | 1 | 0 | 0 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::stop::output_tests` | private-logic | 1 | 52 | 0 | 4 | 0 |
| `domain_computation::primary_graph::provider::committed_dispatch_outbox::layout_tests` | private-logic | 1 | 52 | 0 | 0 | 1 |
| `domain_computation::primary_graph::provider::pending_application_publication::registry::correction_tests` | private-logic | 1 | 52 | 0 | 2 | 1 |
| `domain_computation::primary_graph::workflow::definition::compilation::plan::tests` | private-logic | 1 | 52 | 0 | 5 | 0 |
| `domain_computation::primary_graph::application_attempt::denial::allocation_error_source_tests` | public-surface | 1 | 51 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_attempt::idempotency::workflow_transition::tests` | public-surface | 3 | 51 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::comparison_tests` | support | 0 | 51 | 1 | 1 | 2 |
| `domain_computation::primary_graph::index_refresh::read_denial_tests` | runtime-root-private | 1 | 51 | 1 | 1 | 1 |
| `domain_computation::primary_graph::invariant_projection::operation_reader::decision_plan::orbit::tests` | private-logic | 1 | 51 | 0 | 0 | 3 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::consumed_output::capacity_tests` | private-logic | 1 | 50 | 0 | 2 | 0 |
| `published_checkpoint` | support | 0 | 50 | 2 | 0 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::required_continuations::held::state_unavailable` | private-logic | 1 | 49 | 0 | 3 | 1 |
| `domain_computation::primary_graph::application_query::batch::planning::tests` | private-logic | 1 | 49 | 0 | 0 | 1 |
| `domain_computation::primary_graph::live_delivery::source::testing` | support | 0 | 49 | 0 | 2 | 0 |
| `domain_computation::primary_graph::publication_request_denial::tests` | runtime-root-private | 1 | 49 | 1 | 1 | 0 |
| `domain_computation::application_aftermath::recovery_progression::mechanism_tests` | private-logic | 2 | 48 | 0 | 0 | 3 |
| `domain_computation::artifact_owner::native_access::provider_value_view::tests` | public-surface | 2 | 48 | 0 | 0 | 0 |
| `domain_computation::primary_graph::invariant_projection::operation_reader::decision_reads::aspect_presence::tests` | runtime-root-private | 1 | 48 | 1 | 4 | 0 |
| `domain_computation::primary_graph::request_allocation_control::tests` | private-logic | 2 | 48 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_branch::residue` | public-surface | 1 | 47 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_contribution::request_execution::test_placement::scope_tests` | public-surface | 3 | 46 | 0 | 0 | 0 |
| `domain_computation::primary_graph::output_lineage::required_settlement::tests` | private-logic | 2 | 46 | 0 | 5 | 1 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::precommit_snapshot::tests` | runtime-root-private | 1 | 46 | 1 | 1 | 3 |
| `domain_computation::primary_graph::schema_layout::provider_inbound_completion::tests` | private-logic | 1 | 46 | 0 | 0 | 1 |
| `domain_computation::primary_graph::application_attempt::retained_decision_facts::handoff` | support | 0 | 45 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_query::observed_source::source_identity::epoch_construction` | support | 0 | 45 | 0 | 5 | 0 |
| `domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::admit::owner_read_tests` | private-logic | 2 | 45 | 0 | 1 | 0 |
| `domain_computation::primary_graph::provider::inbound_completion::read::provenance_tests` | private-logic | 1 | 45 | 0 | 4 | 0 |
| `domain_computation::primary_graph::workflow::definition::inbound_codec::tests` | private-logic | 1 | 45 | 0 | 1 | 0 |
| `domain_computation::authorization::capability_observation::grant_selection::late_cancellation` | support | 0 | 44 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::selection::retained_liveness::counted_read_stops` | private-logic | 1 | 44 | 0 | 2 | 3 |
| `domain_computation::primary_graph::provider::aftermath_causality::counted_read_stops` | runtime-root-private | 1 | 43 | 1 | 3 | 3 |
| `domain_computation::primary_graph::application_installation::program::demand::unavailable::custody_order` | runtime-root-private | 1 | 42 | 1 | 1 | 1 |
| `domain_computation::primary_graph::conditional_operation::installation::application_binding_scope::tests` | private-logic | 1 | 42 | 0 | 3 | 0 |
| `domain_computation::primary_graph::invariant_projection::identity_encoding::tests` | public-surface | 1 | 42 | 0 | 0 | 0 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::index_preparation_tests` | private-logic | 2 | 42 | 0 | 1 | 0 |
| `domain_computation::primary_graph::provider::branch_commit_coordination::tests` | private-logic | 2 | 42 | 0 | 1 | 2 |
| `domain_computation::primary_graph::workflow::definition::global_actions::tests` | private-logic | 1 | 42 | 0 | 1 | 0 |
| `domain_computation::primary_graph::workflow::operation_output::origin::tests` | private-logic | 1 | 42 | 0 | 4 | 0 |
| `domain_computation::provider_session::graph_provider::receipt_association::tests` | private-logic | 1 | 42 | 0 | 2 | 1 |
| `domain_computation::convergence_epoch::iteration_owner::workflow::association::test_host_policy` | support | 0 | 41 | 0 | 2 | 2 |
| `domain_computation::primary_graph::conditional_operation::execution_provenance::classification::tests` | private-logic | 1 | 40 | 0 | 1 | 0 |
| `domain_computation::primary_graph::conditional_operation::installation::output_readiness::tests` | private-logic | 2 | 40 | 0 | 2 | 0 |
| `domain_computation::primary_graph::workflow::schema::relations::tests` | private-logic | 1 | 40 | 0 | 6 | 0 |
| `domain_computation::primary_graph::application_output_demand::currentness::consumed::tests` | private-logic | 1 | 39 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_query::read_execution::tree_materialization::field_presence::tests` | private-logic | 1 | 39 | 0 | 2 | 0 |
| `domain_computation::primary_graph::product_operation::program_adoption::preparation::tests` | private-logic | 1 | 39 | 0 | 1 | 0 |
| `domain_computation::primary_graph::program_occurrence::activation_cell::tests` | private-logic | 3 | 39 | 0 | 1 | 0 |
| `domain_computation::primary_graph::structural_walk::key::key_ordering_tests` | private-logic | 1 | 39 | 0 | 2 | 0 |
| `domain_computation::application_aftermath::external_effect::inbound::custody::maintenance::tests` | private-logic | 1 | 38 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::compare_and_commit::commit_receipt::certification` | private-logic | 2 | 38 | 0 | 1 | 2 |
| `domain_computation::primary_graph::schema_layout::platform_identity_allocator::tests` | private-logic | 3 | 38 | 0 | 1 | 0 |
| `domain_computation::convergence_epoch::iteration_owner::direct::association::test_host_policy` | support | 0 | 37 | 0 | 2 | 2 |
| `domain_computation::primary_graph::application_attempt::provider_binding::effect_accumulator::tests` | private-logic | 1 | 37 | 0 | 6 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::own_effect_observation` | support | 0 | 37 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::post_commit::source_retirement::tests` | private-logic | 1 | 37 | 0 | 1 | 0 |
| `domain_computation::primary_graph::output_lineage::invalidation::commit_touches::relation_interest_tests` | private-logic | 1 | 37 | 0 | 4 | 0 |
| `domain_computation::provider_session::graph_obligation::inspection_terminal::court` | support | 0 | 37 | 0 | 1 | 0 |
| `test_request_policy` | support | 0 | 37 | 3 | 0 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::frontier::tests` | private-logic | 1 | 36 | 0 | 5 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::execution::outcome::commit_denial::tests` | private-logic | 1 | 36 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_query::output_source_observation::current_control::tests` | public-surface | 3 | 36 | 0 | 0 | 0 |
| `domain_computation::primary_graph::output_lineage::current_output::receipt_stops` | runtime-root-private | 1 | 36 | 1 | 0 | 1 |
| `domain_computation::primary_graph::provider::idempotency::denial_mapping_tests` | private-logic | 2 | 36 | 0 | 4 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::commit_resolution::read_fold_tests` | private-logic | 1 | 35 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::workflow_transition_program::preparation::actor_denial::tests` | private-logic | 1 | 35 | 0 | 1 | 0 |
| `domain_computation::primary_graph::invariant_projection::entity_resident::tests` | public-surface | 1 | 35 | 0 | 0 | 0 |
| `domain_computation::primary_graph::invariant_projection::operation_reader::decision_plan::field_presence::tests` | private-logic | 1 | 35 | 0 | 1 | 1 |
| `domain_computation::primary_graph::output_lineage::recorded_output::span_tests` | private-logic | 1 | 35 | 0 | 9 | 1 |
| `domain_computation::primary_graph::provider::managed_cleanup_control::tests` | private-logic | 2 | 35 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::source_custody::reservation_control::tests` | private-logic | 1 | 34 | 0 | 2 | 0 |
| `domain_computation::primary_graph::output_lineage::invalidation::logical_marking::read_observation` | support | 0 | 34 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_attempt::denial::native_storage_tests` | public-surface | 1 | 33 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_attempt::dependency_identity::canonical_encoding::observation_identity::tests` | private-logic | 1 | 33 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::post_commit::discovery::tests` | private-logic | 1 | 33 | 0 | 1 | 1 |
| `domain_computation::primary_graph::application_query::basis::admission::product_carriage_tests` | runtime-root-private | 1 | 33 | 1 | 0 | 3 |
| `domain_computation::primary_graph::workflow::instance::transition::outcome::tests` | private-logic | 1 | 33 | 0 | 2 | 0 |
| `domain_computation::provider_session::decision_read_set::failure::footprint_tests` | public-surface | 1 | 33 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::execution::input_cutoff::refusal::tests` | private-logic | 1 | 32 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::required_work::proof_fixture` | support | 0 | 32 | 0 | 4 | 1 |
| `domain_computation::primary_graph::provider::fault_port::output_comparison_cancellation_tests` | private-logic | 2 | 32 | 0 | 1 | 0 |
| `domain_computation::primary_graph::workflow::correction::tests` | private-logic | 1 | 32 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_query::read_execution::entry_observation::observation_tests` | private-logic | 1 | 31 | 0 | 1 | 0 |
| `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::failure::tests` | private-logic | 2 | 31 | 0 | 2 | 0 |
| `domain_computation::primary_graph::provider::invariant_execution::validation_control::tests` | private-logic | 1 | 31 | 0 | 1 | 0 |
| `domain_computation::primary_graph::workflow::definition::compilation::plan::dispatch::tests` | private-logic | 1 | 31 | 0 | 3 | 0 |
| `domain_computation::execution_runtime::product_world::owner_cleanup::registry::testing` | support | 0 | 30 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::commit_resolution::index_preparation_tests` | private-logic | 1 | 30 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::join_observation::scope_tests` | private-logic | 2 | 30 | 0 | 2 | 0 |
| `domain_computation::primary_graph::output_lineage::invalidation::settlement::posting_continuation::tests` | private-logic | 1 | 30 | 0 | 2 | 0 |
| `domain_computation::primary_graph::product_operation::program_adoption::custody::recovery::readmission::foreign_phase_tests` | private-logic | 1 | 30 | 0 | 0 | 2 |
| `domain_computation::primary_graph::provider::pending_application_publication::aftermath::tests` | private-logic | 1 | 30 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::queued::tests` | private-logic | 2 | 29 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::selection::read_stop_tests` | runtime-root-private | 1 | 29 | 1 | 1 | 2 |
| `domain_computation::primary_graph::output_lineage::recorded_source_identity::tests` | private-logic | 1 | 29 | 0 | 3 | 0 |
| `domain_computation::primary_graph::application_attempt::workflow_transition_program::publication::operation::approval_denial::tests` | private-logic | 1 | 28 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::workflow_transition_program::publication::required_assessment::tests` | public-surface | 1 | 28 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::sealing::tests` | private-logic | 1 | 28 | 0 | 4 | 0 |
| `domain_computation::primary_graph::application_installation::profile::tests` | public-surface | 1 | 28 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_query::observed_source::denial::tests` | runtime-root-public | 1 | 28 | 1 | 0 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::execution::outcome::publication_record_tests` | private-logic | 1 | 27 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_query::basis::admission_denial::tests` | private-logic | 2 | 27 | 0 | 0 | 1 |
| `domain_computation::primary_graph::application_query::derived_view::retention::reconstruction_observation` | support | 0 | 27 | 0 | 1 | 0 |
| `domain_computation::primary_graph::provider::branch_commit_coordination::correction::tests` | private-logic | 1 | 27 | 0 | 1 | 0 |
| `domain_computation::application_aftermath::retained_preimage_tests` | private-logic | 1 | 26 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::observation::read_rendezvous` | support | 0 | 26 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_attempt::workflow_instance_program::migration::lineage::request_stops` | runtime-root-private | 1 | 26 | 2 | 1 | 3 |
| `domain_computation::primary_graph::application_contribution::producer::demand::bridge_denial::cause_contracts` | runtime-root-private | 1 | 26 | 1 | 1 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::required_stop::source_query::tests` | private-logic | 2 | 26 | 0 | 2 | 0 |
| `domain_computation::application_aftermath::external_effect::causal_event::tests` | private-logic | 1 | 25 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_entry::mutation::execution::decision_query::cause_tests` | private-logic | 1 | 25 | 0 | 1 | 0 |
| `domain_computation::primary_graph::bootstrap::checkpoint_transition::capture_funding::tests` | runtime-root-public | 1 | 24 | 2 | 0 | 0 |
| `domain_computation::managed_run::truth_read_request::tests` | public-surface | 1 | 23 | 0 | 0 | 0 |
| `domain_computation::primary_graph::invariant_projection::work::tests` | private-logic | 1 | 23 | 0 | 1 | 0 |
| `domain_computation::primary_graph::output_lineage::input_cutoff::verification::pending_upstream::tests` | private-logic | 1 | 23 | 0 | 3 | 0 |
| `domain_computation::primary_graph::provider::publication_fault_control::tests` | private-logic | 1 | 23 | 0 | 2 | 0 |
| `domain_computation::primary_graph::provider::relational_execution_denial::preflight::tests` | private-logic | 1 | 23 | 0 | 2 | 0 |
| `domain_computation::managed_run::lower_admission::atomic_tests` | private-logic | 1 | 22 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::demand::progression::caller_pass_observation::scope_tests` | private-logic | 1 | 22 | 0 | 0 | 1 |
| `domain_computation::primary_graph::output_lineage::input_reuse_key::tests` | private-logic | 1 | 22 | 0 | 3 | 1 |
| `domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::workflow::tests` | public-surface | 1 | 21 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_contribution::producer::execution::denial::tests` | private-logic | 1 | 21 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_contribution::request_execution::advancement::standalone_test_custody` | support | 0 | 21 | 0 | 2 | 1 |
| `domain_computation::primary_graph::application_query::resource_lifecycle::lifecycle_count::tests` | private-logic | 2 | 21 | 0 | 5 | 0 |
| `domain_computation::primary_graph::provider::application_attempt_state::tests` | private-logic | 1 | 21 | 0 | 1 | 2 |
| `domain_computation::primary_graph::workflow::evidence_dependency::tests` | private-logic | 1 | 21 | 0 | 1 | 0 |
| `domain_computation::convergence_epoch::iteration_owner::direct::epoch::test_phase_custody` | support | 0 | 20 | 0 | 0 | 2 |
| `domain_computation::convergence_epoch::iteration_owner::workflow::epoch::test_phase_custody` | support | 0 | 20 | 0 | 0 | 2 |
| `domain_computation::primary_graph::application_contribution::producer::output_evaluation::attempt_tests` | private-logic | 1 | 20 | 0 | 1 | 0 |
| `domain_computation::primary_graph::output_reuse::selection::tests` | private-logic | 1 | 20 | 0 | 3 | 0 |
| `domain_computation::primary_graph::resolution_denial::tests` | private-logic | 1 | 20 | 0 | 3 | 0 |
| `domain_computation::primary_graph::workflow::instance::transition::admission::selection::proposal_need::tests` | private-logic | 1 | 20 | 0 | 1 | 0 |
| `domain_computation::provider_session::graph_obligation::session_identity::tests` | public-surface | 2 | 20 | 0 | 0 | 0 |
| `domain_computation::application_aftermath::external_effect::identity_derivation::tests` | private-logic | 1 | 19 | 0 | 3 | 0 |
| `domain_computation::execution_runtime::product_world::activation::capacity::tests` | private-logic | 1 | 19 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_attempt::effect_program::write_index::tests` | private-logic | 1 | 19 | 0 | 1 | 0 |
| `domain_computation::primary_graph::product_operation::generated_materialization::restoration::no_effect::tests` | private-logic | 1 | 19 | 0 | 1 | 0 |
| `domain_computation::provider_session::graph_obligation::owner_execution::pin_panic` | support | 0 | 19 | 0 | 1 | 0 |
| `domain_computation::artifact_owner::registry_evidence::tests` | private-logic | 1 | 18 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::dependency_identity::producing_implementation::tests` | private-logic | 1 | 18 | 0 | 1 | 0 |
| `domain_computation::primary_graph::workflow::instance::progression::navigation::tests` | private-logic | 1 | 18 | 0 | 0 | 1 |
| `domain_computation::primary_graph::causal_inspection::application_inspection::connection::output_refusal::tests` | public-surface | 1 | 17 | 0 | 0 | 0 |
| `domain_computation::primary_graph::observations::principal_target_read_observation::tests` | private-logic | 1 | 17 | 0 | 2 | 0 |
| `domain_computation::primary_graph::application_attempt::idempotency::workflow_proposal_context::tests` | public-surface | 1 | 16 | 0 | 0 | 0 |
| `domain_computation::primary_graph::application_query::managed_index::nodes::lineage::tests` | private-logic | 1 | 16 | 0 | 1 | 0 |
| `domain_computation::primary_graph::invariant_projection::admission_denial::stack_size_tests` | runtime-root-public | 1 | 16 | 1 | 0 | 0 |
| `domain_computation::primary_graph::invariant_projection::operation_reader::current_output::resolution::verification_denial::tests` | private-logic | 1 | 16 | 0 | 2 | 0 |
| `domain_computation::provider_session::protocol::provider_port::tests` | private-logic | 1 | 16 | 0 | 1 | 0 |
| `domain_computation::execution_observation::tests` | private-logic | 1 | 15 | 0 | 1 | 0 |
| `domain_computation::managed_run::run_affinity::tests` | private-logic | 1 | 15 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_contribution::conditional::dependency::tests` | private-logic | 1 | 15 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_query::one_shot::denial::tests` | private-logic | 1 | 15 | 0 | 1 | 0 |
| `domain_computation::primary_graph::provider::decision_facts::comparison::indexed::read_fold_tests` | runtime-root-private | 1 | 15 | 1 | 1 | 0 |
| `domain_computation::primary_graph::application_installation::program::speculation::discard_fault` | support | 0 | 14 | 1 | 0 | 0 |
| `domain_computation::primary_graph::application_output_demand::registry::record_map::observation_overflow_tests` | private-logic | 1 | 14 | 0 | 3 | 0 |
| `domain_computation::primary_graph::provider::product_retirement::tests` | private-logic | 1 | 14 | 0 | 1 | 0 |
| `domain_computation::primary_graph::invariant_projection::operation_reader::managed_index::cause_tests` | private-logic | 1 | 13 | 0 | 1 | 0 |
| `domain_computation::authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::tests` | private-logic | 1 | 12 | 0 | 1 | 0 |
| `domain_computation::execution_runtime::runtime_identity::tests` | private-logic | 1 | 12 | 0 | 1 | 0 |
| `domain_computation::primary_graph::conditional_operation::temporal_reconstruction::tests` | private-logic | 1 | 12 | 0 | 1 | 0 |
| `domain_computation::primary_graph::product_operation::document_history::step::tests` | private-logic | 1 | 12 | 0 | 1 | 0 |
| `domain_computation::provider_session::execution_attempt_identity::tests` | private-logic | 1 | 12 | 0 | 1 | 0 |
| `domain_computation::convergence_epoch::iteration_owner::direct::yield_transition::test_phase_custody` | support | 0 | 11 | 0 | 0 | 2 |
| `domain_computation::convergence_epoch::iteration_owner::workflow::yield_transition::test_phase_custody` | support | 0 | 11 | 0 | 0 | 2 |
| `domain_computation::primary_graph::application_contribution::conditional::producer_access::tests` | private-logic | 1 | 11 | 0 | 1 | 0 |
| `domain_computation::primary_graph::output_lineage::invalidation::mark_state::projection_identity_tests` | private-logic | 1 | 11 | 0 | 1 | 0 |
| `bridge_host_request` | support | 0 | 10 | 1 | 0 | 0 |
| `domain_computation::managed_run::direct_yield::test_phase_custody` | support | 0 | 10 | 0 | 0 | 1 |
| `domain_computation::managed_run::workflow_yield::test_phase_custody` | support | 0 | 10 | 0 | 0 | 1 |
| `domain_computation::primary_graph::application_attempt::retained_decision_facts::fact_store::tests` | private-logic | 1 | 10 | 0 | 1 | 0 |
| `domain_computation::primary_graph::workflow::proposal::observation::fields::identity_tests` | private-logic | 1 | 9 | 0 | 1 | 0 |
| `domain_computation::primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::session_admission::registration_conflict_tests` | support | 0 | 7 | 0 | 0 | 0 |
| `domain_computation::authorization::application_disclosure::contract::cause_tests` | private-logic | 1 | 6 | 0 | 1 | 0 |

## Blocked courts

| Blocked by | Lines |
|---|---:|
| hooks | 13127 |
| hooks+private items | 145365 |
| private items | 5065 |

## Needs blocking the most court lines (top 40)

| Need | Families | Lines blocked |
|---|---:|---:|
| `hook .test_host_request` | 38 | 130921 |
| `hook .fixture` | 52 | 127185 |
| `hook .borrowed_request_for_test` | 31 | 115862 |
| `hook domain_computation::primary_graph::with_test_advancement` | 8 | 111451 |
| `hook domain_computation::primary_graph::TestHostRequest` | 7 | 104978 |
| `private equality_type_id::EqualityTypeId` | 11 | 99891 |
| `private domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact` | 10 | 95048 |
| `private domain_computation::primary_graph::primary_truth_branch_identity` | 2 | 93949 |
| `hook domain_computation::execution_runtime::product_world::test_product_world_resources` | 8 | 93428 |
| `hook .default_branch` | 20 | 92227 |
| `hook .admit_product_branch` | 8 | 84881 |
| `private domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole` | 7 | 84267 |
| `private domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence` | 4 | 82386 |
| `hook .fixture_binding` | 4 | 82051 |
| `hook .select_product_branch` | 15 | 82020 |
| `private domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness` | 3 | 81899 |
| `private domain_computation::primary_graph::application_attempt::Movement` | 6 | 81722 |
| `hook .decision_facts` | 3 | 81568 |
| `private domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome` | 3 | 80829 |
| `private domain_computation::primary_graph::aggregate_observation::AggregateObservation` | 3 | 79669 |
| `private relational_snapshot_release::release_query_snapshot` | 12 | 79628 |
| `private domain_computation::primary_graph::application_contribution::QueryRequestExecution` | 3 | 79460 |
| `private domain_computation::primary_graph::WorthQueryApplicationObservedFact` | 3 | 76931 |
| `hook .provider_session_resource_count_for_test` | 5 | 76864 |
| `private domain_computation::primary_graph::output_reuse::OutputDependencySelection` | 4 | 76566 |
| `private domain_computation::primary_graph::output_reuse::compare_retained_output_dependencies` | 4 | 76566 |
| `hook domain_computation::primary_graph::application_checkpoint::decode_producer_facts` | 4 | 76220 |
| `hook domain_computation::primary_graph::application_checkpoint::encode_producer_facts` | 4 | 76220 |
| `private domain_computation::primary_graph::application_checkpoint::facts::FactSequence` | 4 | 76220 |
| `private domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied` | 5 | 75776 |
| `hook .active_snapshot_count` | 4 | 75115 |
| `private domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease` | 2 | 75065 |
| `private domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout` | 2 | 75065 |
| `private domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure` | 2 | 74912 |
| `private domain_computation::primary_graph::provider::relational_execution_denial::native_preparation_kind` | 2 | 74910 |
| `hook domain_computation::primary_graph::installed_source_reads_on_this_thread_for_test` | 2 | 74485 |
| `private domain_computation::primary_graph::WorthQueryApplicationSnapshotLease` | 2 | 74485 |
| `hook domain_computation::primary_graph::application_query::component_native_acquisitions_on_this_thread_for_test` | 2 | 74231 |
| `hook .prepare_output_contract_for_test` | 2 | 74228 |
| `private domain_computation::primary_graph::application_attempt::OutputRoleUse` | 2 | 74228 |
