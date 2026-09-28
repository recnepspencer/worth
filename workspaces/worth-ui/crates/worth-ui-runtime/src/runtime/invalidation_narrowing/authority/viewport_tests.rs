use worth_ui_host_contract::{
    UiHostMeasurementObservationValue, UiHostMeasurementRequest, UiMeasurementEvidenceFamily,
    UiMeasurementRequestIdentity, UiViewportExtentObservation, UiViewportExtentRequest,
    WorthUiMeasurementHostAdapter,
};
use worth_ui_inspection::UiEvidenceAuthorityGeneration;

struct ViewportAdapter(f32);

impl WorthUiMeasurementHostAdapter for ViewportAdapter {
    fn observe_measurement(
        &self,
        _request: &UiHostMeasurementRequest,
    ) -> UiHostMeasurementObservationValue {
        UiHostMeasurementObservationValue::ViewportExtent(UiViewportExtentObservation {
            width: self.0,
            height: 600.0,
        })
    }
}

fn viewport_collection_input(
    report: &worth_ui_host_contract::WorthUiHostCapabilityReport,
    evidence_generation: UiEvidenceAuthorityGeneration,
    profile: crate::host::UiHostMeasurementAssumptionProfile,
) -> crate::host::UiHostMeasurementCollectionInput<'_> {
    crate::host::UiHostMeasurementCollectionInput {
        identity: UiMeasurementRequestIdentity::new(900),
        evidence_family: UiMeasurementEvidenceFamily::ViewportExtent,
        need: crate::host::UiHostMeasurementNeed::ViewportExtent(UiViewportExtentRequest),
        capability_report: report,
        evidence_generation,
        normalization_context:
            crate::host::UiHostMeasurementNormalizationContext::viewport_logical_exact(profile),
    }
}

#[test]
fn viewport_fanout_at_receipt_ceiling_commits_every_bounded_neighborhood() {
    let (mut runtime, _, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_viewport_catalog(64);
    let completion = submit_viewport_observation(&mut runtime, 900.0);
    let outcome = completion
        .viewport_resize_outcome()
        .expect("at-limit viewport fanout commits through the ordinary lane");
    assert_eq!(outcome.counters().selected_neighborhoods(), 64);
    assert_eq!(outcome.counters().committed_receipts(), 64);
    assert_eq!(outcome.counters().authority_probes(), 1);
    assert_eq!(outcome.counters().emitted_targets(), 64);
    assert_eq!(outcome.evidence().maximum_committed_receipts(), 64);
}

#[test]
fn viewport_fanout_over_target_ceiling_denies_before_locality() {
    let (mut runtime, _, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_viewport_catalog(129);
    let committed_before = runtime.committed_allocation_scope_count_for_test();
    let completion = submit_viewport_observation(&mut runtime, 900.0);
    let crate::runtime::WorthUiFrameworkTurnCompletion::AllocationInvalidationNarrowingDenied {
        rejection,
    } = completion
    else {
        panic!("over-limit viewport fanout must deny during narrowing preflight");
    };
    assert_eq!(
        rejection.denial(),
        crate::runtime::UiAllocationInvalidationNarrowingDenial::ViewportTargetBudgetExceeded {
            ordinal: 0,
            attempted: 129,
            maximum: 128,
        }
    );
    assert_eq!(rejection.counters().graph_target_lookups(), 1);
    assert_eq!(rejection.counters().authority_probes(), 1);
    assert_eq!(rejection.counters().emitted_targets(), 0);
    assert_eq!(rejection.counters().materialized_host_target_sets(), 0);
    assert_eq!(
        runtime.committed_allocation_scope_count_for_test(),
        committed_before,
        "preflight denial cannot mutate the committed receipt ledger"
    );
}

#[test]
fn cumulative_viewport_target_ceiling_denies_the_first_excess_sample() {
    let (mut runtime, _, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_viewport_catalog(3);
    let committed_before = runtime.committed_allocation_scope_count_for_test();
    let report = crate::evidence::measurement::projection::fact_test_support::capability_report(77);
    let profile = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        &report, 11, 22, 33, 44,
    );
    let completion = runtime.execute_framework_turn(|turn| {
        turn.host_measurement(|source| {
            for sample in 0..43 {
                let width = 800.0 + sample as f32 * 20.0;
                source
                    .collect_and_submit(
                        &ViewportAdapter(width),
                        viewport_collection_input(
                            &report,
                            UiEvidenceAuthorityGeneration::new(17),
                            profile,
                        ),
                    )
                    .expect("viewport observation admits");
            }
        });
    });
    let crate::runtime::WorthUiFrameworkTurnCompletion::AllocationInvalidationNarrowingDenied {
        rejection,
    } = completion
    else {
        panic!("cumulative target work must deny before locality");
    };
    assert_eq!(
        rejection.denial(),
        crate::runtime::UiAllocationInvalidationNarrowingDenial::ViewportTargetBudgetExceeded {
            ordinal: 42,
            attempted: 129,
            maximum: 128,
        }
    );
    assert_eq!(rejection.counters().invalidation_visits(), 43);
    assert_eq!(rejection.counters().authority_probes(), 43);
    assert_eq!(rejection.counters().emitted_targets(), 126);
    assert_eq!(rejection.counters().materialized_host_target_sets(), 42);
    assert_eq!(
        runtime.committed_allocation_scope_count_for_test(),
        committed_before
    );
}

#[test]
fn viewport_receipt_ceiling_denies_post_locality_without_ledger_mutation() {
    let (mut runtime, _, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_viewport_catalog(65);
    let committed_before = runtime.committed_allocation_scope_count_for_test();
    let completion = submit_viewport_observation(&mut runtime, 900.0);
    assert!(matches!(
        completion,
        crate::runtime::WorthUiFrameworkTurnCompletion::ViewportResizeDenied {
            denial: crate::runtime::UiViewportResizeDenial::ReceiptBudgetExceeded {
                selected: 65,
                maximum: 64,
            },
            ..
        }
    ));
    assert_eq!(
        runtime.committed_allocation_scope_count_for_test(),
        committed_before
    );
}

#[test]
fn viewport_fanout_remains_complete_across_separate_successor_turns() {
    let expected = 40;
    let (mut runtime, _, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_viewport_catalog(expected);

    for width in [1_120.0, 960.0] {
        let completion = submit_viewport_observation(&mut runtime, width);
        let outcome = completion
            .viewport_resize_outcome()
            .expect("each viewport successor must resolve through the ordinary lane");
        assert_eq!(
            outcome.counters().selected_neighborhoods(),
            u16::try_from(expected).unwrap(),
            "viewport authority succession cannot lose a previously admitted neighborhood"
        );
        assert_eq!(
            outcome.counters().committed_receipts(),
            u16::try_from(expected).unwrap()
        );
    }
}

fn submit_viewport_observation<'a>(
    runtime: &'a mut crate::runtime::WorthUiRuntimeFrameworkLoop,
    width: f32,
) -> crate::runtime::WorthUiFrameworkTurnCompletion<'a> {
    let report = crate::evidence::measurement::projection::fact_test_support::capability_report(77);
    let profile = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        &report, 11, 22, 33, 44,
    );
    runtime.execute_framework_turn(|turn| {
        turn.host_measurement(|source| {
            source
                .collect_and_submit(
                    &ViewportAdapter(width),
                    viewport_collection_input(
                        &report,
                        UiEvidenceAuthorityGeneration::new(17),
                        profile,
                    ),
                )
                .expect("viewport observation admits");
        });
    })
}

#[test]
fn ordinary_viewport_churn_resolves_with_bounded_receipts_and_no_durable_mutation() {
    let (mut runtime, first_active, second_active, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_production_catalog_activation();
    let report = crate::evidence::measurement::projection::fact_test_support::capability_report(77);
    let profile = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        &report, 11, 22, 33, 44,
    );
    let generation = UiEvidenceAuthorityGeneration::new(17);

    let completion = runtime.execute_framework_turn(|turn| {
        turn.host_measurement(|source| {
            for width in [800.0, 820.0, 840.0, 860.0] {
                source
                    .collect_and_submit(
                        &ViewportAdapter(width),
                        viewport_collection_input(&report, generation, profile),
                    )
                    .expect("ordinary viewport observation admits");
            }
        });
    });

    let outcome = completion.viewport_resize_outcome().unwrap_or_else(|| {
        panic!("viewport churn resolves through ordinary runtime: {completion:?}")
    });
    let counters = outcome.counters();
    assert_eq!(counters.admitted_observations(), 4);
    assert_eq!(counters.selected_neighborhoods(), 2);
    assert_eq!(counters.committed_receipts(), 2);
    assert_eq!(counters.durable_mutations(), 0);
    assert_eq!(counters.authority_probes(), 4);
    assert_eq!(counters.emitted_targets(), 8);
    assert_eq!(counters.materialized_host_target_sets(), 4);
    assert_eq!(outcome.evidence().maximum_committed_receipts(), 64);
    assert!(counters.committed_receipts() <= outcome.evidence().maximum_committed_receipts());

    let evidence = outcome.evidence();
    let committed = outcome.committed_replan();
    let touched = committed
        .receipts()
        .iter()
        .map(|receipt| {
            receipt
                .committed_allocation()
                .allocation_neighborhood()
                .identity()
                .root_graph_node_identity()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        touched,
        [first_active, second_active].into_iter().collect(),
        "the exact shared viewport witness must fan out to every admitted affected neighborhood"
    );
    assert_eq!(
        evidence.root_posture(),
        crate::graph::UiReplanRootPosture::CountedRootWiden {
            reason: crate::graph::UiReplanWidenReason::SharedAncestorRequirement,
        }
    );

    assert_eq!(evidence.frame_epoch(), outcome.frame_epoch());
    assert_eq!(
        evidence.transaction_idempotency_key(),
        outcome.transaction_idempotency_key()
    );
    assert_eq!(evidence.selected_neighborhoods(), 2);
    assert_eq!(evidence.durable_mutations(), 0);
    assert_eq!(
        evidence.selected_neighborhood_identity_digests(),
        outcome.selected_neighborhood_identity_digests()
    );
    assert_ne!(outcome.transaction_idempotency_key(), 0);
}

#[test]
fn mixed_viewport_and_semantic_input_denies_before_locality_or_commit() {
    let (mut runtime, first_active, _, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_production_catalog_activation();
    let report = crate::evidence::measurement::projection::fact_test_support::capability_report(77);
    let profile = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        &report, 11, 22, 33, 44,
    );

    let completion = runtime.execute_framework_turn(|turn| {
        turn.host_measurement(|source| {
            source
                .collect_and_submit(
                    &ViewportAdapter(900.0),
                    viewport_collection_input(
                        &report,
                        UiEvidenceAuthorityGeneration::new(17),
                        profile,
                    ),
                )
                .expect("viewport observation admits at its source boundary");
        });
        turn.interaction(|source| {
            source
                .admit_and_submit(
                    first_active,
                    crate::runtime::WorthUiTransientInteractionState::TextInput,
                )
                .expect("semantic input admits at its source boundary");
        });
    });

    let crate::runtime::WorthUiFrameworkTurnCompletion::AllocationFrameResolutionDenied {
        rejection,
    } = completion
    else {
        panic!("mixed viewport authority must deny during policy resolution");
    };
    assert_eq!(
        rejection.denial(),
        crate::runtime::UiAllocationFrameResolutionDenial::Policy(
            crate::runtime::UiAllocationStreamCompositionDenial::IllegalFamilyPair {
                left: crate::runtime::UiAllocationStreamFamily::TextInput,
                right: crate::runtime::UiAllocationStreamFamily::ViewportObservation,
            },
        )
    );
}

#[test]
fn identical_ordinary_viewport_churn_replays_to_identical_evidence() {
    fn run() -> crate::evidence::UiViewportResizeEvidence {
        let (mut runtime, _, _, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_production_catalog_activation();
        let report =
            crate::evidence::measurement::projection::fact_test_support::capability_report(77);
        let profile = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
            &report, 11, 22, 33, 44,
        );
        let completion = runtime.execute_framework_turn(|turn| {
            turn.host_measurement(|source| {
                for width in [800.0, 820.0, 840.0, 860.0] {
                    source
                        .collect_and_submit(
                            &ViewportAdapter(width),
                            viewport_collection_input(
                                &report,
                                UiEvidenceAuthorityGeneration::new(17),
                                profile,
                            ),
                        )
                        .expect("replayed viewport observation admits");
                }
            });
        });
        let transient = completion
            .viewport_resize_outcome()
            .expect("ordinary replay resolves")
            .evidence();
        assert_ne!(transient.transaction_idempotency_key(), 0);
        transient
    }

    assert_eq!(run(), run());
}

#[test]
fn incompatible_viewport_normalization_cannot_reuse_admitted_target_authority() {
    let (mut runtime, _, _, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_production_catalog_activation();
    let report = crate::evidence::measurement::projection::fact_test_support::capability_report(77);
    let incompatible_profile =
        crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
            &report, 99, 2, 3, 4,
        );

    let completion = runtime.execute_framework_turn(|turn| {
        turn.host_measurement(|source| {
            source
                .collect_and_submit(
                    &ViewportAdapter(900.0),
                    viewport_collection_input(
                        &report,
                        UiEvidenceAuthorityGeneration::new(17),
                        incompatible_profile,
                    ),
                )
                .expect("host observation is internally coherent");
        });
    });

    let crate::runtime::WorthUiFrameworkTurnCompletion::AllocationInvalidationNarrowingDenied {
        rejection,
    } = completion
    else {
        panic!("incompatible normalization must deny before target selection");
    };
    assert_eq!(
        rejection.denial(),
        crate::runtime::UiAllocationInvalidationNarrowingDenial::HostNormalizationAuthorityMismatch {
            ordinal: 0,
        }
    );
}
