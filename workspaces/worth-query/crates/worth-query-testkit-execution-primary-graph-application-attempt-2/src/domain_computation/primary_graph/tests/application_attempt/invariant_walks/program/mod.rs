//! Actual authenticated operations, with their declared reads and effects.

pub use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::program::activation::set_active;
pub use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::program::activation::set_active_on_branch;
pub use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::program::activation::set_node_active;
pub use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::program::edge::replace_next;
pub use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::program::link::link;

#[path = "activation/mod.rs"]
pub mod activation;
#[path = "authorization/mod.rs"]
pub mod authorization;
#[path = "edge/mod.rs"]
pub mod edge;
#[path = "link/mod.rs"]
pub mod link;
