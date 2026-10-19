use super::{
    ApplicationOperationBodyInputEffectRecipe as Recipe,
    ApplicationOperationBodyInputEmission as Emission,
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
        if self.input_emission.is_some() {
            return Err(Denial::DuplicateAttachment);
        }
        let emission = Emission {
            effect: effect.name().to_owned(),
            input_binding: Operation::InputBinding::IDENTITY,
            payload_binding: Effect::PayloadBinding::IDENTITY,
            protocol: Effect::PayloadBinding::PROTOCOL,
            maximum_payload_bytes: Effect::PayloadBinding::MAX_EXTERNAL_BYTES,
            condition: condition.map(|condition| condition.program),
        };
        self.input_emission_recipe = Some(Recipe::declare::<Schema, Operation, Effect, Input>(
            emission.clone(),
        ));
        self.input_emission = Some(emission);
        Ok(self)
    }
}
