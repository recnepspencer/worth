//! Descriptive posture cannot construct or transfer the installed legality proof.
use super::{WorthQueryObservedResultSet, WorthQueryObservedSource};
impl<Q> WorthQueryObservedSource<Q> {
    pub fn has_verified_realized_footprint(&self) -> bool {
        self.source_meaning
            .footprint()
            .maintained_source()
            .is_some_and(|source| source.has_verified_realized_footprint())
    }
}
impl<Q> WorthQueryObservedResultSet<Q> {
    pub fn has_verified_realized_footprint(&self) -> bool {
        self.source.has_verified_realized_footprint()
    }
}
