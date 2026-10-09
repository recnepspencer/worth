use worth_store_physical_format::{
    BlobAbandonmentReasonV1, BlobRecordKind, BlobSessionAbandonedV1, PersistedRecordIdentity,
};

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, BlobSessionId, PhysicalMutationDeadline, ServingPhysicalRuntime,
};

use super::super::{resume::BlobResumeToken, BlobTerminalFailure};
use crate::physical_runtime::blob::append::append_blob_record;

pub(super) fn publish(
    runtime: &ServingPhysicalRuntime,
    token: BlobResumeToken,
    reason: BlobAbandonmentReasonV1,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<PersistedRecordIdentity, BlobTerminalFailure> {
    let terminal = BlobSessionAbandonedV1::new(
        token.store,
        token.session,
        token.declaration_record,
        token.declaration_digest,
        reason,
    )
    .map_err(BlobTerminalFailure::Format)?;
    append_blob_record(
        runtime,
        placement,
        BlobSessionId::from_selected(token.session),
        BlobRecordKind::SessionAbandoned,
        0,
        deadline,
        terminal.encode(),
    )
    .map_err(BlobTerminalFailure::Append)
}
