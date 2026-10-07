use super::*;
use crate::runtime::WorthQueryLiveArtifactTarget;

impl<'a> WorthQueryPreviewSession<'a> {
    pub fn label(&self) -> &str {
        self.label.display()
    }

    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.label
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn basis_admission(&self) -> &WorthQueryPreviewBasisAdmission {
        &self.basis_admission
    }

    pub fn admit_effect_action(
        &self,
        action: WorthQueryEffectAction,
        target_lane: WorthQueryAuthorityLane,
    ) -> Result<WorthQueryEffectAdmission, WorthQueryRuntimeError> {
        self.effect_policy
            .admit(action, target_lane)
            .map_err(WorthQueryRuntimeError::EffectPolicyDenied)
    }

    pub fn use_view<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
    ) -> WorthQueryPreviewHandleBindingEvidence {
        let evidence = WorthQueryPreviewHandleBindingEvidence::live_view(
            &self.label,
            view.name(),
            self.effect_policy,
            &self.basis_admission.evidence(),
        );
        self.handle_bindings.push(evidence.clone());
        evidence
    }

    pub fn use_computed<T>(
        &mut self,
        computed: &WorthQueryDerivedViewHandle<T>,
    ) -> WorthQueryPreviewHandleBindingEvidence {
        let evidence = WorthQueryPreviewHandleBindingEvidence::computed(
            &self.label,
            computed.name(),
            self.effect_policy,
            &self.basis_admission.evidence(),
        );
        self.handle_bindings.push(evidence.clone());
        evidence
    }

    pub fn use_effect<T>(
        &mut self,
        effect: &WorthQueryEffectHandle<T>,
    ) -> Result<WorthQueryPreviewHandleBindingEvidence, WorthQueryRuntimeError> {
        let inspected = self.runtime.inspect_effect(effect)?;
        let disposition = WorthQueryPreviewEffectBindingDisposition::from_policy(
            self.effect_policy,
            inspected.action(),
            inspected.target_lane(),
        )?;
        let evidence = WorthQueryPreviewHandleBindingEvidence::effect(
            &self.label,
            effect.name(),
            inspected.target_lane(),
            self.effect_policy,
            disposition,
            &self.basis_admission.evidence(),
        );
        self.handle_bindings.push(evidence.clone());
        Ok(evidence)
    }

    pub fn run_operation(
        &mut self,
        operation: WorthQueryInstalledOperation,
        inputs: Vec<WorthQueryOperationInput>,
    ) -> Result<WorthQueryRunReceipt, WorthQueryRuntimeError> {
        let query_operation = self.runtime.installed_query_operation(&operation)?;
        admit_authority_requirements(query_operation.authority_requirements())?;
        let bound_inputs = validate_inputs(&query_operation, &inputs)?;
        let mut trace = WorthQueryProgramTrace::new(
            operation.program_identity.as_str(),
            operation.operation_identity.as_str(),
            &bound_inputs,
            query_operation
                .authority_requirements()
                .iter()
                .cloned()
                .collect(),
        );
        trace.record_replay_or_parity(format!("preview-session:{}", self.label.display()));
        let mut outputs = Vec::new();
        let mut write_receipts = Vec::new();
        let mut patch_batches = Vec::new();

        for effect in query_operation.effects() {
            self.admit_operation_effect(effect)?;
            match effect.clone() {
                WorthQueryProgramEffect::DeclareLiveView {
                    name,
                    request,
                    schema_view,
                } => {
                    let _: WorthQueryLiveView<crate::runtime::WorthQueryUnrefinedLiveShape> = self
                        .runtime
                        .declare_live_view(name.clone(), request, schema_view)?;
                    trace.record_declaration(format!("preview-live:{name}"));
                }
                WorthQueryProgramEffect::DeclareDerivedView(view) => {
                    let name = view.name().to_string();
                    self.runtime.declare_derived_view(view)?;
                    trace.record_declaration(format!("preview-derived:{name}"));
                }
                WorthQueryProgramEffect::Write(command) => {
                    self.admit_preview_write_intent()?;
                    let receipt = self.stage_command(command)?;
                    trace.record_write_receipt(receipt.commit_identity().clone());
                    write_receipts.push(receipt);
                }
                WorthQueryProgramEffect::WriteTemplate(template) => {
                    self.admit_preview_write_intent()?;
                    let command = template.bind(&bound_inputs)?;
                    let receipt = self.stage_command(command)?;
                    trace.record_write_receipt(receipt.commit_identity().clone());
                    write_receipts.push(receipt);
                }
                WorthQueryProgramEffect::ReadLive { view_name } => {
                    let target = self
                        .runtime
                        .live_subscriptions
                        .get(&WorthQueryLiveArtifactTarget::from_view_name(&view_name))
                        .map(|state| {
                            WorthQueryLiveArtifactTarget::from_subscription_installation(
                                &state.installation,
                            )
                        })
                        .unwrap_or_else(|| {
                            WorthQueryLiveArtifactTarget::from_view_name(view_name.clone())
                        });
                    let rows = self.runtime.backend.live_entities_for_target(&target)?;
                    outputs.push(WorthQueryOperationOutput::from_live_read_entities(
                        format!("preview-live:{view_name}"),
                        rows,
                    ));
                    trace.record_replay_or_parity(format!("preview-read-live:{view_name}"));
                }
                WorthQueryProgramEffect::DrainPatches { view_name } => {
                    patch_batches.push(WorthQueryPatchBatch {
                        view_name,
                        live_patches: Vec::new(),
                        query_delivery_batches: Vec::new(),
                        derived_patch_notes: vec![format!(
                            "preview:{}:patch-drain-deferred",
                            self.label.display()
                        )],
                        derived_patches: Vec::new(),
                    });
                }
            }
        }

        let run_identity = self.runtime.next_run_identity(&operation);
        self.runtime.run_traces.insert(run_identity.clone(), trace);
        self.writes.extend(write_receipts.iter().cloned());
        Ok(WorthQueryRunReceipt {
            run_identity,
            operation,
            outputs,
            write_receipts,
            patch_batches,
        })
    }
}
