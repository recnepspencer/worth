//! Original-source reconstruction is DATA processing under one admitted operation.
pub(in crate::durability) mod admission_projection;
mod branches;
pub use admission_projection::CapturedRelationalCheckpointAdmissionImage;
mod canonical;
mod capture;
mod contracts;
mod covered_prefix;
mod current_metadata;
mod destination_members;
mod draft;
pub(in crate::durability::authority) mod interim;
mod metadata;
mod original_image;
mod prepare;
mod regions;
mod roots;
mod schemas;
mod source;
mod symbols;
mod verified_originals;
use crate::durability::data::{
    RelationalNativeCheckpoint, RelationalNativeCheckpointCaptureDenial as Denial,
};
pub use interim::{
    AdmittedNativeInterimReceiver, CapturedNativeInterimAdmission, CapturedNativeInterimComponent,
    CapturedNativeInterimOrigin, NativeInterimBootstrapContinuation,
};

/// The all-root producer's strong result, retaining its original read and ACK custody.
/// Descriptive bytes and ordinary captures have no conversion into this result.
#[derive(Debug)]
pub struct CompleteRelationalNativeCheckpoint {
    checkpoint: RelationalNativeCheckpoint,
    initial_predecessor: Option<[u8; 32]>,
    source_artifact: crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact,
    admission_image: CapturedRelationalCheckpointAdmissionImage,
}
impl CompleteRelationalNativeCheckpoint {
    pub fn admission_image(&self) -> &CapturedRelationalCheckpointAdmissionImage {
        &self.admission_image
    }
    /// The recorded predecessor of this producer's accepted initialization.
    pub fn initial_predecessor(&self) -> Option<[u8; 32]> {
        self.initial_predecessor
    }
    pub fn checkpoint(&self) -> &RelationalNativeCheckpoint {
        &self.checkpoint
    }
    pub fn source_artifact(
        &self,
    ) -> &crate::durability::source_initial_inputs::RelationalCompleteSourceArtifact {
        &self.source_artifact
    }
}

fn root_stop(cause: crate::branch::RelationalBranchRootCaptureDenial) -> Denial {
    match cause {
        crate::branch::RelationalBranchRootCaptureDenial::Request(cause) => Denial::Request(cause),
        crate::branch::RelationalBranchRootCaptureDenial::StorageUnavailable(cause) => cause.into(),
        cause => Denial::Durability(crate::durability::data::DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::CorruptCanonicalArtifact,
            format!("original all-root reconstruction disagrees: {cause:?}"),
        )),
    }
}

impl super::DurabilityAuthority<'_> {
    pub(crate) fn complete_native_checkpoint_with_policy(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        placement: crate::durability::data::RelationalCheckpointArtifactPolicy<'_>,
    ) -> Result<CompleteRelationalNativeCheckpoint, Denial> {
        prepare::prepare(self.runtime, execution, placement)
    }
}
#[cfg(test)]
mod tests;
