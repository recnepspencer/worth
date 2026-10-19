use super::{
    ApplicationOperationBodyExternalEffectRecipe as Recipe,
    ApplicationOperationBodyExternalEmission as Emission,
    ApplicationOperationBodyExternalPayloadSource as Source,
};
use crate::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationEffectRef, ApplicationExternalEffectBinding,
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationOperationBodyRequirement, ApplicationOperationMarkerIdentity,
    ApplicationStructuredValueBinding, OperationEmits,
};

impl<Schema, Operation, Input> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    /// Uses the exact operation input as the typed external payload. A Met
    /// condition emits; Unmet or NotApplicable suppresses. Failures still deny.
    pub fn emit_external_input<Effect>(
        mut self,
        effect: ApplicationEffectRef<Schema, Effect, Input>,
        condition: Option<ApplicationOperationBodyRequirement<Schema, Operation>>,
    ) -> Result<Self, Denial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Effect: ApplicationEffectMarkerIdentity<Schema> + OperationEmits<Operation> + 'static,
        Effect::PayloadBinding: ApplicationExternalEffectBinding<Value = Input>,
        Input: Clone + Send + Sync + 'static,
    {
        if self.external_emission.is_some() {
            return Err(Denial::DuplicateAttachment);
        }
        let emission = Emission {
            source: Source::OperationInput,
            effect: effect.name().to_owned(),
            input_binding: Operation::InputBinding::IDENTITY,
            payload_binding: Effect::PayloadBinding::IDENTITY,
            protocol: Effect::PayloadBinding::PROTOCOL,
            maximum_payload_bytes: Effect::PayloadBinding::MAX_EXTERNAL_BYTES,
            condition: condition.map(|condition| condition.program),
        };
        self.external_emission_recipe =
            Some(Recipe::declare::<Schema, Operation, Effect, Input, Input>(
                emission.clone(),
            ));
        self.external_emission = Some(emission);
        Ok(self)
    }

    /// Formats a payload from the exact admitted native source context, not the
    /// public query DTO or caller-provided graph values. The codec is pure.
    pub fn emit_external_query_context<Binding, Query, Parameters, QueryResult, Scope, Effect>(
        mut self,
        effect: ApplicationEffectRef<
            Schema,
            Effect,
            crate::application_query::ApplicationQueryExpressionSnapshot,
        >,
        query: crate::application_query::ApplicationQueryDefinition<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        condition: Option<ApplicationOperationBodyRequirement<Schema, Operation>>,
    ) -> Result<Self, Denial>
    where
        Schema: crate::application_schema::ApplicationSchema,
        Binding: crate::application_operation::ApplicationMutationBinding<
            Schema,
            Operation = Operation,
            Input = Input,
        >,
        Binding::SourceExpectation:
            crate::application_operation::ApplicationMutationSourceExpectation<
                Schema,
                Query = Query,
            >,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Effect: ApplicationEffectMarkerIdentity<Schema> + OperationEmits<Operation> + 'static,
        Effect::PayloadBinding: ApplicationExternalEffectBinding<
            Value = crate::application_query::ApplicationQueryExpressionSnapshot,
        >,
        Input: 'static,
    {
        if self.external_emission.is_some() {
            return Err(Denial::DuplicateAttachment);
        }
        if !query.parameters().is_empty()
            || query.cardinality()
                != crate::application_query::ApplicationQueryCardinality::ExactlyOne
        {
            return Err(
                super::super::ApplicationOperationBodyRequirementDenial::UnsupportedQueryContext
                    .into(),
            );
        }
        let emission = Emission {
            source: Source::QueryContext {
                mutation_binding: Binding::IDENTITY.to_owned(),
                query: Box::new(query.into_erased()),
            },
            effect: effect.name().to_owned(),
            input_binding: Operation::InputBinding::IDENTITY,
            payload_binding: Effect::PayloadBinding::IDENTITY,
            protocol: Effect::PayloadBinding::PROTOCOL,
            maximum_payload_bytes: Effect::PayloadBinding::MAX_EXTERNAL_BYTES,
            condition: condition.map(|condition| condition.program),
        };
        self.external_emission_recipe = Some(Recipe::declare::<
            Schema,
            Operation,
            Effect,
            Input,
            crate::application_query::ApplicationQueryExpressionSnapshot,
        >(emission.clone()));
        self.external_emission = Some(emission);
        Ok(self)
    }
}
