//! Original borrowed capability admission and recovery cross actual native boundaries.
use super::super::super::entry_correction::EntryCorrection;
use super::super::source_delivery::CorrectEntryAction;
use super::*;
use worth_query_decl::facade::application_schema::{
    ApplicationEncodedScalarValue, TypedMutationPreconditions,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQuerySourceCompletion, WorthQuerySourceOnlyAdvance,
};

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
    let refused = wrong.advance(&request);
    assert!(
        !matches!(refused, WorthQuerySourceOnlyAdvance::Completed(_)),
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
    let WorthQuerySourceOnlyAdvance::Progress(operation) = operation.advance(&request) else {
        panic!("the real fault leaves unpublished source custody");
    };
    assert_eq!(operation.posture(), worth_query_host::facade::application_entry::WorthQuerySourceOnlyPosture::ProductUnpublished);
    drop(request);
    drop(principal);
    drop(scope);
    let (fresh_scope, fresh_principal) = authenticate(&application);
    let fresh = application.request(&fresh_principal, &fresh_scope);
    let operation = match operation.advance(&fresh) {
        WorthQuerySourceOnlyAdvance::Progress(operation) => operation,
        _ => panic!("capability-bound recovery publishes the original source"),
    };
    let completed = match operation.advance(&fresh) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        _ => panic!("the original keyed read proves source completion"),
    };
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered {
            publication: Some(_),
            ..
        }
    ));
    let catalog = application
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(catalog.rows().len(), 1);
    let inspector = application
        .readmit_product_publication_recovery(catalog.rows()[0].handle())
        .unwrap();
    let held = inspector.inspect().unwrap();
    let (completed, denial) = completed
        .close(&fresh)
        .err()
        .expect("real caller capability refuses record release");
    assert!(matches!(denial, worth_query_host::facade::application_entry::WorthQueryOperationCloseDenial::Recovery(worth_query_host::facade::product::WorthQueryProductUnpublishedRecoveryReleaseDenial::World(worth_query_host::facade::primary_graph::RuntimeWorldRecoveryDenial::CallerCapabilityLive))));
    drop(held);
    completed
        .close(&fresh)
        .unwrap_or_else(|(_, denial)| panic!("native close retry: {denial:?}"));
    assert!(application
        .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
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
    let WorthQuerySourceOnlyAdvance::Completed(completed) =
        operation.advance(&request, std::num::NonZeroUsize::new(8).unwrap())
    else {
        panic!("the actual commit retains an observation");
    };
    let WorthQuerySourceCompletion::Fresh {
        retained: Some(retained),
        ..
    } = completed.evidence()
    else {
        panic!("the actual commit retains an observation");
    };
    let current = request.retain_current_read().unwrap();
    assert_eq!(retained.selected_commit(), current.selected_commit());
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("fresh retained close: {denial:?}"));
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
    use worth_query_host::facade::primary_graph::bound_advancement_requests_on_this_thread_for_test as bound;
    struct RestoreBudget(Option<worth_foundational::ExecutionBudget>);
    impl Drop for RestoreBudget {
        fn drop(&mut self) {
            bound(self.0);
        }
    }
    let budget = RestoreBudget(bound(Some(worth_foundational::ExecutionBudget::new(
        std::num::NonZeroUsize::MIN,
        1,
        u64::MAX,
    ))));
    let refused = operation.advance(&request);
    drop(budget);
    let operation = match refused {
        WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
            assert!(!matches!(denial, worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Execution(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::NestedOpening)), "memory pressure is not a nested opening");
            operation
        }
        _ => panic!("the native leased memory bound must refuse"),
    };
    let completed = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        _ => panic!("the original operation succeeds after native capacity is released"),
    };
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Fresh { retained: None, .. }
    ));
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
    completed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("ordinary source close: {denial:?}"));
}
