use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandProgress;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

use super::*;

#[test]
fn direct_demand_reports_its_own_contact_not_the_reused_outputs_history() {
    let _guard = checkpoint_recovery_test_guard();
    let provider_observation = crate::producer::ProviderContacts::default();
    let application = support::observed::install(None, &provider_observation);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let first = settle(&request, &application);
    assert_eq!(first.root_producer_contacts_in_this_demand(), 1);
    drop(first);

    provider_observation.reset();
    let reused = direct_demand(&request);
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert_eq!(provider_observation.count(), 0);
    assert!(
        reused
            .readiness_delivery()
            .expect("the committed output retains its historical delivery")
            .producer_contact_count()
            > 0
    );
}

#[test]
fn restored_output_reuses_under_receiving_host_policy_without_provider_contact() {
    let _guard = checkpoint_recovery_test_guard();
    let provider_observation = crate::producer::ProviderContacts::default();
    let application = support::observed::install(None, &provider_observation);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application
        .with_host_advancement(|host_phase| {
            application
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .expect("installed host call opens");
    drop(application);

    let restored = support::observed::install_program::<CheckpointProgram>(
        Some(checkpoint),
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::new(
            NonZeroUsize::new(32).unwrap(),
        ),
        &provider_observation,
    );
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    provider_observation.reset();
    let reused = direct_demand(&request);
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert_eq!(provider_observation.count(), 0);
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
    let checkpoint = application
        .with_host_advancement(|host_phase| {
            application
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .expect("installed host call opens");
    drop(application);

    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let controls = WorthQueryOutputDemandControls::default();
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .controls(controls)
        .start()
        .expect("the original final output is restorable");
    let mut settled = None;
    for _ in 0..512 {
        match demand.advance(&request).expect("the final demand advances") {
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => std::thread::yield_now(),
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
    let controls = WorthQueryOutputDemandControls::default();
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls)
        .start()
        .expect("the direct output demand starts");
    for _ in 0..512 {
        match demand.advance(request).expect("the direct demand advances") {
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => std::thread::yield_now(),
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => return settled,
        }
    }
    panic!("the direct output demand did not settle within its synchronous bound")
}
