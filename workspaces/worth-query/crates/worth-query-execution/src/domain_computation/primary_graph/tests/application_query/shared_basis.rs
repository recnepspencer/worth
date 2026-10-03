//! Exact-custody owner proofs; these do not claim public wave completion.
use super::super::fixture::installed_authorization_world;
use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::InvalidationEditAdmission,
    product_operation::SelectedQueryBasisRetentionStop, WorthQueryApplicationBasisReleaseOutcome,
};
use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};

fn admission(work: u64, bytes: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: bytes,
    })
}

#[test]
fn shared_basis_keeps_one_exact_native_owner_until_last_explicit_release() {
    let world = installed_authorization_world(true);
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();
    let mut meter = admission(100_000, 100_000);
    let selected = world.selected_product();
    let snapshot = selected.application_basis().snapshot_handle().clone();
    let identity = selected.application_basis().identity().clone();
    let shared = match selected.prepare_shared_query_basis(&mut meter) {
        Ok(shared) => shared,
        Err((_, stop)) => panic!("funded sharing must prepare: {stop:?}"),
    };
    let (product_a, basis_a) = world
        .application
        .retain_selected_query_basis_admitted(&shared, &mut meter)
        .unwrap();
    let (product_b, basis_b) = world
        .application
        .retain_selected_query_basis_admitted(&shared, &mut meter)
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
    assert!(world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| runtime.read_truth().project_snapshot(&snapshot).is_some()));
    drop(product_b);
    let last = basis_b.release();
    assert!(last.released());
    assert_eq!(last.identity(), &identity);
    assert_eq!(observer.observe().active(), before.active());
    assert!(world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| runtime.read_truth().project_snapshot(&snapshot).is_none()));
}

#[test]
fn shared_basis_promotion_refuses_before_allocation_and_returns_exclusive_custody() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let identity = selected.application_basis().identity().clone();
    let mut denied = admission(100, 0);
    let (selected, stop) = match selected.prepare_shared_query_basis(&mut denied) {
        Ok(_) => panic!("zero preparation bytes cannot fund the new final owner"),
        Err(denial) => denial,
    };
    assert!(matches!(
        stop,
        CompanionPreflightStop::PreparationMemoryExhausted { maximum: 0, .. }
    ));
    assert_eq!(denied.charged_bytes(), 0);
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
    let mut funded = admission(100_000, 100_000);
    let shared = match world
        .selected_product()
        .prepare_shared_query_basis(&mut funded)
    {
        Ok(shared) => shared,
        Err((_, stop)) => panic!("funded sharing must prepare: {stop:?}"),
    };
    let mut measured = admission(100_000, 100_000);
    let (product, basis) = world
        .application
        .retain_selected_query_basis_admitted(&shared, &mut measured)
        .unwrap();
    let required_bytes = measured.charged_bytes();
    let required_work = measured.charged_work();
    let mut work_short = admission(required_work - 1, 100_000);
    let error = match world
        .application
        .retain_selected_query_basis_admitted(&shared, &mut work_short)
    {
        Ok(_) => panic!("one Work visit short must refuse before the identity copy"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        SelectedQueryBasisRetentionStop::Admission(CompanionPreflightStop::WorkExhausted { .. })
    ));
    assert_eq!(work_short.charged_bytes(), 0);
    let mut short = admission(100_000, required_bytes - 1);
    let error = match world
        .application
        .retain_selected_query_basis_admitted(&shared, &mut short)
    {
        Ok(_) => panic!("one byte short must refuse the identity copy"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        SelectedQueryBasisRetentionStop::Admission(
            CompanionPreflightStop::PreparationMemoryExhausted { .. }
        )
    ));
    assert_eq!(short.charged_bytes(), 0);
    let error = match foreign
        .application
        .retain_selected_query_basis_admitted(&shared, &mut funded)
    {
        Ok(_) => panic!("another Query runtime cannot share this selected basis"),
        Err(error) => error,
    };
    assert!(matches!(error, SelectedQueryBasisRetentionStop::Basis(_)));
    drop(shared);
    drop(product);
    assert!(
        basis.release().released(),
        "failed clones retained no extra share"
    );
    assert_eq!(observer.observe().active(), before.active());
}

#[test]
fn shared_basis_drop_releases_the_final_native_owner() {
    let world = installed_authorization_world(true);
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();
    let mut meter = admission(100_000, 100_000);
    let selected = world.selected_product();
    let snapshot = selected.application_basis().snapshot_handle().clone();
    let shared = match selected.prepare_shared_query_basis(&mut meter) {
        Ok(shared) => shared,
        Err((_, stop)) => panic!("funded sharing must prepare: {stop:?}"),
    };
    let (product, basis) = world
        .application
        .retain_selected_query_basis_admitted(&shared, &mut meter)
        .unwrap();
    drop(shared);
    drop(product);
    drop(basis);
    assert_eq!(observer.observe().active(), before.active());
    assert!(world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| runtime.read_truth().project_snapshot(&snapshot).is_none()));
}

#[test]
fn shared_basis_real_query_completes_without_claiming_physical_release() {
    use super::super::fixture::{live_scope, status_parameter, AccountStatus};
    use crate::domain_computation::primary_graph::{
        application_query::WorthQueryApplicationQueryControls,
        WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
    };
    use std::{num::NonZeroUsize, time::Duration};
    use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .selected_product()
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = super::installed_query(&world);
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let mut meter = admission(100_000, 100_000);
    let shared = match world
        .selected_product()
        .prepare_shared_query_basis(&mut meter)
    {
        Ok(shared) => shared,
        Err((_, stop)) => panic!("funded sharing must prepare: {stop:?}"),
    };
    let (product, basis) = world
        .application
        .retain_selected_query_basis_admitted(&shared, &mut meter)
        .unwrap();
    let identity = basis.identity().clone();
    let plan = world
        .application
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_string())
                .unwrap(),
            WorthQueryApplicationQueryControls::selected_read_one_shot(
                product,
                basis,
                NonZeroUsize::new(32).unwrap(),
                NonZeroUsize::new(4096).unwrap(),
                &request,
                &mut meter,
            )
            .unwrap(),
        )
        .unwrap();
    let result = world
        .application
        .execute_application_query_one_shot(plan)
        .unwrap();
    let release = result.receipt().read_completion().basis_release();
    assert_eq!(release.identity(), &identity);
    assert_eq!(
        release.outcome(),
        WorthQueryApplicationBasisReleaseOutcome::SharedCustodyRetained
    );
    assert!(release.custody_released());
    assert!(!release.released());
    assert!(shared.selected().application_basis().is_live());
}
