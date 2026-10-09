use worth_store_io_scheduler::foreground_reservation::admitted_point_read_reservation_for_security_scope_for_certification_test;
use worth_store_physical_format::{
    PhysicalFutureChunkId, PhysicalFutureChunkReference, PhysicalGeneration,
};
use worth_store_physical_isolation::{
    stable_physical_read_plan_for_certification_test, ChunkMigrationReadInterlockPlan,
    FutureChunkStabilityBasis, PhysicalChunkStabilityPlaceholder,
};
use worth_store_tiering::ColdPlacementState;

use crate::lifecycle::generation_registry_test_support::{
    lifecycle_receipt_for_publication, root_publication,
};
use crate::placement::admission::test_support::{admit_external_placement, admit_inline_placement};
use crate::{BlobAuthorityClassification, LifecycleReceipt};

use super::{
    BlobPlacementMovementAuthority, BlobPlacementMovementColdOutcome, BlobPlacementMovementDenial,
    BlobPlacementMovementFreshness, BlobPlacementMovementReadPlanBasis,
    BlobPlacementMovementRequest,
};

#[test]
fn movement_plan_retains_lower_read_plan_without_executing_bytes() {
    let lifecycle = lifecycle();
    let read_plan = read_plan();
    let source = admit_inline_placement(lifecycle.reachability());
    let target = admit_external_placement(lifecycle.reachability());
    let reservation = admitted_point_read_reservation_for_security_scope_for_certification_test(
        lifecycle.declaration().security_metadata().identity(),
    );
    let plan = BlobPlacementMovementAuthority::for_planning()
        .plan_movement(BlobPlacementMovementRequest::new(
            lifecycle,
            source,
            target,
            read_plan,
            reservation.into(),
            BlobPlacementMovementColdOutcome::from_state(ColdPlacementState::HotAvailable),
            BlobPlacementMovementFreshness::Current,
        ))
        .expect("mechanism movement eligibility should admit");
    assert_eq!(plan.read_plan(), read_plan);
    assert_eq!(plan.read_plan().completion(), read_plan.completion());
}

#[test]
fn movement_plan_requires_read_plan_bookkeeping() {
    let lifecycle = lifecycle();
    let source = admit_inline_placement(lifecycle.reachability());
    let target = admit_external_placement(lifecycle.reachability());
    let reservation = admitted_point_read_reservation_for_security_scope_for_certification_test(
        lifecycle.declaration().security_metadata().identity(),
    );
    let denied = BlobPlacementMovementAuthority::for_planning()
        .plan_movement(BlobPlacementMovementRequest::without_movement_read_plan(
            lifecycle,
            source,
            target,
            reservation.into(),
            BlobPlacementMovementColdOutcome::from_state(ColdPlacementState::HotAvailable),
            BlobPlacementMovementFreshness::Current,
        ))
        .expect_err("missing lower read plan should deny planning");
    assert!(matches!(
        denied,
        BlobPlacementMovementDenial::MissingMovementReadPlan { .. }
    ));
}

fn lifecycle() -> LifecycleReceipt {
    let case = "phase2.movement.plan";
    let (publication, stored_digest) = root_publication(case);
    lifecycle_receipt_for_publication(
        case,
        publication.chunk_tree_root().clone(),
        publication.logical_content_digest().clone(),
        stored_digest,
        BlobAuthorityClassification::StoreOwnedPhysicalBlob,
    )
}

fn read_plan() -> BlobPlacementMovementReadPlanBasis {
    let plan = stable_physical_read_plan_for_certification_test(12);
    let barrier = plan.reachability_barrier();
    let root = plan.root();
    let epoch = root.future_chunk_publication_epoch_placeholder().epoch();
    let reference = PhysicalFutureChunkReference::stability_placeholder(
        PhysicalFutureChunkId::from_raw(17).unwrap(),
        PhysicalGeneration::from_raw(1).unwrap(),
    );
    let basis = FutureChunkStabilityBasis::from_stability_receipt(reference, epoch, barrier);
    let placeholder =
        PhysicalChunkStabilityPlaceholder::admit_with_epoch(reference, epoch, basis).unwrap();
    BlobPlacementMovementReadPlanBasis::from_completed_plan_and_interlock(
        plan.into_execution_ready_handle().complete_plan(),
        ChunkMigrationReadInterlockPlan::admit(placeholder).unwrap(),
    )
}
