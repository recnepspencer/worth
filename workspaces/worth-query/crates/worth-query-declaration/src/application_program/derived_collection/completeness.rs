use super::ApplicationCollectionLineage;

/// The pure value contract checked by a collection's installed computation.
///
/// The value retains per-measure completeness and contributor lineage. Pending
/// evidence is not empty membership: an unavailable physical input may leave a
/// physical measure pending while a membership-only count remains complete.
/// This check observes the completed value, grants no publication authority and
/// receives no graph reader. Runtime evidence must establish the domain claim.
pub trait ApplicationCollectionCompleteness: 'static {
    type Value: 'static;
    type Stopped: 'static;
    type Lineage: ApplicationCollectionLineage;

    const IDENTITY: &'static str;

    fn check(value: &Self::Value) -> Result<(), Self::Stopped>;
}
