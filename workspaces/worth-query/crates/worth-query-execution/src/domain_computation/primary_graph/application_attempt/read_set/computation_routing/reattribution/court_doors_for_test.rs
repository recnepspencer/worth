//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::fact::WorthQueryApplicationFactKey;

impl SealedComputationFacts {
    #[doc(hidden)]
    pub fn reattribute_for_test(
        &self,
        calls: &BTreeSet<ComputationRead>,
        fresh: &BTreeMap<ComputationRead, Vec<&WorthQueryApplicationFactKey>>,
        control: StorageControl<'_, '_>,
    ) -> Result<Option<Self>, StoreDenial> {
        Self::reattribute(self, calls, fresh, control)
    }
}
