//! The name references a recorded discriminator carries, found from its
//! self-describing tags alone, so a read can admit each one at its basis
//! before any typed decode.

use std::collections::BTreeSet;

use super::decoder::{
    ApplicationNameDiscriminatorDecodeDenial, ApplicationNameDiscriminatorDecoder,
};
use super::{
    TAG_BOOL, TAG_FIELD, TAG_I16, TAG_I32, TAG_I64, TAG_I8, TAG_NAME_REF, TAG_NESTED_DISCRIMINATOR,
    TAG_OPTION_NONE, TAG_OPTION_SOME, TAG_PAIR, TAG_TEXT, TAG_U16, TAG_U32, TAG_U64, TAG_U8,
    TAG_UUID, TAG_VARIANT,
};

/// Every 32-byte name digest `bytes` references, in encoding order,
/// including those inside nested discriminators. Empty bytes reference none.
///
/// Complexity: O(len).
pub(crate) fn recorded_name_references(
    bytes: &[u8],
) -> Result<Vec<[u8; 32]>, ApplicationNameDiscriminatorDecodeDenial> {
    let mut references = Vec::new();
    if bytes.is_empty() {
        return Ok(references);
    }
    let admits_none = BTreeSet::new();
    let mut decoder = ApplicationNameDiscriminatorDecoder::new(bytes, &admits_none);
    decoder.header()?;
    // A nested body is itself a run of tagged items, so it is walked in line
    // after its identity and length prefix.
    while decoder.offset() < bytes.len() {
        let offset = decoder.offset();
        match decoder.next_tag()? {
            TAG_VARIANT | TAG_FIELD | TAG_TEXT => {
                decoder.len_prefixed()?;
            }
            TAG_BOOL | TAG_I8 | TAG_U8 => {
                decoder.raw::<1>()?;
            }
            TAG_I16 | TAG_U16 => {
                decoder.raw::<2>()?;
            }
            TAG_I32 | TAG_U32 => {
                decoder.raw::<4>()?;
            }
            TAG_I64 | TAG_U64 => {
                decoder.raw::<8>()?;
            }
            TAG_UUID => {
                decoder.raw::<16>()?;
            }
            TAG_NAME_REF => references.push(decoder.raw::<32>()?),
            TAG_OPTION_NONE | TAG_OPTION_SOME | TAG_PAIR => {}
            TAG_NESTED_DISCRIMINATOR => {
                decoder.len_prefixed()?;
                decoder.raw::<8>()?;
            }
            found => {
                return Err(ApplicationNameDiscriminatorDecodeDenial::UnexpectedTag {
                    offset,
                    found,
                });
            }
        }
    }
    Ok(references)
}
