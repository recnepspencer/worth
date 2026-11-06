use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::AspectKey;
use worth_runtime_bridge::facade::{
    BridgeIdentityEvidence, BridgePolicyDeclaration, BridgePolicyDeclarationIdentity,
    BridgeRouteIdentity, BridgeWritebackCausalityIdentity, BridgeWritebackEffectIdentity,
    BridgeWritebackEffectIntent, BridgeWritebackIdempotenceIdentity,
    BridgeWritebackNativeCausalityInputs, RuntimeBridge,
};

use crate::application::{query_truth_commit_identity, query_truth_snapshot_identity};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::workflow::QueryWritebackDeclaration;

use super::execution::EffectExecutionDenialKind;

pub(crate) fn execute_lowered_writeback(
    runtime: &RuntimeBridge,
    declaration: &QueryWritebackDeclaration,
) -> Result<
    worth_runtime_bridge::facade::BridgeAdmittedWritebackExecution,
    (EffectExecutionDenialKind, String),
> {
    let request = worth_runtime_bridge::facade::BridgeAdmittedWritebackExecutionRequest::new(
        BridgePolicyDeclaration::new(
            BridgePolicyDeclarationIdentity::from_bridge_evidence(
                &effect_writeback_bridge_evidence("policy", declaration),
            ),
            declaration.causality_binding().request_kind(),
            worth_runtime_bridge::facade::BridgeExecutionPolicyClass::DeterministicCanonical,
            worth_runtime_bridge::facade::BridgeDiagnosticsTier::Standard,
            true,
            true,
        ),
        declaration.bridge_declaration().clone(),
        BridgeWritebackNativeCausalityInputs::new(
            BridgeWritebackCausalityIdentity::from_bridge_evidence(
                &effect_writeback_bridge_evidence("causality", declaration),
            ),
            query_truth_commit_identity(
                "effect-causality",
                effect_writeback_truth_commit_evidence("truth-commit-causality", declaration)
                    .as_str(),
            ),
            BridgeRouteIdentity::from_bridge_evidence(&effect_writeback_bridge_evidence(
                "route",
                declaration,
            )),
            query_truth_snapshot_identity(
                "effect-evaluation",
                effect_writeback_truth_snapshot_evidence("truth-snapshot-evaluation", declaration)
                    .as_str(),
            ),
            query_truth_snapshot_identity(
                "effect-basis",
                effect_writeback_truth_snapshot_evidence("truth-snapshot-basis", declaration)
                    .as_str(),
            ),
        ),
        BridgeWritebackEffectIdentity::from_bridge_evidence(&effect_writeback_bridge_evidence(
            "effect",
            declaration,
        )),
        query_writeback_effect_intent(declaration)?,
        BridgeWritebackIdempotenceIdentity::from_bridge_evidence(
            &effect_writeback_bridge_evidence("idempotence", declaration),
        ),
        declaration.bridge_declaration().idempotence_class(),
    );
    runtime
        .execute_admitted_writeback(request)
        .map_err(map_bridge_writeback_execution_error)
}

fn effect_writeback_bridge_evidence(
    role: &str,
    declaration: &QueryWritebackDeclaration,
) -> Result<BridgeIdentityEvidence, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority,
    )
    .field_shape(WorthQueryEvidenceTag::new("role"), role)
    .field_bridge_authority_identity(
        WorthQueryEvidenceTag::new("bridge_declaration"),
        &declaration
            .bridge_declaration()
            .declaration_identity()
            .bridge_trust_boundary(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("lowering"),
        declaration.lowering_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("causality"),
        declaration.causality_binding().causality_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("basis"),
        declaration.causality_binding().basis_identity(),
    )
    .seal()?
    .bridge_evidence_identity())
}

fn effect_writeback_truth_commit_evidence(
    role: &str,
    declaration: &QueryWritebackDeclaration,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority,
    )
    .field_shape(WorthQueryEvidenceTag::new("role"), role)
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("causality"),
        declaration.causality_binding().causality_identity(),
    )
    .seal()?)
}

fn effect_writeback_truth_snapshot_evidence(
    role: &str,
    declaration: &QueryWritebackDeclaration,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let basis_identity = match role {
        "truth-snapshot-basis" => declaration.causality_binding().basis_identity(),
        _ => declaration.lowering_identity(),
    };
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority,
    )
    .field_shape(WorthQueryEvidenceTag::new("role"), role)
    .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_identity)
    .seal()?)
}

fn query_writeback_effect_intent(
    declaration: &QueryWritebackDeclaration,
) -> Result<BridgeWritebackEffectIntent, (EffectExecutionDenialKind, String)> {
    BridgeWritebackEffectIntent::validated_scalar_patch(
        declaration.bridge_declaration().effect_class(),
        AspectKey::new("query.writeback.effect")
            .expect("static query writeback effect aspect key is valid"),
        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
            declaration.lowering_for_reporting().to_string(),
        ),
    )
    .map_err(|error| {
        (
            EffectExecutionDenialKind::BridgeWritebackExecutionFailed,
            format!("{error:?}"),
        )
    })
}

fn map_bridge_writeback_execution_error(
    error: worth_runtime_bridge::facade::BridgeAdmittedWritebackExecutionError,
) -> (EffectExecutionDenialKind, String) {
    match error {
        worth_runtime_bridge::facade::BridgeAdmittedWritebackExecutionError::PolicyAdmission(
            rejection,
        ) => (
            EffectExecutionDenialKind::BridgePolicyAdmissionFailed,
            format!("{rejection:?}"),
        ),
        worth_runtime_bridge::facade::BridgeAdmittedWritebackExecutionError::Writeback(error) => (
            EffectExecutionDenialKind::BridgeWritebackExecutionFailed,
            format!("{error:?}"),
        ),
    }
}
