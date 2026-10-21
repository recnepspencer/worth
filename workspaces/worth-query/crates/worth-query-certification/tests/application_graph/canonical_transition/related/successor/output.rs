//! Real V2 wire owner; refusal observation never supplies a synthetic denial.
use super::{model::NestedValue, wire_model::DerivedV2 as Wire};
use worth_foundational::facade::{
    CanonicalValue, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds, ValueShapeBody,
};
use worth_query_decl::facade::worth_query_value_binding;
#[derive(Clone, Debug, PartialEq)]
pub struct DerivedValue(pub Wire);
impl serde::Serialize for DerivedValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
thread_local! { static DECODES:std::cell::Cell<u32> = const {std::cell::Cell::new(0)}; static IGNORED:std::cell::Cell<u32> = const {std::cell::Cell::new(0)}; }
pub fn reset_decodes() {
    DECODES.set(0);
    IGNORED.set(0);
}
pub fn decodes() -> u32 {
    DECODES.get()
}
pub fn ignored_refusals() -> u32 {
    IGNORED.get()
}
impl CanonicalValue for DerivedValue {
    const WIRE_NULLABLE: bool = Wire::WIRE_NULLABLE;
    const SUPPORTS_ADMISSION: bool = true;
    const SHAPE_BODY: ValueShapeBody = { Wire::SHAPE_BODY };
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), ValueCodecDenial> {
        self.0.validate_bounds(bounds)
    }
    fn deserialize_bounded<'de, D: serde::Deserializer<'de>>(
        decoder: D,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, D::Error> {
        DECODES.set(DECODES.get() + 1);
        Wire::deserialize_bounded(decoder, bounds).map(Self)
    }
    fn deserialize_admitted<'de, D: serde::Deserializer<'de>>(
        decoder: D,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, D::Error> {
        DECODES.set(DECODES.get() + 1);
        Wire::deserialize_admitted(decoder, bounds, &mut |work, bytes| {
            if admission(work, bytes).is_err() {
                IGNORED.set(IGNORED.get() + 1);
            }
            Ok(())
        })
        .map(Self)
    }
}
pub fn calculate(source: &NestedValue) -> DerivedValue {
    DerivedValue(Wire {
        caption: format!("V2 {}", source.title.to_uppercase()),
        total_rank: source
            .children
            .iter()
            .map(|child| i64::from(child.rank))
            .sum::<i64>()
            + 1000,
        labels: source
            .children
            .iter()
            .map(|child| format!("v2:{}", child.tag.0))
            .collect(),
    })
}
pub fn literals(value: &DerivedValue) -> (&str, i64, &[String]) {
    (&value.0.caption, value.0.total_rank, &value.0.labels)
}
pub const HANDLER_IDENTITY: &str = "canonical-related.produce.handler.v2";
pub const PROVIDER_IDENTITY: &str = "canonical-related.provider.v2";
worth_query_value_binding! { pub DerivedBinding for DerivedValue { identity:"canonical-related.derived.codec.v2",scalar:String,canonical:ValueDecodeBounds::new(8192,512,32,12), } }
#[path = "../output/roles.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "all output roles are compiled against the actual target schema"
)]
mod roles;
pub use roles::*;

pub const PRESERVE_HANDLER_IDENTITY: &str = "canonical-related.preserve.handler.v2";
pub const PRESERVE_PROVIDER_IDENTITY: &str = "canonical-related.preserve.provider.v2";
