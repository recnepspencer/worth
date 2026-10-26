//! Actual authenticated operations, with their declared reads and effects.
mod activation;
pub(in crate::domain_computation::primary_graph) mod authorization;
mod edge;
mod link;

pub(super) use activation::{set_active, set_active_on_branch, set_node_active};
pub(super) use edge::replace_next;
pub(super) use link::link;
