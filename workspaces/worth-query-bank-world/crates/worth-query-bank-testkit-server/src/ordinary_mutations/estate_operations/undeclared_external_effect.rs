//! Money movement that declares no external effect pays nothing for one.
//!
//! The rail is real and installed; an ordinary transfer simply never reaches
//! it. This is the counter-assertion that makes the dispatch proof meaningful:
//! the outbox row, the transport call, and the dispatch canonicalization are
//! all consequences of a declaration, not of committing a mutation.

pub use std::sync::Arc;

pub use bank_domain::model::Money;
pub use bank_domain::proposals::BankIdempotencyKey;
pub use bank_domain::schema::SendMoney;
pub use bank_server::{mutations, BankMutationControls};

pub use super::external_effect_dispatch::rail_transport::{spawn_rail, BankEstateRailTransport};
pub use crate::facade::fixture::{ordinary_read_world, principal_id, OWNER, RECIPIENT};
pub use crate::facade::support::request_scope;
