//! One record domain's kind-prefixed membership and key-shaped commitment.
use super::maintenance::DirectoryError;
use super::paged::{PagedIds, PagedMembership};
use crate::identity::data::{KindId, PartitionId, RecordId};
use crate::internal_port::storage::substrate::digest_index::DigestIndexOwnerPort as _;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::storage::substrate::{DigestIndex, SharedMap};
use std::collections::BTreeSet;
use std::sync::Arc;

/// A resident membership: ordered keys and their commitment.
#[derive(Debug, Clone, Default)]
pub(super) struct ResidentMembership {
    records: SharedMap<u128, ()>,
    digest: DigestIndex,
}

impl ResidentMembership {
    /// Coexistence bound of one materialized entry (map node, digest leaf and
    /// the transient ordered key).
    pub(super) const ENTRY_BYTES: u64 = 256;

    pub(super) fn from_sorted_keys(keys: Vec<u128>) -> Result<Self, DirectoryError> {
        if keys.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(DirectoryError::Corrupt);
        }
        let mut digest = DigestIndex::default();
        for key in &keys {
            digest
                .set(*key, Some([1; 32]))
                .map_err(|_| DirectoryError::Corrupt)?;
        }
        Ok(Self {
            records: SharedMap::from_sorted_unique(keys.into_iter().map(|key| (key, ())).collect()),
            digest,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.records.len()
    }

    pub(super) fn digest(&self) -> [u8; 32] {
        self.digest.digest()
    }

    fn has_kind(&self, selected: KindId) -> bool {
        self.records
            .iter_from(&((selected.0 as u128) << 96))
            .next()
            .is_some_and(|(key, _)| kind(*key) == selected)
    }

    /// Present kinds, seeking once per kind rather than once per record.
    fn kinds(&self) -> BTreeSet<KindId> {
        let mut kinds = BTreeSet::new();
        let mut from = 0_u128;
        while let Some((key, _)) = self.records.iter_from(&from).next() {
            let present = kind(*key);
            kinds.insert(present);
            let Some(next) = present.0.checked_add(1) else {
                break;
            };
            from = (next as u128) << 96;
        }
        kinds
    }
}

#[derive(Debug, Clone)]
pub(super) enum Membership {
    Resident(ResidentMembership),
    Paged(Arc<PagedMembership>),
}

impl Default for Membership {
    fn default() -> Self {
        Self::Resident(ResidentMembership::default())
    }
}

/// One kind's members; paged reads may fail to fault a page.
pub(super) enum MembershipIds<'a, K> {
    Resident(Box<dyn Iterator<Item = RecordId<K>> + 'a>),
    Paged(PagedIds<'a, K>),
}

impl<K: Copy> Iterator for MembershipIds<'_, K> {
    type Item = Result<RecordId<K>, Cause>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Resident(ids) => ids.next().map(Ok),
            Self::Paged(ids) => ids.next(),
        }
    }
}

impl Membership {
    pub(super) fn digest(&self) -> [u8; 32] {
        match self {
            Self::Resident(resident) => resident.digest(),
            Self::Paged(paged) => paged.digest(),
        }
    }

    pub(super) fn pages(&self) -> Option<&Arc<super::paged::RecordDirectoryPages>> {
        match self {
            Self::Resident(_) => None,
            Self::Paged(paged) => Some(paged.pages()),
        }
    }

    /// Whether this domain is still backed by cold directory pages.
    pub(super) fn is_paged(&self) -> bool {
        matches!(self, Self::Paged(_))
    }

    pub(super) fn has_kind(&self, kind: KindId) -> bool {
        match self {
            Self::Resident(resident) => resident.has_kind(kind),
            Self::Paged(paged) => paged.has_kind(kind),
        }
    }

    /// Kinds whose membership may differ from `previous`: exact for resident
    /// deltas, otherwise every kind present on either side.
    pub(super) fn changed_kinds(&self, previous: &Self) -> BTreeSet<KindId> {
        match (self, previous) {
            (Self::Resident(current), Self::Resident(previous)) => delta(current, previous),
            (Self::Paged(current), Self::Paged(previous)) if current.same_image(previous) => {
                BTreeSet::new()
            }
            (Self::Resident(current), Self::Paged(previous)) if previous.resident().is_some() => {
                delta(current, previous.resident().expect("checked above"))
            }
            _ => {
                let mut kinds = self.kind_set();
                kinds.extend(previous.kind_set());
                kinds
            }
        }
    }

    fn kind_set(&self) -> BTreeSet<KindId> {
        match self {
            Self::Resident(resident) => resident.kinds(),
            Self::Paged(paged) => paged.kinds().iter().copied().collect(),
        }
    }

    pub(super) fn ids<K: Copy>(
        &self,
        selected: KindId,
        partition: PartitionId,
    ) -> MembershipIds<'_, K> {
        let resident = match self {
            Self::Resident(resident) => resident,
            Self::Paged(paged) => match paged.resident() {
                Some(resident) => resident,
                None => return MembershipIds::Paged(paged.ids(selected, partition)),
            },
        };
        MembershipIds::Resident(Box::new(
            resident
                .records
                .iter_from(&((selected.0 as u128) << 96))
                .take_while(move |(key, _)| kind(**key) == selected)
                .map(move |(key, _)| identity(*key, partition)),
        ))
    }

    /// Every key in order, materializing a paged domain once.
    pub(super) fn sorted_keys(
        &self,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
    ) -> Result<Vec<u128>, DirectoryError> {
        let resident = self.resident(prepare)?;
        prepare(resident.len() as u64, resident.len() as u64 * 16)?;
        Ok(resident.records.keys().copied().collect())
    }

    fn resident(
        &self,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
    ) -> Result<&ResidentMembership, DirectoryError> {
        match self {
            Self::Resident(resident) => Ok(resident),
            Self::Paged(paged) => paged.materialize(prepare),
        }
    }

    pub(super) fn cold(
        lane: &ReconstructionLane<'_>,
        entries: Vec<u128>,
    ) -> Result<Self, DirectoryError> {
        let digests: Vec<_> = entries.iter().map(|key| (*key, [1; 32])).collect();
        let digest = DigestIndex::cold(lane, &digests).map_err(|_| DirectoryError::Corrupt)?;
        Ok(Self::Resident(ResidentMembership {
            records: SharedMap::from_sorted_unique(
                entries.into_iter().map(|key| (key, ())).collect(),
            ),
            digest,
        }))
    }

    /// Replaces a paged domain with its verified resident image.
    pub(super) fn make_resident(
        &mut self,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
    ) -> Result<(), DirectoryError> {
        if let Self::Paged(paged) = self {
            // Shared with every clone of `paged`, so deltas stay structural.
            *self = Self::Resident(paged.materialize(prepare)?.clone());
        }
        Ok(())
    }

    pub(super) fn update(
        &mut self,
        key: u128,
        present: bool,
        prepare: &mut dyn FnMut(u64, u64) -> Result<(), DirectoryError>,
    ) -> Result<(), DirectoryError> {
        self.make_resident(prepare)?;
        let Self::Resident(resident) = self else {
            unreachable!("paged membership materialized above")
        };
        if resident.records.contains_key(&key) == present {
            return Ok(());
        }
        let (units, digest_bytes) = resident.digest.edit_quote(key);
        let bytes = resident
            .records
            .adoption_allocation_bound()
            .and_then(|bytes| bytes.checked_add(digest_bytes))
            .ok_or(DirectoryError::Corrupt)?;
        prepare(
            units + resident.records.navigation_work_bound() as u64,
            bytes,
        )?;
        if present {
            resident.records.insert(key, ());
        } else {
            resident.records.remove(&key);
        }
        resident
            .digest
            .set(key, present.then_some([1; 32]))
            .map_err(|_| DirectoryError::Corrupt)
    }
}

fn delta(current: &ResidentMembership, previous: &ResidentMembership) -> BTreeSet<KindId> {
    current
        .records
        .changed_keys_since(&previous.records)
        .into_iter()
        .map(kind)
        .collect()
}

pub(super) fn key<K: Copy>(kind: KindId, record: RecordId<K>) -> u128 {
    ((kind.0 as u128) << 96)
        | ((record.local_slot_value() as u128) << 32)
        | record.generation_value() as u128
}
pub(super) fn kind(key: u128) -> KindId {
    KindId((key >> 96) as u32)
}
pub(super) fn identity<K>(key: u128, partition: PartitionId) -> RecordId<K> {
    RecordId::new(partition, (key >> 32) as u64, key as u32)
}
