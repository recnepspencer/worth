use super::{
    ApplicationContentBinding, ApplicationContentCodecDenial as Denial,
    ApplicationContentControl as Control, ApplicationContentEncoder as Encoder,
    ApplicationContentFormatDeclaration,
};
use std::any::{Any, TypeId};

pub type ErasedContentLength = fn(&dyn Any, &mut dyn Control) -> Result<usize, Denial>;
pub type ErasedContentEncode = fn(&dyn Any, &mut dyn Encoder) -> Result<(), Denial>;
pub type ErasedContentValidate = fn(&[u8], &mut dyn Control) -> Result<(), Denial>;
pub type ErasedContentVerify = fn(&dyn Any, &[u8], &mut dyn Control) -> Result<(), Denial>;

/// Typed provenance, never serialized or reconstructed from a descriptor.
///
/// Descriptive metadata cannot substitute for an installed executable recipe:
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_schema::{
///     ApplicationContentFormatDeclaration, ApplicationContentBindingRecipe,
/// };
/// fn cannot_install_description(description: ApplicationContentFormatDeclaration)
///     -> ApplicationContentBindingRecipe {
///     description
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ApplicationContentBindingRecipe {
    declaration: ApplicationContentFormatDeclaration,
    binding_type: TypeId,
    value_type: TypeId,
    length: ErasedContentLength,
    encode: ErasedContentEncode,
    validate: ErasedContentValidate,
    verify: ErasedContentVerify,
}
impl ApplicationContentBindingRecipe {
    pub(crate) fn of<F: ApplicationContentBinding>() -> Self {
        Self {
            declaration: ApplicationContentFormatDeclaration {
                identity: *F::IDENTITY.bytes(),
                name: F::NAME.to_owned(),
                version: F::VERSION,
                maximum_frame_bytes: F::MAXIMUM_FRAME_BYTES as u64,
            },
            binding_type: TypeId::of::<F>(),
            value_type: TypeId::of::<F::Value>(),
            length: length::<F>,
            encode: encode::<F>,
            validate: F::validate,
            verify: verify::<F>,
        }
    }
    pub fn declaration(&self) -> &ApplicationContentFormatDeclaration {
        &self.declaration
    }
    pub fn binding_type(&self) -> TypeId {
        self.binding_type
    }
    pub fn value_type(&self) -> TypeId {
        self.value_type
    }
    pub fn length(&self) -> ErasedContentLength {
        self.length
    }
    pub fn encode(&self) -> ErasedContentEncode {
        self.encode
    }
    pub fn validate(&self) -> ErasedContentValidate {
        self.validate
    }
    pub fn verify(&self) -> ErasedContentVerify {
        self.verify
    }
}
impl PartialEq for ApplicationContentBindingRecipe {
    fn eq(&self, other: &Self) -> bool {
        self.declaration == other.declaration
            && self.binding_type == other.binding_type
            && self.value_type == other.value_type
    }
}
impl Eq for ApplicationContentBindingRecipe {}
fn input<F: ApplicationContentBinding>(value: &dyn Any) -> Result<&F::Value, Denial> {
    value.downcast_ref().ok_or(Denial::InputTypeMismatch)
}
fn length<F: ApplicationContentBinding>(
    value: &dyn Any,
    control: &mut dyn Control,
) -> Result<usize, Denial> {
    F::payload_length(input::<F>(value)?, control)
}
fn encode<F: ApplicationContentBinding>(
    value: &dyn Any,
    encoder: &mut dyn Encoder,
) -> Result<(), Denial> {
    F::encode(input::<F>(value)?, encoder)
}
fn verify<F: ApplicationContentBinding>(
    value: &dyn Any,
    bytes: &[u8],
    control: &mut dyn Control,
) -> Result<(), Denial> {
    F::verify_encoding(input::<F>(value)?, bytes, control)
}
