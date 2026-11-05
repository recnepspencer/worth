//! Moved whole from `domain_computation::primary_graph::application_attempt::fact::movement`: the items below and every impl they own.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Movement {
    Unmoved,
    Moved,
}
