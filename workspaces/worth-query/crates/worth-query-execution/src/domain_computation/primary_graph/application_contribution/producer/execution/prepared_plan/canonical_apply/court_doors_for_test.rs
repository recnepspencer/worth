//! Sealed test doors over this module's private members.
use super::*;

impl<'owner, Key, Value, Stop> CanonicalApplyCursor<'owner, Key, Value, Stop> {
    #[doc(hidden)]
    pub fn slots_for_test(&self) -> &Vec<(
        OwnerPlan<'owner, Value, Stop>,
        Result<
            crate::domain_computation::primary_graph::vocabulary::application_contribution::wave_dispatch::ComputedMember,
            crate::domain_computation::request_execution::WorthQueryAdvancementDenial,
        >,
    )>{
        &self.slots
    }
}

impl<'owner, Key, Value, Stop> CanonicalApplyCursor<'owner, Key, Value, Stop> {
    #[doc(hidden)]
    pub fn apply_for_test(
        self,
        published: impl FnMut(Key, Result<Value, Stop>) -> Result<(), Stop>,
        refused: impl FnOnce(
            Key,
            crate::domain_computation::request_execution::WorthQueryAdvancementDenial,
        ) -> Stop,
    ) -> Result<(), Stop> {
        Self::apply(self, published, refused)
    }
}

impl<'owner, Key, Value, Stop> CanonicalApplyCursor<'owner, Key, Value, Stop> {
    #[doc(hidden)]
    pub fn compute_for_test(
        keys: CanonicalUniqueVec<Key>,
        plans: Vec<OwnerPlan<'owner, Value, Stop>>,
        execution: &QueryRequestExecution<'_>,
    ) -> Self {
        Self::compute(keys, plans, execution)
    }
}
