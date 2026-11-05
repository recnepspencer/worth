//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;

impl CallShape {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}

impl CallShape {
    #[doc(hidden)]
    pub fn prepare_transition_for_test(
        self: &Arc<Self>,
        inserted: &BTreeSet<ComputationRead>,
        retired: &BTreeSet<ComputationRead>,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<CallShapeTransition>, CallSequenceDenial> {
        Self::prepare_transition(self, inserted, retired, execution)
    }
}
