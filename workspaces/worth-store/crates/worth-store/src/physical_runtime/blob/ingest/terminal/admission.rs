use crate::physical_runtime::{
    durability::{CompletedDurableCheckpointWitness, PhysicalBlobTerminalAdmissionDenial},
    AdmittedBlobScope, AdmittedRecordPlacementPolicy, BlobSessionId, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::BlobAbandonmentReasonV1;

use super::super::resume::BlobResumeToken;
use super::{
    publication, selection, BlobTerminalDisposition, BlobTerminalFailure, BlobTerminalLimits,
    BlobTerminalReceipt,
};

impl ServingPhysicalRuntime {
    pub(in crate::physical_runtime) fn abort_blob_ingest(
        &self,
        token: BlobResumeToken,
        scope: &AdmittedBlobScope,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        limits: BlobTerminalLimits,
    ) -> Result<BlobTerminalReceipt, BlobTerminalFailure> {
        self.terminalize_blob_ingest(
            token,
            scope,
            placement,
            deadline,
            limits,
            TerminalIntent::ExplicitAbort,
        )
    }

    pub(in crate::physical_runtime) fn expire_blob_ingest(
        &self,
        token: BlobResumeToken,
        scope: &AdmittedBlobScope,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        limits: BlobTerminalLimits,
    ) -> Result<BlobTerminalReceipt, BlobTerminalFailure> {
        self.terminalize_blob_ingest(
            token,
            scope,
            placement,
            deadline,
            limits,
            TerminalIntent::CheckpointExpiry,
        )
    }

    fn terminalize_blob_ingest(
        &self,
        token: BlobResumeToken,
        scope: &AdmittedBlobScope,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        limits: BlobTerminalLimits,
        intent: TerminalIntent,
    ) -> Result<BlobTerminalReceipt, BlobTerminalFailure> {
        self.blobs()
            .map_err(|_| BlobTerminalFailure::ServingRequiresInspection)?;
        if token.store != self.store_identity().bytes() {
            return Err(BlobTerminalFailure::ForeignStore);
        }
        let session = BlobSessionId::from_selected(token.session);
        let (reader, mut claim, completed_checkpoint) = self
            .claimed_blob_reader_with_checkpoint(session)
            .map_err(|cause| BlobTerminalFailure::Claim(cause.into()))?;
        let selected = selection::select(reader, token, scope, limits, completed_checkpoint)?;
        if let Some(record) = selected.existing {
            return Ok(BlobTerminalReceipt::new(
                session,
                record,
                BlobTerminalDisposition::AlreadyAbandoned,
            ));
        }
        let reason = intent.reason(completed_checkpoint, selected.maximum_checkpoint_sequence)?;
        self.blobs()
            .map_err(|_| BlobTerminalFailure::ServingRequiresInspection)?;
        self.promote_blob_terminal(&mut claim)
            .map_err(|cause| match cause {
                PhysicalBlobTerminalAdmissionDenial::Claim(cause) => {
                    BlobTerminalFailure::Claim(cause.into())
                }
                PhysicalBlobTerminalAdmissionDenial::PendingPublication => {
                    BlobTerminalFailure::PendingPublication
                }
            })?;
        // Both the selected-root protection and terminal claim remain live
        // until the physical append has completed or returned an uncertain
        // effect fate. Neither guard's Drop asserts durable abandonment.
        let record = publication::publish(self, token, reason, placement, deadline)?;
        Ok(BlobTerminalReceipt::new(
            session,
            record,
            BlobTerminalDisposition::NewlyAbandoned,
        ))
    }
}

#[derive(Clone, Copy)]
enum TerminalIntent {
    ExplicitAbort,
    CheckpointExpiry,
}

impl TerminalIntent {
    fn reason(
        self,
        completed: Option<CompletedDurableCheckpointWitness>,
        maximum: u64,
    ) -> Result<BlobAbandonmentReasonV1, BlobTerminalFailure> {
        match self {
            Self::ExplicitAbort => Ok(BlobAbandonmentReasonV1::ExplicitAbort),
            Self::CheckpointExpiry => {
                let selected = completed.map_or(0, |witness| witness.sequence().get());
                let Some(witness) = completed.filter(|witness| witness.sequence().get() > maximum)
                else {
                    return Err(BlobTerminalFailure::NotExpired {
                        selected_checkpoint_sequence: selected,
                        maximum_checkpoint_sequence: maximum,
                    });
                };
                Ok(BlobAbandonmentReasonV1::CheckpointExpired {
                    checkpoint_sequence: witness.sequence(),
                })
            }
        }
    }
}
