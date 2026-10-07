use std::num::{NonZeroU64, NonZeroUsize};

/// Host-selected finite capacity for one native transaction's staged intents.
///
/// These resources bound overlay backing and touched read/write loci. They do
/// not change candidate admission, publication-record limits, prepared-root
/// capacity, invariant rules, or the source/schema authority of the application.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryTransactionStagingResources {
    maximum_overlay_bytes: NonZeroU64,
    maximum_footprint_loci: NonZeroUsize,
}

impl WorthQueryTransactionStagingResources {
    pub const fn bounded(
        maximum_overlay_bytes: NonZeroU64,
        maximum_footprint_loci: NonZeroUsize,
    ) -> Self {
        Self {
            maximum_overlay_bytes,
            maximum_footprint_loci,
        }
    }

    pub const fn maximum_overlay_bytes(self) -> u64 {
        self.maximum_overlay_bytes.get()
    }

    pub const fn maximum_footprint_loci(self) -> usize {
        self.maximum_footprint_loci.get()
    }
}
