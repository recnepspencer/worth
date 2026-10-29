//! Separate producer-created meaning; authored nested source is never output.
pub use super::{model::NestedValue, schema::*};
pub use worth_foundational::facade::WorthValue;
pub use worth_query_decl::facade::worth_query_value_binding;
pub use worth_query_host::facade::declaration::application_operation::*;

#[derive(Clone, Debug, PartialEq, WorthValue)]
pub struct DerivedV1 {
    pub caption: String,
    pub rank_sum: i64,
    pub tags: Vec<String>,
}
pub use DerivedV1 as DerivedValue;
#[derive(Clone, Debug, PartialEq, WorthValue)]
pub struct DerivedV2 {
    pub caption: String,
    pub total_rank: i64,
    pub labels: Vec<String>,
}
pub fn calculate(source: &NestedValue) -> DerivedV1 {
    DerivedV1 {
        caption: source.title.to_uppercase(),
        rank_sum: source
            .children
            .iter()
            .map(|child| i64::from(child.rank))
            .sum(),
        tags: source
            .children
            .iter()
            .map(|child| child.tag.0.clone())
            .collect(),
    }
}
worth_query_value_binding! { pub DerivedBinding for DerivedV1 {
    identity: "canonical-transition.derived.codec.v1", scalar: String,
    canonical,
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

pub struct PreservedOutputs;
pub struct PreservedOutput;
impl WorthQueryApplicationOutputRole for PreservedOutput {
    type Schema = CanonicalSchema;
    type Contract = PreservedOutputs;
    type Entity = Derived;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "derived-report";
}
impl ApplicationMutationOutputContract<CanonicalSchema> for PreservedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[PreservedOutput::DESCRIPTOR];
}
