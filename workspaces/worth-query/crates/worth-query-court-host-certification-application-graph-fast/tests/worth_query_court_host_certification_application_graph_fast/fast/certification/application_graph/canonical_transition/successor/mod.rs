pub(crate) use worth_query_worlds::facade::certification::canonical_transition::successor::*;

pub(crate) mod acknowledged_capture;

pub(crate) mod adoption;

pub(crate) mod historical;

pub(crate) mod pre_effect_indexes;

#[test]
fn actual_v2_installation_produces_and_reads_its_supported_incompatible_output() {
    let authored = model::authored_value();
    let application = host::install(authored.clone()).unwrap();
    let source_id = host::source_id(&application);
    output::reset_decodes();
    let scope = worth_query_worlds::facade::certification::document_retention_model::operator_identity::request_scope();
    let principal = worth_query_worlds::facade::certification::document_retention_model::operator_identity::authenticate_operator(
        application.installed_schema(),
        &scope,
    );
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(application.runtime().current_world());
    let mut demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("real V2 producer must settle");
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    assert!(settled.application_commit_receipt().is_some());
    let derived = host::read_derived(&application);
    assert_eq!(derived.0.caption, "ASSEMBLY Β");
    assert_eq!(derived.0.total_rank, 12);
    assert_eq!(derived.0.labels, ["left", "right"]);
    assert!(
        output::decodes() > 0,
        "ordinary installed V2 read must enter its actual decoder"
    );
    assert_eq!(host::read(&application), authored);
    assert_eq!(host::source_id(&application), source_id);
    let checkpoint = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    drop(demand);
    drop(application);
    let restored = host::restore(checkpoint);
    assert_eq!(host::source_id(&restored), source_id);
    assert_eq!(host::read_derived(&restored), output::calculate(&authored));
    assert_eq!(host::read(&restored), authored);
    assert_edit_regenerates(&restored);
}

#[test]
fn exact_v2_restore_requires_explicit_transition_before_any_derived_decode() {
    let checkpoint = predecessor_checkpoint();
    output::reset_decodes();
    let denial = installation::in_memory_program_from_checkpoint(
        host::program(),
        schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host::limits(),
        checkpoint.clone(),
    )
    .err()
    .expect("incompatible recovered V1 field cannot enter V2 ordinary restore");
    assert_eq!(
        output::decodes(),
        0,
        "codec fence must precede V2 payload interpretation"
    );
    let installation::WorthQueryInMemoryApplicationDenial::Graph(denial) = denial else {
        panic!("expected native checkpoint graph refusal: {denial:?}");
    };
    assert_eq!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected);
    assert!(
        denial
            .subject()
            .contains("recovered derived codec requires authenticated native transition admission"),
        "unexpected recovery fence: {denial:?}"
    );
    unchanged_revision::assert_missing_revision_refused(checkpoint);
}
