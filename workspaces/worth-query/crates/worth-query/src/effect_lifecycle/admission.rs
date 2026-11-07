use crate::basis_lifecycle::BasisFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::workflow::{admit_query_workflow_declaration, WorkflowAdmissionFailureClass};

use super::eligibility::{
    AdmittedEffectIntent, AdvisoryEffectEligibility, DeferredEffectEligibility,
    DeniedEffectEligibility, EffectEligibility, EffectEligibilityOutcome,
    RebindRequiredEffectEligibility,
};
use super::normalized::NormalizedEffectIntent;
use super::support_contract::EffectDeferredSupportContract;
use super::support_matrix::{support_decision_for, EffectSupportCause, EffectSupportPosture};
use super::taxonomy::DeniedEffectEligibilityKind;

pub fn evaluate_effect_eligibility(
    normalized: NormalizedEffectIntent,
) -> Result<EffectEligibilityOutcome, IdentityDerivationDenial> {
    let support = support_decision_for(normalized.basis_family(), normalized.family())?;
    Ok(match support.posture() {
        EffectSupportPosture::Admitted => {
            admit_supported_effect(normalized, support.rows_consulted())?
        }
        EffectSupportPosture::Advisory => {
            EffectEligibilityOutcome::Advisory(AdvisoryEffectEligibility::new(
                normalized,
                support.cause(),
                "effect authoring path is advisory only and cannot be lowered without authoritative rebind",
                support.rows_consulted(),
            )?)
        }
        EffectSupportPosture::RebindRequired => {
            let message = rebind_required_message(&normalized);
            EffectEligibilityOutcome::RebindRequired(RebindRequiredEffectEligibility::new(
                &normalized,
                message,
                support.cause().as_str(),
                support.rows_consulted(),
            )?)
        }
        EffectSupportPosture::Deferred => {
            let (contract, message) = deferred_effect(support.cause());
            EffectEligibilityOutcome::Deferred(DeferredEffectEligibility::new(
                contract,
                &normalized,
                message,
                support.cause().as_str(),
                support.rows_consulted(),
            )?)
        }
        EffectSupportPosture::Denied | EffectSupportPosture::Unsupported => {
            EffectEligibilityOutcome::Denied(denied_effect(
                normalized,
                support.cause(),
                support.rows_consulted(),
            )?)
        }
    })
}

fn rebind_required_message(normalized: &NormalizedEffectIntent) -> &'static str {
    match normalized.family() {
        super::EffectFamily::Mutation => {
            "preview-backed mutation must rebind to an authoritative basis before lowering"
        }
        super::EffectFamily::Writeback => {
            "preview-backed writeback must rebind to an authoritative basis before lowering"
        }
        super::EffectFamily::Merge => {
            "preview-backed merge must rebind to an authoritative basis before lowering"
        }
    }
}

fn admit_supported_effect(
    normalized: NormalizedEffectIntent,
    support_row_count: usize,
) -> Result<EffectEligibilityOutcome, IdentityDerivationDenial> {
    Ok(
        match admit_query_workflow_declaration(
            normalized.workflow_binding(),
            normalized.workflow_request().clone(),
        ) {
            Ok(workflow_declaration) => EffectEligibilityOutcome::Admitted(EffectEligibility::new(
                normalized,
                workflow_declaration,
                support_row_count,
            )?),
            Err(error) => match error.failure_class() {
                WorkflowAdmissionFailureClass::ExplicitRebindRequired => {
                    EffectEligibilityOutcome::RebindRequired(RebindRequiredEffectEligibility::new(
                        &normalized,
                        error.message(),
                        DeniedEffectEligibilityKind::PreviewRebindRequired.as_str(),
                        support_row_count,
                    )?)
                }
                WorkflowAdmissionFailureClass::PreviewReadOnlyAuthorityRequestForbidden => {
                    EffectEligibilityOutcome::Denied(DeniedEffectEligibility::new(
                        DeniedEffectEligibilityKind::PreviewReadOnlyExecutionForbidden,
                        &normalized,
                        error.message(),
                        DeniedEffectEligibilityKind::PreviewReadOnlyExecutionForbidden.as_str(),
                        support_row_count,
                    )?)
                }
                WorkflowAdmissionFailureClass::UnsupportedAuthorityTargetFamily
                | WorkflowAdmissionFailureClass::InvalidBasisPairing
                | WorkflowAdmissionFailureClass::UnsupportedBasisFamily => {
                    EffectEligibilityOutcome::Denied(DeniedEffectEligibility::new(
                        DeniedEffectEligibilityKind::AuthorityTargetMismatch,
                        &normalized,
                        error.message(),
                        DeniedEffectEligibilityKind::AuthorityTargetMismatch.as_str(),
                        support_row_count,
                    )?)
                }
                WorkflowAdmissionFailureClass::ForbiddenWorkflowBroadening => {
                    EffectEligibilityOutcome::Denied(DeniedEffectEligibility::new(
                        DeniedEffectEligibilityKind::WorkflowBroadeningForbidden,
                        &normalized,
                        error.message(),
                        DeniedEffectEligibilityKind::WorkflowBroadeningForbidden.as_str(),
                        support_row_count,
                    )?)
                }
                _ => EffectEligibilityOutcome::Denied(DeniedEffectEligibility::new(
                    DeniedEffectEligibilityKind::WorkflowAdmissionDenied,
                    &normalized,
                    error.message(),
                    DeniedEffectEligibilityKind::WorkflowAdmissionDenied.as_str(),
                    support_row_count,
                )?),
            },
        },
    )
}

fn denied_effect(
    normalized: NormalizedEffectIntent,
    cause: EffectSupportCause,
    support_row_count: usize,
) -> Result<DeniedEffectEligibility, IdentityDerivationDenial> {
    let (kind, message) = match cause {
        EffectSupportCause::BranchAuthorityRequired => (
            DeniedEffectEligibilityKind::BranchAuthorityRequired,
            "merge effects require branch-authoritative basis rather than tenant or policy scoped mutation preparation",
        ),
        _ => (
            DeniedEffectEligibilityKind::UnsupportedForBasisFamily,
            unsupported_effect_message(&normalized),
        ),
    };
    Ok(DeniedEffectEligibility::new(
        kind,
        &normalized,
        message,
        cause.as_str(),
        support_row_count,
    )?)
}

fn deferred_effect(cause: EffectSupportCause) -> (EffectDeferredSupportContract, &'static str) {
    match cause {
        EffectSupportCause::StoreBackedExecutionDeferred => (
            super::support_contract::deferred_support_contract(cause)
                .expect("store-backed deferred cause should have exact contract"),
            "store-backed effect execution parity remains deferred and leaves zero operational residue",
        ),
        EffectSupportCause::DurableReplayDeferred => (
            super::support_contract::deferred_support_contract(cause)
                .expect("durable deferred cause should have exact contract"),
            "durable replay and restart-stable effect continuation remain deferred and leave zero operational residue",
        ),
        _ => unreachable!("deferred effect posture requires an exact deferred support contract"),
    }
}

fn unsupported_effect_message(normalized: &NormalizedEffectIntent) -> &'static str {
    match (normalized.basis_family(), normalized.family()) {
        (BasisFamily::TenantScoped | BasisFamily::PolicyScoped, super::EffectFamily::Merge) => {
            "merge effects require branch-authoritative basis rather than tenant or policy scoped mutation preparation"
        }
        (BasisFamily::Preview | BasisFamily::PreviewDerived, super::EffectFamily::Merge) => {
            "preview-scoped effect authoring does not support merge execution without authoritative rebind"
        }
        _ => "effect family is not supported on this basis family",
    }
}

pub fn admit_effect_intent(eligibility: EffectEligibility) -> AdmittedEffectIntent {
    AdmittedEffectIntent::new(eligibility)
}
