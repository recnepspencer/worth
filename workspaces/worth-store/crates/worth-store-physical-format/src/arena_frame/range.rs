/// Durable arena identity. It locates a container, never a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtentArenaId(u64);

impl ExtentArenaId {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A finite byte range in one arena; publication and protection grant its use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtentArenaRange {
    arena: ExtentArenaId,
    offset: u64,
    length: u64,
}

impl ExtentArenaRange {
    pub const fn new(arena: ExtentArenaId, offset: u64, length: u64) -> Option<Self> {
        if length == 0 || offset.checked_add(length).is_none() {
            None
        } else {
            Some(Self {
                arena,
                offset,
                length,
            })
        }
    }

    pub const fn arena(self) -> ExtentArenaId {
        self.arena
    }
    pub const fn offset(self) -> u64 {
        self.offset
    }
    pub const fn length(self) -> u64 {
        self.length
    }
    pub const fn end(self) -> u64 {
        self.offset + self.length
    }
    pub const fn overlaps(self, other: Self) -> bool {
        self.arena.get() == other.arena.get()
            && self.offset < other.end()
            && other.offset < self.end()
    }
}
