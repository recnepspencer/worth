//! Declarative residency ceilings for one runtime's working set (Store M4 D1).
//!
//! A runtime with a budget binds every cold-backed region it reopens to one
//! of two pools and evicts unborrowed regions to stay within it; genuine
//! saturation refuses with typed `MemoryExhausted`. Head and historical pools
//! are separate, so historical reads can never evict or starve head regions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidencyBudget {
    /// Resident bytes for branch heads and the current mirror.
    pub head_working_set_bytes: u64,
    /// Resident bytes for retained historical roots and time-travel reads.
    pub historical_working_set_bytes: u64,
    /// Encoded member bytes concurrently being read from cold storage.
    pub max_in_flight_fill_bytes: u64,
    /// Upper bound on one rolling-window step, in bytes of declared demand.
    pub streaming_window_bytes: u32,
}
