//! Real COW continuation backing denial, independently observed at media.

use std::{cell::Cell, num::NonZeroU64};

use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store::physical_runtime::{
    PhysicalLayoutMaintenanceFailure, PhysicalMutationDeadline,
    PhysicalOperationAllocationScope as Scope, PhysicalRecordResidencyFailureReason,
    PhysicalResidencyRetryPosture, ServingPhysicalRuntime,
};
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use super::{admitted_blob_scope, selected_marker, shared_format, world};

const MAINTENANCE_BYTES: u64 = 8 << 20;
// This is the documented retained-owner envelope, not an allocator census:
// 256 bytes of control backing plus five 24-byte record slots.
const RETAINED_BYTES: u64 = 256 + 5 * 24;
const GROWTH_BYTES: u64 = 256 + 10 * 24;

#[test]
fn continuation_backing_denies_before_cow_and_successful_twin_appends() {
    std::thread::Builder::new()
        .name("cow-continuation-backing".to_owned())
        .stack_size(16 << 20)
        .spawn(run)
        .expect("continuation worker")
        .join()
        .expect("continuation worker did not panic");
}

fn run() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_maintenance_scope(
        "c11-cow-continuation-backing",
        shared_format(),
        512,
        NonZeroU64::new(MAINTENANCE_BYTES).unwrap(),
    )
    .expect("genuine 32 MiB operation world with narrower Maintenance scope");
    let scope = admitted_blob_scope("c11.cow.continuation.backing");
    let payload = world::payload();
    let sources = std::array::from_fn(|_| {
        world::publish_one(&world, &scope, &payload);
        selected_marker(world.serving())
    });
    let serving = world.serving();
    let policy = serving.residency_observation().admitted_policy();
    assert_eq!(policy.operation_bytes(), 32 << 20);
    assert_eq!(policy.scope_bytes(Scope::Maintenance), MAINTENANCE_BYTES);
    assert_eq!(maintenance_bytes(serving), 0);
    let denied_before = Cell::new(None);
    let failure = serving
        .certification_try_selected_catalog_continuation(
            sources,
            world.placement(),
            deadline(),
            || {
                // Writer scratch is gone, but replacement backing is still live.
                assert_eq!(maintenance_bytes(serving), RETAINED_BYTES);
                let collision = serving
                    .physical_allocations()
                    .admit_maintenance(
                        NonZeroU64::new(MAINTENANCE_BYTES - RETAINED_BYTES - 1).unwrap(),
                    )
                    .expect("real competing Maintenance owner");
                assert_eq!(maintenance_bytes(serving), MAINTENANCE_BYTES - 1);
                denied_before.set(Some((generation(serving), media_writes(serving))));
                Some(collision)
            },
        )
        .expect_err("third insertion must grow its replacement backing");
    let allocation = match failure {
        PhysicalLayoutMaintenanceFailure::WriterAllocation(allocation) => allocation,
        other => panic!("exact pre-COW growth cause required: {other:?}"),
    };
    assert_eq!(
        allocation.reason(),
        PhysicalRecordResidencyFailureReason::PhysicalPressure
    );
    let pressure = allocation.pressure().expect("typed continuation pressure");
    assert_eq!(pressure.scope(), Scope::Maintenance);
    assert_eq!(pressure.requested(), GROWTH_BYTES);
    assert_eq!(pressure.admitted(), MAINTENANCE_BYTES - 1);
    assert_eq!(pressure.limit(), MAINTENANCE_BYTES);
    assert_eq!(
        pressure.retry_posture(),
        PhysicalResidencyRetryPosture::AfterAllocationRelease
    );
    assert!(!pressure.effect_may_have_started());
    let (root_before, media_before) = denied_before.get().expect("observed before continuation");
    assert_eq!(generation(serving), root_before);
    assert_eq!(media_writes(serving), media_before);
    assert_eq!(
        maintenance_bytes(serving),
        0,
        "consumed chain and collision both dispose"
    );

    let successful_before = Cell::new(None);
    serving
        .certification_try_selected_catalog_continuation(
            sources,
            world.placement(),
            deadline(),
            || {
                assert_eq!(maintenance_bytes(serving), RETAINED_BYTES);
                successful_before.set(Some((generation(serving), media_writes(serving))));
                None
            },
        )
        .expect("same real continuation without pressure must append");
    let (root_before, media_before) = successful_before.get().unwrap();
    assert!(generation(serving) > root_before);
    let after = media_writes(serving);
    assert!(after.2 + after.3 > media_before.2 + media_before.3);
    assert_eq!(maintenance_bytes(serving), 0);
    world.close();
}

fn maintenance_bytes(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .residency_observation()
        .counters()
        .active_operation_bytes_for(Scope::Maintenance)
}

fn generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

fn deadline() -> PhysicalMutationDeadline {
    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
}

fn media_writes(serving: &ServingPhysicalRuntime) -> (u64, u64, u64, u64) {
    let media = serving.media_counters();
    (
        media.append_attempts(),
        media.positioned_write_attempts(),
        media.completed_bytes_for(MediaOperationRole::Append),
        media.completed_bytes_for(MediaOperationRole::PositionedWrite),
    )
}
