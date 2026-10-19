//! Compiler-local codec correspondence. Names or decoded records cannot mint it.
use super::ApplicationOperationBodyExternalEmission;
use crate::application_schema::ApplicationValueEncodeDenial;
use crate::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationExternalEffectBinding,
    ApplicationExternalEffectPayloadProjection as Projection, ApplicationOperationMarkerIdentity,
    ApplicationValueValidationDenial,
};
use std::any::{Any, TypeId};

#[derive(Clone)]
pub struct ApplicationOperationBodyExternalEffectRecipe {
    operation: &'static str,
    input_type: TypeId,
    payload_type: TypeId,
    operation_type: TypeId,
    effect_type: TypeId,
    meaning: ApplicationOperationBodyExternalEmission,
    quote: fn(&dyn Any) -> Result<u64, ApplicationValueValidationDenial>,
    project: fn(&dyn Any, &'static str) -> Result<Projection, ApplicationValueEncodeDenial>,
    effect: &'static str,
}
impl std::fmt::Debug for ApplicationOperationBodyExternalEffectRecipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BodyExternalEffectRecipe")
            .field("operation", &self.operation)
            .field("meaning", &self.meaning)
            .finish_non_exhaustive()
    }
}
impl PartialEq for ApplicationOperationBodyExternalEffectRecipe {
    fn eq(&self, other: &Self) -> bool {
        self.operation == other.operation
            && self.input_type == other.input_type
            && self.payload_type == other.payload_type
            && self.operation_type == other.operation_type
            && self.effect_type == other.effect_type
            && self.meaning == other.meaning
    }
}
impl Eq for ApplicationOperationBodyExternalEffectRecipe {}
impl ApplicationOperationBodyExternalEffectRecipe {
    pub(super) fn declare<Schema, Operation, Effect, Input, Payload>(
        meaning: ApplicationOperationBodyExternalEmission,
    ) -> Self
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Effect: ApplicationEffectMarkerIdentity<Schema> + 'static,
        Effect::PayloadBinding: ApplicationExternalEffectBinding<Value = Payload>,
        Input: 'static,
        Payload: Clone + Send + Sync + 'static,
    {
        Self {
            operation: Operation::IDENTIFIER,
            operation_type: TypeId::of::<Operation>(),
            input_type: TypeId::of::<Input>(),
            payload_type: TypeId::of::<Payload>(),
            effect_type: TypeId::of::<Effect>(),
            effect: Effect::IDENTIFIER,
            meaning,
            quote: quote::<Effect::PayloadBinding>,
            project: project::<Effect::PayloadBinding>,
        }
    }
    pub fn operation(&self) -> &'static str {
        self.operation
    }
    pub fn input_type(&self) -> TypeId {
        self.input_type
    }
    pub fn payload_type(&self) -> TypeId {
        self.payload_type
    }
    pub fn operation_type(&self) -> TypeId {
        self.operation_type
    }
    pub fn effect_type(&self) -> TypeId {
        self.effect_type
    }
    pub fn meaning(&self) -> &ApplicationOperationBodyExternalEmission {
        &self.meaning
    }
    pub fn quote(&self, input: &dyn Any) -> Result<u64, ApplicationValueValidationDenial> {
        (self.quote)(input)
    }
    /// Caller must have funded the quote and selected the installed native emission.
    pub fn project(&self, input: &dyn Any) -> Result<Projection, ApplicationValueEncodeDenial> {
        (self.project)(input, self.effect)
    }
}
fn quote<Binding: ApplicationExternalEffectBinding>(
    input: &dyn Any,
) -> Result<u64, ApplicationValueValidationDenial> {
    let input = input.downcast_ref::<Binding::Value>().ok_or_else(|| {
        ApplicationValueValidationDenial::rejected(
            Binding::IDENTITY,
            "wrong native effect payload type",
        )
    })?;
    Binding::validate(input)?;
    Ok(Binding::retained_bytes(input))
}
fn project<Binding: ApplicationExternalEffectBinding>(
    input: &dyn Any,
    effect: &'static str,
) -> Result<Projection, ApplicationValueEncodeDenial>
where
    Binding::Value: Clone + Send + Sync,
{
    let value = input.downcast_ref::<Binding::Value>().ok_or_else(|| {
        ApplicationValueValidationDenial::rejected(
            Binding::IDENTITY,
            "wrong native effect payload type",
        )
    })?;
    Projection::encode::<Binding>(effect, value.clone())
}
