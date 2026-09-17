#[cfg(any(test, feature = "certification-construction"))]
use std::collections::BTreeMap;

#[cfg(any(test, feature = "certification-construction"))]
use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey};
#[cfg(any(test, feature = "certification-construction"))]
use worth_query::facade::foundation::{WorthQueryEntity, WorthQueryEntityIdentity};
#[cfg(any(test, feature = "certification-construction"))]
use worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts;

#[cfg(any(test, feature = "certification-construction"))]
use crate::installed_domain::scalar_text_projection::PLATFORM_PULSE_STATUS_IDENTITY;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiScalarProjectionSourceRecord {
    status: String,
    revision: u64,
}

impl WorthUiScalarProjectionSourceRecord {
    pub fn new(status: impl Into<String>, revision: u64) -> Result<Self, &'static str> {
        let status = status.into();
        if status.is_empty() {
            return Err("scalar projection status must not be empty");
        }
        if status.len() > 65_536 {
            return Err("scalar projection status exceeds the 65,536-byte product budget");
        }
        Ok(Self { status, revision })
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    #[cfg(any(test, feature = "certification-construction"))]
    pub(crate) fn into_query_entity(self) -> WorthQueryEntity {
        WorthQueryEntity::from_native_field_values(
            platform_pulse_entity_identity(),
            BTreeMap::from([
                (
                    field_path("identity", "id"),
                    AspectValue::String(PLATFORM_PULSE_STATUS_IDENTITY.into()),
                ),
                (
                    field_path("query_text", "status"),
                    AspectValue::String(self.status.into()),
                ),
                (
                    field_path("query_revision", "value"),
                    AspectValue::UInt64(self.revision),
                ),
            ]),
        )
    }
}

#[cfg(any(test, feature = "certification-construction"))]
pub(crate) fn platform_pulse_entity_identity() -> WorthQueryEntityIdentity {
    WorthQueryEntityIdentity::from_bridge_record_projection(
        RelationalBridgeRecordIdentityParts::entity(313, 1, 1),
    )
}

#[cfg(any(test, feature = "certification-construction"))]
fn field_path(aspect: &str, field: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new([
        FieldKey::new(aspect).expect("static aspect key must admit"),
        FieldKey::new(field).expect("static field key must admit"),
    ])
    .expect("two-part native field path must admit")
}
