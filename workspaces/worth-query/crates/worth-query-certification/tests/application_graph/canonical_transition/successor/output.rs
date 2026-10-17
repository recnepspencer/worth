//! Actual compiled V2 codec owner; the probe observes decoder entry only.
use super::{model::NestedValue, schema::*, wire_model::DerivedV2 as WireV2};
use worth_foundational::facade::{
    CanonicalValue, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds, ValueShape,
};
use worth_query_decl::facade::worth_query_value_binding;
use worth_query_host::facade::declaration::application_operation::*;

thread_local! { static DECODES: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
pub fn reset_decodes() {
    DECODES.set(0);
}
pub fn decodes() -> u32 {
    DECODES.get()
}
#[derive(Clone, Debug, PartialEq)]
pub struct DerivedV2(pub WireV2);
pub use DerivedV2 as DerivedValue;
impl serde::Serialize for DerivedV2 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
impl CanonicalValue for DerivedV2 {
    const SHAPE_DEPTH: usize = WireV2::SHAPE_DEPTH;
    const WIRE_NULLABLE: bool = WireV2::WIRE_NULLABLE;
    const SUPPORTS_ADMISSION: bool = true;
    fn value_shape() -> ValueShape {
        WireV2::value_shape()
    }
    fn validate_bounds(&self, bounds: ValueDecodeBounds) -> Result<(), ValueCodecDenial> {
        self.0.validate_bounds(bounds)
    }
    fn deserialize_bounded<'de, D: serde::Deserializer<'de>>(
        decoder: D,
        bounds: ValueDecodeBounds,
    ) -> Result<Self, D::Error> {
        DECODES.set(DECODES.get() + 1);
        WireV2::deserialize_bounded(decoder, bounds).map(Self)
    }
    fn deserialize_admitted<'de, D: serde::Deserializer<'de>>(
        decoder: D,
        bounds: ValueDecodeBounds,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self, D::Error> {
        DECODES.set(DECODES.get() + 1);
        WireV2::deserialize_admitted(decoder, bounds, admission).map(Self)
    }
}
pub fn calculate(source: &NestedValue) -> DerivedValue {
    // The same source calculation, expressed in V2's incompatible field names.
    DerivedV2(WireV2 {
        caption: source.title.to_uppercase(),
        total_rank: source
            .children
            .iter()
            .map(|child| i64::from(child.rank))
            .sum(),
        labels: source
            .children
            .iter()
            .map(|child| child.tag.0.clone())
            .collect(),
    })
}
worth_query_value_binding! { pub DerivedBinding for DerivedValue {
    identity: "canonical-transition.derived.codec.v2", scalar: String,
    canonical: ValueDecodeBounds::new(8192, 512, 32, 12),
}}
pub struct DerivedOutputs;
pub struct CreatedOutput;
impl WorthQueryApplicationOutputRole for CreatedOutput {
    type Schema = CanonicalSchema;
    type Contract = DerivedOutputs;
    type Entity = Derived;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-report";
}
impl ApplicationMutationOutputContract<CanonicalSchema> for DerivedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[CreatedOutput::DESCRIPTOR];
}
