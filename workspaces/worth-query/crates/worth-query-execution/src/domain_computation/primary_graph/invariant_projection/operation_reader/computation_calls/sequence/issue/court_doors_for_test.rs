//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;

impl ComputationCallSequence {
    #[doc(hidden)]
    pub fn build_for_test<'a, I>(
        calls: impl Fn() -> I,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Arc<Self>, CallSequenceDenial>
    where
        I: Iterator<Item = (ComputationRead, &'a ComputationCallCharge, Option<u64>)>,
    {
        Self::build(calls, execution)
    }
}
