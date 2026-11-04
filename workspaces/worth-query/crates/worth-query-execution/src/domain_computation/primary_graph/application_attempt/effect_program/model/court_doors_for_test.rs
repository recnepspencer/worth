//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn with_producer_required_invariants_for_test(
        self,
        requirements: &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    ) -> Self {
        Self::with_producer_required_invariants(self, requirements)
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn effects_for_test(&self) -> &Vec<WorthQueryApplicationRealizedEffect> {
        &self.effects
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn effects_mut_for_test(&mut self) -> &mut Vec<WorthQueryApplicationRealizedEffect> {
        &mut self.effects
    }
}
