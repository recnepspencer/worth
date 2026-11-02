use serde_json::Value;

use super::JsonCompatibilityLoweringDenial;
use crate::locators::BoundarySourceLocator;
use crate::values::{AspectValue, ContentRefId};

pub(super) fn lower_json_content_ref(
    source: &BoundarySourceLocator,
    value: &Value,
) -> Result<AspectValue, JsonCompatibilityLoweringDenial> {
    value
        .as_str()
        .and_then(ContentRefId::from_hex)
        .map(AspectValue::ContentRef)
        .ok_or_else(|| JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
            source: source.clone(),
            expected: "lowercase 32-byte content digest",
        })
}
