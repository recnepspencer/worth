//! Read and authenticate a retained generation member without warming its cache.
use worth_relational_contracts::internal_port as contracts;
use super::{captured_decode, cold_checkpoint::DeferredIndexImage};
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_storage::internal_port::durability::medium::owned_read_stop::owned_stop;
use contracts::indexes::data::definition::DerivedIndexId;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::checkpoint_image::{CapturedCheckpointGenerations, captured_inputs};
use worth_relational_storage::internal_port::storage::data::member_frame;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;
use contracts::storage::data::storage_unavailable::{RelationalStorageSubject, RelationalStorageUnavailable};
use contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane;
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
