//! The actual ordinary source/producer/query route through the two facades.
use super::model::causal_inspection::journey;
#[test]
fn ordinary_source_and_performed_producer_connect_to_actual_calculated_query() {
    journey::connected_source_and_output();
}
#[test]
fn equal_rewrites_foreign_receipts_and_head_queries_do_not_invent_producers() {
    journey::equal_values_and_head_reads_cannot_supply_a_cause();
}
#[test]
fn revoked_current_read_refuses_before_retained_source_lookup() {
    journey::current_admission_precedes_source_lookup();
}

#[test]
fn source_only_inspection_needs_no_query_or_projected_authored_field() {
    super::model::causal_inspection::source_only::source_without_a_completed_query_or_projected_input();
}
#[test]
fn source_and_query_metadata_cannot_borrow_payload_permission() {
    super::model::causal_inspection::source_only::source_metadata_is_independent_of_payload_and_query_metadata();
}
#[test]
fn source_inspection_rechecks_revocation_cancellation_and_request_expiry() {
    super::model::causal_inspection::source_only::revoked_cancelled_and_expired_requests_release_source_inspection();
}

#[test]
fn source_inspection_keeps_original_request_payer_until_final_close() {
    super::model::causal_inspection::source_only::source_report_retains_fresh_request_funding_until_close();
}

#[test]
fn calculated_only_query_connects_independently_admitted_original_source() {
    super::model::causal_inspection::independent::calculated_only_connects_original_source_under_independent_governance();
}
#[test]
fn connected_source_and_query_cannot_lend_governance_or_original_ownership() {
    super::model::causal_inspection::independent::neither_binding_can_lend_metadata_diagnostics_or_ownership();
}

#[test]
fn independent_binding_connection_preserves_exact_source_created_revision() {
    super::model::causal_inspection::independent::equal_source_rewrites_do_not_connect_to_the_later_producer();
}

#[test]
fn independent_connection_respects_actual_query_parameters_and_empty_output() {
    super::model::causal_inspection::independent::no_match_parameters_keep_original_output_evidence_unavailable();
}

#[test]
fn independent_connection_rechecks_current_read_cancellation_and_expiry() {
    super::model::causal_inspection::independent::current_admission_cancellation_and_expiry_precede_connection_discovery();
}
