use super::*;

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_writeback_bundle(
    policy: BridgeRuntimePolicy,
) -> PricingWritebackBundle {
    let writeback_authority = RecordingTruthWritebackAuthority::default();
    let runtime = build_pricing_runtime_with_policy_and_writeback_authority(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        policy,
        writeback_authority.clone(),
    );
    let lowered_policy = pricing_lowered_policy(&runtime);
    let contract = runtime
        .admit_writeback_declaration(
            pricing_writeback_declaration(
                "writeback:pricing-authority",
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "pricing-authority",
            ),
            &lowered_policy,
        )
        .expect("pricing writeback declaration should admit");
    let effect = runtime.lower_writeback_effect(
        &contract,
        &pricing_writeback_causality_basis(
            "causality:pricing-authority",
            "truth-trigger:pricing-steel-main",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:pricing-authority"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "pricing-authority",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:pricing-authority"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let (commit_outcome, commit_receipt) = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect("pricing writeback authority should commit the first time");
    let commit_record = runtime
        .diagnostics()
        .last_writeback_execution_record()
        .expect("pricing writeback commit should retain an execution record");
    let commit_replay_bundle =
        runtime.replay_writeback_bundle(&contract, &effect, &idempotence, &commit_outcome);

    let (noop_outcome, noop_receipt) = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect("pricing writeback authority should classify repeated causality as canonical noop");
    let noop_record = runtime
        .diagnostics()
        .last_writeback_execution_record()
        .expect("pricing writeback noop should retain an execution record");
    let noop_replay_bundle =
        runtime.replay_writeback_bundle(&contract, &effect, &idempotence, &noop_outcome);

    let rejecting_runtime = build_pricing_runtime_with_policy_and_writeback_authority(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        policy,
        RejectingPricingWritebackAuthority {
            failure_class: BridgeWritebackFailureClass::MergeAuthorityRejected,
        },
    );
    let rejecting_lowered_policy = pricing_lowered_policy(&rejecting_runtime);
    let rejecting_contract = rejecting_runtime
        .admit_writeback_declaration(
            pricing_writeback_declaration(
                "writeback:pricing-rejection",
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "pricing-rejection",
            ),
            &rejecting_lowered_policy,
        )
        .expect("pricing rejection declaration should admit");
    let rejecting_effect = rejecting_runtime.lower_writeback_effect(
        &rejecting_contract,
        &pricing_writeback_causality_basis(
            "causality:pricing-rejection",
            "truth-trigger:pricing-rubber-shock",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:pricing-rejection"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "pricing-rejection",
        ),
    );
    let rejecting_idempotence = rejecting_runtime.classify_writeback_idempotence(
        &rejecting_effect,
        &rejecting_lowered_policy,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&rejecting_effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:pricing-rejection"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let rejection_error = rejecting_runtime
        .execute_writeback_authority(
            &rejecting_contract,
            &rejecting_effect,
            &rejecting_idempotence,
        )
        .expect_err("pricing writeback rejection should stay typed");
    let rejection_record = rejecting_runtime
        .diagnostics()
        .last_writeback_execution_record()
        .expect("pricing writeback rejection should retain an execution record");

    PricingWritebackBundle {
        family_kind: effect.family_kind(),
        strategy_class: effect.strategy_class(),
        commit_outcome_class: commit_outcome.outcome_class(),
        noop_outcome_class: noop_outcome.outcome_class(),
        commit_replay_semantic_digest: commit_replay_bundle.semantic_digest().to_owned(),
        noop_replay_semantic_digest: noop_replay_bundle.semantic_digest().to_owned(),
        shared_authoritative_artifact: commit_receipt.authoritative_artifact_digest()
            == noop_receipt.authoritative_artifact_digest(),
        authority_commit_count: writeback_authority.committed_causality_count(),
        execution_request_count: noop_record.counters().writeback_request_count(),
        execution_commit_count: commit_record.counters().writeback_commit_count(),
        execution_noop_count: noop_record.counters().writeback_noop_count(),
        rejection_error_kind: rejection_error.kind(),
        rejection_failure_class: rejection_record
            .failure_class()
            .expect("pricing writeback rejection should carry a failure class"),
        rejection_request_emitted: rejection_record.request_digest().is_some(),
        rejection_receipt_emitted: rejection_record.receipt_digest().is_some(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_workload_certification_bundle(
    policy: BridgeRuntimePolicy,
    preview_session_identity: BridgePreviewSessionIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingWorkloadCertificationBundle {
    let hostile_source = InMemoryRelationalBridgeSource::default();
    hostile_source.insert_committed_patch(pricing_patch(
        pricing_patch_envelope_identity(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-missing-snapshot"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-missing-snapshot"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-missing"),
        ),
        "steel",
    ));
    let hostile_runtime = build_pricing_runtime_with_policy(
        hostile_source,
        RecordingSignalBridgeSink::default(),
        policy,
    );

    PricingWorkloadCertificationBundle {
        matrix: capture_pricing_certification_matrix(
            policy,
            preview_session_identity,
            resource_request,
        ),
        aspect: capture_pricing_aspect_bundle(policy, resource_request),
        discard: capture_pricing_discard_bundle(resource_request),
        promotion: capture_pricing_promotion_bundle(resource_request),
        fanout: capture_pricing_fanout_bundle(resource_request),
        restart_replay: capture_pricing_restart_replay_bundle(policy, resource_request),
        restart_failure: capture_pricing_restart_failure_bundle(resource_request),
        writeback: capture_pricing_writeback_bundle(policy),
        provenance: capture_pricing_historical_provenance_bundle(policy, resource_request),
        portfolio: capture_pricing_portfolio_blast_radius_bundle(),
        crisis: capture_pricing_crisis_bundle(),
        strategy: capture_pricing_strategy_bundle(),
        simulation: capture_pricing_simulation_suite(),
        trust_attacks: capture_pricing_trust_attack_bundle(),
        hostile_failure: capture_pricing_missing_snapshot_failure_bundle(
            &hostile_runtime,
            resource_request,
        ),
    }
}

pub(in crate::harness::tests::pricing_shock) fn pricing_historical_source_declaration(
    declaration_id: &str,
) -> SourceDeclaration {
    SourceDeclaration::new(
        SourceDeclarationIdentity::admit_bridge_owned(declaration_id),
        BridgeTruthViewSelector::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
        ),
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::HistoricalRead,
            BridgeSourceCapability::BranchRead,
            BridgeSourceCapability::ReplayContinuityRead,
        ]),
    )
}

pub(in crate::harness::tests::pricing_shock) fn pricing_harness_fixture(
    name: &str,
    policy: BridgeRuntimePolicy,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    let scenario = generated_pricing_scenario();
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![
            pricing_mapping(
                "steel",
                SignalInvalidationScope::admit_bridge_owned("price:bicycle"),
            ),
            pricing_mapping(
                "steel",
                SignalInvalidationScope::admit_bridge_owned("price:wheelbarrow"),
            ),
            pricing_mapping(
                "rubber",
                SignalInvalidationScope::admit_bridge_owned("price:scooter"),
            ),
        ])
        .with_policy(policy)
        .with_source_declaration(pricing_historical_source_declaration(
            "source:pricing-main-history",
        ))
        .with_source_adapter_capabilities(BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::HistoricalRead,
            BridgeSourceCapability::BranchRead,
            BridgeSourceCapability::ReplayContinuityRead,
        ]))
        .with_committed_patch(pricing_patch(
            pricing_patch_envelope_identity(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch:steel-main"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
            ),
            "steel",
        ))
        .with_committed_patch(pricing_patch(
            pricing_patch_envelope_identity(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch:rubber-main"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
            ),
            "rubber",
        ))
        .with_snapshot(scenario.main_snapshot),
    )
    .declare_input("pricing-source")
    .declare_observation("pricing-source")
    .compile()
}
