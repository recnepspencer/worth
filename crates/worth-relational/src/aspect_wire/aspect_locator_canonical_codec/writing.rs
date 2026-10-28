use worth_foundational::facade::{AspectFieldLocator, AspectValueLocator, LocatorAuthority};

use crate::aspect_wire::{encode_string, encode_u32};

#[cfg(test)]
use super::error::AspectValueLocatorCanonicalCodecError;
use super::tags;
#[cfg(test)]
use worth_foundational::facade::BoundarySourceLocator;

pub(crate) fn encode_aspect_value_locator(locator: &AspectValueLocator) -> Vec<u8> {
    let mut bytes = Vec::new();
    match locator {
        AspectValueLocator::WholeAspect(aspect) => {
            bytes.push(tags::WHOLE_ASPECT);
            encode_authority(&mut bytes, aspect.authority());
            encode_string(&mut bytes, aspect.aspect_key().as_str());
        }
        AspectValueLocator::StructField(field) => {
            bytes.push(tags::STRUCT_FIELD);
            encode_authority(&mut bytes, field.aspect().authority());
            encode_string(&mut bytes, field.aspect().aspect_key().as_str());
            encode_u32(&mut bytes, field.field_path().fields().len() as u32);
            for field_key in field.field_path().fields() {
                encode_string(&mut bytes, field_key.as_str());
            }
        }
    }
    bytes
}

pub(crate) fn encode_aspect_field_locator(locator: &AspectFieldLocator) -> Vec<u8> {
    encode_aspect_value_locator(&AspectValueLocator::struct_field(locator.clone()))
}

#[cfg(test)]
pub(crate) fn encode_boundary_source_locator(
    locator: &BoundarySourceLocator,
) -> Result<Vec<u8>, AspectValueLocatorCanonicalCodecError> {
    match locator {
        BoundarySourceLocator::Aspect(aspect) => Ok(encode_aspect_value_locator(
            &AspectValueLocator::whole_aspect(aspect.clone()),
        )),
        BoundarySourceLocator::AspectField(field) => Ok(encode_aspect_value_locator(
            &AspectValueLocator::struct_field(field.clone()),
        )),
        BoundarySourceLocator::BoundaryArtifact(_) => {
            Err(AspectValueLocatorCanonicalCodecError::new(
                "canonical aspect source locator bytes do not encode boundary artifact locators",
            ))
        }
    }
}

fn encode_authority(bytes: &mut Vec<u8>, authority: LocatorAuthority) {
    bytes.push(match authority {
        LocatorAuthority::Authoritative => tags::AUTHORITY_AUTHORITATIVE,
        LocatorAuthority::Derived => tags::AUTHORITY_DERIVED,
        LocatorAuthority::Projected => tags::AUTHORITY_PROJECTED,
        LocatorAuthority::SupportOnly => tags::AUTHORITY_SUPPORT_ONLY,
        LocatorAuthority::Planned => tags::AUTHORITY_PLANNED,
        LocatorAuthority::ReceiptBearing => tags::AUTHORITY_RECEIPT_BEARING,
    });
}
