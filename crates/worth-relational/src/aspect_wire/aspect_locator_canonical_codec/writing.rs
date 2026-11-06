use worth_foundational::facade::{AspectFieldLocator, AspectValueLocator, LocatorAuthority};

use crate::aspect_wire::{CanonicalAspectOutput, CollectCanonicalAspect};

#[cfg(test)]
use super::error::AspectValueLocatorCanonicalCodecError;
use super::tags;
#[cfg(test)]
use worth_foundational::facade::BoundarySourceLocator;

pub(crate) fn encode_aspect_value_locator(locator: &AspectValueLocator) -> Vec<u8> {
    let mut bytes = Vec::new();
    match write_aspect_value_locator(&mut CollectCanonicalAspect(&mut bytes), locator) {
        Ok(()) => bytes,
        Err(never) => match never {},
    }
}

pub(crate) fn encode_aspect_field_locator(locator: &AspectFieldLocator) -> Vec<u8> {
    let mut bytes = Vec::new();
    match write_aspect_field_locator(&mut CollectCanonicalAspect(&mut bytes), locator) {
        Ok(()) => bytes,
        Err(never) => match never {},
    }
}

pub(crate) fn write_aspect_value_locator<O: CanonicalAspectOutput>(
    out: &mut O,
    locator: &AspectValueLocator,
) -> Result<(), O::Error> {
    match locator {
        AspectValueLocator::WholeAspect(aspect) => {
            out.byte(tags::WHOLE_ASPECT)?;
            encode_authority(out, aspect.authority())?;
            out.text(aspect.aspect_key().as_str())?;
        }
        AspectValueLocator::StructField(field) => write_aspect_field_locator(out, field)?,
    }
    Ok(())
}

pub(crate) fn write_aspect_field_locator<O: CanonicalAspectOutput>(
    out: &mut O,
    locator: &AspectFieldLocator,
) -> Result<(), O::Error> {
    out.byte(tags::STRUCT_FIELD)?;
    encode_authority(out, locator.aspect().authority())?;
    out.text(locator.aspect().aspect_key().as_str())?;
    out.length(locator.field_path().fields().len())?;
    for field in locator.field_path().fields() {
        out.text(field.as_str())?;
    }
    Ok(())
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

fn encode_authority<O: CanonicalAspectOutput>(
    bytes: &mut O,
    authority: LocatorAuthority,
) -> Result<(), O::Error> {
    bytes.byte(match authority {
        LocatorAuthority::Authoritative => tags::AUTHORITY_AUTHORITATIVE,
        LocatorAuthority::Derived => tags::AUTHORITY_DERIVED,
        LocatorAuthority::Projected => tags::AUTHORITY_PROJECTED,
        LocatorAuthority::SupportOnly => tags::AUTHORITY_SUPPORT_ONLY,
        LocatorAuthority::Planned => tags::AUTHORITY_PLANNED,
        LocatorAuthority::ReceiptBearing => tags::AUTHORITY_RECEIPT_BEARING,
    })
}
