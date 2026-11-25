pub(crate) use worth_query_testkit_certification_canonical::facade::certification::canonical_transition::*;

#[test]
fn accepted_created_canonical_output_reopens_cleanly_without_changing_source() {
    let authored = model::authored_value();
    assert!(!authored.children.is_empty());
    let dir = worth_query_testkit_document_retention::facade::directory_home::unique_directory(
        "canonical-transition-reopen",
    );
    let host = host::open_directory(dir, authored.clone()).unwrap();
    let source_id = host::source_id(&host);
    let expected = output::calculate(&authored);
    assert_eq!(expected.caption, "ASSEMBLY Β");
    assert_eq!(expected.rank_sum, 12);
    assert_eq!(expected.tags, ["left", "right"]);
    assert_eq!(host::read(&host), authored);
    let runtime = host.runtime();
    let retained_catalogue = host.installed_schema().value_bindings().clone();
    assert!(retained_catalogue.same_installation_as(host.installed_schema().value_bindings()));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = runtime
        .request(&principal, &scope)
        .on_branch(runtime.current_world());
    let mut demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&host)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("the real installed producer must settle");
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    assert!(settled.application_commit_receipt().is_some());
    let created = settled
        .outputs_of::<output::DerivedOutputs>()
        .unwrap()
        .entity::<output::CreatedOutput>()
        .unwrap();
    let created_id = created.entity_id();
    assert_eq!(host::read_derived(&host), expected);
    assert_eq!(
        host::source_id(&host),
        source_id,
        "created output must preserve the native authored source entity"
    );
    assert_eq!(
        host::read(&host),
        authored,
        "derived publication must preserve authored nested source"
    );
    drop(settled);
    drop(demand);
    drop(principal);
    drop(scope);
    let dir = host.checkpoint_and_close_for_reopen();
    let restored = host::open_directory(dir, authored.clone()).unwrap();
    assert!(!retained_catalogue.same_installation_as(restored.installed_schema().value_bindings()));
    assert_eq!(
        host::source_id(&restored),
        source_id,
        "current-format restore must preserve the native authored source entity"
    );
    assert_eq!(host::read(&restored), authored);
    assert_eq!(host::read_derived(&restored), expected);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(restored.installed_schema(), &fresh_scope);
    let fresh = restored
        .runtime()
        .request(&fresh_principal, &fresh_scope)
        .on_branch(restored.runtime().current_world());
    let mut reopened = fresh
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&restored)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(reused) =
        reopened.settle(&fresh).unwrap()
    else {
        panic!("restored native output settles");
    };
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert_eq!(
        reused
            .outputs_of::<output::DerivedOutputs>()
            .unwrap()
            .entity::<output::CreatedOutput>()
            .unwrap()
            .entity_id(),
        created_id
    );
}

#[test]
fn derived_successor_wire_is_incompatible_while_authored_source_codec_is_unchanged() {
    use worth_foundational::facade::{
        decode_canonical_value, encode_canonical_value, ValueDecodeBounds,
    };
    let bounds = ValueDecodeBounds::unrestricted();
    let v1 = output::calculate(&model::authored_value());
    let wire = encode_canonical_value(&v1, bounds).unwrap();
    assert!(decode_canonical_value::<output::DerivedV2>(&wire, bounds).is_err());
    assert_eq!(
        decode_canonical_value::<output::DerivedV1>(&wire, bounds).unwrap(),
        v1
    );
}

pub(crate) mod output;

pub(crate) mod related;

pub(crate) mod producer_freshness;
pub(crate) mod successor;

mod source_authorization;
