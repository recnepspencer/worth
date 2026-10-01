use worth_store_physical_format::{BlobRecordKind, BlobTreeNodeV1, PersistedRecordIdentity};

use super::super::{append::append_blob_record, BlobIngestAllocation, BlobSessionId};
use super::{append_pressure::BlobAppendPressure, resume::RetainedBlobNodes, BlobIngestFailure};
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, ServingPhysicalRuntime,
};

pub(super) struct BlobNodeWriter<'attempt, 'runtime> {
    pub(super) runtime: &'runtime ServingPhysicalRuntime,
    pub(super) placement: AdmittedRecordPlacementPolicy,
    pub(super) session: BlobSessionId,
    pub(super) deadline: PhysicalMutationDeadline,
    pub(super) allocation: &'attempt mut BlobIngestAllocation<'runtime>,
    pub(super) retained: &'attempt mut RetainedBlobNodes,
}

impl BlobNodeWriter<'_, '_> {
    pub(super) fn write(
        &mut self,
        node: BlobTreeNodeV1,
        ordinal: u64,
    ) -> Result<PersistedRecordIdentity, BlobIngestFailure> {
        if let Some(record) = self.retained.select(&node, ordinal)? {
            return Ok(record);
        }
        let encoded = node.encode();
        let _pressure = BlobAppendPressure::admit(self.allocation, encoded.len() as u64)
            .map_err(BlobIngestFailure::Memory)?;
        append_blob_record(
            self.runtime,
            self.placement,
            self.session,
            BlobRecordKind::TreeNode,
            ordinal,
            self.deadline,
            encoded,
        )
        .map_err(|cause| BlobIngestFailure::Append {
            session: self.session,
            kind: BlobRecordKind::TreeNode,
            ordinal,
            cause,
        })
    }
}
