//! Original borrowed capability admission and recovery cross actual native boundaries.
use super::super::super::entry_correction::EntryCorrection;
use super::super::source_delivery::CorrectEntryAction;
use super::*;
use worth_query_decl::facade::application_schema::{
    ApplicationEncodedScalarValue, TypedMutationPreconditions,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryApplicationRetainedMutationOutcome,
    WorthQueryOperationCleanup,
};
use worth_query_host::facade::primary_graph::WorthQueryManagedApplicationRecoveryOutcome;

// The original owning intent has no Clone implementation.
struct OriginalCorrection(EntryCorrection);
impl worth_query_decl::facade::application_operation::ApplicationMutationIntent<CheckpointSchema>
    for OriginalCorrection
{
    type Binding = super::super::super::entry_correction::EntryCorrectionBinding<CheckpointSchema>;
    fn input(&self) -> &EntryCorrection {
        &self.0
    }
    fn scope_binding(&self) -> <Self::Binding as worth_query_decl::facade::application_operation::ApplicationMutationBinding<CheckpointSchema>>::ScopeBinding{
        <EntryCorrection as worth_query_decl::facade::application_operation::ApplicationMutationIntent<CheckpointSchema>>::scope_binding(&self.0)
    }
}

#[test]
fn common_operation_handoff_borrows_capability_preconditions_and_cleanup() {
    let _guard = checkpoint_recovery_test_guard();
    let case = Model::law_cases()
        .into_iter()
        .find(|case| case.name == "Value change")
        .unwrap();
    let application = installation::install_variant::<false, TOTALS_WORK, 1, 0>(
        None,
        Default::default(),
        |graph| {
            case.before.seed(graph);
            correction_grant::seed(graph, SCOPE);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let conditions = TypedMutationPreconditions::new().expect_fact(
        BodyKey::reference::<CheckpointSchema>(),
        ApplicationEncodedScalarValue::try_new(SCOPE.to_owned()).unwrap(),
    );
    let input = EntryCorrection {
        scope_key: SCOPE.to_owned(),
        entry: 1,
        mask: 2.0_f64.to_bits() ^ 3.0_f64.to_bits(),
    };
    let operation = request
        .mutate(OriginalCorrection(input))
        .without_source()
        .preconditions(conditions)
        .idempotency_owned(0x9176_4001)
        .retain_operation::<OracleProgram, CorrectEntryAction>(&application)
        .unwrap_or_else(|(_, denial)| panic!("exact source completion: {denial:?}"));
    let wrong = request
        .mutate(OriginalCorrection(EntryCorrection {
            scope_key: SCOPE.to_owned(),
            entry: 1,
            mask: 1,
        }))
        .without_source()
        .preconditions(TypedMutationPreconditions::new().expect_fact(
            BodyKey::reference::<CheckpointSchema>(),
            ApplicationEncodedScalarValue::try_new("wrong-original-fact".to_owned()).unwrap(),
        ))
        .idempotency_owned(0x9176_4003)
        .retain_operation::<OracleProgram, CorrectEntryAction>(&application)
        .unwrap_or_else(|(_, denial)| {
            panic!("wrong fact is a native admission question: {denial:?}")
        });
    let refused = application
        .with_application_advancement(&scope, |phase| {
            wrong.execute_in_advancement(&phase, &request)
        })
        .unwrap();
    assert!(
        !matches!(
            refused,
            Ok(WorthQueryApplicationRetainedMutationOutcome::Other(
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ))
        ),
        "dropping borrowed original preconditions would falsely commit"
    );
    application
        .with_host_advancement(|phase| {
            application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(application.product_runtime())
                    .unwrap(),
            )
        })
        .unwrap();
    let outcome = application
        .with_application_advancement(&scope, |phase| {
            operation.execute_in_advancement(&phase, &request)
        })
        .unwrap()
        .unwrap();
    let WorthQueryApplicationRetainedMutationOutcome::Other(
        WorthQueryApplicationMutationOutcome::Commit(uncommitted),
    ) = outcome
    else {
        panic!("the real fault leaves unpublished source custody");
    };
    let worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::ProductUnpublished(unpublished) = uncommitted else { panic!("native partial publication"); };
    let recovery = unpublished.into_recovery();
    drop(request);
    drop(principal);
    drop(scope);
    let (fresh_scope, fresh_principal) = authenticate(&application);
    let fresh = application.request(&fresh_principal, &fresh_scope);
    let mut performed = match application
        .with_application_advancement(&fresh_scope, |phase| {
            operation.recover_in_advancement(&phase, &fresh, &recovery)
        })
        .unwrap()
        .unwrap()
    {
        WorthQueryManagedApplicationRecoveryOutcome::Performed(performed) => performed,
        _ => panic!("capability-bound recovery publishes the original source"),
    };
    let mut cleanup = WorthQueryOperationCleanup::default();
    cleanup.absorb_recovered(&mut performed).unwrap();
    let catalog = application
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(catalog.rows().len(), 1);
    let inspector = application
        .readmit_product_publication_recovery(catalog.rows()[0].handle())
        .unwrap();
    let held = inspector.inspect().unwrap();
    let denial = application
        .with_application_advancement(&fresh_scope, |phase| {
            cleanup.close_in_advancement(&phase, application.runtime())
        })
        .unwrap()
        .err()
        .expect("real caller capability refuses record release");
    assert!(matches!(denial, worth_query_host::facade::application_entry::WorthQueryOperationCloseDenial::Recovery(worth_query_host::facade::product::WorthQueryProductUnpublishedRecoveryReleaseDenial::World(worth_query_host::facade::primary_graph::RuntimeWorldRecoveryDenial::CallerCapabilityLive))));
    drop(held);
    application
        .with_application_advancement(&fresh_scope, |phase| {
            cleanup.close_in_advancement(&phase, application.runtime())
        })
        .unwrap()
        .unwrap();
    assert!(application
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    let read = application
        .with_application_advancement(&fresh_scope, |phase| {
            operation.resolve_keyed_in_advancement(&phase, &fresh)
        })
        .unwrap()
        .unwrap();
    assert!(matches!(read.into_resolution(), worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(_)));
    // Recovery and keyed reads must preserve the one XOR effect. The expected host
    // comes from independent authored model truth, never the tested host's records.
    let expected = installation::install(|graph| case.after.seed(graph));
    let (expected_scope, expected_principal) = authenticate(&expected);
    let (_, expected_runs) = demand(
        &expected.request(&expected_principal, &expected_scope),
        &expected,
    );
    let (_, actual_runs) = demand(&fresh, &application);
    assert_eq!(
        actual_runs.last().unwrap().outcome,
        expected_runs.last().unwrap().outcome
    );
}

#[test]
fn common_operation_handoff_retains_actual_commit_observation() {
    let _guard = checkpoint_recovery_test_guard();
    let case = Model::law_cases()
        .into_iter()
        .find(|case| case.name == "Value change")
        .unwrap();
    let application = installation::install(|graph| case.before.seed(graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let edit = case
        .changes
        .into_iter()
        .find_map(|change| match change {
            differential::alphabet::Change::Entry(edit) => Some(edit.commanded(0x9176_4002)),
            _ => None,
        })
        .unwrap();
    let operation = request
        .mutate(edit)
        .without_source()
        .idempotency_owned(0x9176_4002)
        .retain_operation::<OracleProgram, super::super::source_delivery::RetainedEntryAction>(
            &application,
        )
        .unwrap_or_else(|(_, denial)| panic!("retained commit constructor: {denial:?}"));
    let outcome = application
        .with_application_advancement(&scope, |phase| {
            operation.execute_in_advancement(&phase, &request)
        })
        .unwrap()
        .unwrap();
    let WorthQueryApplicationRetainedMutationOutcome::Committed {
        retained, receipt, ..
    } = outcome
    else {
        panic!("the actual commit retains an observation");
    };
    let current = request.retain_current_read().unwrap();
    assert_eq!(retained.selected_commit(), current.selected_commit());
    drop(receipt);
}

#[test]
fn common_operation_handoff_completes_source_without_retained_observation() {
    let _guard = checkpoint_recovery_test_guard();
    let case = Model::law_cases()
        .into_iter()
        .find(|case| case.name == "Value change")
        .unwrap();
    let application = installation::install_variant::<false, TOTALS_WORK, 1, 0>(
        None,
        Default::default(),
        |graph| {
            case.before.seed(graph);
            correction_grant::seed(graph, SCOPE);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let operation = request
        .mutate(OriginalCorrection(EntryCorrection {
            scope_key: SCOPE.to_owned(),
            entry: 1,
            mask: 2.0_f64.to_bits() ^ 3.0_f64.to_bits(),
        }))
        .without_source()
        .preconditions(TypedMutationPreconditions::new().expect_fact(
            BodyKey::reference::<CheckpointSchema>(),
            ApplicationEncodedScalarValue::try_new(SCOPE.to_owned()).unwrap(),
        ))
        .idempotency_owned(0x9176_4004)
        .retain_operation::<OracleProgram, CorrectEntryAction>(&application)
        .unwrap_or_else(|(_, denial)| panic!("source completion constructor: {denial:?}"));
    use worth_query_host::facade::primary_graph::{
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    };
    struct Restore(Placement);
    impl Drop for Restore {
        fn drop(&mut self) {
            place(self.0);
        }
    }
    let _placement = Restore(place(Placement::Leased(std::num::NonZeroUsize::MIN)));
    let refused = application
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .checked_execution_request_for(application.product_runtime())
                .unwrap();
            let worth_query_host::facade::runtime::ExecutionAllocationPolicy::Execution(lease) =
                execution.allocation_policy()
            else {
                panic!("the admitted phase must preserve leased allocation authority");
            };
            let _held = lease
                .reserve_memory(lease.available_memory_bytes())
                .unwrap();
            operation.execute_in_advancement(&phase, &request)
        })
        .unwrap();
    assert!(!matches!(
        refused,
        Ok(WorthQueryApplicationRetainedMutationOutcome::Other(
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ))
    ));
    let read = application
        .with_application_advancement(&scope, |phase| {
            operation.resolve_keyed_in_advancement(&phase, &request)
        })
        .unwrap()
        .unwrap();
    assert!(!matches!(read.into_resolution(), worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(_)));
    let outcome = application
        .with_application_advancement(&scope, |phase| {
            operation.execute_in_advancement(&phase, &request)
        })
        .unwrap()
        .unwrap();
    assert!(matches!(
        outcome,
        WorthQueryApplicationRetainedMutationOutcome::Other(
            WorthQueryApplicationMutationOutcome::Committed { .. }
        )
    ));
    assert!(outcome.retained_read().is_none());
    let replay = application
        .with_application_advancement(&scope, |phase| {
            operation.resolve_keyed_in_advancement(&phase, &request)
        })
        .unwrap()
        .unwrap();
    assert!(matches!(replay.into_resolution(), worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(_)));
    let expected = installation::install(|graph| case.after.seed(graph));
    let (expected_scope, expected_principal) = authenticate(&expected);
    let (_, expected_runs) = demand(
        &expected.request(&expected_principal, &expected_scope),
        &expected,
    );
    let (_, actual_runs) = demand(&request, &application);
    assert_eq!(
        actual_runs.last().unwrap().outcome,
        expected_runs.last().unwrap().outcome
    );
}
