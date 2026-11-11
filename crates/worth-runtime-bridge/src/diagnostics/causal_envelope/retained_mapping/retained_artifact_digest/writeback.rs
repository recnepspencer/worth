use crate::diagnostics::BridgeDiagnosticsFacade;
use crate::identity::BridgeIdentityEvidence;
use crate::policy::BridgeDiagnosticsTier;
use crate::writeback::{
    BridgeMappedWritebackFamilyInputIdentity, BridgeWritebackEffectClass,
    BridgeWritebackExecutionRecordIdentity, BridgeWritebackFailureClass,
    BridgeWritebackFamilyAdmissionRecordIdentity, BridgeWritebackFamilyKind,
    BridgeWritebackMapperEnvelopeIdentity, BridgeWritebackMapperRecordIdentity,
    BridgeWritebackOutcomeClass, BridgeWritebackReplayRecordIdentity, BridgeWritebackStrategyClass,
};

use super::super::digest_basis::{
    compose_retained_causal_mapping_evidence_identity, retained_mapping_bridge_identity_part,
    retained_mapping_shape_part, RetainedCausalMappingDigestArtifact,
};

pub(crate) fn writeback_admission_record_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Result<
    Option<BridgeIdentityEvidence>,
    crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal,
> {
    facade
        .writeback_admission_record_for_identity(
            &BridgeWritebackFamilyAdmissionRecordIdentity::from_retained_evidence_reference(
                reference_identity,
            ),
        )
        .map(|record| writeback_admission_digest(&record))
        .transpose()
}

pub(crate) fn writeback_admission_digest(
    record: &crate::writeback::BridgeWritebackFamilyAdmissionRecord,
) -> Result<BridgeIdentityEvidence, crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal>
{
    let replay_permitted = record.replay_artifacts_permitted().to_string();
    Ok(compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::WritebackAdmissionRecord,
        &[
            retained_mapping_bridge_identity_part(record.record_identity()),
            retained_mapping_shape_part(writeback_family_kind_label(record.family_kind())),
            retained_mapping_shape_part(writeback_effect_class_label(record.effect_class())),
            retained_mapping_shape_part(writeback_strategy_class_label(record.strategy_class())),
            retained_mapping_shape_part(diagnostics_tier_label(record.diagnostics_tier())),
            retained_mapping_shape_part(replay_permitted.as_str()),
        ],
    )?)
}

pub(crate) fn writeback_mapper_envelope_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Result<
    Option<BridgeIdentityEvidence>,
    crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal,
> {
    facade
        .writeback_mapper_envelope_for_identity(
            &BridgeWritebackMapperEnvelopeIdentity::from_retained_evidence_reference(
                reference_identity,
            ),
        )
        .map(|envelope| writeback_mapper_envelope_artifact_digest(&envelope))
        .transpose()
}

pub(crate) fn writeback_mapper_envelope_artifact_digest(
    envelope: &crate::writeback::BridgeWritebackMapperEnvelope,
) -> Result<BridgeIdentityEvidence, crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal>
{
    Ok(compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::WritebackMapperEnvelope,
        &[
            retained_mapping_bridge_identity_part(envelope.envelope_identity()),
            retained_mapping_shape_part(writeback_family_kind_label(envelope.family_kind())),
            retained_mapping_shape_part(writeback_effect_class_label(envelope.effect_class())),
            retained_mapping_shape_part(writeback_strategy_class_label(envelope.strategy_class())),
        ],
    )?)
}

pub(crate) fn writeback_mapped_family_input_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Result<
    Option<BridgeIdentityEvidence>,
    crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal,
> {
    facade
        .writeback_mapped_family_input_for_identity(
            &BridgeMappedWritebackFamilyInputIdentity::from_retained_evidence_reference(
                reference_identity,
            ),
        )
        .map(|mapped_input| writeback_mapped_family_input_artifact_digest(&mapped_input))
        .transpose()
}

pub(crate) fn writeback_mapped_family_input_artifact_digest(
    mapped_input: &crate::writeback::BridgeMappedWritebackFamilyInput,
) -> Result<BridgeIdentityEvidence, crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal>
{
    Ok(compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::WritebackMappedFamilyInput,
        &[
            retained_mapping_bridge_identity_part(mapped_input.mapped_input_identity()),
            retained_mapping_shape_part(writeback_family_kind_label(mapped_input.family_kind())),
            retained_mapping_shape_part(writeback_effect_class_label(mapped_input.effect_class())),
            retained_mapping_shape_part(writeback_strategy_class_label(
                mapped_input.strategy_class(),
            )),
        ],
    )?)
}

pub(crate) fn writeback_mapper_record_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Result<
    Option<BridgeIdentityEvidence>,
    crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal,
> {
    facade
        .writeback_mapper_record_for_identity(
            &BridgeWritebackMapperRecordIdentity::from_retained_evidence_reference(
                reference_identity,
            ),
        )
        .map(|record| writeback_mapper_record_artifact_digest(&record))
        .transpose()
}

pub(crate) fn writeback_mapper_record_artifact_digest(
    record: &crate::writeback::BridgeWritebackMapperRecord,
) -> Result<BridgeIdentityEvidence, crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal>
{
    Ok(compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::WritebackMapperRecord,
        &[
            retained_mapping_bridge_identity_part(record.record_identity()),
            retained_mapping_shape_part(writeback_family_kind_label(record.family_kind())),
            retained_mapping_shape_part(writeback_effect_class_label(record.effect_class())),
            retained_mapping_shape_part(writeback_strategy_class_label(record.strategy_class())),
        ],
    )?)
}

pub(crate) fn writeback_execution_record_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Result<
    Option<BridgeIdentityEvidence>,
    crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal,
> {
    facade
        .retained_writeback_execution_reference_for_identity(
            &BridgeWritebackExecutionRecordIdentity::from_retained_evidence_reference(
                reference_identity,
            ),
        )
        .and_then(
            |reference| match facade.writeback_execution_record_for_reference(&reference) {
                Ok(record) => Some(writeback_execution_artifact_digest(&record)),
                Err(worth_foundational::facade::DiagnosticRecordReadDenial::Absent(
                    worth_foundational::facade::DiagnosticRecordAbsence::NeverRetained
                    | worth_foundational::facade::DiagnosticRecordAbsence::ExpiredUnderCountLimit
                    | worth_foundational::facade::DiagnosticRecordAbsence::NotRetainedAcrossReopen,
                )) => None,
                Err(worth_foundational::facade::DiagnosticRecordReadDenial::ForeignOwner) => {
                    panic!("retained reference belongs to its issuing facade")
                }
            },
        )
        .transpose()
}

pub(crate) fn writeback_execution_artifact_digest(
    record: &crate::writeback::BridgeWritebackExecutionRecord,
) -> Result<BridgeIdentityEvidence, crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal>
{
    let outcome_class = record
        .outcome_class()
        .map(writeback_outcome_class_label)
        .unwrap_or("none");
    let failure_class = record
        .failure_class()
        .map(writeback_failure_class_label)
        .unwrap_or("none");
    Ok(compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::WritebackExecutionRecord,
        &[
            retained_mapping_bridge_identity_part(record.record_identity()),
            retained_mapping_shape_part(writeback_family_kind_label(record.family_kind())),
            retained_mapping_shape_part(writeback_strategy_class_label(record.strategy_class())),
            retained_mapping_shape_part(outcome_class),
            retained_mapping_shape_part(record.request_digest().unwrap_or("none")),
            retained_mapping_shape_part(record.receipt_digest().unwrap_or("none")),
            retained_mapping_shape_part(failure_class),
        ],
    )?)
}

pub(crate) fn writeback_replay_record_digest(
    facade: &BridgeDiagnosticsFacade,
    reference_identity: &BridgeIdentityEvidence,
) -> Result<
    Option<BridgeIdentityEvidence>,
    crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal,
> {
    facade
        .retained_writeback_replay_reference_for_identity(
            &BridgeWritebackReplayRecordIdentity::from_retained_evidence_reference(
                reference_identity,
            ),
        )
        .and_then(
            |reference| match facade.writeback_replay_record_for_reference(&reference) {
                Ok(record) => Some(writeback_replay_artifact_digest(&record)),
                Err(worth_foundational::facade::DiagnosticRecordReadDenial::Absent(
                    worth_foundational::facade::DiagnosticRecordAbsence::NeverRetained
                    | worth_foundational::facade::DiagnosticRecordAbsence::ExpiredUnderCountLimit
                    | worth_foundational::facade::DiagnosticRecordAbsence::NotRetainedAcrossReopen,
                )) => None,
                Err(worth_foundational::facade::DiagnosticRecordReadDenial::ForeignOwner) => {
                    panic!("retained reference belongs to its issuing facade")
                }
            },
        )
        .transpose()
}

pub(crate) fn writeback_replay_artifact_digest(
    record: &crate::writeback::BridgeWritebackReplayRecord,
) -> Result<BridgeIdentityEvidence, crate::diagnostics::causal_envelope::BridgeCausalDigestRefusal>
{
    let failure_class = record
        .failure_class()
        .map(writeback_failure_class_label)
        .unwrap_or("none");
    Ok(compose_retained_causal_mapping_evidence_identity(
        RetainedCausalMappingDigestArtifact::WritebackReplayRecord,
        &[
            retained_mapping_bridge_identity_part(record.record_identity()),
            retained_mapping_shape_part(writeback_family_kind_label(record.family_kind())),
            retained_mapping_shape_part(failure_class),
        ],
    )?)
}

fn writeback_family_kind_label(value: BridgeWritebackFamilyKind) -> &'static str {
    match value {
        BridgeWritebackFamilyKind::ProjectedStateDiff => "projected-state-diff",
        BridgeWritebackFamilyKind::AspectReconciliation => "aspect-reconciliation",
    }
}

fn writeback_effect_class_label(value: BridgeWritebackEffectClass) -> &'static str {
    match value {
        BridgeWritebackEffectClass::ProjectedStateDiff => "projected-state-diff",
        BridgeWritebackEffectClass::AspectReconciliation => "aspect-reconciliation",
    }
}

fn writeback_strategy_class_label(value: BridgeWritebackStrategyClass) -> &'static str {
    match value {
        BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation => {
            "projected-state-diff-reconciliation"
        }
        BridgeWritebackStrategyClass::AspectReconciliationCommit => "aspect-reconciliation-commit",
    }
}

fn writeback_outcome_class_label(value: BridgeWritebackOutcomeClass) -> &'static str {
    match value {
        BridgeWritebackOutcomeClass::CanonicalNoop => "canonical-noop",
        BridgeWritebackOutcomeClass::AuthoritativeCommit => "authoritative-commit",
        BridgeWritebackOutcomeClass::Rejected => "rejected",
    }
}

fn writeback_failure_class_label(value: BridgeWritebackFailureClass) -> &'static str {
    match value {
        BridgeWritebackFailureClass::WritebackNotRequested => "writeback-not-requested",
        BridgeWritebackFailureClass::PolicyRejected => "policy-rejected",
        BridgeWritebackFailureClass::StrategyUnavailable => "strategy-unavailable",
        BridgeWritebackFailureClass::FamilyBindingMismatch => "family-binding-mismatch",
        BridgeWritebackFailureClass::StrategyDescriptorMismatch => "strategy-descriptor-mismatch",
        BridgeWritebackFailureClass::IdempotenceBasisMismatch => "idempotence-basis-mismatch",
        BridgeWritebackFailureClass::CausalityEffectMismatch => "causality-effect-mismatch",
        BridgeWritebackFailureClass::StaleTruthBasis => "stale-truth-basis",
        BridgeWritebackFailureClass::InvariantRejected => "invariant-rejected",
        BridgeWritebackFailureClass::MergeAuthorityRejected => "merge-authority-rejected",
        BridgeWritebackFailureClass::StrategyFailed => "strategy-failed",
        BridgeWritebackFailureClass::StrategyPanicked => "strategy-panicked",
        BridgeWritebackFailureClass::ReplayMismatch => "replay-mismatch",
        BridgeWritebackFailureClass::AuthorityDenied => "authority-denied",
        BridgeWritebackFailureClass::PreviewWritebackRejected => "preview-writeback-rejected",
    }
}

fn diagnostics_tier_label(value: BridgeDiagnosticsTier) -> &'static str {
    match value {
        BridgeDiagnosticsTier::Minimal => "minimal",
        BridgeDiagnosticsTier::Standard => "standard",
        BridgeDiagnosticsTier::Exhaustive => "exhaustive",
    }
}
