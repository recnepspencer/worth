use worth_store_contracts::DurableArtifactFamilyId;

use crate::{IndexedThroughBlobPublication, PersistedRecordIdentity};

const MAGIC: &[u8; 8] = b"WRC11IDX";
const VERSION_V1: u8 = 1;
const VERSION_V2: u8 = 2;
const PREFIX_V1_BYTES: usize = 76;
const PREFIX_V2_BYTES: usize = 101;
const ENTRY_BYTES: usize = 26;
pub const MAX_DERIVED_FAMILY_ROOTS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DerivedFamilyRootEntry {
    family: DurableArtifactFamilyId,
    root_record: PersistedRecordIdentity,
}

impl DerivedFamilyRootEntry {
    pub fn new(
        family: DurableArtifactFamilyId,
        root_record: PersistedRecordIdentity,
    ) -> Option<Self> {
        family_tag(family).map(|_| Self {
            family,
            root_record,
        })
    }

    pub const fn family(self) -> DurableArtifactFamilyId {
        self.family
    }

    pub const fn root_record(self) -> PersistedRecordIdentity {
        self.root_record
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedFamilyRootDirectoryV1 {
    entries: Box<[DerivedFamilyRootEntry]>,
    indexed_through_blob_publication: Option<IndexedThroughBlobPublication>,
    indexed_through_quarantine: Option<PersistedRecordIdentity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedFamilyDirectoryDenial {
    WrongMagic,
    UnsupportedVersion,
    InvalidLength,
    EntryLimit,
    UnknownFamily,
    NonCanonicalOrder,
    InvalidRecordIdentity,
    ReservedFieldNonZero,
}

impl DerivedFamilyRootDirectoryV1 {
    pub fn new(entries: Vec<DerivedFamilyRootEntry>) -> Result<Self, DerivedFamilyDirectoryDenial> {
        validate_entries(&entries)?;
        Ok(Self {
            entries: entries.into_boxed_slice(),
            indexed_through_blob_publication: None,
            indexed_through_quarantine: None,
        })
    }

    pub fn with_indexed_through(mut self, publication: IndexedThroughBlobPublication) -> Self {
        self.indexed_through_blob_publication = Some(publication);
        self
    }

    pub const fn indexed_through_blob_publication(&self) -> Option<IndexedThroughBlobPublication> {
        self.indexed_through_blob_publication
    }

    pub fn with_indexed_through_quarantine(
        mut self,
        quarantine: Option<PersistedRecordIdentity>,
    ) -> Self {
        self.indexed_through_quarantine = quarantine;
        self
    }

    pub const fn indexed_through_quarantine(&self) -> Option<PersistedRecordIdentity> {
        self.indexed_through_quarantine
    }

    pub fn entries(&self) -> &[DerivedFamilyRootEntry] {
        &self.entries
    }

    pub fn find(&self, family: DurableArtifactFamilyId) -> Option<PersistedRecordIdentity> {
        let tag = family_tag(family)?;
        self.entries
            .binary_search_by_key(&tag, |entry| family_tag(entry.family).unwrap())
            .ok()
            .map(|index| self.entries[index].root_record)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![0; PREFIX_V2_BYTES + self.entries.len() * ENTRY_BYTES];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8] = VERSION_V2;
        bytes[9] = self.entries.len() as u8;
        if let Some(publication) = self.indexed_through_blob_publication {
            bytes[10] = 1;
            bytes[12..28].copy_from_slice(&publication.record().allocation_epoch());
            bytes[28..36].copy_from_slice(&publication.record().ordinal().to_le_bytes());
            bytes[36..44].copy_from_slice(&publication.root_generation().to_le_bytes());
            bytes[44..76].copy_from_slice(&publication.encoded_digest());
        }
        if let Some(quarantine) = self.indexed_through_quarantine {
            bytes[76] = 1;
            bytes[77..93].copy_from_slice(&quarantine.allocation_epoch());
            bytes[93..101].copy_from_slice(&quarantine.ordinal().to_le_bytes());
        }
        for (entry, frame) in self
            .entries
            .iter()
            .zip(bytes[PREFIX_V2_BYTES..].chunks_exact_mut(ENTRY_BYTES))
        {
            frame[..2].copy_from_slice(&family_tag(entry.family).unwrap().to_le_bytes());
            frame[2..18].copy_from_slice(&entry.root_record.allocation_epoch());
            frame[18..26].copy_from_slice(&entry.root_record.ordinal().to_le_bytes());
        }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DerivedFamilyDirectoryDenial> {
        if bytes.len() < PREFIX_V1_BYTES || &bytes[..8] != MAGIC {
            return Err(DerivedFamilyDirectoryDenial::WrongMagic);
        }
        let prefix_bytes = match bytes[8] {
            VERSION_V1 => PREFIX_V1_BYTES,
            VERSION_V2 => PREFIX_V2_BYTES,
            _ => return Err(DerivedFamilyDirectoryDenial::UnsupportedVersion),
        };
        if bytes.len() < prefix_bytes {
            return Err(DerivedFamilyDirectoryDenial::InvalidLength);
        }
        if bytes[11] != 0 || bytes[10] > 1 {
            return Err(DerivedFamilyDirectoryDenial::ReservedFieldNonZero);
        }
        let publication = if bytes[10] == 0 {
            if bytes[12..76] != [0; 64] {
                return Err(DerivedFamilyDirectoryDenial::ReservedFieldNonZero);
            }
            None
        } else {
            let record = PersistedRecordIdentity::new(
                bytes[12..28].try_into().unwrap(),
                u64::from_le_bytes(bytes[28..36].try_into().unwrap()),
            )
            .ok_or(DerivedFamilyDirectoryDenial::InvalidRecordIdentity)?;
            Some(
                IndexedThroughBlobPublication::new(
                    u64::from_le_bytes(bytes[36..44].try_into().unwrap()),
                    record,
                    bytes[44..76].try_into().unwrap(),
                )
                .ok_or(DerivedFamilyDirectoryDenial::InvalidRecordIdentity)?,
            )
        };
        let count = bytes[9] as usize;
        if count > MAX_DERIVED_FAMILY_ROOTS {
            return Err(DerivedFamilyDirectoryDenial::EntryLimit);
        }
        if bytes.len() != prefix_bytes + count * ENTRY_BYTES {
            return Err(DerivedFamilyDirectoryDenial::InvalidLength);
        }
        let quarantine = if prefix_bytes == PREFIX_V2_BYTES {
            if bytes[76] > 1 {
                return Err(DerivedFamilyDirectoryDenial::ReservedFieldNonZero);
            }
            if bytes[76] == 0 {
                if bytes[77..101] != [0; 24] {
                    return Err(DerivedFamilyDirectoryDenial::ReservedFieldNonZero);
                }
                None
            } else {
                Some(
                    PersistedRecordIdentity::new(
                        bytes[77..93].try_into().unwrap(),
                        u64::from_le_bytes(bytes[93..101].try_into().unwrap()),
                    )
                    .ok_or(DerivedFamilyDirectoryDenial::InvalidRecordIdentity)?,
                )
            }
        } else {
            None
        };
        let mut entries = Vec::with_capacity(count);
        for frame in bytes[prefix_bytes..].chunks_exact(ENTRY_BYTES) {
            let tag = u16::from_le_bytes(frame[..2].try_into().unwrap());
            let family = family_from_tag(tag).ok_or(DerivedFamilyDirectoryDenial::UnknownFamily)?;
            let record = PersistedRecordIdentity::new(
                frame[2..18].try_into().unwrap(),
                u64::from_le_bytes(frame[18..26].try_into().unwrap()),
            )
            .ok_or(DerivedFamilyDirectoryDenial::InvalidRecordIdentity)?;
            entries.push(DerivedFamilyRootEntry {
                family,
                root_record: record,
            });
        }
        let mut directory = Self::new(entries)?;
        directory.indexed_through_blob_publication = publication;
        directory.indexed_through_quarantine = quarantine;
        Ok(directory)
    }
}

fn validate_entries(
    entries: &[DerivedFamilyRootEntry],
) -> Result<(), DerivedFamilyDirectoryDenial> {
    if entries.len() > MAX_DERIVED_FAMILY_ROOTS {
        return Err(DerivedFamilyDirectoryDenial::EntryLimit);
    }
    let mut previous = 0;
    for entry in entries {
        let tag = family_tag(entry.family).ok_or(DerivedFamilyDirectoryDenial::UnknownFamily)?;
        if tag <= previous {
            return Err(DerivedFamilyDirectoryDenial::NonCanonicalOrder);
        }
        previous = tag;
    }
    Ok(())
}

fn family_tag(family: DurableArtifactFamilyId) -> Option<u16> {
    match family {
        DurableArtifactFamilyId::BlobCatalog => Some(1),
        DurableArtifactFamilyId::DedupeIndex => Some(2),
        _ => None,
    }
}

fn family_from_tag(tag: u16) -> Option<DurableArtifactFamilyId> {
    match tag {
        1 => Some(DurableArtifactFamilyId::BlobCatalog),
        2 => Some(DurableArtifactFamilyId::DedupeIndex),
        _ => None,
    }
}

#[cfg(test)]
#[path = "derived_family_root_directory/tests.rs"]
mod tests;
