use super::*;

#[test]
fn retained_sessions_execute_b_a_b_without_reopening_sources_or_readmitting_slots() {
    use worth_signal::facade::SignalConditionalDecisionClass;

    let opened = Arc::new(AtomicUsize::new(0));
    let predicate = SessionObservationPredicate::default();
    let (mut owner, installation) = installation_fixture_with_runtime(
        super::super::super::semantic_dependencies::runtime_predicate_contract("query:one"),
        &["bridge-main"],
        BridgeConditionalProviderSet::new()
            .condition(predicate.clone())
            .compute(Compute(1)),
        &[],
        |registrations| {
            let mapping = super::super::super::exact_mapping();
            let aspect = super::super::super::aspect_mapping(&mapping);
            registrations
                .into_iter()
                .fold(
                    RuntimeBridgeBuilder::new()
                        .with_committed_patch_source(super::super::super::TestSource)
                        .with_snapshot_read_source(ReusableReaderSource(Arc::clone(&opened)))
                        .with_signal_sink(super::super::super::TestSink)
                        .register_mapping(mapping)
                        .register_aspect_mapping(aspect),
                    |builder, registration| builder.register_semantic_correspondence(registration),
                )
                .build()
                .unwrap()
        },
    );
    let lowering = owner.install(installation).unwrap();
    let owner = owner.seal().unwrap();
    let signal_basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let source_b = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let source_a = crate::truth_identity_fixtures::truth_snapshot(1, 2);
    let session_b = owner
        .admit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                &signal_basis, &source_b,
            ),
        )
        .unwrap();
    let session_a = owner
        .admit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                &signal_basis, &source_a,
            ),
        )
        .unwrap();
    let execute = |session: &crate::facade::BridgeConditionalEvaluationSession,
                   source: &TruthSnapshotIdentity,
                   label: &str,
                   attempt| {
        owner
            .execute_admitted_conditional(
                session,
                BridgeConditionalExecutionRequest {
                    lowering: &lowering,
                    query_binding_identity: "query-binding",
                    query_capability_identity: 1,
                    snapshot_identity: label,
                    truth_branch_identity: None,
                    bridge_snapshot_identity: Some(source),
                    execution_identity: label,
                    attempt,
                },
                &mut (),
            )
            .unwrap()
    };
    let first_b = execute(&session_b, &source_b, "B", 1);
    let first_a = execute(&session_a, &source_a, "A", 1);
    let worth_proof::TransitionOutcome::Success(delivery) = owner
        .deliver_owned_authoritative_change(&signal_basis, 0)
        .unwrap()
    else {
        panic!("B/A/B proof requires one performed dependency transition")
    };
    let _session_a_successor = owner
        .readmit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationReadmissionRequest {
                predecessor: &session_a,
                transitions: &[&delivery],
            },
        )
        .unwrap();
    let session_b = owner
        .readmit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationReadmissionRequest {
                predecessor: &session_b,
                transitions: &[&delivery],
            },
        )
        .unwrap();
    let second_b = execute(&session_b, &source_b, "B", 2);

    assert_eq!(opened.load(Ordering::SeqCst), 2);
    assert_eq!(
        first_b.signal().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert_eq!(first_a.signal().counters().compute_contacts, 1);
    assert_eq!(second_b.signal().counters().compute_contacts, 1);
    assert_eq!(
        *predicate
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
        vec![
            (None, "B".to_string()),
            (None, "A".to_string()),
            (Some("B".to_string()), "B".to_string()),
        ],
        "each retained session owns its own semantic baseline",
    );
}
