use crate::physical_runtime::CompletedPhysicalRecoveryFreshReopen;

/// Store-owned terminal for the one cleanup opportunity attached to a fresh
/// reopen.
///
/// Cleanup may complete, defer, or have no candidates. In every case this
/// value consumes the fresh-reopen authority so the same reopen cannot admit
/// another cleanup batch before recovered-runtime construction.
pub struct ClosedPhysicalRecoveryCleanup {
    reopen: CompletedPhysicalRecoveryFreshReopen,
    descriptive_plan_identity: [u8; 32],
    authority_plan_identity: Option<[u8; 32]>,
    live_media_handle_delta: u64,
}

impl ClosedPhysicalRecoveryCleanup {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.reopen.owned_heap_bytes()
    }

    pub(in crate::physical_runtime) const fn new(
        reopen: CompletedPhysicalRecoveryFreshReopen,
        descriptive_plan_identity: [u8; 32],
        authority_plan_identity: Option<[u8; 32]>,
        live_media_handle_delta: u64,
    ) -> Self {
        Self {
            reopen,
            descriptive_plan_identity,
            authority_plan_identity,
            live_media_handle_delta,
        }
    }

    pub const fn descriptive_plan_identity(&self) -> [u8; 32] {
        self.descriptive_plan_identity
    }

    pub const fn authority_plan_identity(&self) -> Option<[u8; 32]> {
        self.authority_plan_identity
    }

    pub const fn live_media_handle_delta(&self) -> u64 {
        self.live_media_handle_delta
    }

    pub(in crate::physical_runtime) fn into_reopen(self) -> CompletedPhysicalRecoveryFreshReopen {
        self.reopen
    }
}
