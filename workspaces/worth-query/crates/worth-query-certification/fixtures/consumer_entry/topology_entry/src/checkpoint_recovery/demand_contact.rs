use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandProgress,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryPreserveOutput,
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

    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    super::super::producer::reset_provider_contacts();
    assert_small_budget_denied_without_provider(&request);
    let reused = direct_demand(&request);
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
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
    assert!(settled
        .output_correspondence()
        .entity(WorthQueryApplicationOutputRole::<
            FinalPlanarMutationBinding<CheckpointSchema>,
            Body,
            WorthQueryCreateOutput,
        >::from_static("anchor"))
        .is_ok());
    assert!(settled
        .output_correspondence()
        .entity(WorthQueryApplicationOutputRole::<
            FinalPlanarPreserveBinding<CheckpointSchema>,
            Body,
            WorthQueryPreserveOutput,
        >::from_static("anchor"))
        .is_err());
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
