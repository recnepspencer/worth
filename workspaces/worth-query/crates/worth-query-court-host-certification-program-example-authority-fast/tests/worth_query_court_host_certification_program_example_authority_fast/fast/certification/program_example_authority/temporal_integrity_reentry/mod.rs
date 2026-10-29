pub(crate) use worth_query_worlds::facade::certification::program_example_authority::temporal_integrity_reentry::*;

#[test]
fn temporal_reentry_decodes_the_example_integrity_refusal() {
    // The re-entry completes the intent with a lifecycle TemporalIntegrity
    // does not admit, so the commit boundary refuses it.
    let mut app = ExampleApplication::publish_completing("ready", "archived");
    let scope = super::product_workflow_support::adapters::request_scope();
    let actor = principal(&app, &scope);
    let branch = app.runtime.current_world();
    let mut port = app
        .runtime
        .with_host_advancement(|phase| {
            let selected = app.runtime.on_branch(branch).select(&phase).unwrap();
            let execution = phase
                .checked_execution_request_for(app.runtime.product_runtime())
                .unwrap();
            selected
                .conditional_clock(&execution, &app.conditional.clock)
                .unwrap()
        })
        .unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let observed = match port.observe(&ExecutionRequest::serial(&request)) {
        Observation::Accepted(observed) => observed,
        Observation::Failed(cause) => panic!("conditional observation failed: {cause:?}"),
        _ => panic!("a fresh clock reading must be accepted"),
    };
    assert_eq!(observed.committed_operation_count(), 0);
    let [wake] = observed.execution_provenance() else {
        panic!(
            "one due wake must be evaluated: {:?}",
            observed.execution_provenance()
        );
    };
    assert_eq!(wake.signal_decision(), Some(SignalDecision::Eligible));
    assert_eq!(wake.terminal(), Terminal::Failed, "{wake:?}");
    let Some(Cause::CustomInvariant(denial)) = wake.cause() else {
        panic!("the integrity refusal must be the wake's cause: {wake:?}")
    };
    assert_eq!(
        denial.decode_refusal::<TemporalIntegrityRefusalBinding>(),
        Ok(TemporalIntegrityRefusal::InvalidState {
            revision: Some(2),
            lifecycle: Some("archived".to_owned()),
        })
    );
    drop(port);
    // Nothing of the refused completion was published.
    let row = read_row(&app, branch, &actor, &scope);
    assert_eq!((row.revision, row.lifecycle.as_str()), (1, "active"));
    app.runtime.close_conditional_runtime().unwrap();
}
