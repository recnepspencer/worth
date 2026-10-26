//! Bounded, iterative reconstruction of earlier metadata from history members.
use super::region_image::{
    RegionHistoryBase, RegionHistoryImage, RegionHistoryRow, RelationalRegionImage,
    RelationalRegionImageKind,
};
use super::RelationalResidencyDemandFailure as Failure;
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
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) trait HistoryRowKind {
    type Metadata: Clone + Serialize;
    fn rows(image: &RegionHistoryImage) -> &[RegionHistoryRow<Self::Metadata>];
}

impl HistoryRowKind for EntityRecordKind {
    type Metadata = VersionedEntityMetadataImage;
    fn rows(image: &RegionHistoryImage) -> &[RegionHistoryRow<Self::Metadata>] {
        &image.entities
    }
}

impl HistoryRowKind for RelationRecordKind {
    type Metadata = VersionedRelationMetadataImage;
    fn rows(image: &RegionHistoryImage) -> &[RegionHistoryRow<Self::Metadata>] {
        &image.relations
    }
}

struct CachedHistory {
    partition: PartitionId,
    schema: [u8; 32],
    kind_commitment: [u8; 32],
    image: Arc<RegionHistoryImage>,
    directories: Vec<Arc<RelationalHistoryMemberDirectory>>,
}

pub(super) struct HistoryFolder<'request, 'scope, 'host> {
    execution: &'request worth_execution::ExecutionRequest<'scope, 'host>,
    partition: PartitionId,
    limit: u64,
    input_bytes: u64,
    output_bytes: u64,
    cache: BTreeMap<RelationalDurableMemberLocator, CachedHistory>,
}

impl<'request, 'scope, 'host> HistoryFolder<'request, 'scope, 'host> {
    pub(super) fn new(
        execution: &'request worth_execution::ExecutionRequest<'scope, 'host>,
        partition: PartitionId,
        limit: u64,
    ) -> Self {
        Self {
            execution,
            partition,
            limit,
            input_bytes: 0,
            output_bytes: 0,
            cache: BTreeMap::new(),
        }
    }

    pub(super) fn read(
        &mut self,
        source: &dyn RelationalDurableColdSource,
        directory: &Arc<RelationalHistoryMemberDirectory>,
        reference: &RegionHistoryBase,
    ) -> Result<Arc<RegionHistoryImage>, Failure> {
        crate::execution::check_request(self.execution).map_err(Failure::Request)?;
        let partition = self.partition;
        if reference.partition_id != partition {
            return Err(unavailable(partition, Cause::Damaged));
        }
        if let Some(cached) = self.cache.get_mut(&reference.member) {
            if cached.partition != reference.partition_id
                || cached.schema != reference.schema_carrier_digest
            {
                return Err(unavailable(partition, Cause::Damaged));
            }
            if cached.kind_commitment != reference.kind_commitment {
                return Err(unavailable(partition, Cause::IntegrityMismatch));
            }
            if !cached
                .directories
                .iter()
                .any(|owner| Arc::ptr_eq(owner, directory))
            {
                verify_directory(&cached.image, directory, reference.member)
                    .map_err(|cause| unavailable(partition, cause))?;
                cached.directories.push(directory.clone());
            }
            return Ok(cached.image.clone());
        }
        let remaining = self.limit - self.input_bytes;
        let length = self
            .input_bytes
            .checked_add(reference.member.length)
            .ok_or_else(|| {
                unavailable(
                    partition,
                    Cause::LimitExceeded {
                        length: u64::MAX,
                        limit: self.limit,
                    },
                )
            })?;
        if length > self.limit {
            return Err(unavailable(
                partition,
                Cause::LimitExceeded {
                    length,
                    limit: self.limit,
                },
            ));
        }
        self.input_bytes = length;
        let bytes = source
            .read_member(reference.member, remaining)
            .map_err(|cause| unavailable(partition, cause))?;
        crate::execution::check_request(self.execution).map_err(Failure::Request)?;
        let frame = RelationalRegionImage::decode(
            &bytes,
            reference.member,
            reference.partition_id,
            reference.schema_carrier_digest,
            RelationalRegionImageKind::History,
        )
        .map_err(|cause| unavailable(partition, cause))?;
        let image: RegionHistoryImage = rmp_serde::from_slice(frame.payload)
            .map_err(|_| unavailable(partition, Cause::Damaged))?;
        super::kind_commitment::verify(&image.kind_projection, reference.kind_commitment)
            .map_err(|cause| unavailable(partition, cause))?;
        super::history_kind_cache::verify(&image).map_err(|cause| unavailable(partition, cause))?;
        verify_directory(&image, directory, reference.member)
            .map_err(|cause| unavailable(partition, cause))?;
        let image = Arc::new(image);
        self.cache.insert(
            reference.member,
            CachedHistory {
                partition: reference.partition_id,
                schema: reference.schema_carrier_digest,
                kind_commitment: reference.kind_commitment,
                image: image.clone(),
                directories: vec![directory.clone()],
            },
        );
        Ok(image)
    }

    pub(super) fn fold_rows<K: HistoryRowKind>(
        &mut self,
        source: &dyn RelationalDurableColdSource,
        directory: &Arc<RelationalHistoryMemberDirectory>,
        root: &RegionHistoryBase,
        rows: usize,
    ) -> Result<Vec<Vec<K::Metadata>>, Failure> {
        let mut result = Vec::new();
        result
            .try_reserve_exact(rows)
            .map_err(|_| unavailable(self.partition, Cause::MemoryExhausted))?;
        for row in 0..rows {
            let mut reference = root.clone();
            reference.row_index = row as u64;
            result.push(self.fold_row::<K>(source, directory, reference)?);
        }
        Ok(result)
    }

    pub(super) fn fold_row<K: HistoryRowKind>(
        &mut self,
        source: &dyn RelationalDurableColdSource,
        directory: &Arc<RelationalHistoryMemberDirectory>,
        mut reference: RegionHistoryBase,
    ) -> Result<Vec<K::Metadata>, Failure> {
        let mut chunks = Vec::new();
        loop {
            let image = self.read(source, directory, &reference)?;
            let index = usize::try_from(reference.row_index)
                .map_err(|_| unavailable(self.partition, Cause::Damaged))?;
            let row = K::rows(&image)
                .get(index)
                .ok_or_else(|| unavailable(self.partition, Cause::Damaged))?;
            let next = row.prefix.clone();
            chunks
                .try_reserve(1)
                .map_err(|_| unavailable(self.partition, Cause::MemoryExhausted))?;
            chunks.push((image, index));
            match next {
                Some(next) => reference = next,
                None => break,
            }
        }
        let mut result = Vec::new();
        for (image, index) in chunks.into_iter().rev() {
            let earlier = &K::rows(&image)[index].earlier;
            for metadata in earlier {
                self.charge_output(metadata)?;
                result
                    .try_reserve(1)
                    .map_err(|_| unavailable(self.partition, Cause::MemoryExhausted))?;
                result.push(metadata.clone());
            }
        }
        Ok(result)
    }

    pub(super) fn charge_output(&mut self, metadata: &impl Serialize) -> Result<(), Failure> {
        crate::execution::check_request(self.execution).map_err(Failure::Request)?;
        let mut counter = OutputLength {
            length: self.output_bytes,
            limit: self.limit,
            exceeded: false,
        };
        let encoded = rmp_serde::encode::write(&mut counter, metadata);
        self.output_bytes = counter.length;
        if counter.exceeded {
            Err(unavailable(
                self.partition,
                Cause::LimitExceeded {
                    length: counter.length,
                    limit: self.limit,
                },
            ))
        } else {
            encoded.map_err(|_| unavailable(self.partition, Cause::Damaged))
        }
    }
}

fn verify_directory(
    image: &RegionHistoryImage,
    directory: &RelationalHistoryMemberDirectory,
    member: RelationalDurableMemberLocator,
) -> Result<(), Cause> {
    if image.direct_bases().as_slice() != directory.bases(member) {
        Err(Cause::Damaged)
    } else {
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

struct OutputLength {
    length: u64,
    limit: u64,
    exceeded: bool,
}
impl std::io::Write for OutputLength {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let Some(length) = self.length.checked_add(bytes.len() as u64) else {
            self.length = u64::MAX;
            self.exceeded = true;
            return Err(std::io::Error::other("expanded history length overflow"));
        };
        self.length = length;
        if self.length > self.limit {
            self.exceeded = true;
            Err(std::io::Error::other(
                "expanded history exceeds its read limit",
            ))
        } else {
            Ok(bytes.len())
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
