//! One discriminator carried inside another, framed by its own identity.

use super::{
    sealed, ApplicationNameDiscriminator, ApplicationNameDiscriminatorEncoder,
    ApplicationNameDiscriminatorValue, TAG_NESTED_DISCRIMINATOR,
};

/// Carries one [`ApplicationNameDiscriminator`] where a closed value is
/// expected: inside an `Option` or a pair. A field that is itself a
/// discriminator needs no wrapper and takes the same wire form.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NestedDiscriminator<D>(pub D);

impl<D: ApplicationNameDiscriminator> sealed::Sealed for NestedDiscriminator<D> {}
impl<D: ApplicationNameDiscriminator> ApplicationNameDiscriminatorValue for NestedDiscriminator<D> {
    fn encode_discriminator_value(&self, encoder: &mut ApplicationNameDiscriminatorEncoder) {
        encode_nested(&self.0, encoder);
    }
}

/// Writes `nested` as one value: its identity, then its length-prefixed
/// body. Name references inside it stay declared on the outer encoder.
///
/// Complexity: O(B) for the B bytes `nested` encodes to.
pub(super) fn encode_nested<D: ApplicationNameDiscriminator>(
    nested: &D,
    encoder: &mut ApplicationNameDiscriminatorEncoder,
) {
    let mut body = ApplicationNameDiscriminatorEncoder::default();
    nested.encode_canonical(&mut body);
    let (body_bytes, nested_refs, nested_slots) = body.finish_with_same_commit_names();
    encoder.write_tag(TAG_NESTED_DISCRIMINATOR);
    encoder.write_len_prefixed_str(D::DISCRIMINATOR_IDENTITY);
    // The body follows its own eight-byte length prefix.
    let body_start = encoder.bytes.len() + 8;
    encoder.write_len_prefixed_bytes(&body_bytes);
    for digest in nested_refs {
        encoder.record_name_ref(digest);
    }
    for slot in nested_slots {
        encoder.same_commit_names.push(slot.rebased(body_start));
    }
}

/// Reads one nested discriminator written by `encode_nested`: its identity,
/// which must be `D`'s, then its length-prefixed body, read completely.
///
/// Complexity: O(B) for the B bytes of the nested body.
pub(super) fn decode_nested<D: super::ApplicationNameDiscriminatorDecode>(
    decoder: &mut super::ApplicationNameDiscriminatorDecoder<'_>,
) -> Result<D, super::ApplicationNameDiscriminatorDecodeDenial> {
    decoder.tag(TAG_NESTED_DISCRIMINATOR)?;
    let found = decoder.text()?;
    if found != D::DISCRIMINATOR_IDENTITY {
        return Err(
            super::ApplicationNameDiscriminatorDecodeDenial::IdentityMismatch {
                expected: D::DISCRIMINATOR_IDENTITY.to_owned(),
                found: found.to_owned(),
            },
        );
    }
    let body = decoder.len_prefixed()?;
    let mut body = decoder.nested(body);
    let nested = D::decode_canonical(&mut body)?;
    body.finish()?;
    Ok(nested)
}
