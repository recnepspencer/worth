//! Genuine no-source durable intent survives native checkpoint reconstruction.
use super::*;
use crate::application_program::historical::{FixedAction, FixedProgram, Original as StableIntent};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_topology_entry::PlanarInitialAdjustment;
#[test]
fn fixed_restored_historical_commit_is_terminal_and_closes_without_rerun() {
    let dir = installation::unique_directory("fixed-historical");
    let foreign = crate::installed_schema();
    let program = ApplicationProgramAuthoring::<ConsumerSchema, FixedProgram>::begin()
        .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the completed schema is valid"))
        .validated_program()
        .unwrap();
    let home = installation::open_directory_selected_program(&dir, &foreign, program);
    let application = home.runtime();
    let adapter = authentication::admit(application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = application.request(&principal, &scope);
    let input = PlanarInitialAdjustment {
        scope_key: "anchor-b".to_owned(),
        replacement_y: length(2),
    };
    // This existing handler reports real decision/candidate contacts by the same input fields.
    let contacts = PlanarSourceAdjustment {
        scope_key: input.scope_key.clone(),
        replacement_y: input.replacement_y,
    };
    reset_planar_source_adjustment_contacts(&contacts);
    let operation = request
        .mutate(StableIntent(input.clone()))
        .without_source()
        .idempotency_owned(10_094)
        .unwrap_or_else(|(_, _, d)| panic!("capture: {d:?}"))
        .retain_durable_operation::<FixedProgram, FixedAction>(&request, application)
        .unwrap_or_else(|(_, d)| panic!("construction: {d:?}"));
    let operation = match operation.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        Advance::Terminal { operation, denial } => panic!(
            "original source: {denial:?}, native={:?}",
            operation.native_outcome()
        ),
        Advance::Refused { denial, .. } => panic!("original source admission: {denial:?}"),
        _ => panic!("original source must perform before capture"),
    };
    assert_eq!(planar_source_adjustment_contacts(&contacts), (1, 1));
    let mut operation = operation;
    let settled = loop {
        match operation.advance(&request) {
            Advance::Progress {
                operation: next, ..
            } => operation = next,
            Advance::Settled(settled) => break settled,
            Advance::Terminal { operation, denial } => panic!(
                "original settlement: {denial:?}, native={:?}",
                operation.native_outcome()
            ),
            Advance::Refused { denial, .. } => panic!("original settlement admission: {denial:?}"),
        }
    };
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("original close: {d:?}"));
    drop(request);
    drop(principal);
    drop(adapter);
    installation::close_directory_program(home);
    let program = ApplicationProgramAuthoring::<ConsumerSchema, FixedProgram>::begin()
        .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the completed schema is valid"))
        .validated_program()
        .unwrap();
    let reopened = installation::open_directory_selected_program(&dir, &foreign, program);
    let restored = reopened.runtime();
    let adapter = authentication::admit(restored.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = restored.request(&principal, &scope);
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let replay = request
        .mutate(StableIntent(input.clone()))
        .without_source()
        .idempotency_owned(10_094)
        .unwrap_or_else(|(_, _, d)| panic!("restored capture: {d:?}"))
        .retain_durable_operation::<FixedProgram, FixedAction>(&request, &restored)
        .unwrap_or_else(|(_, d)| panic!("restored construction: {d:?}"));
    // Recorded replay first observes the original, then asks the real registry
    // for retained output custody (the phase cutover in 6b1c8ff677).
    let replay = match replay.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceReplayObserved,
        } => operation,
        Advance::Progress { .. } => {
            panic!("unexpected original replay phase")
        }
        Advance::Terminal { operation, denial } => panic!(
            "original lookup: {denial:?}, native={:?}",
            operation.native_outcome()
        ),
        Advance::Refused { denial, .. } => panic!("original lookup admission: {denial:?}"),
        Advance::Settled(_) => panic!("historical original lookup cannot settle a live operation"),
    };
    assert_eq!(planar_source_adjustment_contacts(&contacts), (1, 1));
    let terminal = match replay.advance(&request) {
        Advance::Terminal {
            operation,
            denial:
                worth_query_host::facade::application_entry::WorthQueryOperationAdvanceDenial::Outputs(
                    cause,
                ),
        } => {
            assert_eq!(cause.recovery_posture(), worth_query_host::facade::application_entry::WorthQueryRequiredOutputRecoveryPosture::Terminal);
            assert!(
                matches!(cause, worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(ref native)) if native.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable)
            );
            operation
        }
        Advance::Terminal { operation, denial } => panic!(
            "historical terminal: {denial:?}, native={:?}",
            operation.native_outcome()
        ),
        Advance::Refused { denial, .. } => {
            panic!("missing output custody must be terminal: {denial:?}")
        }
        Advance::Progress { .. } => {
            panic!("history cannot recreate output custody")
        }
        Advance::Settled(_) => panic!("history cannot settle without retained output custody"),
    };
    let outcome = terminal.native_outcome().unwrap();
    assert!(matches!(outcome, worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::RecoveredCommitted(_)), "restored native outcome: {outcome:?}");
    assert!(outcome.receipt().is_none());
    assert!(outcome.result().is_none());
    assert!(terminal.recovered_progress().is_none());
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2)
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    let cancellation=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    cancellation.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let (terminal, denial) = terminal
        .close(&restored.request(&principal, &stopped))
        .err()
        .expect("stopped close preserves historical owner");
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryOperationCloseDenial::Execution(_)
    ));
    assert!(matches!(terminal.native_outcome(),Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::RecoveredCommitted(_))));
    terminal
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("historical close: {d:?}"));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(planar_source_adjustment_contacts(&contacts), (1, 1));
    drop(head);
    drop(request);
    drop(principal);
    drop(adapter);
    installation::close_directory_program(reopened);
    let _ = std::fs::remove_dir_all(&dir);
}
