//! Portable locator retained by the performed publication, independently of its cache.

pub(super) struct NativePriorCheckpointLocator {
    pub(super) producer: String,
    pub(super) source: [u8; 32],
}
