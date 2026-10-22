//! Separate producer-created meaning; authored nested source is never output.
use super::{model::NestedValue, schema::*};
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::worth_query_value_binding;
use worth_query_host::facade::declaration::application_operation::*;

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

#[test]
fn authored_edit_regenerates_the_actual_v1_output_without_replacing_its_entity() {
    use super::{host, model, producer, program};
    use crate::document_retention_model::operator_identity::{
        authenticate_operator, request_scope,
    };
    use worth_query_host::facade::application_entry::*;
    let application = host::install(model::authored_value()).unwrap();
    let source_id = host::source_id(&application);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(application.runtime().current_world());
    let mut initial = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(created) =
        initial.settle(&request).unwrap()
    else {
        panic!("the native V1 initial producer must settle");
    };
    assert_eq!(created.producer_contacts_in_this_demand(), 1);
    let created_id = created
        .outputs_of::<DerivedOutputs>()
        .unwrap()
        .entity::<CreatedOutput>()
        .unwrap()
        .entity_id();
    drop(initial);
    let mut edited = model::authored_value();
    edited.title = "regenerated γ".to_owned();
    edited.children[0].rank = 40;
    edited.children[1].rank = -3;
    edited.children[0].tag.0 = "updated-left".to_owned();
    let regenerated = host::edit_source(&application, edited.clone(), 0x271_0101);
    let preserved_id = regenerated
        .outputs_of::<PreservedOutputs>()
        .unwrap()
        .entity::<PreservedOutput>()
        .unwrap()
        .entity_id();
    assert_eq!(preserved_id, created_id);
    let derived = host::read_derived(&application);
    assert_eq!(derived.caption, "REGENERATED Γ");
    assert_eq!(derived.rank_sum, 37);
    assert_eq!(derived.tags, ["updated-left", "right"]);
    assert_eq!(host::source_id(&application), source_id);
    assert_eq!(host::read(&application), edited);
}
