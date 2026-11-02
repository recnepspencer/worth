//! Existing Query admission over genuine physical Home and Native artifacts.
use super::bootstrap_publication::{build, meaning, prepared, DirectoryHome};
use fixture::{schema::HomeIdentitySchema, HomeProgram};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::application_home_integration::*;
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryBootstrapAdvancementPhase,
    },
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    WorthQueryApplicationHomeOpenRefused,
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};
use worth_store_application::*;
mod administration;
mod idempotency;
mod operations;
mod selected;
mod stops;
use operations::{balance, fresh_closed, set};
#[derive(Debug)]
struct ObservationMaintenanceMustRemainUnentered;
impl WorthQueryHomeMaintenance for ObservationMaintenanceMustRemainUnentered {
    fn after_advancement(&self, _: &ExecutionRequest<'_, '_>) {
        panic!("the observational attachment must never advance or schedule maintenance");
    }
}

type Existing = PreparedExistingHomeQueryRuntime<HomeIdentitySchema, HomeProgram>;
type NativePreparation = PreparedHomeNativeMeaning<HomeIdentitySchema, HomeProgram>;
type NativeMeaning = ReadmittedExistingHomeNativeMeaning<HomeIdentitySchema, HomeProgram>;
type NativeRefused = QueryExistingHomeNativeReadmissionRefused<HomeIdentitySchema, HomeProgram>;

fn close_world(home: DirectoryHome, execution: &ExecutionRequest<'_, '_>) {
    let outcome = home
        .close(execution)
        .unwrap_or_else(|refused| panic!("actual World close refused: {:?}", refused.reason()));
    match outcome {
        ApplicationHomeCloseOutcome::Closed { .. } => {}
        ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
            panic!("actual World close retained custody: {reason:?}")
        }
    }
}

/// This observational attachment has no permission to admit an operation.
fn no_operation_admission(_: worth_foundational::facade::DurableStateFamily, _: u8) -> Option<u8> {
    panic!("the Bridge/source observation proof must never invoke Directory operation admission")
}

fn with_native<R>(
    path: &std::path::Path,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    operation: impl FnOnce(&worth_relational::facade::runtime::RelationalRuntime) -> R,
) -> R {
    let (query, mut home) = unshared(path, phase);
    let (native, _, _) = home.recovered_query_finalization_parts().unwrap();
    let result = operation(native);
    drop(query);
    assert!(matches!(
        home.close(&phase.execution_request()).unwrap(),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
    result
}
fn finalize_native(
    query: NativePreparation,
    home: &mut UnsharedReadmittedApplicationHome,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
) -> Result<NativeMeaning, NativeRefused> {
    let (native, authority, outcome) = home.recovered_query_finalization_parts().unwrap();
    query.finish_existing_home_native_readmission(phase, native, authority, outcome)
}

fn unshared(
    path: &std::path::Path,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
) -> (
    PreparedHomeNativeMeaning<HomeIdentitySchema, HomeProgram>,
    UnsharedReadmittedApplicationHome,
) {
    let query = meaning();
    let execution = phase.execution_request();
    let request = ApplicationHomeRequest::new(
        path.to_path_buf(),
        query.home_relational_definition(),
        query.admitted_home_binding().unwrap(),
        build(),
    );
    let home = admit_home(recover_home(request, &execution).unwrap(), &execution).unwrap();
    let mut home =
        attach_relational(home, &execution).unwrap_or_else(|_| panic!("actual Native attaches"));
    let query = query
        .install_home_native_schema(phase, home.native_for_schema_installation())
        .unwrap_or_else(|refused| panic!("actual existing schema: {:?}", refused.reason()));
    let home = home
        .readmit(query.home_initial_schema_receipt(), &execution)
        .unwrap_or_else(|_| panic!("actual Native recovery"))
        .consume(|home| home);
    (query, home)
}

fn existing(
    path: &std::path::Path,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
) -> (
    Existing,
    DirectoryHome,
    worth_runtime_bridge::facade::RuntimeBridgeRelationalSource,
) {
    let (query, mut home) = unshared(path, phase);
    let execution = phase.execution_request();
    let query = finalize_native(query, &mut home, phase).unwrap_or_else(|refused| {
        panic!(
            "actual existing Native finalization: {:?}",
            refused.reason()
        )
    });
    let bridge = query.home_bridge_definition().unwrap();
    let (budgets, clock, policy, execution_authority, conditionals) = query.home_world_resources();
    let home = readmit_query_world(
        home.share(),
        ApplicationHomeWorldDefinition::new(
            bridge,
            "primary-application-source",
            budgets,
            clock,
            policy,
            execution_authority,
            conditionals,
        ),
        &execution,
    )
    .unwrap_or_else(|_| panic!("actual World admission"))
    .consume(|home| home);
    let (source, bridge, conditional) = home.query_composition_parts().unwrap();
    let witness = source.clone();
    let query = query
        .attach_existing_home_registered_source(phase, source, bridge, conditional)
        .unwrap_or_else(|refused| panic!("actual existing source: {:?}", refused.reason()));
    (query, home, witness)
}

fn reopen_refusal(
    path: &std::path::Path,
    execution: ExecutionRequest<'_, '_>,
) -> WorthQueryApplicationHomeOpenRefused<HomeIdentitySchema, HomeProgram> {
    let request = fixture::request_with_initializer(
        path.to_path_buf(),
        build(),
        Arc::new(AtomicUsize::new(0)),
        |_, _, _| panic!("existing artifacts must never initialize"),
    );
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .err()
        .expect("actual existing artifacts require owned refusal")
}

#[test]
fn latest_native_owner_rows_refuse_their_actual_family_kind_and_record_before_service() {
    for (name, family) in [
        ("worth-query-workflow-lineage", "workflow"),
        ("worth-query-dispatch-outbox", "outbox"),
        ("worth-query-inbound-completion", "inbound"),
        ("worth-query-aftermath-causality", "aftermath"),
    ] {
        let path = tempfile::tempdir().unwrap();
        let lease = checkpoint_capture::lease(128 * 1024 * 1024);
        let execution = ExecutionRequest::leased(&lease);
        fresh_closed(path.path(), execution);
        let (kind, record) = with_bootstrap_advancement_from_request(execution, |phase| {
            let (query, mut home) = unshared(path.path(), &phase);
            let (native, _, _) = home.recovered_query_finalization_parts().unwrap();
            let result = administration::create_required_row(native, &execution, name);
            drop(query);
            assert!(matches!(
                home.close(&execution).unwrap(),
                ApplicationHomeCloseOutcome::Closed { .. }
            ));
            result
        })
        .unwrap();
        let refused = reopen_refusal(path.path(), execution);
        match &refused {
            WorthQueryApplicationHomeOpenRefused::ExistingNativeFinalization { query, .. } => {
                assert_eq!(
                    query.reason().subject(),
                    format!(
                        "Home unsupported Native {family} entity kind {} record {record:?}",
                        kind.0
                    )
                )
            }
            other => panic!("latest Native owner gate must refuse first: {other:?}"),
        }
        refused
            .close(&execution)
            .unwrap_or_else(|refused| panic!("owned family close: {refused:?}"));
    }
}

#[test]
fn unassociated_native_nine_cannot_replace_selected_world_eight_or_preflight_bridge_head() {
    use worth_foundational::facade::AspectValue;
    use worth_runtime_bridge::facade::{TruthBranchHeadSource, TruthBranchIdentity};
    let path = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    let contacts = Arc::new(AtomicUsize::new(0));
    let request = fixture::request(
        path.path().to_path_buf(),
        build(),
        Arc::new(AtomicUsize::new(0)),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("actual writer: {refused:?}"));
    let writer_receipt = match set(home.runtime(), 8) {
        WorthQueryApplicationMutationOutcome::Committed { receipt, result: 8 } => {
            receipt.commit_reference().clone()
        }
        other => panic!("actual World balance eight: {other:?}"),
    };
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("writer close: {refused:?}"));
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, mut home) = unshared(path.path(), &phase);
        let (native, _, _) = home.recovered_query_finalization_parts().unwrap();
        let accounts = administration::rows(native, &execution, "Account");
        let [account] = accounts.as_slice() else {
            panic!("actual account row")
        };
        administration::update(
            native,
            &execution,
            account.entity_id,
            "AccountIdentity",
            "Balance",
            AspectValue::UInt64(9),
        );
        let rows = administration::rows(native, &execution, "Account");
        assert_eq!(
            administration::field(&rows[0], "AccountIdentity", "Balance"),
            &AspectValue::UInt64(9)
        );
        drop(query);
        assert!(matches!(
            home.close(&execution).unwrap(),
            ApplicationHomeCloseOutcome::Closed { .. }
        ));
    })
    .unwrap();
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query, mut home, source) = existing(path.path(), &phase);
        let root = home.select_query_root(&execution).unwrap();
        let native = home.relational().runtime();
        let selected = native
            .history()
            .branch_head_for_observation(&root.basis().relational_basis().observation())
            .unwrap()
            .unwrap();
        assert_eq!(selected, writer_receipt);
        let (_, latest_basis) = native
            .observe_branch(&native.main_branch_identity())
            .unwrap();
        let latest = native
            .history()
            .branch_head_for_observation(&latest_basis.observation())
            .unwrap()
            .unwrap();
        assert_ne!(selected.commit_id, latest.commit_id);
        assert_ne!(selected.version_id, latest.version_id);
        let ports = home.world();
        let query = query
            .attach_existing_home_world(
                &phase,
                ports.publication_port(),
                ports.branch_port(),
                ports.observation_port(),
                ports.inspection_port(),
                ports.recovery_port(),
                root.clone(),
                no_operation_admission,
                Arc::new(ObservationMaintenanceMustRemainUnentered),
            )
            .unwrap_or_else(|refused| {
                panic!("same selected World attachment: {}", refused.detail())
            })
            .consume(|query| query.into_home_program_runtime());
        let envelope = source
            .load_branch_head_patch(
                &TruthBranchIdentity::from_relational_branch_id(selected.branch_id.0.clone()),
                execution,
            )
            .unwrap();
        assert_eq!(
            envelope
                .snapshot_identity()
                .relational_snapshot_parts()
                .unwrap()
                .version_id(),
            selected.version_id.as_u64()
        );
        assert_ne!(
            envelope
                .snapshot_identity()
                .relational_snapshot_parts()
                .unwrap()
                .version_id(),
            latest.version_id.as_u64()
        );
        assert_eq!(contacts.load(Ordering::SeqCst), 1);
        drop(envelope);
        drop(query);
        drop(source);
        drop(root);
        drop(latest_basis);
        close_world(home, &execution);
    })
    .unwrap();
    // Public Host service uses its actual canonical Directory policy separately.
    let request = fixture::request_with_initializer(
        path.path().to_path_buf(),
        build(),
        contacts,
        |_, _, _| panic!("same Home cannot initialize"),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("public selected World service: {refused:?}"));
    assert_eq!(balance(home.runtime()), 8);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("public selected World close: {refused:?}"));
}

#[test]
fn another_actual_home_world_basis_cannot_attach_to_this_existing_source() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    fresh_closed(first.path(), execution);
    fresh_closed(second.path(), execution);
    with_bootstrap_advancement_from_request(execution, |phase| {
        let (query_a, mut home_a, source_a) = existing(first.path(), &phase);
        let (query_b, home_b, source_b) = existing(second.path(), &phase);
        let root_a = home_a.select_query_root(&execution).unwrap();
        let ports = home_b.world();
        let refused = query_b
            .attach_existing_home_world(
                &phase,
                ports.publication_port(),
                ports.branch_port(),
                ports.observation_port(),
                ports.inspection_port(),
                ports.recovery_port(),
                root_a.clone(),
                no_operation_admission,
                Arc::new(ObservationMaintenanceMustRemainUnentered),
            )
            .err()
            .expect("a genuine foreign Native observation is not this source's basis");
        assert!(
            refused
                .detail()
                .contains("Home completion selected basis denied:"),
            "{}",
            refused.detail()
        );
        drop(refused);
        drop(query_a);
        drop(source_a);
        drop(source_b);
        drop(root_a);
        close_world(home_b, &execution);
        close_world(home_a, &execution);
    })
    .unwrap();
}
