//! Request-funded, iterative reconstruction of earlier metadata DATA.
use super::region_image::{
    RegionHistoryBase, RegionHistoryImage, RegionHistoryRow, RelationalRegionImage,
    RelationalRegionImageKind,
};
use super::{
    region_load::materialization_funding as funding, RelationalResidencyDemandFailure as Failure,
};
use crate::durability::data::{
    RelationalHistoryMemberDirectory, VersionedEntityMetadataImage, VersionedRelationMetadataImage,
};
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::identity::data::PartitionId;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::substrate::{EntityRecordKind, RelationRecordKind};
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
pub(super) trait HistoryRowKind {
    type Metadata: Clone + serde::Serialize;
    fn rows(image: &RegionHistoryImage) -> &[RegionHistoryRow<Self::Metadata>];
    fn clone_quote(value: &Self::Metadata) -> Result<funding::Quote, Failure>;
}
impl HistoryRowKind for EntityRecordKind {
    type Metadata = VersionedEntityMetadataImage;
    fn rows(image: &RegionHistoryImage) -> &[RegionHistoryRow<Self::Metadata>] {
        &image.entities
    }
    fn clone_quote(value: &Self::Metadata) -> Result<funding::Quote, Failure> {
        funding::aspect_state::entity_metadata(value, 0)
    }
}
impl HistoryRowKind for RelationRecordKind {
    type Metadata = VersionedRelationMetadataImage;
    fn rows(image: &RegionHistoryImage) -> &[RegionHistoryRow<Self::Metadata>] {
        &image.relations
    }
    fn clone_quote(value: &Self::Metadata) -> Result<funding::Quote, Failure> {
        funding::aspect_state::relation_metadata(value, 0)
    }
}
/// The decoded image and its reservation escape the folder together.
pub(super) struct FundedHistoryImage {
    pub(super) image: RegionHistoryImage,
    bases: ExecutionOrderedMap<RelationalDurableMemberLocator, ()>,
    _memory: funding::Funding,
}
pub(super) struct FundedRow<M> {
    pub(super) values: Vec<M>,
    memory: funding::Funding,
}
impl<M> FundedRow<M> {
    pub(super) fn append_current(
        &mut self,
        suffix: &mut Vec<M>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        let count = self
            .values
            .len()
            .checked_add(suffix.len())
            .ok_or_else(funding::overflow)?;
        self.memory.grow(
            funding::array::<M>(count.checked_mul(2).ok_or_else(funding::overflow)?)?,
            request,
        )?;
        funding::work(request, suffix.len() as u64)?;
        self.values.try_reserve_exact(suffix.len()).map_err(|_| {
            Failure::Request(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
        })?;
        self.values.append(suffix);
        Ok(())
    }
}
pub(super) struct FundedRows<M> {
    pub(super) rows: Vec<FundedRow<M>>,
    _memory: funding::Funding,
}
impl<M> FundedRows<M> {
    pub(super) fn len(&self) -> usize {
        self.rows.len()
    }
    pub(super) fn allocate(
        count: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Failure> {
        let memory = funding::Funding::reserve(funding::array::<FundedRow<M>>(count)?, request)?;
        Ok(Self {
            rows: Vec::with_capacity(count),
            _memory: memory,
        })
    }
}
#[derive(Clone)]
struct CachedHistory {
    partition: PartitionId,
    schema: [u8; 32],
    kind_commitment: [u8; 32],
    image: Arc<FundedHistoryImage>,
}
pub(super) struct HistoryFolder<'request, 'scope, 'host> {
    execution: &'request ExecutionRequest<'scope, 'host>,
    partition: PartitionId,
    limit: u64,
    input_bytes: u64,
    output_bytes: u64,
    cache: ExecutionOrderedMap<RelationalDurableMemberLocator, CachedHistory>,
    verified: ExecutionOrderedMap<
        (RelationalDurableMemberLocator, usize),
        Arc<RelationalHistoryMemberDirectory>,
    >,
}
impl<'request, 'scope, 'host> HistoryFolder<'request, 'scope, 'host> {
    pub(super) fn new(
        execution: &'request ExecutionRequest<'scope, 'host>,
        partition: PartitionId,
        limit: u64,
    ) -> Self {
        Self {
            execution,
            partition,
            limit,
            input_bytes: 0,
            output_bytes: 0,
            cache: ExecutionOrderedMap::default(),
            verified: ExecutionOrderedMap::default(),
        }
    }
    pub(super) fn read(
        &mut self,
        source: &dyn RelationalDurableColdSource,
        directory: &Arc<RelationalHistoryMemberDirectory>,
        reference: &RegionHistoryBase,
    ) -> Result<Arc<FundedHistoryImage>, Failure> {
        funding::work(
            self.execution,
            2 * (self.cache.len().max(1).ilog2() as u64 + 1),
        )?;
        if reference.partition_id != self.partition {
            return Err(unavailable(self.partition, Cause::Damaged));
        }
        if let Some(cached) = self.cache.get(&reference.member).cloned() {
            if cached.partition != reference.partition_id
                || cached.schema != reference.schema_carrier_digest
            {
                return Err(unavailable(self.partition, Cause::Damaged));
            }
            if cached.kind_commitment != reference.kind_commitment {
                return Err(unavailable(self.partition, Cause::IntegrityMismatch));
            }
            self.verify_directory(&cached.image, directory, reference.member)?;
            return Ok(cached.image);
        }
        let remaining = self.limit - self.input_bytes;
        let length = self.input_bytes.saturating_add(reference.member.length);
        if length > self.limit {
            return Err(unavailable(
                self.partition,
                Cause::LimitExceeded {
                    length,
                    limit: self.limit,
                },
            ));
        }
        self.input_bytes = length;
        let _read = funding::Funding::reserve(
            funding::vector::<u8>(
                usize::try_from(reference.member.length).map_err(|_| funding::overflow())?,
            )?,
            self.execution,
        )?;
        funding::work(self.execution, reference.member.length)?;
        let bytes = source
            .read_member(reference.member, remaining)
            .map_err(|cause| unavailable(self.partition, cause))?;
        let frame = RelationalRegionImage::decode(
            &bytes,
            reference.member,
            reference.partition_id,
            reference.schema_carrier_digest,
            RelationalRegionImageKind::History,
        )
        .map_err(|cause| unavailable(self.partition, cause))?;
        let quote = funding::region_image::history(funding::Raw(frame.payload), 0, self.execution)
            .map_err(|cause| funding::at_region(cause, self.partition))?;
        let memory = funding::Funding::reserve(
            funding::sum(quote.wire, funding::shared::<FundedHistoryImage>()?)?,
            self.execution,
        )?;
        funding::work(self.execution, frame.payload.len() as u64)?;
        let image: RegionHistoryImage = rmp_serde::from_slice(frame.payload)
            .map_err(|_| unavailable(self.partition, Cause::Damaged))?;
        super::kind_commitment::verify(&image.kind_projection, reference.kind_commitment)
            .map_err(|cause| unavailable(self.partition, cause))?;
        super::history_kind_cache::verify(&image)
            .map_err(|cause| unavailable(self.partition, cause))?;
        let mut bases = ExecutionOrderedMap::default();
        for prefix in image
            .entities
            .iter()
            .filter_map(|row| row.prefix.as_ref())
            .chain(image.relations.iter().filter_map(|row| row.prefix.as_ref()))
        {
            funding::work(self.execution, 1)?;
            if bases.get(&prefix.member).is_none() {
                bases
                    .insert_cloned_native_for_request(&prefix.member, &(), *self.execution)
                    .map_err(funding::ordered)?;
            }
        }
        let image = Arc::new(FundedHistoryImage {
            image,
            bases,
            _memory: memory,
        });
        self.verify_directory(&image, directory, reference.member)?;
        self.cache
            .insert_cloned_native_for_request(
                &reference.member,
                &CachedHistory {
                    partition: reference.partition_id,
                    schema: reference.schema_carrier_digest,
                    kind_commitment: reference.kind_commitment,
                    image: image.clone(),
                },
                *self.execution,
            )
            .map_err(funding::ordered)?;
        Ok(image)
    }
    fn verify_directory(
        &mut self,
        image: &FundedHistoryImage,
        directory: &Arc<RelationalHistoryMemberDirectory>,
        member: RelationalDurableMemberLocator,
    ) -> Result<(), Failure> {
        #[expect(
            clippy::disallowed_methods,
            reason = "Allocation identity only deduplicates DATA verification while the directory Arc stays retained"
        )]
        let identity = Arc::as_ptr(directory) as usize;
        let key = (member, identity);
        funding::work(
            self.execution,
            2 * (self.verified.len().max(1).ilog2() as u64 + 1),
        )?;
        if self.verified.get(&key).is_some() {
            return Ok(());
        }
        const DIRECTORY_NAVIGATION_BOUND: u64 = 11 * usize::BITS as u64;
        funding::work(self.execution, DIRECTORY_NAVIGATION_BOUND)?;
        let expected = directory.bases(member);
        if image.bases.len() != expected.len() {
            return Err(unavailable(self.partition, Cause::Damaged));
        }
        for ((actual, _), expected) in image.bases.iter().zip(expected) {
            funding::work(self.execution, 1)?;
            if actual != expected {
                return Err(unavailable(self.partition, Cause::Damaged));
            }
        }
        self.verified
            .insert_cloned_native_for_request(&key, directory, *self.execution)
            .map_err(funding::ordered)?;
        Ok(())
    }
    pub(super) fn fold_rows<K: HistoryRowKind>(
        &mut self,
        source: &dyn RelationalDurableColdSource,
        directory: &Arc<RelationalHistoryMemberDirectory>,
        root: &RegionHistoryBase,
        rows: usize,
    ) -> Result<FundedRows<K::Metadata>, Failure> {
        let mut result = FundedRows::allocate(rows, self.execution)?;
        while result.len() < rows {
            let mut reference = root.clone();
            reference.row_index = result.len() as u64;
            result
                .rows
                .push(self.fold_row::<K>(source, directory, reference)?);
        }
        Ok(result)
    }
    pub(super) fn fold_row<K: HistoryRowKind>(
        &mut self,
        source: &dyn RelationalDurableColdSource,
        directory: &Arc<RelationalHistoryMemberDirectory>,
        mut reference: RegionHistoryBase,
    ) -> Result<FundedRow<K::Metadata>, Failure> {
        let mut chunks = Vec::new();
        let mut scratch = funding::Funding::default();
        loop {
            funding::work(self.execution, 1)?;
            let image = self.read(source, directory, &reference)?;
            let index = usize::try_from(reference.row_index)
                .map_err(|_| unavailable(self.partition, Cause::Damaged))?;
            let row = K::rows(&image.image)
                .get(index)
                .ok_or_else(|| unavailable(self.partition, Cause::Damaged))?;
            let next = row.prefix.clone();
            if chunks.len() == chunks.capacity() {
                let capacity = chunks.capacity().max(4);
                scratch.grow(
                    funding::array::<(Arc<FundedHistoryImage>, usize)>(
                        capacity.checked_mul(2).ok_or_else(funding::overflow)?,
                    )?,
                    self.execution,
                )?;
                chunks
                    .try_reserve_exact(capacity)
                    .map_err(|_| unavailable(self.partition, Cause::MemoryExhausted))?;
            }
            chunks.push((image, index));
            match next {
                Some(next) => reference = next,
                None => break,
            }
        }
        let mut result = FundedRow {
            values: Vec::new(),
            memory: funding::Funding::default(),
        };
        for (image, index) in chunks.into_iter().rev() {
            for metadata in &K::rows(&image.image)[index].earlier {
                self.charge_output(metadata)?;
                let quote = K::clone_quote(metadata)?;
                result.memory.grow(quote.wire, self.execution)?;
                if result.values.len() == result.values.capacity() {
                    let capacity = result.values.capacity().max(4);
                    result.memory.grow(
                        funding::array::<K::Metadata>(
                            capacity.checked_mul(2).ok_or_else(funding::overflow)?,
                        )?,
                        self.execution,
                    )?;
                    result
                        .values
                        .try_reserve_exact(capacity)
                        .map_err(|_| unavailable(self.partition, Cause::MemoryExhausted))?;
                }
                result.values.push(metadata.clone());
            }
        }
        Ok(result)
    }
    pub(super) fn charge_output(
        &mut self,
        metadata: &impl serde::Serialize,
    ) -> Result<(), Failure> {
        let length = funding::aspect_state::metadata_length(metadata, self.execution)?;
        self.output_bytes = self.output_bytes.saturating_add(length);
        if self.output_bytes > self.limit {
            return Err(unavailable(
                self.partition,
                Cause::LimitExceeded {
                    length: self.output_bytes,
                    limit: self.limit,
                },
            ));
        }
        Ok(())
    }
}
fn unavailable(partition: PartitionId, cause: Cause) -> Failure {
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(partition),
        cause,
    }
    .into()
}
