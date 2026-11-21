//! Sealed test doors over this module's private members.
use super::*;

impl ManagedDerivedViewRegistry {
    #[doc(hidden)]
    pub fn prepare_publication_for_test(
        &self,
        basis: super::super::publication::ViewPublicationBasis<'_>,
        changes: &[super::super::publication::ViewChange],
        maximum_work_units: usize,
    ) -> PreparedManagedViewPublication {
        Self::prepare_publication(self, basis, changes, maximum_work_units)
    }
}
