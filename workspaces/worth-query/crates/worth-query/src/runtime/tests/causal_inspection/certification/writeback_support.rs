use worth_runtime_bridge::facade::{
    BridgeDiagnosticsTier, BridgeExecutionPolicyClass, BridgeIdentityEvidence,
    BridgePolicyDeclaration, BridgePolicyDeclarationIdentity, BridgeRequestKind,
    BridgeRouteIdentity, BridgeWritebackAuthoritativeStateBasis, BridgeWritebackCausalityIdentity,
    BridgeWritebackDeclaration, BridgeWritebackDeclarationIdentity, BridgeWritebackEffectClass,
    BridgeWritebackEffectIdentity, BridgeWritebackEffectIntent, BridgeWritebackFamilyKind,
    BridgeWritebackIdempotenceClass, BridgeWritebackIdempotenceIdentity,
    BridgeWritebackNativeCausalityInputs, BridgeWritebackStrategyClass,
    LoweredBridgeExecutionPolicy, RelationalBridgeSnapshotIdentityParts, RuntimeBridge,
    TruthCommitIdentity, TruthSnapshotIdentity,
};

use super::super::materialization::stable_causal_position;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

pub(super) struct RetainedWritebackRecordIdentities {
    pub(super) admission_record_identity: String,
    pub(super) mapper_envelope_identity: String,
    pub(super) mapped_family_input_identity: String,
    pub(super) mapper_record_identity: String,
    pub(super) execution_record_identity: String,
    pub(super) replay_record_identity: String,
}

pub(super) fn retain_writeback_record_identities(
    runtime: &RuntimeBridge,
    commit_identity: &TruthCommitIdentity,
) -> RetainedWritebackRecordIdentities {
    let commit_id = commit_identity
        .relational_commit_id()
        .expect("causal writeback fixture must carry relational commit authority");
    let suffix = format!("commit-{commit_id}");
    let lowered_policy = lowered_writeback_policy(runtime);
    let contract = runtime
        .admit_writeback_declaration(writeback_declaration(&suffix), &lowered_policy)
        .expect("writeback declaration should admit");
    let admission = runtime
        .diagnostics()
        .last_writeback_admission_record()
        .expect("writeback admission record should be retained");
    let causality = BridgeWritebackNativeCausalityInputs::new(
        BridgeWritebackCausalityIdentity::from_bridge_evidence(&bridge_test_evidence(
            "causality",
            &format!("slot:{suffix}"),
        )),
        TruthCommitIdentity::from_relational_commit_id(commit_id + 20_000),
        BridgeRouteIdentity::from_bridge_evidence(&bridge_test_evidence("route", "slot")),
        writeback_snapshot_identity("query-evaluation", commit_id),
        writeback_snapshot_identity("query-truth-view", commit_id),
    );
    let mapped_input = runtime.map_writeback_family_input(
        &contract,
        &causality,
        writeback_effect_intent(BridgeWritebackEffectClass::ProjectedStateDiff, &suffix),
    );
    let mapper_envelope = runtime
        .diagnostics()
        .last_writeback_mapper_envelope()
        .expect("writeback mapper envelope should be retained");
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::from_bridge_evidence(&bridge_test_evidence(
            "effect",
            &format!("slot:{suffix}"),
        )),
        writeback_effect_intent(BridgeWritebackEffectClass::ProjectedStateDiff, &suffix),
    );
    let authoritative_state_basis = BridgeWritebackAuthoritativeStateBasis::from_effect(&effect);
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &authoritative_state_basis,
        BridgeWritebackIdempotenceIdentity::from_bridge_evidence(&bridge_test_evidence(
            "idempotence",
            &format!("slot:{suffix}"),
        )),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let (outcome, _) = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect("writeback authority should execute");
    let mapper_record = runtime
        .diagnostics()
        .last_writeback_mapper_record()
        .expect("writeback mapper record should be retained");
    let execution = runtime
        .diagnostics()
        .last_writeback_execution_record()
        .expect("writeback execution record should be retained");
    let replay_bundle = runtime.replay_writeback_bundle(&contract, &effect, &idempotence, &outcome);
    let drifted_effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::from_bridge_evidence(&bridge_test_evidence(
            "effect",
            &format!("slot:{suffix}:drifted"),
        )),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            &format!("{suffix}:drifted"),
        ),
    );
    let drifted_authoritative_state_basis =
        BridgeWritebackAuthoritativeStateBasis::from_effect(&drifted_effect);
    let drifted_idempotence = runtime.classify_writeback_idempotence(
        &drifted_effect,
        &lowered_policy,
        &drifted_authoritative_state_basis,
        BridgeWritebackIdempotenceIdentity::from_bridge_evidence(&bridge_test_evidence(
            "idempotence",
            &format!("slot:{suffix}:drifted"),
        )),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let drifted_bundle =
        runtime.replay_writeback_bundle(&contract, &drifted_effect, &drifted_idempotence, &outcome);
    runtime
        .validate_replayed_writeback_bundle(&replay_bundle, &drifted_bundle)
        .expect_err("drifted replay should retain a writeback replay record");
    let replay = runtime
        .diagnostics()
        .last_writeback_replay_record()
        .expect("writeback replay record should be retained");

    RetainedWritebackRecordIdentities {
        admission_record_identity: admission
            .record_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
            .to_string(),
        mapper_envelope_identity: mapper_envelope
            .envelope_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
            .to_string(),
        mapped_family_input_identity: mapped_input
            .mapped_input_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
            .to_string(),
        mapper_record_identity: mapper_record
            .record_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
            .to_string(),
        execution_record_identity: execution
            .record_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
            .to_string(),
        replay_record_identity: replay
            .record_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
            .to_string(),
    }
}

fn writeback_snapshot_identity(namespace: &str, commit_id: u64) -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        stable_causal_position(namespace, commit_id.to_string()),
        commit_id,
    ))
}

fn writeback_effect_intent(
    effect_class: BridgeWritebackEffectClass,
    suffix: &str,
) -> BridgeWritebackEffectIntent {
    BridgeWritebackEffectIntent::validated_scalar_patch(
        effect_class,
        worth_foundational::facade::AspectKey::new("worth.query.writeback")
            .expect("valid writeback effect aspect key"),
        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(format!(
            "query-effect:{suffix}"
        )),
    )
    .expect("causal certification writeback effect intent should validate")
}

fn lowered_writeback_policy(runtime: &RuntimeBridge) -> LoweredBridgeExecutionPolicy {
    let contract = runtime
        .admit_policy_declaration(BridgePolicyDeclaration::new(
            BridgePolicyDeclarationIdentity::from_bridge_evidence(&bridge_test_evidence(
                "policy",
                "slot-writeback",
            )),
            BridgeRequestKind::Authoritative,
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeDiagnosticsTier::Standard,
            true,
            true,
        ))
        .expect("writeback policy should admit");
    runtime.lower_admitted_policy(&contract)
}

fn writeback_declaration(suffix: &str) -> BridgeWritebackDeclaration {
    BridgeWritebackDeclaration::writeback_capable(
        BridgeWritebackDeclarationIdentity::from_bridge_evidence(&bridge_test_evidence(
            "writeback",
            &format!("slot:{suffix}"),
        )),
        BridgeRequestKind::Authoritative,
        BridgeWritebackFamilyKind::ProjectedStateDiff,
        BridgeWritebackEffectClass::ProjectedStateDiff,
        BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    )
}

fn bridge_test_evidence(role: &'static str, value: &str) -> BridgeIdentityEvidence {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority)
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_value(WorthQueryEvidenceTag::new("value"), value)
        .seal()
        .expect("fixture digest must fit")
        .bridge_external_identity_evidence()
}
