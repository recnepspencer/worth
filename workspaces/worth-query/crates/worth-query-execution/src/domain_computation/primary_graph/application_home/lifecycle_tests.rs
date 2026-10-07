use super::lifecycle_fixture::{
    assert_open, graph, open, open_with_projection, HomeRecord, HomeValue,
};
use super::{ApplicationHome, WorthQueryApplicationCloseDenial};
use crate::domain_computation::{
    execution_runtime::product_world::WorthQueryHandleDenial,
    primary_graph::application_installation::WorthQueryHomeOpening,
};

#[test]
fn close_then_open_resumes_the_committed_record_and_revokes_retained_handles() {
    let runtime = open(ApplicationHome::memory());
    let retained = graph(&runtime);
    let product = runtime.runtime().product_runtime.clone();
    let branch = runtime.runtime().current_world();
    let home = runtime.close().expect("an idle owner closes");
    assert_eq!(
        retained.with_runtime(|_| ()),
        Err(WorthQueryHandleDenial::Closed)
    );
    assert_eq!(
        retained.with_runtime_mut(|_| ()),
        Err(WorthQueryHandleDenial::Closed)
    );
    assert_eq!(
        retained.with_runtime_mut_unwind_isolated(|_| ()),
        Err(WorthQueryHandleDenial::Closed)
    );
    assert!(matches!(
        product.integration_admit_product_branch(branch),
        Err(
            crate::basis::WorthQueryProductBranchAdmissionDenial::Handle(
                WorthQueryHandleDenial::Closed
            )
        )
    ));
    let resumed = open(home);
    assert!(matches!(
        resumed.opening(),
        WorthQueryHomeOpening::Resumed { .. }
    ));
    assert_open(&resumed);
    let invariant = resumed.runtime().retain_invariant_projection_authority();
    let snapshot = invariant.snapshot().expect("the resumed record projects");
    let records = snapshot
        .entities(HomeRecord::reference())
        .expect("the resumed owner remains open");
    assert_eq!(records.len(), 1);
    assert_eq!(
        snapshot.field(&records[0], HomeValue::reference()),
        Ok(Some(42))
    );
}

#[test]
fn an_in_flight_admission_refuses_without_waiting_or_revoking_any_handle() {
    let runtime = open(ApplicationHome::memory());
    let retained = graph(&runtime);
    let worker = retained.clone();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let admission = std::thread::spawn(move || {
        worker.with_runtime(|_| {
            entered_tx.send(()).expect("the close thread is alive");
            release_rx
                .recv()
                .expect("the close thread releases the admission");
        })
    });
    entered_rx
        .recv()
        .expect("the admission holds the source lock");
    let refusal = runtime
        .close()
        .expect_err("close never waits for the held source lock");
    assert!(matches!(
        refusal.denial,
        WorthQueryApplicationCloseDenial::AdmissionsActive
    ));
    release_tx.send(()).expect("the admission thread is alive");
    admission
        .join()
        .expect("the admission thread returns")
        .expect("a refused close revokes nothing");
    assert_open(&refusal.runtime);
    refusal
        .runtime
        .close()
        .expect("the owner closes after the admission returns");
}

#[test]
fn a_retained_invariant_snapshot_refuses_closed_and_drops_silently() {
    let (runtime, invariant) = open_with_projection(ApplicationHome::memory());
    let snapshot = invariant.snapshot().expect("the open fixture projects");
    let record = snapshot
        .entities(HomeRecord::reference())
        .expect("the fixture is open")
        .remove(0);
    assert_eq!(
        snapshot.field(&record, HomeValue::reference()),
        Ok(Some(42))
    );
    let home = runtime
        .close()
        .expect("a retained read does not keep the owner admitted");
    assert_eq!(
        snapshot.field(&record, HomeValue::reference()),
        Err(WorthQueryHandleDenial::Closed)
    );
    drop(snapshot);
    drop(invariant);
    assert_open(&open(home));
}

#[test]
fn a_capture_error_refuses_before_seal_and_preserves_the_runtime() {
    let runtime = open(ApplicationHome::memory());
    graph(&runtime).source_owner.fail_closing_capture_for_test();
    let refusal = runtime
        .close()
        .expect_err("capture failure must precede the seal");
    assert!(matches!(
        refusal.denial,
        WorthQueryApplicationCloseDenial::Capture(_)
    ));
    assert_open(&refusal.runtime);
    refusal
        .runtime
        .close()
        .expect("capture can be retried on the same owner");
}

#[test]
fn an_outstanding_prepared_commit_refuses_close_and_remains_publishable() {
    use worth_relational::facade::{
        mvcc::RelationalPublicationOutcome, runtime::RelationalRuntimeAdmissionHoldDenial,
    };
    let runtime = open(ApplicationHome::memory());
    let candidate = super::lifecycle_fixture::candidate(&runtime);
    let refusal = runtime
        .close()
        .expect_err("prepared custody must prevent sealing");
    assert_eq!(
        refusal.denial,
        WorthQueryApplicationCloseDenial::Owner(
            RelationalRuntimeAdmissionHoldDenial::PreparedCandidatesOutstanding
        )
    );
    assert_open(&refusal.runtime);
    graph(&refusal.runtime)
        .with_runtime(|native| {
            let RelationalPublicationOutcome::Performed(performed) =
                native.publication_port().compare_and_publish(candidate)
            else {
                panic!("a refused close preserves the prepared candidate")
            };
            let committed = native
                .settlement_port()
                .settle_performed_publication(performed)
                .expect("the preserved candidate settles");
            native
                .snapshots()
                .release_snapshot(&committed.snapshot)
                .expect("the committed read releases");
        })
        .expect("the refused close preserves publication admission");
    refusal
        .runtime
        .close()
        .expect("the owner closes after candidate settlement");
}

#[test]
fn deferred_settlement_refuses_close_without_draining_custody() {
    use worth_relational::facade::{
        mvcc::RelationalPublicationOutcome, runtime::RelationalRuntimeAdmissionHoldDenial,
    };
    let runtime = open(ApplicationHome::memory());
    let candidate = super::lifecycle_fixture::candidate(&runtime);
    runtime
        .runtime()
        .fail_next_durable_append_for_test()
        .expect("the fixture owner is open");
    let error = graph(&runtime)
        .with_runtime(|native| {
            let RelationalPublicationOutcome::Performed(performed) =
                native.publication_port().compare_and_publish(candidate)
            else {
                panic!("the candidate performs before append failure")
            };
            native
                .settlement_port()
                .settle_performed_publication(performed)
                .expect_err("the append fault leaves real deferred settlement")
        })
        .expect("the open owner publishes");
    let settlement = error
        .deferred_settlement()
        .expect("the error retains custody");
    let commit_id = settlement.commit().commit_id;
    let refusal = runtime
        .close()
        .expect_err("unsettled publication must prevent sealing");
    assert_eq!(
        refusal.denial,
        WorthQueryApplicationCloseDenial::Owner(
            RelationalRuntimeAdmissionHoldDenial::PerformedPublicationRequiresSettlement(commit_id)
        )
    );
    assert_open(&refusal.runtime);
    graph(&refusal.runtime)
        .with_runtime_mut(|native| {
            assert!(native
                .settlement_port()
                .retains_pending_settlement(commit_id));
            assert_eq!(
                native
                    .repair_deferred_publication_settlement(settlement)
                    .expect("the refused close preserves settlement custody")
                    .commit_id,
                commit_id
            );
        })
        .expect("the same owner admits settlement repair");
    refusal
        .runtime
        .close()
        .expect("the owner closes after deferred settlement repairs");
}

#[test]
fn reopen_recovers_the_highest_retired_branch_ordinal() {
    let runtime = open(ApplicationHome::memory());
    let root = runtime.runtime().current_world();
    let branch = runtime
        .runtime()
        .branches()
        .fork(root)
        .components(|parts| parts.fork_relational().fork_signal())
        .create()
        .expect("the first public fork succeeds");
    runtime
        .runtime()
        .product_runtime
        .product_branches()
        .close(branch)
        .expect("the highest fork retires completely");
    let names = graph(&runtime)
        .with_runtime(|native| native.branch_names())
        .expect("the owner remains open");
    assert!(names
        .retired()
        .iter()
        .any(|name| name.0 == crate::basis::relational_product_branch_name(1)));
    let home = runtime.close().expect("the fork's owner is quiescent");
    let reopened = open(home);
    let root = reopened.runtime().current_world();
    reopened
        .runtime()
        .branches()
        .fork(root)
        .components(|parts| parts.fork_relational().fork_signal())
        .create()
        .expect("the next public fork uses a fresh ordinal");
    let names = graph(&reopened)
        .with_runtime(|native| native.branch_names())
        .expect("the reopened owner admits reads");
    assert!(names
        .registered()
        .iter()
        .any(|name| name.0 == crate::basis::relational_product_branch_name(2)));
}

#[test]
fn the_maximum_registered_ordinal_exhausts_identity_without_wrapping() {
    let runtime = open(ApplicationHome::memory());
    graph(&runtime)
        .with_runtime(|native| {
            let (_, source) = native
                .observe_fork_source(native.main_branch_identity().branch_id())
                .expect("the fixture main branch has committed state");
            native
                .fork_branch(
                    worth_relational::facade::history::BranchId(
                        crate::basis::relational_product_branch_name(u64::MAX),
                    ),
                    source,
                )
                .expect("the owner registers the exhaustion boundary");
        })
        .expect("the owner is open");
    let reopened = open(runtime.close().expect("the exhaustion fixture closes"));
    for _ in 0..2 {
        assert!(matches!(
            reopened
                .runtime()
                .branches()
                .fork(reopened.runtime().current_world())
                .components(|parts| parts.fork_relational().fork_signal())
                .create(),
            Err(crate::basis::WorthQueryProductBranchCreateError::IdentityExhausted)
        ));
    }
}

#[test]
fn close_refuses_an_in_flight_query_publication_without_waiting() {
    let runtime = open(ApplicationHome::memory());
    let integration = graph(&runtime);
    let publication = integration
        .output_lineage
        .lock()
        .expect("the fixture lineage owner is available");
    let refusal = runtime
        .close()
        .expect_err("close cannot wait for Query publication");
    assert!(matches!(
        refusal.denial,
        WorthQueryApplicationCloseDenial::Capture(_)
    ));
    drop(publication);
    assert_open(&refusal.runtime);
    refusal
        .runtime
        .close()
        .expect("publication release makes the same runtime closeable");
}

#[test]
fn a_prepared_product_source_token_is_revoked_before_installation() {
    use crate::domain_computation::execution_runtime::product_world::{
        test_product_world_resources, WorthQueryProductRuntime,
        WorthQueryProductRuntimeInstallationDenialKind,
    };
    let runtime = open(ApplicationHome::memory());
    let retained = graph(&runtime);
    let identity = retained.with_open_runtime(|native| native.main_branch_identity());
    let token = retained
        .prepare_product_source(&identity)
        .expect("the open owner issues a product source token");
    let bridge = runtime.runtime().bridge.conditional_operations();
    let _home = runtime
        .close()
        .expect("the idle owner closes with an uninstalled token");
    let denial = match WorthQueryProductRuntime::install(
        token,
        &mut bridge
            .write()
            .expect("the fixture retains its Bridge assembly"),
        test_product_world_resources(),
        None,
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("a token issued by a sealed owner installs nothing"),
    };
    assert_eq!(
        denial.kind(),
        WorthQueryProductRuntimeInstallationDenialKind::Handle(WorthQueryHandleDenial::Closed)
    );
}

#[test]
fn retained_application_bases_report_closed_on_read_and_release_and_drop_silently() {
    use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisReleaseOutcome;
    let runtime = open(ApplicationHome::memory());
    let product = runtime
        .runtime()
        .product_runtime
        .integration_admit_product_branch(runtime.runtime().current_world())
        .expect("the open fixture admits its product branch");
    let explicit = runtime
        .runtime()
        .retain_product_application_basis(product.observation())
        .expect("the open fixture retains its basis");
    let silent = runtime
        .runtime()
        .retain_product_application_basis(product.observation())
        .expect("the open fixture retains another basis");
    let home = runtime
        .close()
        .expect("read custody does not admit an operation");
    assert_eq!(explicit.is_live(), Err(WorthQueryHandleDenial::Closed));
    let receipt = explicit.release();
    assert_eq!(
        receipt.outcome(),
        WorthQueryApplicationBasisReleaseOutcome::Handle(WorthQueryHandleDenial::Closed)
    );
    assert!(!receipt.released());
    drop(silent);
    drop(product);
    assert_open(&open(home));
}
