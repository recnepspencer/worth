mod merge;

pub(crate) use merge::{
    WorthQueryLowerRuntimeMergeExecution, WorthQueryOrdinaryMergeExecutionError,
    WorthQueryOrdinaryMergeFailureStage,
};

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::session_label::WorthQuerySessionLabel;
use crate::{
    basis_lifecycle::basis_lifecycle,
    effect_lifecycle::{
        admit_effect_intent, evaluate_effect_eligibility, normalize_raw_effect_intent,
        scope_admitted_effect_plan, EffectDiagnosticsMaterialization, EffectDiagnosticsRequest,
        EffectEligibilityOutcome, EffectExecutionReceipt, ExecutedEffectAuthorityArtifact,
        ExecutedEffectPlan, RawEffectIntent,
    },
    workflow::{
        synthetic_runtime_workflow_binding_scoped_for_snapshot_binding_identity,
        WorkflowAuthorityTargetFamily, WorkflowBindingScopeField, WorkflowBudgetClass,
        WorkflowCostClass, WorkflowDeclarationFamily, WorkflowDeclarationRequest,
        WorkflowFreshnessPolicy, WritebackLoweringInput,
    },
};

use super::{
    WorthQueryInspection, WorthQueryOrdinaryAuthorityAdmission, WorthQueryOrdinaryAuthorityDrift,
    WorthQueryOrdinaryAuthorityFamily, WorthQueryPreviewBasisAdmission, WorthQueryPreviewOutcome,
    WorthQueryRuntime, WorthQueryRuntimeError, WorthQueryWriteCommand,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryOrdinaryWritebackFailureStage {
    Handle(worth_query_execution::facade::primary_graph::WorthQueryHandleDenial),
    Authority,
    Basis,
    Intent,
    Eligibility,
    Lowering,
    BridgeExecution,
}

pub(crate) struct WorthQueryOrdinaryWritebackExecutionError {
    stage: WorthQueryOrdinaryWritebackFailureStage,
    message: String,
}

impl WorthQueryOrdinaryWritebackExecutionError {
    fn new(stage: WorthQueryOrdinaryWritebackFailureStage, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
        }
    }

    pub(crate) fn stage(&self) -> WorthQueryOrdinaryWritebackFailureStage {
        self.stage
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

pub(crate) struct WorthQueryLowerRuntimeWritebackExecution {
    admitted_effect_identity: WorthQueryEvidenceIdentity,
    lowered_plan_identity: WorthQueryEvidenceIdentity,
    receipt: EffectExecutionReceipt,
    diagnostics: Option<EffectDiagnosticsMaterialization>,
}

impl WorthQueryLowerRuntimeWritebackExecution {
    pub(crate) fn admitted_effect_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admitted_effect_identity
    }

    pub(crate) fn lowered_plan_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowered_plan_identity
    }

    pub(crate) fn receipt(&self) -> &EffectExecutionReceipt {
        &self.receipt
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        EffectExecutionReceipt,
        Option<EffectDiagnosticsMaterialization>,
    ) {
        (self.receipt, self.diagnostics)
    }
}

pub(crate) struct WorthQueryLowerRuntimePreviewExecution {
    request_identity: WorthQueryEvidenceIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
    aftermath_identity: WorthQueryEvidenceIdentity,
    inspection_identity: Option<WorthQueryEvidenceIdentity>,
    outcome: WorthQueryPreviewOutcome,
}

impl WorthQueryLowerRuntimePreviewExecution {
    pub(crate) fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub(crate) fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub(crate) fn aftermath_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.aftermath_identity
    }

    pub(crate) fn inspection_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.inspection_identity.as_ref()
    }

    pub(crate) fn outcome(&self) -> &WorthQueryPreviewOutcome {
        &self.outcome
    }

    pub(crate) fn into_outcome(self) -> WorthQueryPreviewOutcome {
        self.outcome
    }
}

impl WorthQueryRuntime {
    pub(crate) fn execute_ordinary_writeback(
        &mut self,
        authority: WorthQueryOrdinaryAuthorityAdmission,
        declaration_identity: &WorthQueryEvidenceIdentity,
        materialize_inspection: bool,
    ) -> Result<WorthQueryLowerRuntimeWritebackExecution, WorthQueryOrdinaryWritebackExecutionError>
    {
        if authority.family() != WorthQueryOrdinaryAuthorityFamily::Writeback {
            return Err(WorthQueryOrdinaryWritebackExecutionError::new(
                WorthQueryOrdinaryWritebackFailureStage::Authority,
                "ordinary writeback requires a writeback authority context",
            ));
        }
        let drift = self.ordinary_authority_drift(&authority);
        if let WorthQueryOrdinaryAuthorityDrift::Handle(denial) = drift {
            return Err(WorthQueryOrdinaryWritebackExecutionError::new(
                WorthQueryOrdinaryWritebackFailureStage::Handle(denial),
                denial.to_string(),
            ));
        }
        if drift != WorthQueryOrdinaryAuthorityDrift::Current {
            return Err(WorthQueryOrdinaryWritebackExecutionError::new(
                WorthQueryOrdinaryWritebackFailureStage::Authority,
                "ordinary writeback authority is no longer current",
            ));
        }

        let scope = WorkflowBindingScopeField::Identity(declaration_identity);
        let binding = synthetic_runtime_workflow_binding_scoped_for_snapshot_binding_identity(
            "ordinary-declarative-writeback",
            &scope,
            authority.snapshot_identity().clone(),
        );
        let basis = basis_lifecycle()
            .branch_head("main", true)
            .prepare_mutation()
            .map_err(|error| {
                WorthQueryOrdinaryWritebackExecutionError::new(
                    WorthQueryOrdinaryWritebackFailureStage::Basis,
                    format!("{error:?}"),
                )
            })?;
        let request = WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::WritebackLoweringNarrow,
            WorkflowAuthorityTargetFamily::BridgeWriteback,
            WorkflowCostClass::WritebackLoweringNarrow,
            WorkflowBudgetClass::AuthorityTargetBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        );
        let normalized = normalize_raw_effect_intent(
            &basis.into(),
            RawEffectIntent::Writeback {
                binding,
                request,
                input: WritebackLoweringInput::projected_state_diff(),
            },
        )
        .map_err(|denial| {
            WorthQueryOrdinaryWritebackExecutionError::new(
                WorthQueryOrdinaryWritebackFailureStage::Intent,
                denial.message(),
            )
        })?;
        let eligibility = match evaluate_effect_eligibility(normalized) {
            EffectEligibilityOutcome::Admitted(eligibility) => eligibility,
            EffectEligibilityOutcome::Advisory(outcome) => {
                return Err(WorthQueryOrdinaryWritebackExecutionError::new(
                    WorthQueryOrdinaryWritebackFailureStage::Eligibility,
                    outcome.decision_trace().message(),
                ));
            }
            EffectEligibilityOutcome::Denied(outcome) => {
                return Err(WorthQueryOrdinaryWritebackExecutionError::new(
                    WorthQueryOrdinaryWritebackFailureStage::Eligibility,
                    outcome.decision_trace().message(),
                ));
            }
            EffectEligibilityOutcome::RebindRequired(outcome) => {
                return Err(WorthQueryOrdinaryWritebackExecutionError::new(
                    WorthQueryOrdinaryWritebackFailureStage::Eligibility,
                    outcome.decision_trace().message(),
                ));
            }
            EffectEligibilityOutcome::Deferred(outcome) => {
                return Err(WorthQueryOrdinaryWritebackExecutionError::new(
                    WorthQueryOrdinaryWritebackFailureStage::Eligibility,
                    outcome.decision_trace().message(),
                ));
            }
        };
        let admitted = admit_effect_intent(eligibility);
        let admitted_effect_identity = admitted.admitted_identity().clone();
        let lowered = scope_admitted_effect_plan(admitted)
            .lower()
            .map_err(|denial| {
                WorthQueryOrdinaryWritebackExecutionError::new(
                    WorthQueryOrdinaryWritebackFailureStage::Lowering,
                    denial.message(),
                )
            })?;
        let lowered_plan_identity = lowered.lowered_effect_execution_plan_identity().clone();
        let declaration = lowered.as_writeback().cloned().ok_or_else(|| {
            WorthQueryOrdinaryWritebackExecutionError::new(
                WorthQueryOrdinaryWritebackFailureStage::Lowering,
                "writeback effect lowering produced a non-writeback artifact",
            )
        })?;
        let execution =
            self.backend
                .execute_query_writeback(&declaration)
                .map_err(|(kind, message)| {
                    WorthQueryOrdinaryWritebackExecutionError::new(
                        WorthQueryOrdinaryWritebackFailureStage::BridgeExecution,
                        format!("{}: {message}", kind.as_str()),
                    )
                })?;
        let receipt = ExecutedEffectPlan::new(
            lowered,
            ExecutedEffectAuthorityArtifact::Writeback { execution },
            1,
        )
        .receipt();
        let diagnostics = materialize_inspection
            .then(|| receipt.materialize_diagnostics(EffectDiagnosticsRequest::forensic()));
        Ok(WorthQueryLowerRuntimeWritebackExecution {
            admitted_effect_identity,
            lowered_plan_identity,
            receipt,
            diagnostics,
        })
    }

    pub(crate) fn execute_ordinary_read_only_preview(
        &mut self,
        basis_admission: WorthQueryPreviewBasisAdmission,
        declaration_identity: &WorthQueryEvidenceIdentity,
        materialize_inspection: bool,
    ) -> Result<WorthQueryLowerRuntimePreviewExecution, WorthQueryRuntimeError> {
        let label = basis_admission.session_label().clone();
        let request_identity = preview_request_identity("read-only", &label, declaration_identity);
        let outcome = {
            let session = self.open_preview_with_admitted_basis(basis_admission)?;
            session.discard()
        };
        let receipt_identity = outcome.closeout_evidence().closeout_identity().clone();
        self.finish_ordinary_preview_execution(
            request_identity,
            receipt_identity,
            outcome,
            materialize_inspection,
        )
    }

    pub(crate) fn execute_ordinary_preview_promotion(
        &mut self,
        basis_admission: WorthQueryPreviewBasisAdmission,
        declaration_identity: &WorthQueryEvidenceIdentity,
        command: WorthQueryWriteCommand,
        materialize_inspection: bool,
    ) -> Result<WorthQueryLowerRuntimePreviewExecution, WorthQueryRuntimeError> {
        let label = basis_admission.session_label().clone();
        let request_identity = preview_request_identity("promotion", &label, declaration_identity);
        let (receipt_identity, outcome) = {
            let mut session = self.open_preview_with_admitted_basis(basis_admission)?;
            let preview_receipt = session.write(command)?;
            let receipt_identity = preview_receipt.commit_evidence_identity().clone();
            (receipt_identity, session.promote()?)
        };
        self.finish_ordinary_preview_execution(
            request_identity,
            receipt_identity,
            outcome,
            materialize_inspection,
        )
    }

    fn finish_ordinary_preview_execution(
        &self,
        request_identity: WorthQueryEvidenceIdentity,
        receipt_identity: WorthQueryEvidenceIdentity,
        outcome: WorthQueryPreviewOutcome,
        materialize_inspection: bool,
    ) -> Result<WorthQueryLowerRuntimePreviewExecution, WorthQueryRuntimeError> {
        let inspection_identity = if materialize_inspection {
            let inspection = match self.inspect(&outcome)? {
                WorthQueryInspection::PreviewOutcome(inspection) => inspection,
                other => panic!("expected preview outcome inspection, got {other:?}"),
            };
            Some(inspection.inspection_identity().clone())
        } else {
            None
        };
        Ok(WorthQueryLowerRuntimePreviewExecution {
            request_identity,
            receipt_identity,
            aftermath_identity: outcome.closeout_evidence().closeout_identity().clone(),
            inspection_identity,
            outcome,
        })
    }
}

fn preview_request_identity(
    family: &'static str,
    label: &WorthQuerySessionLabel,
    declaration_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "ordinary-preview-request",
        )
        .field_shape(WorthQueryEvidenceTag::new("family"), family)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("session_label"),
            label.identity_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("declaration"),
            declaration_identity,
        )
        .seal()
}
