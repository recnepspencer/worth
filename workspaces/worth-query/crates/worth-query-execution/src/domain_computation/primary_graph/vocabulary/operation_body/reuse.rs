//! Moved whole from `domain_computation::primary_graph::operation_body::reuse`: the items below and every impl they own.

pub(in crate::domain_computation::primary_graph) enum CreationDisposition {
    Create,
    Reuse,
}
