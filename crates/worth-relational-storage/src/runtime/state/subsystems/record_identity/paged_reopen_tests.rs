//! `G7` paged reopen court (`S2`, `S10`): a `CHK24` allocator reopened from
//! its four paged tables retains only roots, faults one partition's pages on
//! first use under the same `ResidencyBudget` at 1x and 10x cardinality, and
//! allocates exactly what the eager restore of the same image allocates.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::paged_base::PagedIdentityBase;
use super::*;
use crate::durability::medium::RelationalDurableMemberLocator as Locator;
use crate::storage::data::record_identity_image::*;
use crate::storage::data::record_identity_pages::encode_record_identity;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::chunked::member_io::{ChunkMemberSink, ChunkMemberSource};
use crate::storage::residency::working_set::ResidencyBudget;
use crate::storage::residency::working_set::{ResidencyPoolKind, WorkingSetAccountant};

#[derive(Default)]
struct Members {
    bytes: Mutex<BTreeMap<Locator, Vec<u8>>>,
    reads: AtomicU64,
}

impl ChunkMemberSink for &Members {
    fn put(&mut self, locator: Locator, bytes: Vec<u8>) -> Result<(), Cause> {
        self.bytes.lock().unwrap().insert(locator, bytes);
        Ok(())
    }
}

impl ChunkMemberSource for Members {
    fn read_range(&self, member: Locator, offset: u64, target: &mut [u8]) -> Result<usize, Cause> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        let members = self.bytes.lock().unwrap();
        let bytes = members.get(&member).ok_or(Cause::MissingMember)?;
        let tail = &bytes[offset as usize..];
        let count = tail.len().min(target.len());
        target[..count].copy_from_slice(&tail[..count]);
        Ok(count)
    }
}

const BUDGET: ResidencyBudget = ResidencyBudget {
    head_working_set_bytes: 512 * 1024,
    historical_working_set_bytes: 512 * 1024,
    max_in_flight_fill_bytes: 128 * 1024,
    streaming_window_bytes: 64 * 1024,
};
const SLOTS: u64 = 96;
const ENTITY: DurableRecordGenerationClass = DurableRecordGenerationClass::Entity;

/// Per partition: slots `[0, SLOTS)` allocated once or twice; slots `0 mod 4`
/// reusable, `1 mod 4` reclaimed-and-pending, `3 mod 4` append-pending.
fn image(partitions: u32) -> DurableRecordIdentityState {
    let (mut generations, mut reusable, mut frontiers, mut pending) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for partition in 0..partitions {
        let partition_id = PartitionId(partition);
        for slot in 0..SLOTS {
            let generation = 1 + (slot % 3) as u32;
            generations.push(DurableRecordGenerationHighWater {
                class: ENTITY,
                partition_id,
                slot,
                generation,
            });
            let origin = match slot % 4 {
                0 => {
                    reusable.push(DurableReusableRecordSlot {
                        class: ENTITY,
                        partition_id,
                        slot,
                    });
                    continue;
                }
                1 => DurableRecordReservationOrigin::Reclaimed {
                    prior_generation: generation - 1,
                },
                3 => DurableRecordReservationOrigin::AppendFrontier,
                _ => continue,
            };
            pending.push(DurablePendingRecordReservation {
                class: ENTITY,
                partition_id,
                slot,
                generation,
                origin,
            });
        }
        frontiers.push(DurableRecordSlotFrontier {
            class: ENTITY,
            partition_id,
            next_slot: SLOTS,
        });
    }
    DurableRecordIdentityState::current_with_memory(generations, reusable, frontiers, pending, None)
}

/// The eager restore every pre-`CHK24` reopen performed.
fn eager(image: &DurableRecordIdentityState) -> RecordIdentitySubsystem {
    let owner = RecordIdentitySubsystem::default();
    let class = |_| RecordAllocationClass::Entity;
    for row in &image.generation_high_water {
        owner.restore_generation(
            class(row.class),
            row.partition_id,
            row.slot as usize,
            row.generation,
        );
    }
    for row in &image.append_frontiers {
        owner.restore_frontier(class(row.class), row.partition_id, row.next_slot as usize);
    }
    for row in &image.reusable_slots {
        owner.restore_reusable(class(row.class), row.partition_id, row.slot as usize);
    }
    for row in &image.pending_reservations {
        let origin = match row.origin {
            DurableRecordReservationOrigin::AppendFrontier => {
                RecordAllocationOrigin::AppendFrontier
            }
            DurableRecordReservationOrigin::Reclaimed { prior_generation } => {
                RecordAllocationOrigin::Reclaimed { prior_generation }
            }
        };
        owner
            .restore_pending(
                class(row.class),
                row.partition_id,
                row.slot as usize,
                row.generation,
                origin,
            )
            .unwrap();
    }
    owner
}

fn paged(
    image: &DurableRecordIdentityState,
) -> (
    RecordIdentitySubsystem,
    Arc<Members>,
    Arc<WorkingSetAccountant>,
) {
    let members = Arc::new(Members::default());
    let paged = encode_record_identity(image, &mut &*members).unwrap();
    let accountant = WorkingSetAccountant::new(BUDGET);
    let base = PagedIdentityBase::open(paged, accountant.clone(), members.clone());
    (
        RecordIdentitySubsystem::from_paged(base),
        members,
        accountant,
    )
}

fn reserve(owner: &RecordIdentitySubsystem, partition: u32) -> (usize, u32) {
    let mut batch = owner.begin_allocations();
    let allocation = batch
        .reserve(RecordAllocationClass::Entity, PartitionId(partition))
        .unwrap();
    let answer = (allocation.slot, allocation.generation);
    std::mem::forget(batch);
    answer
}

#[test]
fn paged_reopen_retains_only_roots_and_faults_one_partition_under_one_budget_at_1x_and_10x() {
    let mut touched = Vec::new();
    for scale in [1_u32, 10] {
        let image = image(16 * scale);
        assert_eq!(image.pending_reservations.len(), 16 * scale as usize * 48);
        let (owner, members, accountant) = paged(&image);
        // Reopen: no page read, no working-set charge, whatever the cardinality.
        assert_eq!(members.reads.load(Ordering::Relaxed), 0);
        assert_eq!(accountant.charged_bytes(ResidencyPoolKind::Head), 0);
        // First use of one partition faults only its pages.
        assert_eq!(reserve(&owner, 7), reserve(&eager(&image), 7));
        let charged = accountant.charged_bytes(ResidencyPoolKind::Head);
        assert!(charged > 0 && charged <= BUDGET.head_working_set_bytes);
        touched.push((members.reads.load(Ordering::Relaxed), charged));
    }
    let ((reads_1x, bytes_1x), (reads_10x, bytes_10x)) = (touched[0], touched[1]);
    // 10x the partitions, reservations and reusable slots: the touched
    // window grows by at most one directory level per table.
    assert!(reads_10x <= reads_1x + 8, "{reads_1x} -> {reads_10x}");
    assert!(bytes_10x <= bytes_1x * 2, "{bytes_1x} -> {bytes_10x}");
}

#[test]
fn paged_allocator_reuses_slots_and_continues_reservations_exactly_like_eager_restore() {
    let image = image(24);
    let oracle = eager(&image);
    let (owner, _members, _accountant) = paged(&image);
    // Pending reservations survive reopen unchanged until recovery settles.
    assert_eq!(owner.checkpoint_identity(), oracle.checkpoint_identity());
    // Reclaimed-slot reuse and generation advance in faulted partitions.
    for partition in [3, 3, 5, 3] {
        assert_eq!(reserve(&owner, partition), reserve(&oracle, partition));
    }
    // Reclamation into a partition never faulted merges on its first use.
    for subject in [&owner, &oracle] {
        subject.admit_reclaimed(ReclaimedRecordSlot::new(
            RecordAllocationClass::Entity,
            PartitionId(9),
            2,
        ));
    }
    assert_eq!(owner.checkpoint_identity(), oracle.checkpoint_identity());
    assert_eq!(reserve(&owner, 9), reserve(&oracle, 9));
    // Recovery releases unowned checkpoint reservations; partitions faulted
    // afterwards apply the same release (reclaimed slots return reusable).
    owner.release_unowned_reservations();
    oracle.release_unowned_reservations();
    for partition in [11, 11, 3, 20, 11] {
        assert_eq!(reserve(&owner, partition), reserve(&oracle, partition));
    }
    assert_eq!(owner.checkpoint_identity(), oracle.checkpoint_identity());
    // A detached fork shares the pages and owns no checkpoint reservation.
    let (fork, oracle_fork) = (owner.detached(), oracle.detached());
    assert_eq!(reserve(&fork, 17), reserve(&oracle_fork, 17));
    assert_eq!(
        fork.checkpoint_identity(),
        oracle_fork.checkpoint_identity()
    );
}

#[test]
fn unavailable_identity_page_refuses_allocation_without_reserving() {
    let image = image(4);
    let (owner, members, _accountant) = paged(&image);
    members.bytes.lock().unwrap().clear();
    let denial = owner
        .begin_allocations()
        .reserve(RecordAllocationClass::Entity, PartitionId(1))
        .unwrap_err();
    assert!(matches!(
        denial,
        RecordAllocationDenial::IdentityPageUnavailable {
            partition_id: PartitionId(1),
            cause: Cause::MissingMember,
            ..
        }
    ));
    assert_eq!(owner.lock().faulted.len(), 0);
}

#[test]
fn unavailable_identity_page_refuses_replay_and_exact_reservation() {
    use crate::identity::data::EntityId;
    use crate::transactions::data::primitives::RecordRef;
    let image = image(4);
    let (owner, members, _accountant) = paged(&image);
    members.bytes.lock().unwrap().clear();
    let live = CanonicalRecordAllocation::with_origin(
        0,
        RecordRef::Entity(EntityId::new(PartitionId(1), 0, 5)),
        RecordAllocationOrigin::AppendFrontier,
    );
    let replay = owner
        .stage_replay_allocations(vec![live], Vec::new())
        .unwrap_err();
    assert!(matches!(
        replay,
        RecordAllocationDenial::IdentityPageUnavailable {
            partition_id: PartitionId(1),
            cause: Cause::MissingMember,
            ..
        }
    ));
    let exact = owner
        .reserve_exact(
            0,
            RecordAllocationClass::Entity,
            PartitionId(2),
            0,
            5,
            RecordAllocationOrigin::AppendFrontier,
        )
        .unwrap_err();
    assert!(matches!(
        exact,
        RecordAllocationDenial::IdentityPageUnavailable {
            partition_id: PartitionId(2),
            cause: Cause::MissingMember,
            ..
        }
    ));
    assert_eq!(owner.lock().faulted.len(), 0);
}

#[test]
fn working_set_accountant_evicts_unborrowed_identity_pages_for_region_loads() {
    let image = image(16);
    let (owner, _members, accountant) = paged(&image);
    let _ = reserve(&owner, 3);
    assert!(accountant.charged_bytes(ResidencyPoolKind::Head) > 0);
    // Requesting the full Head pool budget forces WorkingSetAccountant::evict_for
    // to sweep the enrolled PageEvictionGroup and reclaim decoded identity pages.
    let charge = accountant
        .reserve(
            ResidencyPoolKind::Head,
            BUDGET.head_working_set_bytes,
            std::ptr::null(),
        )
        .expect("unborrowed identity pages must yield Head budget to region loads");
    drop(charge);
}

#[test]
fn manifest_roster_names_exactly_the_written_pages_without_reading_any() {
    for partitions in [1_u32, 4, 40] {
        let members = Members::default();
        let paged = encode_record_identity(&image(partitions), &mut &members).unwrap();
        let written = members
            .bytes
            .lock()
            .unwrap()
            .keys()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(paged.pages, written);
        assert_eq!(members.reads.load(Ordering::Relaxed), 0);
    }
}
