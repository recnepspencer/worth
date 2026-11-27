use super::same_commit::{SameCommitNameSlot, SAME_COMMIT_NAME_PLACEHOLDER};
use super::{WorthQueryName, WorthQuerySameCommitName};

/// Canonical domain prefix separating encoded name discriminators from all
/// other canonical byte streams.
pub const NAME_DISCRIMINATOR_DOMAIN_TAG: &[u8] = b"worth.query.name-discriminator.v1\0";

mod decode_values;
mod decoder;
mod field;
mod nested;
mod references;
mod wire_bound;

pub use decode_values::ApplicationNameDiscriminatorValueDecode;
pub use decoder::{
    AdmittedNameReferences, ApplicationNameDiscriminatorDecode,
    ApplicationNameDiscriminatorDecodeDenial, ApplicationNameDiscriminatorDecoder,
    ApplicationNameDiscriminatorFieldDecode,
};
pub use field::ApplicationNameDiscriminatorField;
pub use nested::NestedDiscriminator;
pub(crate) use references::recorded_name_references;
pub(crate) use wire_bound::discriminator_fits_structural_identity;
pub use wire_bound::{
    bound_max, bound_sum, field_bound, max_encoded_bytes, variant_bound,
    MAX_STRUCTURAL_DISCRIMINATOR_BYTES,
};

/// Unnameable outside this crate, so no caller can add a value type or a
/// field form of its own.
pub(super) mod sealed {
    pub trait Sealed {}
    /// How a discriminator field reaches the encoder.
    pub trait FieldForm {}
    /// A value of the closed [`super::ApplicationNameDiscriminatorValue`] algebra.
    pub struct ValueForm;
    /// A nested [`super::ApplicationNameDiscriminator`].
    pub struct NestedForm;
    impl FieldForm for ValueForm {}
    impl FieldForm for NestedForm {}
}

/// Deterministic, domain-tagged canonical encoder for name discriminators.
///
/// Only Foundational discrete values and fixed-size name references
/// ([`WorthQueryName`], [`WorthQuerySameCommitName`]) may be written. Floats,
/// `RecordId`s, and untyped payloads are unrepresentable on this encoder.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplicationNameDiscriminatorEncoder {
    bytes: Vec<u8>,
    referenced_name_digests: Vec<[u8; 32]>,
    same_commit_names: Vec<SameCommitNameSlot>,
}

impl ApplicationNameDiscriminatorEncoder {
    /// Begins a new domain-tagged canonical discriminator byte stream for the
    /// named discriminator type identity.
    pub fn new(discriminator_identity: &str) -> Self {
        let mut encoder = Self {
            bytes: Vec::with_capacity(NAME_DISCRIMINATOR_DOMAIN_TAG.len() + 64),
            referenced_name_digests: Vec::new(),
            same_commit_names: Vec::new(),
        };
        encoder
            .bytes
            .extend_from_slice(NAME_DISCRIMINATOR_DOMAIN_TAG);
        encoder.write_len_prefixed_str(discriminator_identity);
        encoder
    }

    /// Writes a canonical enum variant tag before encoding its fields.
    pub fn variant(&mut self, name: &str) -> &mut Self {
        self.bytes.push(TAG_VARIANT);
        self.write_len_prefixed_str(name);
        self
    }

    /// Writes one named field: a closed Foundational or name-reference value,
    /// or a nested [`ApplicationNameDiscriminator`].
    pub fn field<Form, T>(&mut self, name: &str, value: &T) -> &mut Self
    where
        Form: sealed::FieldForm,
        T: ApplicationNameDiscriminatorField<Form>,
    {
        self.bytes.push(TAG_FIELD);
        self.write_len_prefixed_str(name);
        value.encode_discriminator_field(self);
        self
    }

    /// Writes a positional field without a label.
    pub fn value<Form, T>(&mut self, value: &T) -> &mut Self
    where
        Form: sealed::FieldForm,
        T: ApplicationNameDiscriminatorField<Form>,
    {
        value.encode_discriminator_field(self);
        self
    }

    /// Returns every 32-byte [`WorthQueryName`] digest written into this
    /// encoder stream in exact encoding order.
    pub fn referenced_name_digests(&self) -> &[[u8; 32]] {
        &self.referenced_name_digests
    }

    /// Finishes encoding and returns the canonical byte vector.
    pub fn finish(self) -> Vec<u8> {
        self.bytes
    }

    /// Finishes encoding and returns both the canonical byte vector and every
    /// 32-byte [`WorthQueryName`] digest embedded in it so declaration and
    /// Relational verification cannot disagree.
    pub fn finish_with_references(self) -> (Vec<u8>, Vec<[u8; 32]>) {
        (self.bytes, self.referenced_name_digests)
    }

    /// Finishes encoding and also returns where each same-commit name
    /// reference left its placeholder, in encoding order. The bytes are
    /// canonical only once every placeholder holds its referenced name.
    pub fn finish_with_same_commit_names(
        self,
    ) -> (Vec<u8>, Vec<[u8; 32]>, Vec<SameCommitNameSlot>) {
        (
            self.bytes,
            self.referenced_name_digests,
            self.same_commit_names,
        )
    }

    fn record_name_ref(&mut self, digest: [u8; 32]) {
        self.referenced_name_digests.push(digest);
    }

    fn write_len_prefixed_str(&mut self, text: &str) {
        self.write_len_prefixed_bytes(text.as_bytes());
    }

    fn write_len_prefixed_bytes(&mut self, bytes: &[u8]) {
        let len = u64::try_from(bytes.len()).expect("discriminator payload length fits u64");
        self.bytes.extend_from_slice(&len.to_be_bytes());
        self.bytes.extend_from_slice(bytes);
    }

    fn write_tag(&mut self, tag: u8) {
        self.bytes.push(tag);
    }

    fn write_raw(&mut self, raw: &[u8]) {
        self.bytes.extend_from_slice(raw);
    }
}

const TAG_VARIANT: u8 = 0x01;
const TAG_FIELD: u8 = 0x02;
const TAG_BOOL: u8 = 0x10;
const TAG_I8: u8 = 0x11;
const TAG_I16: u8 = 0x12;
const TAG_I32: u8 = 0x13;
const TAG_I64: u8 = 0x14;
const TAG_U8: u8 = 0x15;
const TAG_U16: u8 = 0x16;
const TAG_U32: u8 = 0x17;
const TAG_U64: u8 = 0x18;
const TAG_TEXT: u8 = 0x19;
const TAG_UUID: u8 = 0x1a;
const TAG_NAME_REF: u8 = 0x20;
const TAG_OPTION_NONE: u8 = 0x30;
const TAG_OPTION_SOME: u8 = 0x31;
const TAG_PAIR: u8 = 0x32;
const TAG_NESTED_DISCRIMINATOR: u8 = 0x33;

/// Closed value algebra permitted inside an [`ApplicationNameDiscriminator`].
///
/// Sealed so callers cannot introduce floats (`f32`/`f64`), native `RecordId`s,
/// or raw unvalidated payloads into persistent name discriminators.
///
/// Attempting to encode a float fails to compile:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_naming::ApplicationNameDiscriminatorEncoder;
///
/// let mut encoder = ApplicationNameDiscriminatorEncoder::new("example.bad");
/// encoder.field("coord", &1.25_f64);
/// ```
///
/// Valid positive counterpart using discrete Foundational values and a
/// fixed-size [`WorthQueryName`] reference:
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     ApplicationNameDiscriminatorEncoder, WorthQueryName,
/// };
///
/// struct Face;
///
/// fn encode_cut(cutter: WorthQueryName<Face>, index: u32) -> Vec<u8> {
///     let mut encoder = ApplicationNameDiscriminatorEncoder::new("example.cut");
///     encoder
///         .variant("SidedCut")
///         .field("cutter", &cutter)
///         .field("index", &index);
///     encoder.finish()
/// }
///
/// let _check: fn(WorthQueryName<Face>, u32) -> Vec<u8> = encode_cut;
/// ```
pub trait ApplicationNameDiscriminatorValue: sealed::Sealed {
    /// The most bytes this value encodes to; `None` when it can grow.
    const MAX_ENCODED_BYTES: Option<usize>;
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder);
}

impl sealed::Sealed for bool {}
impl ApplicationNameDiscriminatorValue for bool {
    const MAX_ENCODED_BYTES: Option<usize> = Some(2);
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.write_tag(TAG_BOOL);
        encoder.write_raw(&[u8::from(*self)]);
    }
}

macro_rules! impl_signed_and_unsigned_discriminator_values {
    ($(($ty:ty, $tag:ident)),* $(,)?) => {
        $(
            impl sealed::Sealed for $ty {}
            impl ApplicationNameDiscriminatorValue for $ty {
                const MAX_ENCODED_BYTES: Option<usize> = Some(1 + std::mem::size_of::<$ty>());
                fn encode_discriminator_value(
                    &self,
                    encoder: &mut ApplicationNameDiscriminatorEncoder,
                ) {
                    encoder.write_tag($tag);
                    encoder.write_raw(&self.to_be_bytes());
                }
            }
        )*
    };
}

impl_signed_and_unsigned_discriminator_values!(
    (i8, TAG_I8),
    (i16, TAG_I16),
    (i32, TAG_I32),
    (i64, TAG_I64),
    (u8, TAG_U8),
    (u16, TAG_U16),
    (u32, TAG_U32),
    (u64, TAG_U64),
);

impl sealed::Sealed for str {}
impl sealed::Sealed for &str {}
impl ApplicationNameDiscriminatorValue for &str {
    const MAX_ENCODED_BYTES: Option<usize> = None;
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.write_tag(TAG_TEXT);
        encoder.write_len_prefixed_str(self);
    }
}

impl sealed::Sealed for String {}
impl ApplicationNameDiscriminatorValue for String {
    const MAX_ENCODED_BYTES: Option<usize> = None;
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.write_tag(TAG_TEXT);
        encoder.write_len_prefixed_str(self);
    }
}

impl sealed::Sealed for [u8; 16] {}
impl ApplicationNameDiscriminatorValue for [u8; 16] {
    const MAX_ENCODED_BYTES: Option<usize> = Some(17);
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.write_tag(TAG_UUID);
        encoder.write_raw(self);
    }
}

impl<K> sealed::Sealed for WorthQueryName<K> {}
impl<K> ApplicationNameDiscriminatorValue for WorthQueryName<K> {
    const MAX_ENCODED_BYTES: Option<usize> = Some(33);
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        // Referenced names enter ONLY by their fixed 32-byte canonical digest
        // so discriminator and birth-event size stays O(1) at arbitrary depth.
        encoder.write_tag(TAG_NAME_REF);
        encoder.write_raw(self.as_bytes());
        encoder.record_name_ref(*self.as_bytes());
    }
}

impl<K> sealed::Sealed for WorthQuerySameCommitName<K> {}
impl<K> ApplicationNameDiscriminatorValue for WorthQuerySameCommitName<K> {
    const MAX_ENCODED_BYTES: Option<usize> = Some(33);
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        // The same wire form as a committed name reference, so a name does
        // not depend on whether the name it references was born earlier or
        // in the same commit.
        encoder.write_tag(TAG_NAME_REF);
        let slot = SameCommitNameSlot::new(encoder.bytes.len(), self.reference().clone());
        encoder.same_commit_names.push(slot);
        encoder.write_raw(&SAME_COMMIT_NAME_PLACEHOLDER);
    }
}

impl<T: ApplicationNameDiscriminatorValue> sealed::Sealed for Option<T> {}
impl<T: ApplicationNameDiscriminatorValue> ApplicationNameDiscriminatorValue for Option<T> {
    const MAX_ENCODED_BYTES: Option<usize> = bound_sum(&[Some(1), T::MAX_ENCODED_BYTES]);
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        match self {
            None => encoder.write_tag(TAG_OPTION_NONE),
            Some(inner) => {
                encoder.write_tag(TAG_OPTION_SOME);
                inner.encode_discriminator_value(encoder);
            }
        }
    }
}

impl<A: ApplicationNameDiscriminatorValue, B: ApplicationNameDiscriminatorValue> sealed::Sealed
    for (A, B)
{
}
impl<A: ApplicationNameDiscriminatorValue, B: ApplicationNameDiscriminatorValue>
    ApplicationNameDiscriminatorValue for (A, B)
{
    const MAX_ENCODED_BYTES: Option<usize> =
        bound_sum(&[Some(1), A::MAX_ENCODED_BYTES, B::MAX_ENCODED_BYTES]);
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encoder.write_tag(TAG_PAIR);
        self.0.encode_discriminator_value(encoder);
        self.1.encode_discriminator_value(encoder);
    }
}

/// Typed local discriminator declared by an application to distinguish sibling
/// persistent names within one scheme and occurrence.
///
/// Implement directly or with [`crate::worth_query_name_discriminator!`].
pub trait ApplicationNameDiscriminator: Sized + 'static {
    /// Stable, portable identity string for this discriminator type.
    const DISCRIMINATOR_IDENTITY: &'static str;

    /// Same-occurrence role names referenced by this discriminator, if any.
    ///
    /// Installation verifies that every referenced role exists in the scheme's
    /// acyclic role order.
    fn referenced_roles() -> &'static [&'static str] {
        &[]
    }

    /// The most bytes [`Self::encode_canonical`] writes, derived from the
    /// field types by `#[derive(ApplicationNameDiscriminator)]`; `None` (the
    /// default) when unknown or when the body can grow.
    const MAX_BODY_BYTES: Option<usize> = None;

    /// Encodes this value's semantic fields into the domain-tagged encoder.
    fn encode_canonical(&self, encoder: &mut ApplicationNameDiscriminatorEncoder);

    /// Produces the complete domain-tagged canonical byte representation.
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut encoder = ApplicationNameDiscriminatorEncoder::new(Self::DISCRIMINATOR_IDENTITY);
        self.encode_canonical(&mut encoder);
        encoder.finish()
    }

    /// Produces both the canonical byte representation and the exact 32-byte
    /// [`WorthQueryName`] digests embedded inside it.
    fn canonical_bytes_with_references(&self) -> (Vec<u8>, Vec<[u8; 32]>) {
        let mut encoder = ApplicationNameDiscriminatorEncoder::new(Self::DISCRIMINATOR_IDENTITY);
        self.encode_canonical(&mut encoder);
        encoder.finish_with_references()
    }
}
