//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisSelectionIdentity;

impl WorthQueryObservedSourceMeaningRegistry {
    #[doc(hidden)]
    pub fn intern_for_test(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
    ) -> Option<Arc<WorthQueryObservedSourceMeaning>> {
        Self::intern(self, query, parameters, footprint, selection)
    }
}

impl WorthQueryObservedSourceMeaningRegistry {
    #[doc(hidden)]
    pub fn intern_compared_for_test(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        equal: &mut dyn FnMut(
            &WorthQueryObservedSourceFootprint,
            &WorthQueryObservedSourceFootprint,
        ) -> Result<bool, Cause>,
    ) -> Result<Option<Arc<WorthQueryObservedSourceMeaning>>, Cause> {
        Self::intern_compared(self, query, parameters, footprint, selection, equal)
    }
}

impl WorthQueryObservedSourceMeaningRegistry {
    #[doc(hidden)]
    pub fn intern_result_set_for_test(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
    ) -> Option<Arc<WorthQueryObservedSourceMeaning>> {
        Self::intern_result_set(self, query, parameters, footprint, selection)
    }
}
