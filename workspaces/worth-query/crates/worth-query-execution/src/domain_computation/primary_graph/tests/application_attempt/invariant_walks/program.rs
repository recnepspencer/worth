//! Actual authenticated operations, with their declared reads and effects.
mod activation;
pub(super) mod authorization;
mod edge;

pub(super) use activation::{set_active, set_active_on_branch};
pub(super) use edge::replace_next;
