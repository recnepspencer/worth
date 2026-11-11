//! Retained field-revision fact shape decoding; no currentness or progression grant.
use super::*;
use worth_foundational::facade::{CanonicalFieldPath, FieldKey, LocatorAuthority};

pub fn decode_field_revision_fact(
    entity: EntityId,
    aspect: Option<String>,
    field: Option<String>,
    native_revision: Option<u64>,
    field_presence: Option<String>,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryApplicationAttemptDenial> {
    let aspect = aspect
        .and_then(AspectKey::new)
        .ok_or_else(|| denial("workflow evidence dependency field aspect is invalid"))?;
    let field = field
        .and_then(FieldKey::new)
        .ok_or_else(|| denial("workflow evidence dependency field key is invalid"))?;
    let native_revision = match (native_revision, field_presence.as_deref()) {
        (Some(version), Some("present")) => Some(
            worth_relational::facade::runtime::RelationalFieldRevision::new(
                VersionId(version),
                worth_relational::facade::runtime::RelationalFieldPresence::Present,
            ),
        ),
        (Some(version), Some("absent")) => Some(
            worth_relational::facade::runtime::RelationalFieldRevision::new(
                VersionId(version),
                worth_relational::facade::runtime::RelationalFieldPresence::Absent,
            ),
        ),
        (None, None) => None,
        _ => {
            return Err(denial(
                "workflow evidence dependency field revision is invalid",
            ))
        }
    };
    Ok(WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: entity,
        locator: AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            aspect,
            CanonicalFieldPath::single(field),
        ),
        native_revision,
    })
}
