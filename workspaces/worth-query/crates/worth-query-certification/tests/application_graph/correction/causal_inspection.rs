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
