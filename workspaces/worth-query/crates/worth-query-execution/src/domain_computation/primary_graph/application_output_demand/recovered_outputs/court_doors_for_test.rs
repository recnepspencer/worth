//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding;

impl WorthQueryRecoveredOutputs {
    #[doc(hidden)]
    pub fn from_checkpoint_for_test(
        checkpoint: DecodedApplicationCheckpoint,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        readmit: impl FnMut(
            &super::super::WorthQueryAcceptedOutputCheckpointIdentity,
        ) -> Result<
            std::sync::Arc<
                crate::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryApplicationOutputCorrespondence,
            >,
            CheckpointDecodeDenial,
        >,
    ) -> Result<Self, Denial> {
        Self::from_checkpoint(checkpoint, execution, readmit)
    }
}

impl WorthQueryRecoveredOutputs {
    #[doc(hidden)]
    pub fn from_records_for_test(
        records: Vec<WorthQueryReadmittedAcceptedOutput>,
        metadata: CheckpointMetadataFunding,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial> {
        Self::from_records(records, metadata, execution)
    }
}

impl WorthQueryRecoveredOutputs {
    #[doc(hidden)]
    pub fn matching_source_partition_for_test(
        &self,
        scope: WorthQueryOperationScopeEntityBinding,
        partition: [u8; 32],
    ) -> impl Iterator<Item = &WorthQueryReadmittedAcceptedOutput> {
        Self::matching_source_partition(self, scope, partition)
    }
}
