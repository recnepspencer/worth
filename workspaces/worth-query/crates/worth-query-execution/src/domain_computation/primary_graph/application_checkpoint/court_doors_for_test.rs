//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCheckpoint {
    #[doc(hidden)]
    pub fn decode_with_execution_for_test(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        Self::decode_with_execution(self, execution)
    }
}

impl WorthQueryApplicationCheckpoint {
    #[doc(hidden)]
    pub fn decode_for_test(self) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        Self::decode(self)
    }
}

impl DecodedApplicationCheckpoint {
    #[doc(hidden)]
    pub fn accepted_outputs_for_test(
        &self,
    ) -> &Vec<super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity>
    {
        &self.accepted_outputs
    }
}

impl DecodedApplicationCheckpoint {
    #[doc(hidden)]
    pub fn bootstrap_commit_id_for_test(&self) -> &worth_relational::facade::history::CommitId {
        &self.bootstrap_commit_id
    }
}

impl DecodedApplicationCheckpoint {
    #[doc(hidden)]
    pub fn native_for_test(
        &self,
    ) -> &worth_relational::facade::durability::RelationalNativeCheckpoint {
        &self.native
    }
}

impl DecodedApplicationCheckpoint {
    #[doc(hidden)]
    pub fn native_into_for_test(
        self,
    ) -> worth_relational::facade::durability::RelationalNativeCheckpoint {
        self.native
    }
}
