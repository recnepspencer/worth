use super::*;

impl WorthQueryRuntime {
    pub(crate) fn resolve_live_artifact_target(
        &self,
        view_name: &str,
    ) -> Result<WorthQueryLiveArtifactTarget, WorthQueryRuntimeError> {
        if self.managed_live_resource_is_abandoned(view_name) {
            return Err(WorthQueryRuntimeError::MissingLiveSubscription(
                view_name.to_string(),
            ));
        }
        let target = WorthQueryLiveArtifactTarget::from_view_name(view_name);
        let state = self.live_subscriptions.get(&target).ok_or_else(|| {
            WorthQueryRuntimeError::MissingLiveSubscription(view_name.to_string())
        })?;
        Ok(WorthQueryLiveArtifactTarget::from_subscription_installation(&state.installation))
    }

    pub(crate) fn inspect_live_view_name_installation(
        &self,
        view_name: &str,
    ) -> Result<&WorthQueryRuntimeLiveSubscriptionInstallation, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Inspect)?;
        if self.managed_live_resource_is_abandoned(view_name) {
            return Err(WorthQueryRuntimeError::MissingLiveSubscription(
                view_name.to_string(),
            ));
        }
        let target = WorthQueryLiveArtifactTarget::from_view_name(view_name);
        self.live_subscriptions
            .get(&target)
            .map(|state| &state.installation)
            .ok_or_else(|| WorthQueryRuntimeError::MissingLiveSubscription(view_name.to_string()))
    }

    pub fn inspect_live_view<T>(
        &self,
        view: &WorthQueryLiveView<T>,
    ) -> Result<&WorthQueryRuntimeLiveSubscriptionInstallation, WorthQueryRuntimeError> {
        self.inspect_live_view_name_installation(view.name())
    }

    pub fn inspect_live_view_explanation<T>(
        &self,
        view: &WorthQueryLiveView<T>,
    ) -> Result<WorthQueryLiveViewInspection, WorthQueryRuntimeError> {
        let target = WorthQueryLiveArtifactTarget::from_subscription_installation(
            view.subscription_installation(),
        );
        let state = self.live_subscriptions.get(&target).ok_or_else(|| {
            WorthQueryRuntimeError::MissingLiveSubscription(view.name().to_string())
        })?;
        Ok(WorthQueryLiveViewInspection::from_state(state))
    }

    pub fn inspect_receipt<'a>(
        &'a self,
        receipt: &'a WorthQueryWriteReceipt,
    ) -> WorthQueryArtifactInspector<'a> {
        self.try_inspect_receipt(receipt)
            .expect("inspect support must be admitted before inspecting receipts")
    }

    pub fn try_inspect_receipt<'a>(
        &'a self,
        receipt: &'a WorthQueryWriteReceipt,
    ) -> Result<WorthQueryArtifactInspector<'a>, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Inspect)?;
        let runtime_evidence = self
            .backend
            .inspect_write_receipt(receipt, &self.evidence_authority)?;
        Ok(WorthQueryArtifactInspector {
            receipt,
            runtime_evidence,
        })
    }

    pub fn inspect_intent_receipt(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &WorthQueryIntentReceipt,
    ) -> Result<WorthQueryIntentReceiptInspection, WorthQueryRuntimeError> {
        match self.inspect(execution, receipt)? {
            WorthQueryInspection::IntentReceipt(inspection) => Ok(inspection),
            other => panic!("expected intent receipt inspection, got {other:?}"),
        }
    }

    pub fn inspect_effect_intent_receipt(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &WorthQueryEffectIntentReceipt,
    ) -> Result<WorthQueryEffectIntentReceiptInspection, WorthQueryRuntimeError> {
        match self.inspect(execution, receipt)? {
            WorthQueryInspection::EffectIntentReceipt(inspection) => Ok(inspection),
            other => panic!("expected effect intent receipt inspection, got {other:?}"),
        }
    }

    pub fn inspect_intent_denial(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        evidence: &WorthQueryIntentDenialEvidence,
    ) -> Result<WorthQueryIntentDenialInspection, WorthQueryRuntimeError> {
        match self.inspect(execution, evidence)? {
            WorthQueryInspection::IntentDenial(inspection) => Ok(inspection),
            other => panic!("expected intent denial inspection, got {other:?}"),
        }
    }

    pub fn inspect_preview_binding(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &WorthQueryPreviewHandleBindingEvidence,
    ) -> Result<WorthQueryPreviewBindingInspection, WorthQueryRuntimeError> {
        match self.inspect(execution, binding)? {
            WorthQueryInspection::PreviewBinding(inspection) => Ok(inspection),
            other => panic!("expected preview binding inspection, got {other:?}"),
        }
    }

    pub fn inspect_preview_outcome(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        outcome: &WorthQueryPreviewOutcome,
    ) -> Result<WorthQueryPreviewOutcomeInspection, WorthQueryRuntimeError> {
        match self.inspect(execution, outcome)? {
            WorthQueryInspection::PreviewOutcome(inspection) => Ok(inspection),
            other => panic!("expected preview outcome inspection, got {other:?}"),
        }
    }

    pub fn inspect_preview_intent_receipt(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &WorthQueryPreviewIntentReceipt,
    ) -> Result<WorthQueryPreviewIntentReceiptInspection, WorthQueryRuntimeError> {
        match self.inspect(execution, receipt)? {
            WorthQueryInspection::PreviewIntentReceipt(inspection) => Ok(inspection),
            other => panic!("expected preview intent receipt inspection, got {other:?}"),
        }
    }

    pub fn inspect_branch_intent_receipt(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &WorthQueryBranchIntentReceipt,
    ) -> Result<WorthQueryBranchIntentReceiptInspection, WorthQueryRuntimeError> {
        match self.inspect(execution, receipt)? {
            WorthQueryInspection::BranchIntentReceipt(inspection) => Ok(inspection),
            other => panic!("expected branch intent receipt inspection, got {other:?}"),
        }
    }

    pub fn inspect_feedback_path<T>(
        &self,
        effect: &WorthQueryEffectHandle<T>,
    ) -> Result<WorthQueryFeedbackPhaseGraphInspection, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Inspect)?;
        let effect_target = WorthQueryEffectTarget::from_name(effect.name());
        let runtime = self
            .effects
            .get(&effect_target)
            .ok_or_else(|| WorthQueryRuntimeError::MissingEffect(effect.name().to_string()))?;
        WorthQueryFeedbackPhaseGraphInspection::from_effect_runtime(runtime).ok_or_else(|| {
            WorthQueryRuntimeError::MissingEffect(format!(
                "{} has no retained feedback delivery",
                effect.name()
            ))
        })
    }

    pub fn inspect_effect_feedback_receipt(
        &self,
        receipt: &WorthQueryEffectIntentReceipt,
    ) -> Result<WorthQueryFeedbackPhaseGraphInspection, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Inspect)?;
        Ok(WorthQueryFeedbackPhaseGraphInspection::from_effect_intent_receipt(receipt))
    }

    pub fn inspect<'a, T>(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: T,
    ) -> Result<WorthQueryInspection, WorthQueryRuntimeError>
    where
        T: Into<WorthQueryInspectionTarget<'a>>,
    {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Inspect)?;
        match target.into() {
            WorthQueryInspectionTarget::AdmittedWorldBasis(basis) => {
                Ok(WorthQueryInspection::BasisLifecycle(
                    WorthQueryBasisLifecycleInspection::from_admitted_world_basis(basis),
                ))
            }
            WorthQueryInspectionTarget::ScopedObservationBasis(basis) => {
                Ok(WorthQueryInspection::BasisLifecycle(basis.into()))
            }
            WorthQueryInspectionTarget::ScopedMutationPreparationBasis(basis) => {
                Ok(WorthQueryInspection::BasisLifecycle(basis.into()))
            }
            WorthQueryInspectionTarget::ScopedInspectionBasis(basis) => {
                Ok(WorthQueryInspection::BasisLifecycle(basis.into()))
            }
            WorthQueryInspectionTarget::ScopedReplayBasis(basis) => {
                Ok(WorthQueryInspection::BasisLifecycle(basis.into()))
            }
            WorthQueryInspectionTarget::ScopedMaterializationBasis(basis) => {
                Ok(WorthQueryInspection::BasisLifecycle(basis.into()))
            }
            WorthQueryInspectionTarget::ScopedSubscriptionDeclarationBasis(basis) => {
                Ok(WorthQueryInspection::BasisLifecycle(basis.into()))
            }
            WorthQueryInspectionTarget::ScopedSubscriptionActivationBasis(basis) => {
                Ok(WorthQueryInspection::BasisLifecycle(basis.into()))
            }
            WorthQueryInspectionTarget::ScopedPreviewCloseoutBasis(basis) => {
                Ok(WorthQueryInspection::BasisLifecycle(basis.into()))
            }
            WorthQueryInspectionTarget::DerivedView { name } => {
                let review = self.review_runtime_derived_inspection(name.to_string())?;
                let handoff = self.resolve_reviewed_admitted_derived_inspection_handoff(review)?;
                let binding = self.prepare_derived_inspection_execution_binding(handoff);
                let result =
                    self.execute_derived_inspection_execution_binding(execution, binding)?;
                Ok(WorthQueryInspection::DerivedView(result.evidence().clone()))
            }
            target => {
                let seed = crate::intent_admission::WorthQueryGenericInspectionIntentSeed::from_target(target)
                    .expect("derived inspection targets should route through the derived inspection lane");
                let review = self.review_unified_inspection(seed)?;
                let handoff = self.resolve_reviewed_admitted_unified_inspection_handoff(review)?;
                let binding = self.prepare_unified_inspection_execution_binding(handoff);
                let result =
                    self.execute_unified_inspection_execution_binding(execution, binding)?;
                Ok(result.inspection().clone())
            }
        }
    }
}
