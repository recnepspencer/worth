//! Moving real root/dependent demands releases borrows, never their native custody.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial as Error;

macro_rules! finish {
    ($owner:expr, $app:expr, $request:expr) => {
        (0..256)
            .find_map(
                |_| match $owner.advance($app.runtime(), &$request).unwrap() {
                    WorthQueryApplicationOutputDemandProgress::Pending => None,
                    WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
                },
            )
            .expect("the original native continuation settles")
    };
}

#[test]
fn owned_root_and_dependent_demands_keep_native_custody_across_fresh_requests() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    // No request, principal, scope or application borrow escapes this entrance.
    let (mut root, mut dependent) = {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let root = request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
            .unwrap()
            .into_continuation();
        let dependent = request
            .demand(ChainDemand("anchor-b".into()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                &application,
            )
            .unwrap()
            .into_continuation();
        (root, dependent)
    };
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let root_output = finish!(root, application, request);
    assert_eq!(root_output.producer_contacts_in_this_demand(), 1);
    take_all_decisions();
    let before = request.retain_current_read().unwrap();

    let foreign = chain_application();
    let (foreign_scope, foreign_principal) = authenticate(&foreign);
    let foreign_request = foreign.request(&foreign_principal, &foreign_scope);
    let foreign_before = foreign_request.retain_current_read().unwrap();
    assert!(matches!(
        dependent.advance(application.runtime(), &foreign_request),
        Err(Error::FreshRequestMismatch)
    ));
    assert!(
        matches!(dependent.advance(foreign.runtime(), &foreign_request),
        Err(Error::Demand(denial)) if matches!(denial.kind(),
            primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand))
    );
    assert_eq!(
        foreign_before.selected_commit(),
        foreign_request
            .retain_current_read()
            .unwrap()
            .selected_commit()
    );

    let cancelled = authentication::WorthQueryCancellationSource::new();
    cancelled.cancel();
    let stopped_scope = authentication::WorthQueryRequestScope::new(None, cancelled.token());
    let stopped_request = application.request(&principal, &stopped_scope);
    assert!(
        matches!(dependent.settle(application.runtime(), &stopped_request),
        Err(Error::Demand(denial)) if matches!(denial.kind(),
            primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(_)))
    );
    assert!(
        take_all_decisions().is_empty(),
        "refused requests must not run a producer"
    );
    assert_eq!(
        before.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );

    let output = finish!(dependent, application, request);
    assert_eq!(output.producer_contacts_in_this_demand(), 1);
    assert_eq!(take_decisions("anchor-b"), vec![vec![2]]);
    let current = request.retain_current_read().unwrap();
    request
        .at(&current)
        .require_current_output_demand(
            &output,
            Some((std::num::NonZeroUsize::new(1_000_000).unwrap()).get()),
        )
        .unwrap();
    let original_commit = output
        .application_commit_receipt()
        .unwrap()
        .commit_reference()
        .clone();
    let reused = finish!(dependent, application, request);
    assert_eq!(
        reused
            .application_commit_receipt()
            .unwrap()
            .commit_reference(),
        &original_commit
    );
    assert!(take_decisions("anchor-b").is_empty());
    assert_eq!(
        current.selected_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    dependent.close();
    assert!(matches!(
        dependent.advance(application.runtime(), &request),
        Err(Error::Closed)
    ));
    drop((dependent, root));
    let mut fresh = request
        .demand(ChainDemand("anchor-b".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let reused = settle!(fresh, request);
    assert_eq!(reused.producer_contacts_in_this_demand(), 0);
    assert!(take_decisions("anchor-b").is_empty());
}
