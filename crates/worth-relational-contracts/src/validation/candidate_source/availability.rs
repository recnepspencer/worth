//! Persisted physical availability of an admitted Source association.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedIndexSourceAvailability {
    Available,
    Unavailable,
}
