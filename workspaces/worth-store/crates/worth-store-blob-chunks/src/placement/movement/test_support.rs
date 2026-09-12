use worth_store::physical_runtime::stability::{
    stable_physical_read_receipt_for_certification_test, StablePhysicalReadReceipt,
};
use worth_store_contracts::StableDigest;
use worth_store_io_scheduler::foreground_reservation::{
    admitted_point_read_reservation_for_certification_test,
    admitted_point_read_reservation_for_security_scope_for_certification_test,
    ForegroundReservationReceipt,
};
use worth_store_physical_format::{
    PhysicalFutureChunkId, PhysicalFutureChunkReference, PhysicalGeneration,
};
use worth_store_physical_isolation::{
    physical_placement_movement_execution_for_certification_test,
    stable_physical_read_plan_for_certification_test, ChunkMigrationReadInterlockPlan,
    FutureChunkStabilityBasis, PhysicalChunkStabilityPlaceholder,
    PhysicalPlacementMovementExecutionReceipt,
};
use worth_store_tiering::ColdPlacementState;

use crate::lifecycle::generation_registry_test_support::{
    lifecycle_receipt_for_publication, lifecycle_receipt_for_publication_with_bytes,
    root_publication, root_publication_with_bytes,
};
use crate::placement::admission::test_support::{
    admit_cold_placement, admit_external_placement, admit_inline_placement,
};
use crate::{
    AdmittedBlobPlacement, BlobAuthorityClassification, BlobStreamingVerifiedRead, ChunkTreeRoot,
    LifecycleReceipt, LogicalContentDigest,
};

use super::{
    BlobMovementVerifiedReadEvidence, BlobPlacementMovementAuthority,
    BlobPlacementMovementColdOutcome, BlobPlacementMovementForegroundReservation,
    BlobPlacementMovementFreshness, BlobPlacementMovementPhysicalExecutionIntent,
    BlobPlacementMovementReadHold, BlobPlacementMovementRequest,
};

pub(crate) struct MovementCase {
    pub(crate) lifecycle: LifecycleReceipt,
    pub(crate) source: AdmittedBlobPlacement,
    pub(crate) target: AdmittedBlobPlacement,
    pub(crate) read: BlobMovementVerifiedReadEvidence,
}

pub(crate) fn movement_case(case: &str) -> MovementCase {
    let returned_lifecycle = lifecycle(case);
    let source = admit_inline_placement(returned_lifecycle.reachability());
    let target = admit_external_placement(returned_lifecycle.reachability());
    let read_hold = movement_read_hold();
    let read_lifecycle = lifecycle(case);
    let read_source = admit_inline_placement(read_lifecycle.reachability());
    let read_target = admit_external_placement(read_lifecycle.reachability());
    let read_reservation = admitted_reservation_for_lifecycle(&read_lifecycle);
    let plan = BlobPlacementMovementAuthority::store_owned()
        .plan_movement(BlobPlacementMovementRequest::new(
            read_lifecycle,
            read_source,
            read_target,
            read_hold,
            read_reservation.into(),
            BlobPlacementMovementColdOutcome::from_state(ColdPlacementState::HotAvailable),
            BlobPlacementMovementFreshness::Current,
        ))
        .expect("movement case should admit read basis");
    let streaming_read = streaming_read_for_plan(&plan, read_hold.guarded_bytes());
    let read = BlobMovementVerifiedReadEvidence::from_streaming_verified_read(
        &plan,
        read_hold,
        &streaming_read,
    )
    .expect("movement case should verify streaming read basis");
    MovementCase {
        lifecycle: returned_lifecycle,
        source,
        target,
        read,
    }
}

pub(crate) fn lifecycle(case: &str) -> LifecycleReceipt {
    let (publication, stored_digest) = root_publication(case);
    lifecycle_receipt_for_publication(
        case,
        publication.chunk_tree_root().clone(),
        publication.logical_content_digest().clone(),
        stored_digest,
        BlobAuthorityClassification::StoreOwnedPhysicalBlob,
    )
}

pub(crate) fn lifecycle_with_bytes(case: &str, bytes: &[u8]) -> LifecycleReceipt {
    let (publication, stored_digest) = root_publication_with_bytes(case, bytes);
    lifecycle_receipt_for_publication_with_bytes(
        case,
        publication.chunk_tree_root().clone(),
        publication.logical_content_digest().clone(),
        stored_digest,
        BlobAuthorityClassification::StoreOwnedPhysicalBlob,
        bytes,
    )
}

pub(crate) fn plan_current(
    case: MovementCase,
) -> Result<super::AdmittedBlobPlacementMovementPlan, super::BlobPlacementMovementDenial> {
    let reservation = admitted_reservation_for_lifecycle(&case.lifecycle);
    BlobPlacementMovementAuthority::store_owned().plan_movement(BlobPlacementMovementRequest::new(
        case.lifecycle,
        case.source,
        case.target,
        movement_read_hold(),
        reservation.into(),
        BlobPlacementMovementColdOutcome::from_state(ColdPlacementState::HotAvailable),
        BlobPlacementMovementFreshness::Current,
    ))
}

pub(crate) fn stale_request(case: MovementCase) -> BlobPlacementMovementRequest {
    let reservation = admitted_reservation_for_lifecycle(&case.lifecycle);
    BlobPlacementMovementRequest::new(
        case.lifecycle,
        case.source,
        case.target,
        movement_read_hold(),
        reservation.into(),
        BlobPlacementMovementColdOutcome::from_state(ColdPlacementState::HotAvailable),
        BlobPlacementMovementFreshness::Stale,
    )
}

pub(crate) fn missing_read_hold_request(case: MovementCase) -> BlobPlacementMovementRequest {
    let reservation = admitted_reservation_for_lifecycle(&case.lifecycle);
    BlobPlacementMovementRequest::without_movement_read_hold(
        case.lifecycle,
        case.source,
        case.target,
        reservation.into(),
        BlobPlacementMovementColdOutcome::from_state(ColdPlacementState::HotAvailable),
        BlobPlacementMovementFreshness::Current,
    )
}

pub(crate) fn unavailable_cold_request(case: MovementCase) -> BlobPlacementMovementRequest {
    let reservation = admitted_reservation_for_lifecycle(&case.lifecycle);
    BlobPlacementMovementRequest::new(
        case.lifecycle,
        case.source,
        case.target,
        movement_read_hold(),
        reservation.into(),
        BlobPlacementMovementColdOutcome::from_state(ColdPlacementState::ColdUnavailable),
        BlobPlacementMovementFreshness::Current,
    )
}

pub(crate) fn violated_reservation_request(case: MovementCase) -> BlobPlacementMovementRequest {
    let violated = copied_scope_reservation().observe_interference(3);
    BlobPlacementMovementRequest::new(
        case.lifecycle,
        case.source,
        case.target,
        movement_read_hold(),
        BlobPlacementMovementForegroundReservation::from(violated),
        BlobPlacementMovementColdOutcome::from_state(ColdPlacementState::HotAvailable),
        BlobPlacementMovementFreshness::Current,
    )
}

pub(crate) fn cold_target(case: &MovementCase) -> AdmittedBlobPlacement {
    admit_cold_placement(case.lifecycle.reachability())
}

pub(crate) fn movement_read_hold() -> BlobPlacementMovementReadHold {
    let stable_read = stable_read();
    BlobPlacementMovementReadHold::from_physical_isolation_stable_read_and_movement_interlock(
        stable_read,
        movement_interlock_with_id(17),
    )
}

pub(crate) fn alternate_movement_read_hold() -> BlobPlacementMovementReadHold {
    let stable_read = stable_read();
    BlobPlacementMovementReadHold::from_physical_isolation_stable_read_and_movement_interlock(
        stable_read,
        movement_interlock_with_id(18),
    )
}

pub(crate) fn physical_execution_for_read_hold(
    plan: &super::AdmittedBlobPlacementMovementPlan,
    read_hold: BlobPlacementMovementReadHold,
) -> PhysicalPlacementMovementExecutionReceipt<BlobPlacementMovementPhysicalExecutionIntent> {
    physical_placement_movement_execution_for_certification_test(
        plan.physical_execution_intent(),
        read_hold.movement_interlock(),
    )
}

pub(crate) fn streaming_read_for_plan(
    plan: &super::AdmittedBlobPlacementMovementPlan,
    bytes_read: u64,
) -> BlobStreamingVerifiedRead {
    BlobStreamingVerifiedRead::for_movement_certification_test(
        plan.basis().object_id().clone(),
        plan.basis().generation(),
        plan.basis().chunk_tree_root().clone(),
        plan.basis().logical_content_digest().clone(),
        bytes_read,
    )
}

pub(crate) fn mismatched_streaming_read(bytes_read: u64) -> BlobStreamingVerifiedRead {
    BlobStreamingVerifiedRead::for_movement_certification_test(
        crate::BlobObjectId::from_declared_digest(stable_digest("sha256:phase17-wrong-object")),
        crate::BlobGeneration::published(99),
        ChunkTreeRoot::from_declared_digest(stable_digest("sha256:phase17-wrong-root")),
        LogicalContentDigest::from_declared_digest(stable_digest("sha256:phase17-wrong-logical")),
        bytes_read,
    )
}

pub(crate) fn same_digest_wrong_identity_streaming_read(
    plan: &super::AdmittedBlobPlacementMovementPlan,
    bytes_read: u64,
) -> BlobStreamingVerifiedRead {
    BlobStreamingVerifiedRead::for_movement_certification_test(
        crate::BlobObjectId::from_declared_digest(stable_digest(
            "sha256:phase17-same-digest-wrong-object",
        )),
        crate::BlobGeneration::published(plan.basis().generation().sequence() + 1),
        plan.basis().chunk_tree_root().clone(),
        plan.basis().logical_content_digest().clone(),
        bytes_read,
    )
}

pub(crate) fn stable_read() -> StablePhysicalReadReceipt {
    stable_physical_read_receipt_for_certification_test(12)
}

pub(crate) fn copied_scope_reservation() -> ForegroundReservationReceipt {
    admitted_point_read_reservation_for_certification_test()
}

pub(crate) fn scoped_reservation_for_lifecycle(
    lifecycle: &LifecycleReceipt,
) -> ForegroundReservationReceipt {
    admitted_reservation_for_lifecycle(lifecycle)
}

fn admitted_reservation_for_lifecycle(
    lifecycle: &LifecycleReceipt,
) -> ForegroundReservationReceipt {
    admitted_point_read_reservation_for_security_scope_for_certification_test(
        lifecycle.declaration().security_metadata().identity(),
    )
}

fn movement_interlock_with_id(id: u64) -> ChunkMigrationReadInterlockPlan {
    let plan = stable_physical_read_plan_for_certification_test(12);
    let barrier = plan.reachability_barrier();
    let root = plan.root();
    let epoch = root.future_chunk_publication_epoch_placeholder().epoch();
    let reference = PhysicalFutureChunkReference::stability_placeholder(
        PhysicalFutureChunkId::from_raw(id).expect("future chunk id"),
        PhysicalGeneration::from_raw(1).expect("future chunk generation"),
    );
    let basis = FutureChunkStabilityBasis::from_stability_receipt(reference, epoch, barrier);
    let placeholder = PhysicalChunkStabilityPlaceholder::admit_with_epoch(reference, epoch, basis)
        .expect("future chunk placeholder should admit");
    ChunkMigrationReadInterlockPlan::admit(placeholder)
        .expect("future chunk movement interlock should admit")
}

fn stable_digest(raw: &str) -> StableDigest {
    StableDigest::new(raw).expect("test stable digest should be nonempty")
}
