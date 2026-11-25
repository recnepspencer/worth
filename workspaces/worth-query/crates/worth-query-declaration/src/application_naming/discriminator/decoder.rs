//! Typed canonical decoding of a recorded discriminator: the exact inverse of
//! [`ApplicationNameDiscriminatorEncoder`].
//!
//! A decoder exists only over bytes a committed birth recorded. Every name
//! reference it yields was admitted by the read that produced those bytes, as
//! a name recorded and visible at that read's basis; bare digests open
//! nothing.

use std::collections::BTreeMap;
use std::fmt;

use super::sealed;
use super::{ApplicationNameDiscriminator, NAME_DISCRIMINATOR_DOMAIN_TAG, TAG_FIELD, TAG_VARIANT};
use crate::application_naming::role::NAME_ROLE_INDEX_FIELD;
use crate::application_naming::ApplicationNameSubjectKind;

/// Every name a read admitted as a discriminator reference, by digest, with
/// the subject class and declared subject its installed scheme governs.
pub type AdmittedNameReferences = BTreeMap<[u8; 32], (ApplicationNameSubjectKind, String)>;

/// Reads one recorded canonical discriminator back into typed values, in
/// exactly the order its encoder wrote them.
///
/// A hand-written [`ApplicationNameDiscriminatorDecode`] mirrors its
/// `encode_canonical`: `variant` where it wrote `variant`, `field` where it
/// wrote `field`, `value` where it wrote `value`. Applications never build
/// one; a recorded birth's `decode_discriminator` hands it over.
pub struct ApplicationNameDiscriminatorDecoder<'bytes> {
    bytes: &'bytes [u8],
    position: usize,
    admitted: &'bytes AdmittedNameReferences,
}

/// Typed refusal of bytes that are not the canonical encoding of the
/// requested discriminator type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationNameDiscriminatorDecodeDenial {
    /// The bytes do not open with the discriminator domain tag.
    NotADiscriminator,
    /// The bytes encode another discriminator type.
    IdentityMismatch { expected: String, found: String },
    /// The bytes end inside the value starting at `offset`.
    Truncated { offset: usize },
    /// Bytes remain at `offset` after the discriminator's last value.
    TrailingBytes { offset: usize },
    /// The value at `offset` carries another tag than the decoded type writes.
    UnexpectedTag { offset: usize, found: u8 },
    /// An enum variant the decoded type does not declare.
    UnknownVariant { variant: String },
    /// A field label other than the one the decoded type writes next.
    UnexpectedField { expected: String, found: String },
    /// The value at `offset` is malformed: text that is not UTF-8, or a
    /// boolean byte other than 0 or 1.
    InvalidValue { offset: usize },
    /// A name reference that is not a name recorded and visible at the
    /// read's basis.
    UnrecordedName { digest: [u8; 32] },
    /// A referenced name the read admitted under another subject class or
    /// declared subject than the field's kind: a name of one kind never
    /// decodes as a name of another.
    NameKindMismatch {
        digest: [u8; 32],
        expected: (ApplicationNameSubjectKind, String),
        found: (ApplicationNameSubjectKind, String),
    },
    /// A field type that cannot hold a recorded value: a same-commit name
    /// (use `NameRef<K>`) or borrowed text (use `String`).
    UndecodableField,
}

impl fmt::Display for ApplicationNameDiscriminatorDecodeDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotADiscriminator => formatter.write_str("bytes are not a name discriminator"),
            Self::IdentityMismatch { expected, found } => write!(
                formatter,
                "discriminator `{found}` is not the requested `{expected}`"
            ),
            Self::Truncated { offset } => write!(formatter, "discriminator ends inside {offset}"),
            Self::TrailingBytes { offset } => write!(formatter, "trailing bytes at {offset}"),
            Self::UnexpectedTag { offset, found } => {
                write!(formatter, "unexpected tag {found:#04x} at {offset}")
            }
            Self::UnknownVariant { variant } => write!(formatter, "unknown variant `{variant}`"),
            Self::UnexpectedField { expected, found } => {
                write!(formatter, "field `{found}` where `{expected}` belongs")
            }
            Self::InvalidValue { offset } => write!(formatter, "malformed value at {offset}"),
            Self::UnrecordedName { .. } => {
                formatter.write_str("a referenced name is not recorded at the read's basis")
            }
            Self::NameKindMismatch {
                expected, found, ..
            } => write!(
                formatter,
                "a referenced {} `{}` is not a {} `{}`",
                found.0.canonical_name(),
                found.1,
                expected.0.canonical_name(),
                expected.1
            ),
            Self::UndecodableField => formatter.write_str("field type holds no recorded value"),
        }
    }
}

impl std::error::Error for ApplicationNameDiscriminatorDecodeDenial {}

type Decoded<T> = Result<T, ApplicationNameDiscriminatorDecodeDenial>;

/// A discriminator that reads back from its recorded canonical bytes.
///
/// `#[derive(ApplicationNameDiscriminator)]` implements it beside the
/// encoding. A hand-written implementation reads exactly what its
/// `encode_canonical` writes, in the same order.
pub trait ApplicationNameDiscriminatorDecode: ApplicationNameDiscriminator {
    /// Whether this type's own encoding opens with a role index
    /// ([`super::super::IndexedRoleDiscriminator`]); a recorded birth then
    /// leaves the index for it to read.
    const LEADS_WITH_ROLE_INDEX: bool = false;

    /// Reads this value's semantic fields, after the identity header.
    fn decode_canonical(decoder: &mut ApplicationNameDiscriminatorDecoder<'_>) -> Decoded<Self>;
}

/// A field [`ApplicationNameDiscriminatorDecoder::field`] and
/// [`ApplicationNameDiscriminatorDecoder::value`] read back: every decodable
/// closed value, and every decodable nested discriminator.
pub trait ApplicationNameDiscriminatorFieldDecode<Form: sealed::FieldForm>: Sized {
    fn decode_discriminator_field(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self>;
}

impl<'bytes> ApplicationNameDiscriminatorDecoder<'bytes> {
    /// A decoder at the start of `bytes`, admitting exactly the name digests
    /// in `admitted`, each only as the kind it was admitted under.
    pub(crate) fn new(bytes: &'bytes [u8], admitted: &'bytes AdmittedNameReferences) -> Self {
        Self {
            bytes,
            position: 0,
            admitted,
        }
    }

    /// A decoder over a nested body, sharing this decoder's admission.
    pub(super) fn nested(&self, body: &'bytes [u8]) -> Self {
        Self::new(body, self.admitted)
    }

    /// Reads the domain tag and returns the identity it frames.
    pub(crate) fn header(&mut self) -> Decoded<&'bytes str> {
        let tag = NAME_DISCRIMINATOR_DOMAIN_TAG;
        if !self.bytes.starts_with(tag) {
            return Err(ApplicationNameDiscriminatorDecodeDenial::NotADiscriminator);
        }
        self.position = tag.len();
        self.text()
    }

    /// Consumes a leading role-index field, when the next field is one.
    pub(crate) fn leading_role_index(&mut self) -> Decoded<Option<u32>> {
        let start = self.position;
        if self.bytes.get(start) != Some(&TAG_FIELD) {
            return Ok(None);
        }
        self.position += 1;
        if self.text()? != NAME_ROLE_INDEX_FIELD {
            self.position = start;
            return Ok(None);
        }
        self.value::<sealed::ValueForm, u32>().map(Some)
    }

    /// Reads the variant tag an enum's `encode_canonical` wrote.
    pub fn variant(&mut self) -> Decoded<&'bytes str> {
        self.tag(TAG_VARIANT)?;
        self.text()
    }

    /// The refusal of a variant name the decoded enum does not declare.
    pub fn unknown_variant(&self, variant: &str) -> ApplicationNameDiscriminatorDecodeDenial {
        ApplicationNameDiscriminatorDecodeDenial::UnknownVariant {
            variant: variant.to_owned(),
        }
    }

    /// Reads one named field written under exactly `name`.
    pub fn field<Form, T>(&mut self, name: &str) -> Decoded<T>
    where
        Form: sealed::FieldForm,
        T: ApplicationNameDiscriminatorFieldDecode<Form>,
    {
        self.tag(TAG_FIELD)?;
        let found = self.text()?;
        if found != name {
            return Err(ApplicationNameDiscriminatorDecodeDenial::UnexpectedField {
                expected: name.to_owned(),
                found: found.to_owned(),
            });
        }
        T::decode_discriminator_field(self)
    }

    /// Reads one positional field.
    pub fn value<Form, T>(&mut self) -> Decoded<T>
    where
        Form: sealed::FieldForm,
        T: ApplicationNameDiscriminatorFieldDecode<Form>,
    {
        T::decode_discriminator_field(self)
    }

    /// Reads which repetition of an indexed role a birth is, where
    /// [`super::ApplicationNameDiscriminatorEncoder::role_index`] wrote it.
    pub fn role_index(&mut self) -> Decoded<u32> {
        self.field::<sealed::ValueForm, u32>(NAME_ROLE_INDEX_FIELD)
    }

    /// Refuses any byte left after the last value.
    pub(crate) fn finish(self) -> Decoded<()> {
        match self.position == self.bytes.len() {
            true => Ok(()),
            false => Err(ApplicationNameDiscriminatorDecodeDenial::TrailingBytes {
                offset: self.position,
            }),
        }
    }

    /// Admits `digest` only when the read admitted it as a name of exactly
    /// subject class `kind` and declared subject `subject`.
    pub(super) fn admit_name(
        &self,
        digest: [u8; 32],
        kind: ApplicationNameSubjectKind,
        subject: &str,
    ) -> Decoded<()> {
        let Some((found_kind, found_subject)) = self.admitted.get(&digest) else {
            return Err(ApplicationNameDiscriminatorDecodeDenial::UnrecordedName { digest });
        };
        if *found_kind == kind && found_subject == subject {
            return Ok(());
        }
        Err(ApplicationNameDiscriminatorDecodeDenial::NameKindMismatch {
            digest,
            expected: (kind, subject.to_owned()),
            found: (*found_kind, found_subject.clone()),
        })
    }

    pub(super) fn tag(&mut self, expected: u8) -> Decoded<()> {
        let offset = self.position;
        match self.next_tag()? {
            found if found == expected => Ok(()),
            found => Err(ApplicationNameDiscriminatorDecodeDenial::UnexpectedTag { offset, found }),
        }
    }

    pub(super) fn next_tag(&mut self) -> Decoded<u8> {
        Ok(self.raw::<1>()?[0])
    }

    pub(super) fn raw<const N: usize>(&mut self) -> Decoded<[u8; N]> {
        let bytes = self.slice(N)?;
        Ok(bytes.try_into().expect("slice has exactly N bytes"))
    }

    /// The next length-prefixed byte run.
    pub(super) fn len_prefixed(&mut self) -> Decoded<&'bytes [u8]> {
        let offset = self.position;
        let len = u64::from_be_bytes(self.raw::<8>()?);
        let len = usize::try_from(len)
            .map_err(|_| ApplicationNameDiscriminatorDecodeDenial::Truncated { offset })?;
        self.slice(len)
    }

    pub(super) fn text(&mut self) -> Decoded<&'bytes str> {
        let offset = self.position;
        std::str::from_utf8(self.len_prefixed()?)
            .map_err(|_| ApplicationNameDiscriminatorDecodeDenial::InvalidValue { offset })
    }

    pub(super) fn offset(&self) -> usize {
        self.position
    }

    fn slice(&mut self, len: usize) -> Decoded<&'bytes [u8]> {
        let offset = self.position;
        let end = offset
            .checked_add(len)
            .filter(|end| *end <= self.bytes.len())
            .ok_or(ApplicationNameDiscriminatorDecodeDenial::Truncated { offset })?;
        self.position = end;
        Ok(&self.bytes[offset..end])
    }
}

impl<T: super::ApplicationNameDiscriminatorValueDecode>
    ApplicationNameDiscriminatorFieldDecode<sealed::ValueForm> for T
{
    fn decode_discriminator_field(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        T::decode_discriminator_value(decoder)
    }
}

impl<D: ApplicationNameDiscriminatorDecode>
    ApplicationNameDiscriminatorFieldDecode<sealed::NestedForm> for D
{
    fn decode_discriminator_field(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        super::nested::decode_nested(decoder)
    }
}
