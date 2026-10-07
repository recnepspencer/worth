use super::*;
use crate::runtime::{WorthQueryDerivedMaterializationTarget, WorthQueryLiveArtifactTarget};

impl<'a> WorthQueryPreviewSession<'a> {
    pub(super) fn admit_preview_write_intent(&self) -> Result<(), WorthQueryRuntimeError> {
        self.admit_effect_action(
            WorthQueryEffectAction::WriteIntent,
            WorthQueryAuthorityLane::PreviewTruth,
        )
        .map(|_| ())
    }

    pub(super) fn stage_command(
        &mut self,
        command: WorthQueryWriteCommand,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let receipt = WorthQueryWriteReceipt::preview(
            &self.label,
            self.pending_commands.len() + 1,
            &command,
            self.runtime.current_snapshot_identity()?,
        );
        self.pending_commands.push(command);
        self.route_preview_execution(&receipt);
        Ok(receipt)
    }

    pub fn preview_execution_evidence(&self) -> &[WorthQueryPreviewExecutionEvidence] {
        &self.execution_evidence
    }

    pub fn preview_intent_receipts(&self) -> &[WorthQueryPreviewIntentReceipt] {
        &self.intent_receipts
    }

    pub(super) fn route_preview_execution(&mut self, receipt: &WorthQueryWriteReceipt) {
        let mut live_affected: BTreeMap<WorthQueryLiveArtifactTarget, Vec<WorthQueryAspectTouch>> =
            BTreeMap::new();
        let mut computed_affected: BTreeMap<
            WorthQueryDerivedMaterializationTarget,
            Vec<WorthQueryAspectTouch>,
        > = BTreeMap::new();

        for binding in self
            .handle_bindings
            .iter()
            .filter(|binding| binding.family == WorthQueryPreviewHandleBindingFamily::LiveView)
        {
            let target = WorthQueryLiveArtifactTarget::from_view_name(binding.handle_name());
            let Some(state) = self.runtime.live_subscriptions.get(&target) else {
                continue;
            };
            let affected_aspects = relevant_live_aspects(&state.request, receipt.deltas());
            if affected_aspects.is_empty() {
                continue;
            }
            live_affected.insert(target, affected_aspects.clone());
            self.execution_evidence
                .push(WorthQueryPreviewExecutionEvidence::for_aspect_touches(
                    &self.basis_admission,
                    WorthQueryPreviewExecutionKind::LivePatch,
                    binding.handle_name(),
                    binding.source_lane(),
                    WorthQueryAuthorityLane::PreviewTruth,
                    receipt.commit_evidence_identity(),
                    affected_aspects,
                ));
        }

        for binding in self
            .handle_bindings
            .iter()
            .filter(|binding| binding.family == WorthQueryPreviewHandleBindingFamily::ComputedView)
        {
            let target = WorthQueryDerivedMaterializationTarget::new(binding.handle_name());
            let Some(runtime) = self.runtime.derived_views.get(&target) else {
                continue;
            };
            let affected_aspects =
                relevant_computed_aspects(runtime, &live_affected, &computed_affected);
            if affected_aspects.is_empty() {
                continue;
            }
            computed_affected.insert(
                WorthQueryDerivedMaterializationTarget::new(binding.handle_name().to_string()),
                affected_aspects.clone(),
            );
            self.execution_evidence
                .push(WorthQueryPreviewExecutionEvidence::for_aspect_touches(
                    &self.basis_admission,
                    WorthQueryPreviewExecutionKind::ComputedPatch,
                    binding.handle_name(),
                    binding.source_lane(),
                    WorthQueryAuthorityLane::PreviewTruth,
                    receipt.commit_evidence_identity(),
                    affected_aspects,
                ));
        }

        let mut pending_effect_evidence = Vec::new();
        for binding in self
            .handle_bindings
            .iter()
            .filter(|binding| binding.family == WorthQueryPreviewHandleBindingFamily::Effect)
        {
            let Some(disposition) = binding.effect_disposition() else {
                continue;
            };
            let Ok(inspected) = self.runtime.inspect_effect_by_name(binding.handle_name()) else {
                continue;
            };
            let affected_aspects =
                relevant_effect_aspects(&inspected, &live_affected, &computed_affected);
            if affected_aspects.is_empty() {
                continue;
            }
            let kind = match disposition {
                WorthQueryPreviewEffectBindingDisposition::RedirectedDelivery
                | WorthQueryPreviewEffectBindingDisposition::AuthoritativeAllowed => {
                    WorthQueryPreviewExecutionKind::EffectDelivery
                }
                WorthQueryPreviewEffectBindingDisposition::SandboxedWriteIntent => {
                    WorthQueryPreviewExecutionKind::PendingWriteIntent
                }
                WorthQueryPreviewEffectBindingDisposition::Muted
                | WorthQueryPreviewEffectBindingDisposition::MutedByDeriveOnly => {
                    WorthQueryPreviewExecutionKind::MutedEffect
                }
            };
            pending_effect_evidence.push(WorthQueryPreviewExecutionEvidence::for_aspect_touches(
                &self.basis_admission,
                kind,
                binding.handle_name(),
                binding.source_lane(),
                WorthQueryAuthorityLane::PreviewTruth,
                receipt.commit_evidence_identity(),
                affected_aspects,
            ));
        }
        self.execution_evidence.extend(pending_effect_evidence);
    }
}
