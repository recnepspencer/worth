//! Genuine no-source durable intent survives native checkpoint reconstruction.
use super::*;
use crate::application_program::historical::{
    DiscoveredAction, DiscoveredProgram, Original as StableIntent,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_topology_entry::PlanarInitialAdjustment;
#[test]
fn discovered_restored_historical_commit_is_terminal_and_closes_without_rerun() {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, DiscoveredProgram>::begin()
        .validated_program()
        .unwrap();
    let world = installation::install_selected_program(
        &crate::installed_schema(),
        4096,
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &[],
        program,
    )
    .unwrap();
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
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
        .idempotency_owned(110_024)
        .unwrap_or_else(|(_, _, d)| panic!("capture: {d:?}"))
        .retain_operation::<DiscoveredProgram, DiscoveredAction>(&world.application)
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
    drop(operation);
    let program = ApplicationProgramAuthoring::<ConsumerSchema, DiscoveredProgram>::begin()
        .validated_program()
        .unwrap();
    let checkpoint=world.application.with_host_advancement(|phase|world.application.runtime()
        .capture_application_checkpoint(&phase,worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy::SystemAllocation).unwrap()).unwrap();
    let restored = installation::restore_selected_program(&world, program, checkpoint);
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
        .idempotency_owned(110_024)
        .unwrap_or_else(|(_, _, d)| panic!("restored capture: {d:?}"))
        .retain_operation::<DiscoveredProgram, DiscoveredAction>(&restored)
        .unwrap_or_else(|(_, d)| panic!("restored construction: {d:?}"));
    let terminal=match replay.advance(&request) {
        Advance::Terminal{operation,denial:worth_query_host::facade::application_entry::WorthQueryOperationAdvanceDenial::NativeTerminal}=>operation,
        Advance::Terminal{operation,denial}=>panic!("restored terminal: {denial:?}, native={:?}",operation.native_outcome()),
        Advance::Refused{denial,..}=>panic!("restored admission: {denial:?}"),
        _=>panic!("history cannot complete outputs or recreate a receipt"),
    };
    let outcome = terminal.native_outcome().unwrap();
    assert!(matches!(outcome, worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_)), "restored native outcome: {outcome:?}");
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
            cancellation.token(),
        );
    let (terminal, denial) = terminal
        .close(&restored.request(&principal, &stopped))
        .err()
        .expect("stopped close preserves historical owner");
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryOperationCloseDenial::Execution(_)
    ));
    assert!(matches!(terminal.native_outcome(),Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_))));
    terminal
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("historical close: {d:?}"));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(planar_source_adjustment_contacts(&contacts), (1, 1));
}
