mod binding;
mod binding_compaction;
mod dirty_basis;
mod footer;
mod stream_header;
mod stream_projection;

pub(in crate::integrity_ingress) use binding::IntegrityAdmittedCheckpointBinding;
pub(in crate::integrity_ingress) use binding_compaction::IntegrityAdmittedCheckpointBindingCompaction;
pub(in crate::integrity_ingress) use dirty_basis::IntegrityAdmittedCheckpointDirtyBasis;
pub(in crate::integrity_ingress) use footer::IntegrityAdmittedCheckpointFooter;
pub(in crate::integrity_ingress) use stream_header::IntegrityAdmittedCheckpointStreamHeader;
pub(crate) use stream_projection::{IntegrityAdmittedCheckpointStream, OwnerCheckpointProjection};

#[cfg(test)]
pub(super) fn owner_valid_compile_contracts() {
    stream_header::owner_valid_compile_contract();
    dirty_basis::owner_valid_compile_contract();
    binding_compaction::owner_valid_compile_contract();
    binding::owner_valid_compile_contract();
    footer::owner_valid_compile_contract();
}
