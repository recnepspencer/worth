//! Read and authenticate a retained generation member without warming its cache.
use super::{captured_decode, cold_checkpoint::DeferredIndexImage};
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::durability::medium::owned_read_stop::owned_stop;
use crate::indexes::data::DerivedIndexId;
use crate::runtime::state::subsystems::indexing::checkpoint_image::{
    captured_inputs, CapturedCheckpointGenerations,
};
use crate::storage::data::member_frame::{self, MemberKind};
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};
use crate::storage::substrate::maintenance::ReconstructionLane;
use worth_execution::ExecutionRequest;

pub(crate) fn read(
    lane: &ReconstructionLane<'_>,
    image: &DeferredIndexImage,
    request: &ExecutionRequest<'_, '_>,
) -> Result<CapturedCheckpointGenerations, RelationalRecoveryStop> {
    let directory = image.directory();
    let key = directory.manifest.derived_indexes;
    let bytes = directory
        .canonical_read
        .read_member(&directory.source, request, key, 1 << 30)
        .map_err(|cause| owned_stop(cause, RelationalStorageSubject::Index(DerivedIndexId(0))))?;
    let payload =
        member_frame::decode(bytes.bytes(), key, MemberKind::DerivedIndex).map_err(|cause| {
            RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Index(DerivedIndexId(0)),
                cause,
            }
        })?;
    let captured = captured_decode::decode(lane, payload, request)?;
    captured_inputs::validate(
        lane,
        image.history(),
        &directory.manifest.index_definitions,
        directory.manifest.index_generation_frontier,
        &captured,
        request,
    )?;
    Ok(captured)
}
