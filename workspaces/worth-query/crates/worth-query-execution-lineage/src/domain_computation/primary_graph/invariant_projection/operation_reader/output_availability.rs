//! Non-driving observation value types for installed required outputs.
use super::*;

pub mod denial;
pub use denial::WorthQueryOutputAvailabilityDenial;

/// Result of a non-driving observation through the original checked reader.
/// Its source and completion dependencies remain in that reader; this value
/// neither starts production nor substitutes for a fresh native currentness read.
pub struct WorthQueryObservedOutputAvailability<Schema, Entity> {
    state: WorthQueryOutputAvailabilityState<Schema, Entity>,
}

/// A current scoped output identity or positively qualified pending source.
/// Refused, missing and unqualified sources are reported as typed denials,
/// rather than being converted into `Pending`.
pub enum WorthQueryOutputAvailabilityState<Schema, Entity> {
    Current(WorthQueryInvariantEntityIdentity<Schema, Entity>),
    Pending,
}

impl<Schema, Entity> WorthQueryObservedOutputAvailability<Schema, Entity> {
    pub fn from_state(state: WorthQueryOutputAvailabilityState<Schema, Entity>) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &WorthQueryOutputAvailabilityState<Schema, Entity> {
        &self.state
    }
}
