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

impl<'a> worth_query_execution_lineage::facade::ports::ManagedViewPublicationRequestPort<'a>
    for super::super::publication::ViewPublicationBasis<'a>
{
    type Change = super::super::publication::ViewChange;
    type Prepared = PreparedManagedViewPublication;

    fn prepare_on_managed_views(
        self,
        port: &dyn worth_query_execution_lineage::facade::ports::ManagedDerivedViewPort,
        changes: &[Self::Change],
        maximum_work_units: usize,
    ) -> Self::Prepared {
        port.as_any()
            .downcast_ref::<ManagedDerivedViewRegistry>()
            .expect("primary graph carries ManagedDerivedViewRegistry")
            .prepare_publication_for_test(self, changes, maximum_work_units)
    }
}
