//! One ordinary typed effect a body emits, its payload derived from the exact
//! operation input through the payload type's `From<&Input>`.
use super::{ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder};
use crate::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationEffectRef, ApplicationOperationMarkerIdentity,
    ApplicationSchemaDeclarationDenial, ApplicationSchemaMember as Member,
    ApplicationStructuredValueBinding, ApplicationValueEncodeDenial,
    ApplicationValueValidationDenial, OperationEmits,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use std::any::{Any, TypeId};
use std::sync::Arc;

/// Portable meaning: emit `effect` once, with the payload the declared
/// payload binding derives from the operation input.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyEffectEmission {
    pub effect: String,
    pub input_binding: WorthQueryPortableTypeIdentity,
    pub payload_binding: WorthQueryPortableTypeIdentity,
}

/// Compiler-local payload derivation. Names or decoded bodies cannot mint it.
#[derive(Clone)]
pub struct ApplicationOperationBodyEffectRecipe {
    operation: &'static str,
    operation_type: TypeId,
    input_type: TypeId,
    effect_type: TypeId,
    effect: &'static str,
    meaning: ApplicationOperationBodyEffectEmission,
    derive: fn(&dyn Any) -> Result<Arc<dyn Any + Send + Sync>, ApplicationValueEncodeDenial>,
    payload_type: TypeId,
}

/// A validated typed payload derived from one operation input.
pub struct ApplicationOperationBodyEffectPayload {
    pub effect: &'static str,
    pub payload_binding: WorthQueryPortableTypeIdentity,
    pub payload_type: TypeId,
    pub payload: Arc<dyn Any + Send + Sync>,
}

impl std::fmt::Debug for ApplicationOperationBodyEffectRecipe {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BodyEffectRecipe")
            .field("operation", &self.operation)
            .field("meaning", &self.meaning)
            .finish_non_exhaustive()
    }
}

impl PartialEq for ApplicationOperationBodyEffectRecipe {
    fn eq(&self, other: &Self) -> bool {
        self.operation == other.operation
            && self.operation_type == other.operation_type
            && self.input_type == other.input_type
            && self.effect_type == other.effect_type
            && self.payload_type == other.payload_type
            && self.meaning == other.meaning
    }
}
impl Eq for ApplicationOperationBodyEffectRecipe {}

impl ApplicationOperationBodyEffectRecipe {
    pub fn operation(&self) -> &'static str {
        self.operation
    }
    pub fn operation_type(&self) -> TypeId {
        self.operation_type
    }
    pub fn input_type(&self) -> TypeId {
        self.input_type
    }
    pub fn effect_type(&self) -> TypeId {
        self.effect_type
    }
    pub fn payload_type(&self) -> TypeId {
        self.payload_type
    }
    pub fn meaning(&self) -> &ApplicationOperationBodyEffectEmission {
        &self.meaning
    }
    /// Derives and validates the payload; the caller supplies the exact
    /// typed operation input of the installed body.
    pub fn derive(
        &self,
        input: &dyn Any,
    ) -> Result<ApplicationOperationBodyEffectPayload, ApplicationValueEncodeDenial> {
        Ok(ApplicationOperationBodyEffectPayload {
            effect: self.effect,
            payload_binding: self.meaning.payload_binding.clone(),
            payload_type: self.payload_type,
            payload: (self.derive)(input)?,
        })
    }
}

fn derive<Input, Binding>(
    input: &dyn Any,
) -> Result<Arc<dyn Any + Send + Sync>, ApplicationValueEncodeDenial>
where
    Input: 'static,
    Binding: ApplicationStructuredValueBinding,
    Binding::Value: for<'input> From<&'input Input> + Send + Sync + 'static,
{
    let input = input.downcast_ref::<Input>().ok_or_else(|| {
        ApplicationValueValidationDenial::rejected(Binding::IDENTITY, "wrong operation input type")
    })?;
    let payload = Binding::Value::from(input);
    Binding::validate(&payload)?;
    Ok(Arc::new(payload))
}

impl<Schema, Operation, Input> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    /// Emits `effect` once in the body's commit, with the payload
    /// `Payload::from(&input)`. One ordinary effect per body.
    pub fn emit_effect<Effect, Payload>(
        mut self,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
    ) -> Result<Self, Denial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Effect: ApplicationEffectMarkerIdentity<Schema> + OperationEmits<Operation> + 'static,
        Effect::PayloadBinding: ApplicationStructuredValueBinding<Value = Payload>,
        Payload: for<'input> From<&'input Input> + Send + Sync + 'static,
        Input: 'static,
    {
        if self.effect_emission.is_some() {
            return Err(Denial::DuplicateAttachment);
        }
        let meaning = ApplicationOperationBodyEffectEmission {
            effect: effect.name().to_owned(),
            input_binding: Operation::InputBinding::IDENTITY,
            payload_binding: Effect::PayloadBinding::IDENTITY,
        };
        self.effect_emission_recipe = Some(ApplicationOperationBodyEffectRecipe {
            operation: Operation::IDENTIFIER,
            operation_type: TypeId::of::<Operation>(),
            input_type: TypeId::of::<Input>(),
            effect_type: TypeId::of::<Effect>(),
            effect: effect.name(),
            meaning: meaning.clone(),
            derive: derive::<Input, Effect::PayloadBinding>,
            payload_type: TypeId::of::<Payload>(),
        });
        self.effect_emission = Some(meaning);
        Ok(self)
    }
}

/// The emitted effect and the operation input must be declared schema members,
/// and the operation must hold the effect's emit program.
pub(crate) fn validate_effect_emission(
    operation: &str,
    emission: &ApplicationOperationBodyEffectEmission,
    members: &[Member],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let declared = members.iter().any(|member| matches!(member, Member::Effect { effect, payload_type } if effect == &emission.effect && payload_type == &emission.payload_binding))
        && members.iter().any(|member| matches!(member, Member::Operation { operation: name, input_type } if name == operation && input_type == &emission.input_binding))
        && members.iter().any(|member| matches!(member, Member::OperationProgram { operation: name, target: crate::application_schema::ApplicationOperationProgramTarget::Emit { effect } }
            if name == operation && effect == &emission.effect));
    declared
        .then_some(())
        .ok_or_else(|| ApplicationSchemaDeclarationDenial::program_body_dependency(operation))
}
