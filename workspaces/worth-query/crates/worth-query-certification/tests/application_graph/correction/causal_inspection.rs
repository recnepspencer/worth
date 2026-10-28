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
