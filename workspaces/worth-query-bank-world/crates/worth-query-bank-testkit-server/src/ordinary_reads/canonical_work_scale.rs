pub use std::time::Instant;

pub use bank_domain::model::{BusinessId, Money};
pub use bank_domain::proposals::BankIdempotencyKey;
pub use bank_domain::schema::InitiateBusinessPayment;
pub use bank_server::{mutations, queries, BankMutationControls, BankReadControls};

pub use super::canonical_scale_fixture::canonical_scale_world;
pub use super::fixture::{
    ordinary_read_world_with_pending_payments, over_budget_discovery_world, principal_id, APPROVER,
    OWNER, RECIPIENT,
};
pub use crate::facade::support::request_scope;

pub fn controls(maximum_results: usize) -> BankReadControls {
    BankReadControls::current(request_scope(), maximum_results)
        .expect("high-operation controls are bounded")
}
