//! An admitted second owner of one immutable observed query source.

use super::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity;

pub(in crate::domain_computation::primary_graph) enum WorthQueryObservedSourceCloneStop<Stop> {
    AccountingOverflow,
    Admission(Stop),
}

impl<Query> WorthQueryObservedSource<Query> {
    /// The ordinary clone and this request-owned clone use the same retyped
    /// copy. All nested ownership is fixed or shared Arc custody; this copy
    /// allocates no new nested backing. Its inline value subsequently moves
    /// into the required registry's independently funded retained wrapper.
    pub(in crate::domain_computation::primary_graph) fn clone_admitted<Stop>(
        &self,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, WorthQueryObservedSourceCloneStop<Stop>> {
        // Inspect the enum before counting the Product identity's four extra
        // Arc retains: branch name and three component admission identities.
        admit(1, 0).map_err(WorthQueryObservedSourceCloneStop::Admission)?;
        let retained_arcs = match &self.selection {
            WorthQueryApplicationBasisSelectionIdentity::Relational => 6_u64,
            WorthQueryApplicationBasisSelectionIdentity::Product(_) => 10_u64,
        };
        let initialized = u64::try_from(std::mem::size_of::<Self>())
            .ok()
            .and_then(|bytes| bytes.checked_add(retained_arcs))
            .ok_or(WorthQueryObservedSourceCloneStop::AccountingOverflow)?;
        admit(initialized, 0).map_err(WorthQueryObservedSourceCloneStop::Admission)?;
        Ok(self.retyped())
    }
}
