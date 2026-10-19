//! The exact decision reader supplies guard observations and original input.
use super::*;
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    pub(in crate::domain_computation::primary_graph) fn body_input_emission(
        &mut self,
        recipe: std::sync::Arc<worth_query_declaration::facade::application_schema::ApplicationOperationBodyInputEffectRecipe>,
        input: &Binding::Input,
    ) -> Result<crate::domain_computation::primary_graph::operation_body::emission::BodyInputEffectSelection, HandlerExecutionDenial>{
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        self.context_use.set(self.context_use.get().scope());
        if recipe.meaning().condition.iter().flat_map(|condition| condition.operands()).any(|operand| matches!(operand.source(), worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource::PrincipalIdentity { .. })) {
            let _ = self.principal_identity();
        }
        crate::domain_computation::primary_graph::operation_body::emission::prepare(
            self.reader,
            self.scope,
            recipe,
            input,
            self.execution,
        )
    }
}
