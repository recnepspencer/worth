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

impl<Schema, Entity> WorthQueryApplicationEffectEntity<Schema, Entity> {
    #[doc(hidden)]
    pub fn new_for_test(
        reference: EntityReference,
        entity: String,
        created_effect: Option<usize>,
        program: Arc<()>,
        _marker: PhantomData<fn() -> (Schema, Entity)>,
    ) -> Self {
        Self {
            reference,
            entity,
            created_effect,
            program,
            _marker,
        }
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn read_set_for_test(
        &self,
    ) -> &WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    > {
        &self.read_set
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn read_set_mut_for_test(
        &mut self,
    ) -> &mut WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    > {
        &mut self.read_set
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn read_set_into_for_test(
        self,
    ) -> WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    > {
        self.read_set
    }
}
