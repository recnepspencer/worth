//! Candidate source changes are acquired before evaluation and reborrowed exactly.
use super::{CustomInvariantExecutionContext, CustomInvariantScopePlanner};
use crate::derived_relation::{
    DerivedRelationChangeData, DerivedRelationHandle, DerivedRelationReadDenial,
    PreparedDerivedRelationChanges,
};

impl CustomInvariantScopePlanner<'_> {
    /// Pull back the actual candidate's structural frontier through an installed
    /// derived recipe. No supplied record list or descriptor grants source access.
    /// Scalar projection changes must also route through their own dependencies.
    pub fn prepare_derived_relation_changes(
        &self,
        definition: &DerivedRelationHandle,
    ) -> Result<PreparedDerivedRelationChanges, DerivedRelationReadDenial> {
        crate::derived_relation::prepare_changes(
            &self.relations(),
            &self.committed_relations(),
            self.touched(),
            definition,
        )
    }
}

impl CustomInvariantExecutionContext<'_> {
    /// Reborrow the prepared frontier only under the original candidate and
    /// this receiver's full declared audience. No graph read is opened by IDs.
    pub fn read_prepared_derived_relation_changes<'a>(
        &self,
        prepared: &'a PreparedDerivedRelationChanges,
    ) -> Result<&'a DerivedRelationChangeData, DerivedRelationReadDenial> {
        crate::derived_relation::read_changes(
            &self.relations(),
            &self.committed_relations(),
            prepared,
        )
    }
}
