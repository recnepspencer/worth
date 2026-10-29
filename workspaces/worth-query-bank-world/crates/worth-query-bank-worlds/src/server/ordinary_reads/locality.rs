pub use bank_server::{
    queries, BankApplicationOneShotDenialKind, BankApplicationQueryDenial, BankReadControls,
};

pub use super::fixture::{ordinary_read_world, over_budget_discovery_world, OWNER};
pub use crate::facade::server::support::request_scope;

pub fn controls(maximum_results: usize) -> BankReadControls {
    BankReadControls::current(request_scope(), maximum_results).unwrap()
}
