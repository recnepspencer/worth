//! A decision seals complete-content comparison; candidate construction never rereads it.
pub(in crate::domain_computation::primary_graph) enum CreationDisposition {
    Create,
    Reuse,
}
