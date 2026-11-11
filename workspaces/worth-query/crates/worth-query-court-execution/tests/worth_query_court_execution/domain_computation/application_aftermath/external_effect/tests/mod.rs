use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect;
pub use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::correlation::ExternalEffectCorrelationBasis;
pub use worth_query_execution::facade::domain_computation::ExternalEffectCorrelationIdentity;
pub use worth_query_execution::facade::domain_computation::ExternalEffectNoEffectCause;
pub use worth_query_execution::facade::domain_computation::ExternalEffectPosture;
pub use worth_query_execution::facade::domain_computation::ExternalEffectPostureKind;
pub use worth_query_execution::facade::domain_computation::ExternalRailTransportFault;
pub use worth_query_execution::facade::domain_computation::WorthQueryDispatchOutboxRecord;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchPostureKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
pub use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;
pub use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::correlation::derive_external_effect_correlation_identity;
pub use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::outbox::dispatch_outbox_create_intent;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::provider::committed_dispatch_outbox::test_support::commit_distinct_records_and_admit_fixture;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::provider::committed_dispatch_outbox::test_support::commit_observe_and_admit_fixture;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::provider::committed_dispatch_outbox::test_support::commit_observe_and_admit_twice_fixture;
pub use worth_foundational::facade::CanonicalDigestId;
pub use worth_foundational::facade::BoundaryProtocolCompatibilityWindow;
pub use worth_foundational::facade::BoundaryProtocolUnsupportedVersionPosture;
pub use worth_foundational::facade::BoundaryProtocolVersion;

pub struct FixedTransport(WorthQueryExternalTransportOutcome);

impl WorthQueryExternalEffectTransport for FixedTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0
    }
}

#[test]
fn completed_dispatch_has_four_distinct_exact_predecessors() {
    let dispatch = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
        &FixedTransport(WorthQueryExternalTransportOutcome::Completed),
        admitted_from_fresh_runtime(7),
        || {},
    )
    .unwrap();
    let ladder = dispatch.causal_ladder();
    let observation = ladder.observation().expect("completion is owner-observed");
    assert_eq!(
        ladder.provider_commit().kind(),
        ExternalEffectPostureKind::ProviderCommit
    );
    assert_eq!(
        ladder.emission().kind(),
        ExternalEffectPostureKind::EmittedApplicationCausality
    );
    assert_eq!(
        ladder.attempt().kind(),
        ExternalEffectPostureKind::DispatchAttempt
    );
    assert_eq!(
        observation.kind(),
        ExternalEffectPostureKind::ExternalCompletion
    );
    let identities = [
        ladder.provider_commit().identity().bytes(),
        ladder.emission().identity().bytes(),
        ladder.attempt().identity().bytes(),
        observation.identity().bytes(),
    ];
    for left in 0..identities.len() {
        for right in (left + 1)..identities.len() {
            assert_ne!(identities[left], identities[right]);
        }
    }
    assert_predecessor(ladder.emission(), ladder.provider_commit());
    assert_predecessor(ladder.attempt(), ladder.emission());
    assert_predecessor(observation, ladder.attempt());
    assert_eq!(dispatch.canonical_work().digest_derivations(), 4);
}

#[test]
fn transport_faults_never_fabricate_owner_observation_or_completion() {
    for (index, outcome) in [
        WorthQueryExternalTransportOutcome::TimedOut,
        WorthQueryExternalTransportOutcome::Disconnected,
        WorthQueryExternalTransportOutcome::LostResponse,
        WorthQueryExternalTransportOutcome::DuplicateAcknowledgement,
    ]
    .into_iter()
    .enumerate()
    {
        let dispatch = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
            &FixedTransport(outcome),
            admitted_from_fresh_runtime(index as u64 + 9),
            || {},
        )
        .unwrap();
        assert!(!dispatch.is_external_completion());
        assert!(dispatch.causal_ladder().observation().is_none());
        assert_eq!(dispatch.canonical_work().digest_derivations(), 3);
    }
}

#[test]
fn a_rejected_payload_is_proven_no_effect_and_never_unresolved() {
    let dispatch = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
        &FixedTransport(WorthQueryExternalTransportOutcome::Rejected),
        admitted_from_fresh_runtime(31),
        || {},
    )
    .unwrap();
    assert_eq!(
        dispatch.posture().kind(),
        WorthQueryExternalDispatchPostureKind::ProvenNoEffect
    );
    let proof = dispatch
        .posture()
        .proven_no_effect()
        .expect("the external owner's rejection is its own proof");
    assert_eq!(proof.cause(), ExternalEffectNoEffectCause::PayloadRejected);
    assert_eq!(proof.attempt_posture(), dispatch.causal_ladder().attempt());
    assert_eq!(
        dispatch.fault(),
        None,
        "proven no effect is not an unresolved transport fault"
    );
    assert!(dispatch.posture().classification().is_none());
    assert_eq!(dispatch.canonical_work().digest_derivations(), 3);
    assert!(!dispatch.is_external_completion());
    assert!(dispatch.causal_ladder().observation().is_none());
}

#[test]
fn unsupported_protocol_version_preserves_exact_external_owner_causality() {
    let unsupported = BoundaryProtocolCompatibilityWindow::inclusive(
        BoundaryProtocolVersion::new(1),
        BoundaryProtocolVersion::new(2),
    )
    .admit(BoundaryProtocolVersion::new(3))
    .unwrap_err();
    assert_eq!(
        unsupported.posture(),
        BoundaryProtocolUnsupportedVersionPosture::ExceedsWindow
    );
    let dispatch = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
        &FixedTransport(
            WorthQueryExternalTransportOutcome::UnsupportedProtocolVersion(unsupported),
        ),
        admitted_from_fresh_runtime(10),
        || {},
    )
    .unwrap();

    assert_eq!(
        dispatch.fault(),
        Some(ExternalRailTransportFault::UnsupportedProtocolVersion(
            unsupported
        ))
    );
    assert!(!dispatch.is_external_completion());
    assert!(dispatch.causal_ladder().observation().is_none());
}

#[test]
fn distinct_attempt_ordinals_produce_distinct_attempt_identities() {
    let (first_admitted, second_admitted) = commit_observe_and_admit_twice_fixture(11);
    assert_ne!(
        first_admitted.ordinal_for_test(),
        second_admitted.ordinal_for_test()
    );
    let first = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
        &FixedTransport(WorthQueryExternalTransportOutcome::LostResponse),
        first_admitted,
        || {},
    )
    .unwrap();
    let second = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
        &FixedTransport(WorthQueryExternalTransportOutcome::Completed),
        second_admitted,
        || {},
    )
    .unwrap();
    assert_eq!(
        first.causal_ladder().emission().identity(),
        second.causal_ladder().emission().identity()
    );
    assert_ne!(
        first.causal_ladder().attempt().identity(),
        second.causal_ladder().attempt().identity()
    );
}

#[test]
fn equal_attempt_ordinals_in_distinct_query_runtimes_cannot_collide() {
    let (first_admitted, first_commit, first_record_ref, first_runtime) =
        commit_observe_and_admit_fixture(12);
    let (second_admitted, second_commit, second_record_ref, second_runtime) =
        commit_observe_and_admit_fixture(12);
    assert_eq!(first_commit, second_commit);
    assert_eq!(first_record_ref, second_record_ref);
    assert_ne!(first_runtime, second_runtime);
    assert_eq!(
        first_admitted.ordinal_for_test(),
        second_admitted.ordinal_for_test()
    );
    let first = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
        &FixedTransport(WorthQueryExternalTransportOutcome::LostResponse),
        first_admitted,
        || {},
    )
    .unwrap();
    let second = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
        &FixedTransport(WorthQueryExternalTransportOutcome::LostResponse),
        second_admitted,
        || {},
    )
    .unwrap();

    assert_ne!(
        first.causal_ladder().provider_commit().identity(),
        second.causal_ladder().provider_commit().identity()
    );
    assert_ne!(
        first.causal_ladder().attempt().identity(),
        second.causal_ladder().attempt().identity()
    );
}

#[test]
fn identical_values_at_distinct_record_identities_have_distinct_causal_roots() {
    let (first_admitted, second_admitted, first_ref, second_ref) =
        commit_distinct_records_and_admit_fixture();
    assert_ne!(first_ref, second_ref);
    let first = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
        &FixedTransport(WorthQueryExternalTransportOutcome::LostResponse),
        first_admitted,
        || {},
    )
    .unwrap();
    let second = worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::dispatch::dispatch_external_effect(
        &FixedTransport(WorthQueryExternalTransportOutcome::LostResponse),
        second_admitted,
        || {},
    )
    .unwrap();

    assert_ne!(
        first.causal_ladder().provider_commit().identity(),
        second.causal_ladder().provider_commit().identity()
    );
}

#[test]
fn undeclared_external_effect_pays_zero_outbox_intents() {
    assert!(dispatch_outbox_create_intent(None, None).is_none());
}

#[test]
fn correlation_rejects_provider_string_as_identity_basis_field() {
    let left = correlation(1);
    let right = correlation(2);
    assert_ne!(left, right);
}

#[test]
fn external_effect_source_rejects_cdc_checkpoint_vocabulary() {
    const CDC_RESIDUE_FORBIDDEN: &[&str] = &[
        "SubscriberCheckpoint",
        "subscriber_checkpoint",
        "CdcSubscriber",
        "SubscriberResumeRequest",
    ];
    let sources = [
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/posture.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/correlation.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/classification.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/outbox.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/identity.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/identity_derivation.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/causal_event.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/dispatch.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/observation.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/transport.rs"),
        include_str!("../../../../../../../worth-query-execution/src/domain_computation/application_aftermath/external_effect/mod.rs"),
    ];
    for source in sources {
        for forbidden in CDC_RESIDUE_FORBIDDEN {
            assert!(!source.contains(forbidden));
        }
    }
}

pub fn correlation(outcome_identity: u64) -> ExternalEffectCorrelationIdentity {
    derive_external_effect_correlation_identity(ExternalEffectCorrelationBasis {
        correlation_family:
            worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                "estate-death-notice-rail",
            )
            .unwrap(),
        operation_slot: "notify-death",
        operation_version: 1,
        outcome_identity,
        idempotency_key: &[0xAB; 32],
        branch: "main",
    })
    .unwrap()
}

pub fn assert_predecessor(successor: &ExternalEffectPosture, predecessor: &ExternalEffectPosture) {
    assert_eq!(
        successor.predecessor().unwrap().predecessor(),
        predecessor.identity()
    );
}

pub fn admitted_from_fresh_runtime(
    outcome_identity: u64,
) -> worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::causal_event::WorthQueryAdmittedExternalDispatchAttempt<'static>{
    commit_observe_and_admit_fixture(outcome_identity as u8).0
}

#[test]
fn correlation_digest_restore_is_exact() {
    let digest = CanonicalDigestId::new([0x42; 32]);
    assert_eq!(
        ExternalEffectCorrelationIdentity::from_digest_for_test(digest).digest(),
        &digest
    );
}

#[path = "protocol_stability/mod.rs"]
pub mod protocol_stability;

mod rejection_time;
