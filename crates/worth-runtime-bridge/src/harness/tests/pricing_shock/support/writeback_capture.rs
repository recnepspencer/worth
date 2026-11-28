use super::*;

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_writeback_bundle(
    policy: BridgeRuntimePolicy,
    truth: &RelationalPricing,
) -> PricingWritebackBundle {
    let writeback_authority = RecordingTruthWritebackAuthority::default();
    let runtime = truth.writeback_runtime(policy, writeback_authority.clone());
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

    let (noop_outcome, noop_receipt) = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect("pricing writeback authority should classify repeated causality as canonical noop");

    PricingWritebackBundle {
        family_kind: effect.family_kind(),
        strategy_class: effect.strategy_class(),
        commit_outcome_class: commit_outcome.outcome_class(),
        noop_outcome_class: noop_outcome.outcome_class(),
        committed_effect_intent_digest: commit_receipt.effect_intent().digest().to_owned(),
        retried_effect_intent_digest: noop_receipt.effect_intent().digest().to_owned(),
        shared_authoritative_artifact: commit_receipt.authoritative_artifact_digest()
            == noop_receipt.authoritative_artifact_digest(),
        authority_commit_count: writeback_authority.committed_causality_count(),
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_workload_certification_bundle(
    policy: BridgeRuntimePolicy,
    preview_session_identity: BridgePreviewSessionIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingWorkloadCertificationBundle {
    let scenario = generated_pricing_scenario();
    let truth = RelationalPricing::reference(resource_request, &scenario, None);
    capture_pricing_workload_certification_bundle_at_truth(
        policy,
        &truth,
        preview_session_identity,
        resource_request,
    )
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_workload_certification_bundle_at_truth(
    policy: BridgeRuntimePolicy,
    truth: &RelationalPricing,
    preview_session_identity: BridgePreviewSessionIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingWorkloadCertificationBundle {
    PricingWorkloadCertificationBundle {
        matrix: capture_pricing_certification_matrix(
            policy,
            truth,
            preview_session_identity,
            resource_request,
        ),
        aspect: capture_pricing_aspect_bundle(policy, truth, resource_request),
        discard: capture_pricing_discard_bundle(resource_request),
        promotion: capture_pricing_promotion_bundle(resource_request),
        fanout: capture_pricing_fanout_bundle(resource_request),
        restart_replay: capture_pricing_restart_replay_bundle(policy, truth, resource_request),
        restart_failure: capture_pricing_restart_failure_bundle(resource_request),
        writeback: capture_pricing_writeback_bundle(policy, truth),
        provenance: capture_pricing_historical_provenance_bundle(policy, resource_request),
        portfolio: capture_pricing_portfolio_blast_radius_bundle(),
        crisis: capture_pricing_crisis_bundle(),
        strategy: capture_pricing_strategy_bundle(),
        simulation: capture_pricing_simulation_suite(),
        trust_attacks: capture_pricing_trust_attack_bundle(truth),
        hostile_failure: capture_pricing_missing_snapshot_failure_bundle(
            truth,
            policy,
            resource_request,
        ),
    }
}

fn pricing_historical_source_declaration(
    declaration_id: &str,
    commit: TruthCommitIdentity,
) -> SourceDeclaration {
    SourceDeclaration::new(
        SourceDeclarationIdentity::admit_bridge_owned(declaration_id),
        BridgeTruthViewSelector::historical_commit(
            TruthBranchIdentity::from_relational_branch_id("main"),
            commit,
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
    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let truth = RelationalPricing::reference(execution, &scenario, None);
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(truth.mappings())
            .with_policy(policy)
            .with_source_declaration(pricing_historical_source_declaration(
                "source:pricing-main-history",
                truth.steel_commit.clone(),
            ))
            .with_source_adapter_capabilities(BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ]))
            .with_relational_truth(truth.into_scenario()),
    )
    .declare_input("pricing-source")
    .declare_observation("pricing-source")
    .compile()
}
