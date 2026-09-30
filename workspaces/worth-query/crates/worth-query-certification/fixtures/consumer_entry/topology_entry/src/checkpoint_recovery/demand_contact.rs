use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandProgress,
};

use super::*;

#[test]
fn direct_demand_reports_its_own_contact_not_the_reused_outputs_history() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let first = settle(&request, &application);
    assert_eq!(first.root_producer_contacts_in_this_demand(), 1);
    drop(first);

    super::super::producer::reset_provider_contacts();
    let reused = direct_demand(&request);
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert_eq!(super::super::producer::provider_contacts(), 0);
    assert!(
        reused
            .readiness_delivery()
            .expect("the committed output retains its historical delivery")
            .producer_contact_count()
            > 0
    );
    assert_small_budget_denied_without_provider(&request);
}

#[test]
fn restored_output_retains_its_resource_profile_without_provider_contact() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application.capture_application_checkpoint().unwrap();
    drop(application);

    let restored = support::install_with_demand_profile(
        Some(checkpoint),
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::bounded(
            NonZeroUsize::new(4_096).unwrap(),
            NonZeroUsize::new(4_096).unwrap(),
            NonZeroUsize::new(8_192).unwrap(),
            NonZeroUsize::new(32).unwrap(),
        ),
    );
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    super::super::producer::reset_provider_contacts();
    assert_small_budget_denied_without_provider(&request);
    let reused = direct_demand(&request);
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert_eq!(super::super::producer::provider_contacts(), 0);
}

#[test]
fn restored_output_denies_smaller_host_work_without_provider_contact_or_effects() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application.capture_application_checkpoint().unwrap();
    drop(application);

    super::super::producer::reset_provider_contacts();
    let restored = support::install_with_demand_profile(
        Some(checkpoint),
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::bounded(
            NonZeroUsize::new(4_096).unwrap(),
            NonZeroUsize::new(4_095).unwrap(),
            NonZeroUsize::new(8_192).unwrap(),
            NonZeroUsize::new(32).unwrap(),
        ),
    );
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let before = request.retain_read().unwrap();
    let output = PlanarOutputRead {
        body_key: "final:anchor-a".to_owned(),
    };
    let before_output = request.query(output.clone()).execute().unwrap();
    assert!(!before_output.rows().is_empty());

    // No caller restriction: the persisted 4,096-work estimate must be checked
    // against this fresh host's 4,095 limit without asking the producer again.
    let denied = match request.demand(PlanarOutputDemand::new("anchor-a")).start() {
        Ok(_) => panic!("restored resource requirements exceed the reopened host policy"),
        Err(denied) => denied,
    };
    assert!(matches!(
        denied,
        WorthQueryApplicationOutputDemandDenial::Demand(cause)
            if cause.kind() == WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                && cause.subject() == "worth.query.certification.planar-initial.v1"
    ));
    assert_eq!(super::super::producer::provider_contacts(), 0);
    let after = request.retain_read().unwrap();
    let after_output = request.query(output).execute().unwrap();
    assert_eq!(before.selected_commit(), after.selected_commit());
    assert_eq!(before_output.rows(), after_output.rows());
    assert_eq!(super::super::producer::provider_contacts(), 0);
}

#[test]
fn restored_final_output_keeps_original_create_producer_and_zero_contact() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application.capture_application_checkpoint().unwrap();
    drop(application);

    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .controls(controls)
        .start()
        .expect("the original final output is restorable");
    let mut settled = None;
    for _ in 0..512 {
        match demand.advance(&request).expect("the final demand advances") {
            WorthQueryApplicationOutputDemandProgress::Pending => std::thread::yield_now(),
            WorthQueryApplicationOutputDemandProgress::Settled(output) => {
                settled = Some(output);
                break;
            }
        }
    }
    let settled = settled.expect("the original final output settles within its bounded graph");
    assert_eq!(settled.producer_contacts_in_this_demand(), 0);
    // The readmitted correspondence is erased: reading it as another
    // contract is refused once, at the typed view.
    assert_eq!(
        settled.outputs_of::<FinalPlanarPreserveOutputs>().err(),
        Some(primary_graph::WorthQueryApplicationOutputProjectionDenial::ForeignContract)
    );
    let outputs = settled
        .outputs_of::<FinalPlanarOutputs>()
        .expect("the readmitted outputs were committed under the final contract");
    assert!(outputs
        .entity::<FinalAnchorOutput<CheckpointSchema>>()
        .is_ok());
    // Readmitted optional roles stay values: the bound closing vertex is
    // present and the never-bound auxiliary role is absent, not missing.
    assert!(outputs
        .entity::<FinalClosingOutput<CheckpointSchema>>()
        .expect("the readmitted closing role reads as a value")
        .is_some());
    assert!(outputs
        .entity::<FinalAuxiliaryOutput<CheckpointSchema>>()
        .expect("the readmitted auxiliary role reads as a value")
        .is_none());
}

fn assert_small_budget_denied_without_provider(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        CheckpointSchema,
    >,
) {
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_191).unwrap(),
    );
    let denied = match request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls)
        .start()
    {
        Ok(_) => panic!("a smaller retained-byte budget must deny reuse"),
        Err(denied) => denied,
    };
    assert!(matches!(
        denied,
        WorthQueryApplicationOutputDemandDenial::Demand(cause)
            if cause.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
    ));
    assert_eq!(super::super::producer::provider_contacts(), 0);
}

fn direct_demand<'application, 'principal, 'scope>(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        'principal,
        'scope,
        CheckpointSchema,
    >,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandSettlement<
    PlanarQuery,
> {
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls)
        .start()
        .expect("the direct output demand starts");
    for _ in 0..512 {
        match demand.advance(request).expect("the direct demand advances") {
            WorthQueryApplicationOutputDemandProgress::Pending => std::thread::yield_now(),
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => return settled,
        }
    }
    panic!("the direct output demand did not settle within its synchronous bound")
}
