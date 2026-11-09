//! Grouped Native case payloads survive actual authored correction.
use super::*;
pub(super) mod inspection;
pub(super) mod schema;
#[test]
fn authored_cascade_restores_both_cases_of_each_grouped_membership() {
    correction_courts::require_lifecycle::<Model<29>>(1, &[10, 101, 102], false);
}
