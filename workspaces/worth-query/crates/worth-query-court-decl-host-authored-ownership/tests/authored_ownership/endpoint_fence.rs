//! Query creates real custody; the installed receiver probes Native's endpoint verb.
use super::{
    correction_courts::install_definition,
    coupled::multi::{create, settle, TwoProgram},
    lifecycle::*,
    metadata_guard::memory_basis_state,
    *,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    publication::AuthoredRetirementRefusal,
    transactions::{RecordRef, TransactionCommitError},
};
#[test]
fn external_owned_edge_endpoint_retarget_refuses_before_any_effect() {
    run::<6>();
}
#[test]
fn an_owned_edge_with_only_its_target_as_member_refuses_external_retargeting() {
    run::<66>();
}
fn run<const MODE: u8>() {
    type S<const M: u8> = Model<M>;
    // TwoProgram declares correction output material, which Directory Home refuses
    // with UnsupportedCorrectionOutputMaterial; this court tests Native's in-process
    // endpoint retarget fence on MemoryOnly and checks full World + Native + Query
    // basis commitment equality before and after the refused probe.
    let host = install_definition::<S<MODE>, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let body = create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let target = create(&host, &request, 40, vec![])
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S<MODE>>>()
        .unwrap()
        .entity_id();
    let expected = super::current_fence::denied(&host)
        .authored_retirement_refusal()
        .unwrap();
    let AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { publication, owner } =
        expected
    else {
        panic!("{expected:?}")
    };
    let stage = body
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<super::co_roles::CoCreated<S<MODE>, 0>>()
        .unwrap()
        .unwrap()
        .entity_id();
    if MODE == 6 {
        host.runtime()
            .probe_owned_edge_retarget_for_test(body.product_branch(), owner, stage, &scope)
            .unwrap()
            .expect("unchanged endpoints keep existing custody");
    } else {
        let external = host
            .runtime()
            .current_ownership_contacts_for_record_for_test(stage, &scope)
            .unwrap();
        assert!(
            external.publication_rows.is_empty(),
            "edge source is not a member or scope"
        );
        assert!(
            external.presence_rows.is_empty(),
            "edge source scopes no route"
        );
    }
    let before_world = host
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(host.current_world())
                .select(&phase)
                .unwrap()
                .product()
                .observation_for_test()
                .clone()
        })
        .unwrap();
    let before = memory_basis_state(&host);
    let refusal = host
        .runtime()
        .probe_owned_edge_retarget_for_test(body.product_branch(), owner, target, &scope)
        .unwrap()
        .expect_err("an external endpoint change must fail at Native admission");
    let TransactionCommitError::Publication { error, .. } = refusal else {
        panic!("{refusal:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { publication, owner })
    );
    assert!(
        matches!(error.context.affected_records.as_slice(),[RecordRef::Relation(_),RecordRef::Entity(actual)] if *actual==owner)
    );
    let after = memory_basis_state(&host);
    assert_eq!(
        after, before,
        "Native preparation published no World, Native, or Query state change"
    );
    host.with_host_advancement(|phase| {
        let selected = host
            .runtime()
            .on_branch(host.current_world())
            .select(&phase)
            .unwrap();
        assert_eq!(
            &before_world,
            selected.product().observation_for_test(),
            "no World product movement"
        );
    })
    .unwrap();
    settle(&host, &request, 10);
    assert!(request.query(query::Input { id: 20 }).execute().is_ok());
}
