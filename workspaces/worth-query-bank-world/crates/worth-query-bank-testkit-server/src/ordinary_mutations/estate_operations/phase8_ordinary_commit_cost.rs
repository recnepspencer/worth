//! R8.51 — ordinary commit pays nothing for aftermath when no external or
//! recovery work is required.
//!
//! Gate 8.2 discipline: prove the aftermath machinery is live and available,
//! then assert the ordinary path's external_dispatch / undo_admission /
//! redo_admission counters are exactly zero.

pub use std::sync::Arc;

pub use bank_domain::model::Money;
pub use bank_domain::proposals::BankIdempotencyKey;
pub use bank_domain::schema::SendMoney;
pub use bank_server::{mutations, BankMutationControls};

pub use super::external_effect_dispatch::rail_transport::{spawn_rail, BankEstateRailTransport};
pub use crate::facade::fixture::{ordinary_read_world, principal_id, OWNER, RECIPIENT};
pub use crate::facade::support::request_scope;
