//! Moved whole from `domain_computation::primary_graph::application_attempt::fact`: the items below and every impl they own.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryApplicationAdjacencyDirection {
    Outgoing,
    Incoming,
}
