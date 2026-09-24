use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedPresentationAttemptIdentity,
    UiSurfaceBindingGeneration,
};

use super::outcome::{
    UiMountedPresentationOutcome, UiMountedPresentationReceipt, UiMountedRejectedFrame,
    UiMountedSurfacePresentationReceipt, UiMountedSurfacePresentationRejection,
};

pub(super) struct UiIndeterminatePresentationEvidence(
    Box<UiIndeterminatePresentationEvidenceInner>,
);

struct UiIndeterminatePresentationEvidenceInner {
    affected: Vec<UiSurfaceBindingGeneration>,
    completed: Vec<UiMountedSurfacePresentationReceipt>,
    semantic_receipts: Vec<worth_ui_query_binding::WorthUiPresentationRecoveryReceipt>,
    recovery_required: Vec<worth_ui_query_binding::WorthUiPresentationRecoveryRequiredReceipt>,
    additional_adapter_cost: Option<worth_ui_host_contract::UiHostPresentationCostReport>,
    physical_recovery_bindings: Vec<UiSurfaceBindingGeneration>,
}

impl UiIndeterminatePresentationEvidence {
    pub(super) fn new(
        affected: Vec<UiSurfaceBindingGeneration>,
        completed: Vec<UiMountedSurfacePresentationReceipt>,
    ) -> Self {
        Self(Box::new(UiIndeterminatePresentationEvidenceInner {
            affected,
            completed,
            semantic_receipts: Vec::new(),
            recovery_required: Vec::new(),
            additional_adapter_cost: None,
            physical_recovery_bindings: Vec::new(),
        }))
    }

    pub(super) fn with_recovery_required(
        mut self,
        recovery_required: Vec<worth_ui_query_binding::WorthUiPresentationRecoveryRequiredReceipt>,
    ) -> Self {
        self.0.recovery_required = recovery_required;
        self
    }

    pub(super) fn with_physical_recovery_bindings(
        mut self,
        bindings: Vec<UiSurfaceBindingGeneration>,
    ) -> Self {
        self.0.physical_recovery_bindings.extend(bindings);
        self
    }

    pub(super) fn with_semantic_receipts(
        mut self,
        semantic_receipts: Vec<worth_ui_query_binding::WorthUiPresentationRecoveryReceipt>,
    ) -> Self {
        self.0.semantic_receipts = semantic_receipts;
        self
    }

    pub(super) fn with_additional_adapter_cost(
        mut self,
        cost: worth_ui_host_contract::UiHostPresentationCostReport,
    ) -> Self {
        self.0.additional_adapter_cost = Some(cost);
        self
    }

    pub(super) fn into_terminal_parts(
        self,
        mounting_cost: super::super::UiMountCostReport,
    ) -> (
        Vec<UiSurfaceBindingGeneration>,
        super::super::UiMountCostReport,
        Vec<worth_ui_query_binding::WorthUiPresentationRecoveryReceipt>,
        Vec<worth_ui_query_binding::WorthUiPresentationRecoveryRequiredReceipt>,
        Vec<UiSurfaceBindingGeneration>,
    ) {
        let inner = *self.0;
        let composed = UiMountedPresentationReceipt::compose_cost(mounting_cost, &inner.completed)
            .and_then(|cost| match inner.additional_adapter_cost {
                Some(additional) => cost.with_adapter(additional),
                None => Ok(cost),
            });
        let cost = composed
            .map(|cost| {
                cost.reclassified(super::super::UiMountWorkClass::IndeterminatePresentation)
            })
            .unwrap_or_else(|_| {
                mounting_cost
                    .reclassified(super::super::UiMountWorkClass::IndeterminatePresentation)
                    .with_cost_overflow()
                    .expect("one cost overflow marker fits accounting")
            });
        (
            inner.affected,
            cost,
            inner.semantic_receipts,
            inner.recovery_required,
            inner.physical_recovery_bindings,
        )
    }
}

pub(super) fn rejected_outcome(
    attempt: UiMountedPresentationAttemptIdentity,
    frame: super::super::UiPreparedMountedFrame,
    retention: super::super::retention::UiMountedRetentionReservation,
    rejections: Vec<UiMountedSurfacePresentationRejection>,
) -> UiMountedPresentationOutcome {
    drop(retention);
    UiMountedPresentationOutcome::RejectedBeforeEffects(UiMountedRejectedFrame::new(
        attempt, frame, rejections,
    ))
}

pub(super) fn aggregate_affected(
    completed: &[UiMountedSurfacePresentationReceipt],
    pending: &[super::state::UiPendingMountedSurface],
    rejected: &[UiMountedSurfacePresentationRejection],
) -> Vec<UiSurfaceBindingGeneration> {
    completed
        .iter()
        .map(UiMountedSurfacePresentationReceipt::binding)
        .chain(pending.iter().map(|surface| surface.binding))
        .chain(rejected.iter().map(|rejection| rejection.binding()))
        .collect()
}

pub(super) fn frame_rejections(
    frame: &super::super::UiPreparedMountedFrame,
    denial: UiHostSurfacePresentationDenial,
) -> Vec<UiMountedSurfacePresentationRejection> {
    frame
        .surfaces()
        .iter()
        .map(|surface| {
            UiMountedSurfacePresentationRejection::new(surface.requirement().binding(), denial)
        })
        .collect()
}

pub(super) fn completion_effects_satisfy(
    expected: &[worth_ui_host_contract::UiMountedEffectFamily],
    observed: &[worth_ui_host_contract::UiMountedEffectFamily],
    cost: worth_ui_host_contract::UiHostPresentationCostReport,
) -> bool {
    if observed == expected {
        return true;
    }
    let expected_without_paint = expected
        .iter()
        .copied()
        .filter(|effect| *effect != worth_ui_host_contract::UiMountedEffectFamily::NativePaint)
        .collect::<Vec<_>>();
    observed == expected_without_paint
        && expected.len() == observed.len() + 1
        && cost.presented_surfaces() == 0
        && cost.presented_pixels() == 0
        && cost.gpu_writes() == 0
        && cost.render_passes() == 0
        && cost.surface_copies() == 0
        && cost.surface_acquisitions() == 0
        && cost.queue_submissions() == 0
        && cost.presents() == 0
}

#[cfg(test)]
mod tests {
    use super::completion_effects_satisfy;
    use worth_ui_host_contract::{
        UiHostPresentationCostInput, UiHostPresentationCostReport, UiMountedEffectFamily,
    };

    #[test]
    fn offscreen_delta_may_advance_without_counterfeit_native_paint() {
        let expected = [UiMountedEffectFamily::NativePaint];
        assert!(completion_effects_satisfy(
            &expected,
            &[],
            UiHostPresentationCostReport::default(),
        ));
        let copied = UiHostPresentationCostReport::from_adapter(UiHostPresentationCostInput {
            surface_copies: 1,
            ..Default::default()
        });
        assert!(!completion_effects_satisfy(&expected, &[], copied));
        assert!(!completion_effects_satisfy(
            &[UiMountedEffectFamily::IdentityOverlay],
            &[],
            UiHostPresentationCostReport::default(),
        ));
    }
}
