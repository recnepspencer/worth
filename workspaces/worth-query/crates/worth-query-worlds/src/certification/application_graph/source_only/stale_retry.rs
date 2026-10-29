//! One native registered attempt loses its basis, then retries its original meaning.
pub use super::{
    fixture,
    host::Host,
    readback::{observe_head, read_retention},
};
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use std::{num::NonZeroUsize, time::Duration};
pub use worth_query_host::facade::{
    application_entry::*,
    primary_graph::{
        WorthQueryApplicationCommitDenialKind as Kind,
        WorthQueryApplicationCommitDenialStage as Stage, WorthQueryApplicationUncommitted as U,
        WorthQueryInvariantExecutionDenialKind as InvariantKind,
    },
};

#[path = "stale_retry/race.rs"]
pub mod race;
pub use race::{assert_stale, stale_owner};
