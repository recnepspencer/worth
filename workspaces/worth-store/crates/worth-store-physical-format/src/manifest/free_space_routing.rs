use crate::{DurableFrameDenial, RecordAllocationClass, RecordFreeSpaceManifestEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FreeSpaceKey {
    class: RecordAllocationClass,
    owner: u64,
    offset: u64,
}

impl FreeSpaceKey {
    pub const fn inline(owner: u64) -> Option<Self> {
        if owner == 0 {
            None
        } else {
            Some(Self {
                class: RecordAllocationClass::InlinePage,
                owner,
                offset: 0,
            })
        }
    }
    pub const fn arena(arena: crate::ExtentArenaId, offset: u64) -> Self {
        Self {
            class: RecordAllocationClass::ExtentArena,
            owner: arena.get(),
            offset,
        }
    }
    pub const fn offset(self) -> u64 {
        self.offset
    }
    pub const fn class(self) -> RecordAllocationClass {
        self.class
    }
    pub const fn owner(self) -> u64 {
        self.owner
    }
}

impl From<RecordFreeSpaceManifestEntry> for FreeSpaceKey {
    fn from(entry: RecordFreeSpaceManifestEntry) -> Self {
        match entry.region() {
            crate::RecordFreeSpaceRegion::Inline(value) => {
                Self::inline(value.owner()).expect("admitted inline frontier")
            }
            crate::RecordFreeSpaceRegion::Arena(range) => {
                Self::arena(range.arena(), range.offset())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeSpaceBlockReference {
    generation: u64,
    block: u64,
    level: u16,
    checksum: u32,
    first: FreeSpaceKey,
    last: FreeSpaceKey,
}

impl FreeSpaceBlockReference {
    pub fn new(
        generation: u64,
        block: u64,
        level: u16,
        checksum: u32,
        first: FreeSpaceKey,
        last: FreeSpaceKey,
    ) -> Option<Self> {
        (generation != 0 && block != 0 && first <= last).then_some(Self {
            generation,
            block,
            level,
            checksum,
            first,
            last,
        })
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
    pub const fn block(self) -> u64 {
        self.block
    }
    pub const fn level(self) -> u16 {
        self.level
    }
    pub const fn checksum(self) -> u32 {
        self.checksum
    }
    pub const fn first(self) -> FreeSpaceKey {
        self.first
    }
    pub const fn last(self) -> FreeSpaceKey {
        self.last
    }
    pub fn contains(self, key: FreeSpaceKey) -> bool {
        self.first <= key && key <= self.last
    }
}

pub(crate) fn encode_reference(target: &mut [u8], reference: FreeSpaceBlockReference) {
    target[..8].copy_from_slice(&reference.generation().to_le_bytes());
    target[8..16].copy_from_slice(&reference.block().to_le_bytes());
    target[16..18].copy_from_slice(&reference.level().to_le_bytes());
    target[20..24].copy_from_slice(&reference.checksum().to_le_bytes());
    encode_key(&mut target[24..48], reference.first());
    encode_key(&mut target[48..72], reference.last());
}

pub(crate) fn decode_reference(bytes: &[u8]) -> Option<FreeSpaceBlockReference> {
    if bytes[18..20] != [0; 2] {
        return None;
    }
    FreeSpaceBlockReference::new(
        read_u64(bytes, 0),
        read_u64(bytes, 8),
        u16::from_le_bytes(bytes[16..18].try_into().ok()?),
        u32::from_le_bytes(bytes[20..24].try_into().ok()?),
        decode_key(&bytes[24..48])?,
        decode_key(&bytes[48..72])?,
    )
}

fn encode_key(target: &mut [u8], key: FreeSpaceKey) {
    target[0] = key.class() as u8;
    target[8..16].copy_from_slice(&key.owner().to_le_bytes());
    target[16..24].copy_from_slice(&key.offset().to_le_bytes());
}

fn decode_key(bytes: &[u8]) -> Option<FreeSpaceKey> {
    if bytes[1..8] != [0; 7] {
        return None;
    }
    match bytes[0] {
        1 if read_u64(bytes, 16) == 0 => FreeSpaceKey::inline(read_u64(bytes, 8)),
        2 => Some(FreeSpaceKey::arena(
            crate::ExtentArenaId::new(read_u64(bytes, 8))?,
            read_u64(bytes, 16),
        )),
        _ => None,
    }
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeSpaceRoutingDenial {
    Frame(DurableFrameDenial),
    Malformed,
    IdentityOrCapacity,
    InvalidReference,
    CanonicalOrder,
}
