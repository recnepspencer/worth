pub(crate) use worth_query_testkit_certification_canonical::facade::certification::canonical_transition::output::*;

#[test]
fn authored_edit_regenerates_the_actual_v1_output_without_replacing_its_entity() {
    use super::{host, model, producer, program};
    use worth_query_host::facade::application_entry::*;
    use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
        authenticate_operator, request_scope,
    };
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
