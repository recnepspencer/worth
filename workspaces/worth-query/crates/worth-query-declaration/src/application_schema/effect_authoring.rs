use std::marker::PhantomData;

use super::authoring_context::ApplicationOperationProgramAdmission;
use super::capabilities::OperationEmits;
use super::references::{ApplicationEffectRef, ApplicationOperationRef};
use super::{
    ApplicationEffectMarkerIdentity, ApplicationOperationMarkerIdentity,
    ApplicationSchemaAuthoringContext, ApplicationSchemaAuthoringDenial,
    ApplicationSchemaBindingIdentity, ApplicationStructuredValueBinding,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

pub struct TypedEffectIntent<Schema, Operation> {
    operation: &'static str,
    binding: Option<ApplicationSchemaBindingIdentity>,
    effects: Vec<(&'static str, WorthQueryPortableTypeIdentity)>,
    _marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema, Operation> Clone for TypedEffectIntent<Schema, Operation> {
    fn clone(&self) -> Self {
        Self {
            operation: self.operation,
            binding: self.binding.clone(),
            effects: self.effects.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Schema, Operation> std::fmt::Debug for TypedEffectIntent<Schema, Operation> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TypedEffectIntent")
            .field("operation", &self.operation)
            .field("binding", &self.binding)
            .field("effects", &self.effects)
            .finish_non_exhaustive()
    }
}

impl<Schema, Operation> PartialEq for TypedEffectIntent<Schema, Operation> {
    fn eq(&self, other: &Self) -> bool {
        self.operation == other.operation
            && self.binding == other.binding
            && self.effects == other.effects
    }
}

impl<Schema, Operation> TypedEffectIntent<Schema, Operation> {
    pub fn binding(&self) -> Option<&ApplicationSchemaBindingIdentity> {
        self.binding.as_ref()
    }

    pub const fn operation(&self) -> &'static str {
        self.operation
    }

    pub fn effects(&self) -> &[(&'static str, WorthQueryPortableTypeIdentity)] {
        &self.effects
    }
}

pub struct TypedEffectIntentBuilder<Schema, Operation, Input> {
    operation: &'static str,
    context: Option<ApplicationSchemaAuthoringContext>,
    denial: Option<ApplicationSchemaAuthoringDenial>,
    effects: Vec<(&'static str, WorthQueryPortableTypeIdentity)>,
    _marker: PhantomData<fn(Input) -> (Schema, Operation)>,
}

impl<Schema, Operation: 'static, Input> TypedEffectIntentBuilder<Schema, Operation, Input>
where
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: 'static,
{
    pub fn new(operation: ApplicationOperationRef<Schema, Operation, Input>) -> Self {
        Self {
            operation: operation.name(),
            context: None,
            denial: None,
            effects: Vec::new(),
            _marker: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn with_installed_context(mut self, context: ApplicationSchemaAuthoringContext) -> Self {
        self.denial = context
            .admit_operation::<Operation, Input>(self.operation, Operation::InputBinding::IDENTITY)
            .err();
        self.context = Some(context);
        self
    }

    pub fn emit<Effect, Payload>(
        mut self,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
        _payload: Payload,
    ) -> Self
    where
        Effect: ApplicationEffectMarkerIdentity<Schema> + OperationEmits<Operation> + 'static,
        Effect::PayloadBinding: ApplicationStructuredValueBinding<Value = Payload>,
        Payload: 'static,
    {
        if self.denial.is_none() {
            self.denial = self.context.as_ref().and_then(|context| {
                context
                    .admit_effect::<Effect, Payload>(effect.name(), effect.payload_identity())
                    .and_then(|()| {
                        context.admit_operation_program(
                            self.operation,
                            ApplicationOperationProgramAdmission::Emit(effect.name()),
                        )
                    })
                    .err()
            });
        }
        self.effects
            .push((effect.name(), effect.payload_identity()));
        self
    }

    pub fn build(
        self,
    ) -> Result<TypedEffectIntent<Schema, Operation>, ApplicationSchemaAuthoringDenial> {
        if let Some(denial) = self.denial {
            return Err(denial);
        }
        Ok(TypedEffectIntent {
            operation: self.operation,
            binding: self.context.map(|context| context.binding().clone()),
            effects: self.effects,
            _marker: PhantomData,
        })
    }
}
