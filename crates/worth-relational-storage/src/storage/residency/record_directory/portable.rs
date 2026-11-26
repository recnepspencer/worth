//! The directory's `CHK24` checkpoint form: sealing into paged members and
//! read-free readmission from a recorded descriptor. Ordinary publications
//! never build this image.
use super::{
    maintenance::DirectoryError,
    membership::key,
    paged::{PagedMembership, RecordDirectoryPages},
    *,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::record_directory_pages::{encode_record_directory, MembershipImage};
use crate::storage::overlay::PartitionState;
use crate::storage::residency::chunked::cold_member_port::CollectedMembers;
use crate::storage::residency::chunked::member_io::ChunkMemberSink;
use crate::storage::substrate::maintenance::ReconstructionLane;

/// A sealed directory: the leaf descriptor and the page members it names.
#[derive(Debug)]
pub struct SealedRecordDirectory {
    pub descriptor: Arc<PagedRecordDirectory>,
    pub members: Vec<(RelationalDurableMemberLocator, Vec<u8>)>,
}

/// Computes locators without retaining page bytes.
struct DiscardedPages;

impl ChunkMemberSink for DiscardedPages {
    fn put(
        &mut self,
        _locator: RelationalDurableMemberLocator,
        _bytes: Vec<u8>,
    ) -> Result<(), crate::storage::data::RelationalStorageUnavailableCause> {
        Ok(())
    }
}

impl RegionRecordDirectory {
    /// The leaf descriptor together with its page members, for capture.
    pub fn seal(
        &self,
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SealedRecordDirectory, DirectoryError> {
        lane.recover(execution, |_lane, work| {
            let mut sink = CollectedMembers::default();
            let descriptor = self.encode(&mut sink, &mut |units, bytes| {
                work.prepare(units, bytes).map_err(Into::into)
            })?;
            Ok(SealedRecordDirectory {
                descriptor,
                members: sink.members,
            })
        })
    }

    /// The leaf descriptor without page bytes. A readmitted directory answers
    /// from its recorded descriptor and reads no page.
    pub fn descriptor(
        &self,
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<PagedRecordDirectory>, DirectoryError> {
        if let Some(descriptor) = self.paged.get() {
            return Ok(descriptor.clone());
        }
        lane.recover(execution, |_lane, work| {
            self.encode(&mut DiscardedPages, &mut |units, bytes| {
                work.prepare(units, bytes).map_err(Into::into)
            })
        })
    }

    fn encode(
        &self,
        sink: &mut dyn ChunkMemberSink,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
    ) -> Result<Arc<PagedRecordDirectory>, DirectoryError> {
        let entities = self.entities.sorted_keys(prepare)?;
        let relations = self.relations.sorted_keys(prepare)?;
        let descriptor = encode_record_directory(
            MembershipImage {
                keys: &entities,
                digest: self.entities.digest(),
            },
            MembershipImage {
                keys: &relations,
                digest: self.relations.digest(),
            },
            sink,
        )
        .map_err(DirectoryError::Unavailable)?;
        Ok(self.paged.get_or_init(|| Arc::new(descriptor)).clone())
    }

    /// Readmits a recorded leaf directory without reading any page: kind
    /// presence and commitments answer from `descriptor`; scans fault pages
    /// through `pages`.
    pub fn from_paged(
        partition: PartitionId,
        descriptor: Arc<PagedRecordDirectory>,
        pages: Arc<RecordDirectoryPages>,
    ) -> Arc<Self> {
        let open = |membership| {
            Membership::Paged(Arc::new(PagedMembership::open(membership, pages.clone())))
        };
        Arc::new(Self {
            partition,
            entities: open(&descriptor.entities),
            relations: open(&descriptor.relations),
            paged: OnceLock::from(descriptor),
        })
    }

    /// Serves a paged domain from `partition`'s resident rows when they
    /// reproduce the recorded commitment; otherwise the domain stays paged.
    pub fn adopt_resident(&self, partition: &PartitionState) {
        if partition.partition_id != self.partition
            || [&self.entities, &self.relations]
                .iter()
                .all(|domain| !domain.awaits_adoption())
        {
            return;
        }
        if let Ok((entities, relations)) = partition_keys(partition, &mut |_, _| Ok(())) {
            self.entities.adopt(entities);
            self.relations.adopt(relations);
        }
    }

    pub fn capture(
        lane: &ReconstructionLane<'_>,
        partition: &PartitionState,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
    ) -> Result<Arc<Self>, DirectoryError> {
        let (entities, relations) = partition_keys(partition, prepare)?;
        let total = (entities.len() as u64)
            .checked_add(relations.len() as u64)
            .ok_or(DirectoryError::Corrupt)?;
        prepare(
            total,
            total.saturating_mul(membership::ResidentMembership::ENTRY_BYTES),
        )?;
        Ok(Arc::new(Self {
            partition: partition.partition_id,
            entities: Membership::cold(lane, entities)?,
            relations: Membership::cold(lane, relations)?,
            paged: OnceLock::new(),
        }))
    }

    pub fn verify_member(
        lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expected: &PagedRecordDirectory,
        partition: &PartitionState,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        let failure = |error: DirectoryError| error.into_failure(partition.partition_id);
        let actual = lane
            .recover(execution, |lane, work| {
                Self::capture(lane, partition, &mut |units, bytes| {
                    work.prepare(units, bytes).map_err(Into::into)
                })
            })
            .map_err(failure)?;
        if actual
            .descriptor(lane, execution)
            .map_err(failure)?
            .as_ref()
            != expected
        {
            return Err(failure(DirectoryError::Corrupt));
        }
        Ok(())
    }
}

/// The sorted kind-prefixed membership keys of `partition`'s live rows.
fn partition_keys(
    partition: &PartitionState,
    prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
) -> Result<(Vec<u128>, Vec<u128>), DirectoryError> {
    let entry = std::mem::size_of::<u128>() as u64;
    let mut entities = Vec::new();
    for slot in partition.entity_arena.live_bitset.set_slots() {
        prepare(1, entry)?;
        let record = partition
            .entity_arena
            .get_slot(slot)
            .ok_or(DirectoryError::Corrupt)?;
        entities.push(key(
            record.kind_id().ok_or(DirectoryError::Corrupt)?,
            EntityId::new(partition.partition_id, slot as u64, record.generation()),
        ));
    }
    let mut relations = Vec::new();
    for slot in partition.relation_arena.live_bitset.set_slots() {
        prepare(1, entry)?;
        let record = partition
            .relation_arena
            .get_slot(slot)
            .ok_or(DirectoryError::Corrupt)?;
        relations.push(key(
            record.kind_id().ok_or(DirectoryError::Corrupt)?,
            RelationId::new(partition.partition_id, slot as u64, record.generation()),
        ));
    }
    entities.sort_unstable();
    relations.sort_unstable();
    Ok((entities, relations))
}

/// Read the directory using the same reconstruction oracle as its owner tests.
#[doc(hidden)]
pub trait RegionRecordDirectoryFixturePort {
    fn descriptor_oracle_for_test(
        &self,
        lane: &ReconstructionLane<'_>,
    ) -> Result<Arc<PagedRecordDirectory>, DirectoryError>;
}
impl RegionRecordDirectoryFixturePort for RegionRecordDirectory {
    fn descriptor_oracle_for_test(
        &self,
        lane: &ReconstructionLane<'_>,
    ) -> Result<Arc<PagedRecordDirectory>, DirectoryError> {
        crate::storage::residency::resident_history::run_fixture_reconstruction(
            lane,
            |_lane, work| {
                self.encode(&mut DiscardedPages, &mut |units, bytes| {
                    work.prepare(units, bytes).map_err(Into::into)
                })
            },
        )
    }
}
