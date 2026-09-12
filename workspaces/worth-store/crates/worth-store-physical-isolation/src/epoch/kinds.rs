macro_rules! scoped_epoch {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(u64);

        impl $name {
            pub(crate) const fn from_admitted_physical_basis(value: u64) -> Self {
                Self(value)
            }

            pub(crate) const fn has_same_epoch_value(self, other: Self) -> bool {
                self.0 == other.0
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }
    };
}

scoped_epoch!(RootEpoch);
scoped_epoch!(ManifestEpoch);
scoped_epoch!(SegmentEpoch);
scoped_epoch!(ExtentEpoch);
scoped_epoch!(PageEpoch);
scoped_epoch!(ChunkEpoch);

#[cfg(any(test, feature = "certification-authority"))]
pub(crate) fn root_epoch_from_entry_seed(seed: u64) -> RootEpoch {
    RootEpoch::from_admitted_physical_basis(derive_physical_epoch(seed, "root", 0))
}

#[cfg(any(test, feature = "certification-authority"))]
pub(crate) fn manifest_epoch_from_entry_seed(seed: u64) -> ManifestEpoch {
    ManifestEpoch::from_admitted_physical_basis(derive_physical_epoch(seed, "manifest", 0))
}

pub(crate) fn segment_epoch_from_publication(scope: u64, ordinal: u64) -> SegmentEpoch {
    SegmentEpoch::from_admitted_physical_basis(derive_physical_epoch(scope, "segment", ordinal))
}

pub(crate) fn extent_epoch_from_publication(scope: u64, ordinal: u64) -> ExtentEpoch {
    ExtentEpoch::from_admitted_physical_basis(derive_physical_epoch(scope, "extent", ordinal))
}

pub(crate) fn page_epoch_from_publication(scope: u64, ordinal: u64) -> PageEpoch {
    PageEpoch::from_admitted_physical_basis(derive_physical_epoch(scope, "page", ordinal))
}

pub(crate) fn chunk_epoch_from_future_publication(scope: u64, ordinal: u64) -> ChunkEpoch {
    ChunkEpoch::from_admitted_physical_basis(derive_physical_epoch(scope, "future-chunk", ordinal))
}

fn derive_physical_epoch(seed: u64, domain: &str, ordinal: u64) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    mix_u64(&mut hash, seed);
    mix_bytes(&mut hash, domain.as_bytes());
    mix_u64(&mut hash, ordinal);
    if hash == 0 {
        1
    } else {
        hash
    }
}

fn mix_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

fn mix_u64(hash: &mut u64, value: u64) {
    mix_bytes(hash, &value.to_le_bytes());
}
