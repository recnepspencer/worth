//! Moved whole from `domain_computation::primary_graph::application_attempt::precondition_binding`: the items below and every impl they own.

#[derive(Debug)]
pub(in crate::domain_computation) enum EmptyMutationPreconditionAdmissionStop {
    Nonempty,
}
