use worth_store_formal_models::{
    map_active_lease, map_expiry, map_identity_reuse_attempt, map_owned_copy,
    map_reclaim_eligibility, map_release, map_revocation, LeaseReclaimAction,
};
use worth_store_physical_isolation::{
    AllocatorPublicationReceipt, BackupReachabilityLeaseIndexSnapshot,
    CrashStableReclaimReuseFence, GenerationAdvanceReceipt, HazardLeaseTable,
    HazardLeaseTableCapacity, LeaseExpiryPosture, PhysicalOrderingContract, PhysicalOrderingSite,
    ReclaimEligibilityProof,
};
use worth_store_test_support::harness::physical_isolation::{
    epoch_scope::current_generation_page_reference, reclaim::ReclaimFixture,
};

pub(in crate::courtroom::protocol_models) fn execute_ordinary_lease_lifecycle(
) -> Vec<LeaseReclaimAction> {
    execute_ordinary_lease_lifecycle_traces()
        .into_iter()
        .flatten()
        .collect()
}

pub(in crate::courtroom::protocol_models) fn execute_ordinary_lease_lifecycle_traces(
) -> Vec<Vec<LeaseReclaimAction>> {
    let mut release = Vec::new();
    execute_release(&mut release);
    let mut revocation = Vec::new();
    execute_revocation(&mut revocation);
    let mut owned_copy = Vec::new();
    execute_owned_copy(&mut owned_copy);
    let mut expiry = Vec::new();
    execute_expiry_without_authority(&mut expiry);
    let mut reclaim_and_reuse = Vec::new();
    execute_reclaim_and_reuse(&mut reclaim_and_reuse);
    vec![release, revocation, owned_copy, expiry, reclaim_and_reuse]
}

fn execute_release(actions: &mut Vec<LeaseReclaimAction>) {
    let fixture = ReclaimFixture::new(4);
    let mut table = one_slot_table();
    let active = table.acquire(fixture.root, fixture.lease).unwrap();
    actions.push(map_active_lease(active));
    actions.push(map_release(table.release(active).unwrap()));
}

fn execute_revocation(actions: &mut Vec<LeaseReclaimAction>) {
    let fixture = ReclaimFixture::new(5);
    let mut table = one_slot_table();
    let active = table.acquire(fixture.root, fixture.lease).unwrap();
    actions.push(map_active_lease(active));
    actions.push(map_revocation(table.revoke(active).unwrap()));
}

fn execute_owned_copy(actions: &mut Vec<LeaseReclaimAction>) {
    let fixture = ReclaimFixture::new(6);
    let mut table = one_slot_table();
    let active = table.acquire(fixture.root, fixture.lease).unwrap();
    actions.push(map_active_lease(active));
    actions.push(map_owned_copy(table.convert_to_owned_copy(active).unwrap()));
}

fn execute_expiry_without_authority(actions: &mut Vec<LeaseReclaimAction>) {
    let fixture = ReclaimFixture::new(7);
    let mut table = one_slot_table();
    let active = table.acquire(fixture.root, fixture.lease).unwrap();
    actions.push(map_active_lease(active));
    actions.push(map_expiry(LeaseExpiryPosture::expired_without_authority(
        active.slot(),
        active.generation(),
    )));
}

fn execute_reclaim_and_reuse(actions: &mut Vec<LeaseReclaimAction>) {
    let generation = 17;
    let fixture = ReclaimFixture::new(generation);
    let mut live_table = one_slot_table();
    let active = live_table
        .acquire(fixture.root, fixture.lease.clone())
        .unwrap();
    actions.push(map_active_lease(active));
    let blocked = ReclaimEligibilityProof::admit(
        fixture.executed_reachability(),
        live_table.live_index_snapshot(),
        BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();
    actions.push(map_reclaim_eligibility(&blocked));
    actions.push(map_release(live_table.release(active).unwrap()));

    let eligible = ReclaimEligibilityProof::admit(
        fixture.executed_reachability(),
        live_table.live_index_snapshot(),
        BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();
    actions.push(map_reclaim_eligibility(&eligible));
    let removal = eligible.admit_reachability_removal().unwrap();
    let allocator = AllocatorPublicationReceipt::from_ordering(
        PhysicalOrderingContract::acquire_release_for(PhysicalOrderingSite::AllocatorPublication),
    )
    .unwrap();
    let admitted_generation = generation_advance(generation, generation + 1);
    let denied_generation = generation_advance(generation + 1, generation + 2);
    let denied = CrashStableReclaimReuseFence::admit_after_reclaim(
        removal.clone(),
        denied_generation,
        allocator,
    );
    actions.push(map_identity_reuse_attempt(&denied));
    let admitted =
        CrashStableReclaimReuseFence::admit_after_reclaim(removal, admitted_generation, allocator);
    actions.push(map_identity_reuse_attempt(&admitted));
}

fn generation_advance(old: u64, new: u64) -> GenerationAdvanceReceipt {
    GenerationAdvanceReceipt::from_identity_reuse(
        current_generation_page_reference(old),
        current_generation_page_reference(new),
        PhysicalOrderingContract::acquire_release_for(PhysicalOrderingSite::GenerationAdvancement),
    )
    .unwrap()
}

fn one_slot_table() -> HazardLeaseTable {
    HazardLeaseTable::with_capacity(HazardLeaseTableCapacity::bounded_slots(1).unwrap())
}
