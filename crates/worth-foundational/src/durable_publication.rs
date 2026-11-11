//! The owner-neutral fate of one durable publication attempt.

/// The medium's answer for the exact artifact offered by its owning runtime.
///
/// `Committed` permits that owner to mint its subject-bound cutover proof.
/// Neither other answer permits live cutover. `Indeterminate` retains recovery
/// custody until the medium establishes the original attempt's fate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurablePublicationFate<Detail> {
    Committed,
    ProvenNoEffect { detail: Detail },
    Indeterminate { detail: Detail },
}
