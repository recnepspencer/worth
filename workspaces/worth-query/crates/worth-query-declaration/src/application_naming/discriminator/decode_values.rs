//! How each closed discriminator value reads back from its canonical tag.

use super::decoder::{
    ApplicationNameDiscriminatorDecodeDenial, ApplicationNameDiscriminatorDecoder,
};
use super::{
    ApplicationNameDiscriminatorValue, NestedDiscriminator, TAG_BOOL, TAG_I16, TAG_I32, TAG_I64,
    TAG_I8, TAG_NAME_REF, TAG_OPTION_NONE, TAG_OPTION_SOME, TAG_PAIR, TAG_TEXT, TAG_U16, TAG_U32,
    TAG_U64, TAG_U8, TAG_UUID,
};
use crate::application_naming::kind::{subject_kind_of, subject_name_of};
use crate::application_naming::{
    NameRef, WorthQueryName, WorthQueryNamedKind, WorthQuerySameCommitName,
};

type Denial = ApplicationNameDiscriminatorDecodeDenial;
type Decoded<T> = Result<T, Denial>;

/// A closed discriminator value that reads back from its canonical tag.
///
/// Sealed through [`ApplicationNameDiscriminatorValue`]: only the encoder's
/// own value algebra decodes.
pub trait ApplicationNameDiscriminatorValueDecode:
    ApplicationNameDiscriminatorValue + Sized
{
    fn decode_discriminator_value(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self>;
}

impl ApplicationNameDiscriminatorValueDecode for bool {
    fn decode_discriminator_value(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        decoder.tag(TAG_BOOL)?;
        let offset = decoder.offset();
        match decoder.raw::<1>()? {
            [0] => Ok(false),
            [1] => Ok(true),
            _ => Err(Denial::InvalidValue { offset }),
        }
    }
}

macro_rules! impl_integer_decode {
    ($(($ty:ty, $tag:ident)),* $(,)?) => {
        $(
            impl ApplicationNameDiscriminatorValueDecode for $ty {
                fn decode_discriminator_value(
                    decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
                ) -> Decoded<Self> {
                    decoder.tag($tag)?;
                    Ok(<$ty>::from_be_bytes(decoder.raw()?))
                }
            }
        )*
    };
}

impl_integer_decode!(
    (i8, TAG_I8),
    (i16, TAG_I16),
    (i32, TAG_I32),
    (i64, TAG_I64),
    (u8, TAG_U8),
    (u16, TAG_U16),
    (u32, TAG_U32),
    (u64, TAG_U64),
);

impl ApplicationNameDiscriminatorValueDecode for String {
    fn decode_discriminator_value(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        decoder.tag(TAG_TEXT)?;
        decoder.text().map(str::to_owned)
    }
}

/// Borrowed text encodes but cannot be read back into a recorded value;
/// declare the field as `String`.
impl ApplicationNameDiscriminatorValueDecode for &'static str {
    fn decode_discriminator_value(
        _: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        Err(Denial::UndecodableField)
    }
}

impl ApplicationNameDiscriminatorValueDecode for [u8; 16] {
    fn decode_discriminator_value(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        decoder.tag(TAG_UUID)?;
        decoder.raw()
    }
}

/// A recorded name reference reads back only as a name the read admitted
/// at its basis under exactly `K`'s subject class and declared subject.
impl<K: WorthQueryNamedKind> ApplicationNameDiscriminatorValueDecode for WorthQueryName<K> {
    fn decode_discriminator_value(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        decoder.tag(TAG_NAME_REF)?;
        let digest = decoder.raw::<32>()?;
        decoder.admit_name(digest, subject_kind_of::<K>(), subject_name_of::<K>())?;
        Ok(WorthQueryName::from_admitted_digest(digest))
    }
}

/// A recorded reference is always to a committed name.
impl<K: WorthQueryNamedKind> ApplicationNameDiscriminatorValueDecode for NameRef<K> {
    fn decode_discriminator_value(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        WorthQueryName::decode_discriminator_value(decoder).map(NameRef::Committed)
    }
}

/// A recorded birth carries the committed name a same-commit reference
/// became; declare the field as `NameRef<K>` to read it back.
impl<K> ApplicationNameDiscriminatorValueDecode for WorthQuerySameCommitName<K> {
    fn decode_discriminator_value(
        _: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        Err(Denial::UndecodableField)
    }
}

impl<T: ApplicationNameDiscriminatorValueDecode> ApplicationNameDiscriminatorValueDecode
    for Option<T>
{
    fn decode_discriminator_value(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        let offset = decoder.offset();
        match decoder.next_tag()? {
            TAG_OPTION_NONE => Ok(None),
            TAG_OPTION_SOME => T::decode_discriminator_value(decoder).map(Some),
            found => Err(Denial::UnexpectedTag { offset, found }),
        }
    }
}

impl<A, B> ApplicationNameDiscriminatorValueDecode for (A, B)
where
    A: ApplicationNameDiscriminatorValueDecode,
    B: ApplicationNameDiscriminatorValueDecode,
{
    fn decode_discriminator_value(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        decoder.tag(TAG_PAIR)?;
        let first = A::decode_discriminator_value(decoder)?;
        Ok((first, B::decode_discriminator_value(decoder)?))
    }
}

impl<D: super::ApplicationNameDiscriminatorDecode> ApplicationNameDiscriminatorValueDecode
    for NestedDiscriminator<D>
{
    fn decode_discriminator_value(
        decoder: &mut ApplicationNameDiscriminatorDecoder<'_>,
    ) -> Decoded<Self> {
        super::nested::decode_nested(decoder).map(NestedDiscriminator)
    }
}
