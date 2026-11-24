//! Original-source reconstruction is DATA processing under one admitted operation.
pub mod admission_projection;
pub mod branches;
pub use admission_projection::CapturedRelationalCheckpointAdmissionImage;
pub mod canonical;
pub mod capture;
pub mod contracts;
pub mod covered_prefix;
pub mod current_metadata;
pub mod destination_members;
pub mod draft;
pub mod interim;
pub mod metadata;
pub mod original_image;
pub mod prepare;
pub mod regions;
pub mod roots;
pub mod schemas;
pub mod source;
pub mod symbols;
pub mod verified_originals;
use crate::durability::data::RelationalNativeCheckpoint;
pub use interim::{
    CapturedNativeInterimAdmission, CapturedNativeInterimComponent, CapturedNativeInterimOrigin,
};
use worth_relational_contracts::internal_port::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;

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

fn root_stop(
    cause: worth_relational_contracts::internal_port::branch::root_capture_denial::RelationalBranchRootCaptureDenial,
) -> Denial {
    match cause {
        worth_relational_contracts::internal_port::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Request(cause) => Denial::Request(cause),
        worth_relational_contracts::internal_port::branch::root_capture_denial::RelationalBranchRootCaptureDenial::StorageUnavailable(cause) => cause.into(),
        cause => Denial::Durability(worth_relational_contracts::internal_port::durability::data::recovery_errors::DurabilityError::new(
            worth_relational_contracts::internal_port::durability::data::recovery_errors::RecoveryFailureClass::CorruptCanonicalArtifact,
            format!("original all-root reconstruction disagrees: {cause:?}"),
        )),
    }
}
