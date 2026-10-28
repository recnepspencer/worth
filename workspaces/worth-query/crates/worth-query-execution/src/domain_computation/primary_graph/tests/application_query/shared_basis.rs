//! Exact-custody owner proofs; these do not claim public wave completion.
use super::super::fixture::installed_authorization_world;
use crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop;
use crate::domain_computation::primary_graph::WorthQueryApplicationBasisReleaseOutcome;
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[test]
fn shared_basis_keeps_one_exact_native_owner_until_last_explicit_release() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();

    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let snapshot = selected.application_basis().snapshot_handle().clone();
    let identity = selected.application_basis().identity().clone();
    let shared = match selected.prepare_shared_query_basis() {
        Ok(shared) => shared,
        Err((_, stop)) => panic!("funded sharing must prepare: {stop:?}"),
    };
    let (product_a, basis_a) = world
        .application
        .retain_selected_query_basis_admitted(&shared)
        .unwrap();
    let (product_b, basis_b) = world
        .application
        .retain_selected_query_basis_admitted(&shared)
        .unwrap();
    assert_eq!(basis_a.identity(), &identity);
    assert_eq!(basis_b.snapshot_handle(), &snapshot);
    assert_eq!(observer.observe().acquisitions(), before.acquisitions() + 1);
    assert_eq!(observer.observe().active(), before.active() + 1);
    drop(shared);
    drop(product_a);
    let first = basis_a.release();
    assert!(!first.released());
    assert!(first.custody_released());
    assert_eq!(
        first.outcome(),
        WorthQueryApplicationBasisReleaseOutcome::SharedCustodyRetained
    );
    assert_eq!(first.outcome().relational_retention(), None);
    world
        .application
        .with_host_advancement(|_phase| {
            assert!(world.application.primary_provider.graph.with_runtime(
                request_execution,
                |runtime| runtime.read_truth().project_snapshot(&snapshot).is_some()
            ))
        })
        .expect("the fixture read owns its installed host call");
    drop(product_b);
    let last = basis_b.release();
    assert!(last.released());
    assert_eq!(last.identity(), &identity);
    assert_eq!(observer.observe().active(), before.active());
    world
        .application
        .with_host_advancement(|_phase| {
            assert!(world.application.primary_provider.graph.with_runtime(
                request_execution,
                |runtime| runtime.read_truth().project_snapshot(&snapshot).is_none()
            ))
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn exclusive_basis_refuses_identity_copy_and_releases_its_sole_owner() {
    let world = installed_authorization_world(true);
    let observer = world.application.application_query_basis_observer();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let identity = selected.application_basis().identity().clone();
    // ee07da2b0e removed the work/byte forecast from promotion, so promotion
    // has no quota refusal. Before promotion the exclusive owner still refuses
    // every identity copy without allocating a shared owner.
    let before = observer.observe();
    let stop = match selected.application_basis().retain_shared() {
        Ok(_) => panic!("exclusive custody cannot be copied before promotion"),
        Err(stop) => stop,
    };
    assert_eq!(stop, CompanionPreflightStop::SelectedSourceMismatch);
    assert_eq!(observer.observe().active(), before.active());

    let (_, product, basis) = selected.into_parts();
    drop(product);
    let receipt = basis.release();
    assert_eq!(receipt.identity(), &identity);
    assert!(
        receipt.released(),
        "refusal preserved the sole exclusive owner"
    );
}

#[test]
fn denied_identity_copy_and_foreign_owner_leave_no_hidden_shared_custody() {
    let world = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();

    let shared = world
        .application
        .with_host_advancement(|phase| {
            match world
                .selected_product(&(phase).test_host_request())
                .prepare_shared_query_basis()
            {
                Ok(shared) => shared,
                Err((_, stop)) => panic!("funded sharing must prepare: {stop:?}"),
            }
        })
        .expect("the fixture read owns its installed host call");

    let (product, basis) = world
        .application
        .retain_selected_query_basis_admitted(&shared)
        .unwrap();

    assert_eq!(observer.observe().active(), before.active() + 1);
    // ee07da2b0e removed the work/byte forecasts from identity copies: a
    // further copy is admitted and shares the one native owner.
    let (copy_product, copy) = match world
        .application
        .retain_selected_query_basis_admitted(&shared)
    {
        Ok(copy) => copy,
        Err(error) => panic!("an identity copy carries no quota forecast: {error:?}"),
    };
    assert_eq!(copy.identity(), basis.identity());
    assert_eq!(observer.observe().active(), before.active() + 1);
    drop(copy_product);
    let copied = copy.release();
    assert!(!copied.released());
    assert_eq!(
        copied.outcome(),
        WorthQueryApplicationBasisReleaseOutcome::SharedCustodyRetained
    );

    let error = match foreign
        .application
        .retain_selected_query_basis_admitted(&shared)
    {
        Ok(_) => panic!("another Query runtime cannot share this selected basis"),
        Err(error) => error,
    };
    assert!(matches!(error, SelectedQueryBasisRetentionStop::Basis));
    drop(shared);
    drop(product);
    assert!(
        basis.release().released(),
        "the released copy and refused foreign owner retained no extra share"
    );
    assert_eq!(observer.observe().active(), before.active());
}

#[test]
fn shared_basis_drop_releases_the_final_native_owner() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();

    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let snapshot = selected.application_basis().snapshot_handle().clone();
    let shared = match selected.prepare_shared_query_basis() {
        Ok(shared) => shared,
        Err((_, stop)) => panic!("funded sharing must prepare: {stop:?}"),
    };
    let (product, basis) = world
        .application
        .retain_selected_query_basis_admitted(&shared)
        .unwrap();
    drop(shared);
    drop(product);
    drop(basis);
    assert_eq!(observer.observe().active(), before.active());
    world
        .application
        .with_host_advancement(|_phase| {
            assert!(world.application.primary_provider.graph.with_runtime(
                request_execution,
                |runtime| runtime.read_truth().project_snapshot(&snapshot).is_none()
            ))
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn shared_basis_real_query_completes_without_claiming_physical_release() {
    use super::super::fixture::live_scope;
    use super::super::fixture::status_parameter;
    use super::super::fixture::AccountStatus;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryControls;
    use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessContext;
    use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;
    use std::num::NonZeroUsize;
    use std::time::Duration;
    use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_string(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let query = super::installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);

    let shared = world
        .application
        .with_host_advancement(|phase| {
            match world
                .selected_product(&(phase).test_host_request())
                .prepare_shared_query_basis()
            {
                Ok(shared) => shared,
                Err((_, stop)) => panic!("funded sharing must prepare: {stop:?}"),
            }
        })
        .expect("the fixture read owns its installed host call");
    let (product, basis) = world
        .application
        .retain_selected_query_basis_admitted(&shared)
        .unwrap();
    let identity = basis.identity().clone();
    let plan = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .admit_application_query(
                    &host_custody.borrowed_request_for_test(),
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    WorthQueryApplicationQueryControls::selected_read_one_shot(
                        product,
                        basis,
                        NonZeroUsize::new(32).unwrap(),
                        &request,
                    )
                    .unwrap(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let release = result.receipt().read_completion().basis_release();
    assert_eq!(release.identity(), &identity);
    assert_eq!(
        release.outcome(),
        WorthQueryApplicationBasisReleaseOutcome::SharedCustodyRetained
    );
    assert!(release.custody_released());
    assert!(!release.released());
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert!(shared
                .selected()
                .application_basis()
                .is_live(&host_custody.borrowed_request_for_test(),))
        })
        .expect("the fixture read owns its installed host call");
}
