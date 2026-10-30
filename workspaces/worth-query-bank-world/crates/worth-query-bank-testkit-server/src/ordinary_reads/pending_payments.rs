pub use bank_domain::model::CustomerRole;
pub use bank_server::{
    queries, BankApplicationOneShotDenialKind, BankApplicationQueryDenial, BankReadControls,
};

pub use super::fixture::{
    ordinary_read_world, ordinary_read_world_with_pending_payments,
    over_budget_discovery_world_with_role, APPROVER, OWNER, VIEWER,
};
pub use crate::facade::support::request_scope;

pub fn controls(maximum_results: usize) -> BankReadControls {
    BankReadControls::current(request_scope(), maximum_results).unwrap()
}
