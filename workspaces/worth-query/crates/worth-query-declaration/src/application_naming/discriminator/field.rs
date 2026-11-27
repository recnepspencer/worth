//! What one field of a discriminator may be: a closed value or a nested
//! discriminator, chosen by the field's type.

use super::nested::encode_nested;
use super::{
    sealed, ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameDiscriminatorValue,
};

/// A field [`ApplicationNameDiscriminatorEncoder::field`] and
/// [`ApplicationNameDiscriminatorEncoder::value`] accept.
///
/// It has exactly two implementations: every
/// [`ApplicationNameDiscriminatorValue`], and every
/// [`ApplicationNameDiscriminator`], which nests under its own identity. The
/// `Form` parameter tells the two apart and is inferred; its types cannot be
/// named outside this crate, so no caller can add a third.
///
/// A discriminator nests as a field without a wrapper, in the wire form of
/// [`super::NestedDiscriminator`]:
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder, NestedDiscriminator,
/// };
///
/// #[derive(Clone, Copy)]
/// struct Side(u8);
/// impl ApplicationNameDiscriminator for Side {
///     const DISCRIMINATOR_IDENTITY: &'static str = "example.side.v1";
///     fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
///         encoder.field("side", &self.0);
///     }
/// }
///
/// let mut direct = ApplicationNameDiscriminatorEncoder::new("example.cut.v1");
/// direct.field("side", &Side(1));
/// let mut wrapped = ApplicationNameDiscriminatorEncoder::new("example.cut.v1");
/// wrapped.field("side", &NestedDiscriminator(Side(1)));
/// assert_eq!(direct.finish(), wrapped.finish());
/// ```
///
/// A float is not a field:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_naming::ApplicationNameDiscriminatorEncoder;
///
/// let mut encoder = ApplicationNameDiscriminatorEncoder::new("example.bad");
/// encoder.value(&1.5_f32);
/// ```
///
/// Nor is a type that is neither a closed value nor a discriminator, such as
/// a record handle:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_naming::ApplicationNameDiscriminatorEncoder;
///
/// struct RecordId(u64);
///
/// let mut encoder = ApplicationNameDiscriminatorEncoder::new("example.bad");
/// encoder.field("record", &RecordId(7));
/// ```
///
/// And the form that would admit one cannot be named:
///
/// ```compile_fail,E0603
/// use worth_query_declaration::facade::application_naming::{
///     ApplicationNameDiscriminatorEncoder, ApplicationNameDiscriminatorField,
/// };
/// use worth_query_declaration::application_naming::discriminator::sealed::ValueForm;
///
/// struct RecordId(u64);
/// impl ApplicationNameDiscriminatorField<ValueForm> for RecordId {
///     fn encode_discriminator_field(&self, _: &mut ApplicationNameDiscriminatorEncoder) {}
/// }
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be a field of a persistent-name discriminator",
    label = "not a deterministic discriminator value",
    note = "a field is a discrete Foundational value, a `WorthQueryName<K>`, a `NameRef<K>` or a nested `ApplicationNameDiscriminator`; floats, record ids and raw payloads are refused"
)]
pub trait ApplicationNameDiscriminatorField<Form: sealed::FieldForm> {
    /// The most bytes this field's value encodes to; `None` when it can grow.
    const MAX_ENCODED_BYTES: Option<usize>;
    fn encode_discriminator_field(&self, encoder: &mut ApplicationNameDiscriminatorEncoder);
}

impl<T: ApplicationNameDiscriminatorValue> ApplicationNameDiscriminatorField<sealed::ValueForm>
    for T
{
    const MAX_ENCODED_BYTES: Option<usize> = T::MAX_ENCODED_BYTES;
    fn encode_discriminator_field(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        self.encode_discriminator_value(encoder);
    }
}

impl<D: ApplicationNameDiscriminator> ApplicationNameDiscriminatorField<sealed::NestedForm> for D {
    const MAX_ENCODED_BYTES: Option<usize> = super::wire_bound::nested_bound::<D>();
    fn encode_discriminator_field(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encode_nested(self, encoder);
    }
}
