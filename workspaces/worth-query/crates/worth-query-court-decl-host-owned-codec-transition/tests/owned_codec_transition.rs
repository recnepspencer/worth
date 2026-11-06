//! A real owned publication is retired by Native's authenticated codec transition.
use v1::{successor as v2, *};
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_testkit_certification_canonical::facade::certification::source_owned_transition as v1;

#[test]
fn same_route_codec_retirement_covers_the_source_owned_origin() {
    let authored = model::authored_value();
    let before = host::install(authored.clone()).unwrap();
    let source = host::source_id(&before);
    let scope = request_scope();
    let principal = authenticate_operator(before.installed_schema(), &scope);
    let request = before.runtime().request(&principal, &scope);
    let mut demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&before)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("actual V1 producer must settle")
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    let old = settled
        .outputs_of::<output::DerivedOutputs>()
        .unwrap()
        .entity::<output::CreatedOutput>()
        .unwrap()
        .entity_id();
    let checkpoint = before
        .runtime()
        .with_host_advancement(|phase| {
            before
                .runtime()
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    let original = checkpoint.bytes().to_vec();
    let predecessor = installation::WorthQueryCheckpointProgramPredecessor::new(
        &host::program().revision().to_string(),
    )
    .unwrap();
    drop(demand);
    drop(before);
    let after = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        v2::host::program(),
        installation::WorthQueryApplicationProgramRoster::new(),
        v2::schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        v2::host::limits(),
        checkpoint.clone(),
        predecessor,
        installation::WorthQueryCheckpointTransitionResources::bounded(4096, 32, 16384).unwrap(),
        CapturePolicy::SystemAllocation,
        |_, _| Ok(()),
    )
    .expect("same-route codec retirement carries actual ownership coverage");
    assert_eq!(checkpoint.bytes(), original);
    assert_eq!(v2::host::source_id(&after), source);
    assert_eq!(v2::host::read(&after), authored);
    // Reopen while the route is codec-retired, before a fresh publication replaces it.
    // Its independent presence and retained publication row must still agree.
    let retired = after
        .runtime()
        .with_host_advancement(|phase| {
            after
                .runtime()
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    drop(after);
    // A codec-retired route has no live output retirement origin. Its authored
    // source may now be deleted ordinarily; the position owner must remove its
    // retained presence in that same real Query commit.
    let deleted = v2::host::restore(retired.clone());
    let delete_scope = request_scope();
    let delete_principal = authenticate_operator(deleted.installed_schema(), &delete_scope);
    let delete_request = deleted.runtime().request(&delete_principal, &delete_scope);
    let outcome = delete_request
        .advanced()
        .mutate(v2::retire_source::Input {
            identity: "authored-nested".to_owned(),
        })
        .without_source()
        .idempotency(&8127u64)
        .execute_in_program(
            &deleted,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    let contacts = deleted
        .runtime()
        .current_ownership_contacts_for_identity_for_test(source, &delete_scope)
        .unwrap();
    assert!(contacts.publication_rows.is_empty());
    assert!(contacts.presence_rows.is_empty());
    let removed = deleted
        .runtime()
        .with_host_advancement(|phase| {
            deleted
                .runtime()
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    drop(deleted);
    let deleted = v2::host::restore(removed);
    let contacts = deleted
        .runtime()
        .current_ownership_contacts_for_identity_for_test(source, &delete_scope)
        .unwrap();
    assert!(contacts.publication_rows.is_empty());
    assert!(contacts.presence_rows.is_empty());
    drop(deleted);
    let after = v2::host::restore(retired);
    assert_eq!(v2::host::source_id(&after), source);
    let scope = request_scope();
    let principal = authenticate_operator(after.installed_schema(), &scope);
    let request = after.runtime().request(&principal, &scope);
    let mut demand = request
        .demand(v2::producer::Demand)
        .start_in_program::<_, v2::program::Root>(&after)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&request).unwrap()
    else {
        panic!("actual V2 producer must settle")
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    let fresh = settled
        .outputs_of::<v2::output::DerivedOutputs>()
        .unwrap()
        .entity::<v2::output::CreatedOutput>()
        .unwrap()
        .entity_id();
    assert_ne!(old, fresh);
    assert_eq!(
        v2::host::read_derived(&after),
        v2::output::calculate(&authored)
    );
    let checkpoint = after
        .runtime()
        .with_host_advancement(|phase| {
            after
                .runtime()
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    drop(demand);
    drop(after);
    let reopened = v2::host::restore(checkpoint);
    assert_eq!(v2::host::source_id(&reopened), source);
    assert_eq!(v2::host::read(&reopened), authored);
    v2::assert_edit_regenerates(&reopened);
}
