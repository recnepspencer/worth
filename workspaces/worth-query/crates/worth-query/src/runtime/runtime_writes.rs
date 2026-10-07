use super::*;
use crate::intent_admission::WorthQueryIntentEligibilityTraceEvidence;

pub(crate) struct WorthQueryWriteAdmissionExecutionRecord {
    pub family: WorthQueryIntentAdmissionFamily,
    pub entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
    pub execution_seam: WorthQueryIntentAdmissionExecutionSeam,
    pub request_detail: String,
    pub request_digest: String,
    pub eligibility_trace: WorthQueryIntentEligibilityTraceEvidence,
    pub decision_digest: String,
    pub handoff_digest: String,
    pub binding_digest: String,
}

impl WorthQueryRuntime {
    pub(crate) fn build_authoritative_mutation_intent_seed(
        &self,
        command: WorthQueryWriteCommand,
    ) -> crate::intent_admission::WorthQueryAuthoritativeMutationIntentSeed {
        use crate::intent_admission::WorthQueryAuthoritativeMutationPreflight as Preflight;

        let (preflight, admitted_mutation) = match self.admit_backend_write_command(command.clone())
        {
            Err(WorthQueryRuntimeError::MutationContractDenied(denial)) => {
                (Preflight::ContractDenied(denial), None)
            }
            Err(other) => panic!("unexpected native mutation admission error: {other}"),
            Ok(admitted_mutation) => {
                let preflight = if let Some(reference) = command.symbolic_target_reference() {
                    Preflight::TargetReferenceDenied(WorthQuerySymbolicTargetReferenceDenial::new(
                        reference,
                        WorthQuerySymbolicTargetReferenceDenialKind::RequiresBatchContext,
                        "same-batch symbolic target references require batch execution",
                    ))
                } else if let Some(reference) = command.symbolic_aspect_references().first() {
                    Preflight::TargetReferenceDenied(WorthQuerySymbolicTargetReferenceDenial::new(
                        reference.reference(),
                        WorthQuerySymbolicTargetReferenceDenialKind::RequiresBatchContext,
                        "same-batch symbolic aspect references require batch execution",
                    ))
                } else if let Some(binding) = command.existing_truth_binding() {
                    match self.backend.admit_existing_truth_binding(binding) {
                        Err(denial) => Preflight::BindingDenied(denial),
                        Ok(()) => self.scalar_mutation_post_binding_preflight(&command),
                    }
                } else {
                    self.scalar_mutation_post_binding_preflight(&command)
                };
                let successor =
                    matches!(preflight, Preflight::Admitted { .. }).then_some(admitted_mutation);
                (preflight, successor)
            }
        };

        crate::intent_admission::WorthQueryAuthoritativeMutationIntentSeed::new(
            command,
            preflight,
            admitted_mutation,
        )
    }

    fn scalar_mutation_post_binding_preflight(
        &self,
        command: &WorthQueryWriteCommand,
    ) -> crate::intent_admission::WorthQueryAuthoritativeMutationPreflight {
        use crate::intent_admission::WorthQueryAuthoritativeMutationPreflight as Preflight;

        match admit_continuity_intent(command) {
            Err(denial) => Preflight::ContinuityDenied(denial),
            Ok(()) => match admit_naming_intent(command) {
                Err(denial) => Preflight::NamingDenied(denial),
                Ok(()) => match self.verified_existing_assertion_for_command(command) {
                    Ok(verified_existing_truth_assertion) => Preflight::Admitted {
                        verified_existing_truth_assertion,
                    },
                    Err(WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial)) => {
                        Preflight::AssertionDenied(denial)
                    }
                    Err(other) => panic!("unexpected scalar mutation preflight error: {other}"),
                },
            },
        }
    }

    pub(super) fn verified_existing_assertion_for_command(
        &self,
        command: &WorthQueryWriteCommand,
    ) -> Result<Option<WorthQueryVerifiedExistingTruthAssertion>, WorthQueryRuntimeError> {
        match command {
            WorthQueryWriteCommand::VerifyExistingAspects {
                binding, aspects, ..
            }
            | WorthQueryWriteCommand::VerifyThenUpdateExistingAspects {
                binding,
                asserted_aspects: aspects,
                ..
            }
            | WorthQueryWriteCommand::VerifyThenDeleteExistingAspects {
                binding,
                asserted_aspects: aspects,
                ..
            } => Ok(Some(
                self.backend
                    .verify_existing_truth_assertion(binding, aspects)
                    .map_err(WorthQueryRuntimeError::ExistingTruthAssertionDenied)?,
            )),
            _ => Ok(None),
        }
    }

    pub(super) fn lower_backend_write_command(
        command: WorthQueryWriteCommand,
    ) -> WorthQueryWriteCommand {
        match command {
            WorthQueryWriteCommand::VerifyThenUpdateExistingAspects {
                binding,
                aspects,
                metadata,
                naming_intent,
                continuity_intent,
                ..
            } => WorthQueryWriteCommand::UpdateExistingAspects {
                binding,
                aspects,
                metadata,
                naming_intent,
                continuity_intent,
            },
            WorthQueryWriteCommand::VerifyThenDeleteExistingAspects {
                binding,
                touched_aspects,
                metadata,
                naming_intent,
                ..
            } => WorthQueryWriteCommand::DeleteExistingAspects {
                binding,
                touched_aspects,
                metadata,
                naming_intent,
            },
            other => other,
        }
    }

    pub(super) fn admit_backend_write_command(
        &self,
        command: WorthQueryWriteCommand,
    ) -> Result<WorthQueryBackendAdmissibleMutation, WorthQueryRuntimeError> {
        WorthQueryBackendAdmissibleMutation::from_authored_command(
            command,
            &self.native_aspect_contracts,
        )
        .map_err(WorthQueryRuntimeError::MutationContractDenied)
    }

    pub(super) fn preflight_native_mutation_contracts(
        &self,
        commands: &[WorthQueryWriteCommand],
    ) -> Result<(), WorthQueryRuntimeError> {
        for command in commands {
            crate::runtime::native_aspect_contracts::admit_authoritative_mutation_patch(
                command,
                &self.native_aspect_contracts,
            )
            .map_err(WorthQueryRuntimeError::MutationContractDenied)?;
        }
        Ok(())
    }

    pub fn probe_existing(
        &self,
        request: WorthQueryExistingTruthProbeRequest,
    ) -> Result<WorthQueryExistingTruthProbe, WorthQueryRuntimeError> {
        Ok(self
            .probe_existing_intent(request)
            .execute()?
            .probe()
            .clone())
    }

    pub(crate) fn execute_authoritative_write_command_direct(
        &mut self,
        command: WorthQueryWriteCommand,
        admitted_mutation: WorthQueryBackendAdmissibleMutation,
        verified_existing_truth_assertion: Option<WorthQueryVerifiedExistingTruthAssertion>,
        shared_admission: Option<WorthQueryWriteAdmissionExecutionRecord>,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.reap_abandoned_managed_live_resources()?;
        let prepared_routing = WorthQueryPreparedAuthoritativeMutationRouting::from_direct_command(
            &command,
            verified_existing_truth_assertion,
        );
        let receipt = self.execute_backend_or_synthetic_write(
            command,
            admitted_mutation,
            prepared_routing.declared_aspect_value_digest(),
        )?;
        let receipt = self.attach_optional_mutation_bundles(
            receipt,
            prepared_routing.existing_truth_binding(),
            prepared_routing.continuity_intent(),
            prepared_routing.naming_intent(),
        );
        let execution_provenance =
            self.shared_write_execution_provenance(shared_admission.as_ref(), &receipt);
        let decision_trace_envelope = self.shared_write_decision_trace_envelope(
            shared_admission.as_ref(),
            prepared_routing.mutation_family(),
            &receipt,
        );
        let receipt = self.route_authoritative_mutation_receipt(prepared_routing.complete(
            receipt,
            WorthQueryAuthoritativeMutationExecutionEvidence {
                decision_trace_envelope,
                execution_provenance,
            },
        ))?;
        self.journal_replay.record_write_receipt(&receipt);
        Ok(receipt)
    }

    fn execute_backend_or_synthetic_write(
        &mut self,
        command: WorthQueryWriteCommand,
        admitted_mutation: WorthQueryBackendAdmissibleMutation,
        declared_aspect_value_digest: Option<&crate::evidence_identity::WorthQueryEvidenceIdentity>,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryRuntimeError> {
        match &command {
            WorthQueryWriteCommand::AssertExistingAspects { binding, .. }
            | WorthQueryWriteCommand::VerifyExistingAspects { binding, .. } => {
                Ok(synthetic_existing_assertion_receipt(
                    binding,
                    &self.current_snapshot_identity()?,
                    declared_aspect_value_digest,
                ))
            }
            _ => self.backend.write(admitted_mutation).map_err(Into::into),
        }
    }

    fn attach_optional_mutation_bundles(
        &self,
        mut receipt: WorthQueryMutationReceipt,
        existing_truth_binding: Option<&WorthQueryExistingTruthTargetBinding>,
        continuity_intent: Option<&WorthQueryContinuityMutationIntent>,
        naming_intent: Option<&WorthQueryNamingMutationIntent>,
    ) -> WorthQueryMutationReceipt {
        receipt = self.attach_optional_continuity_bundle(
            receipt,
            existing_truth_binding,
            continuity_intent,
        );
        self.attach_optional_naming_bundle(receipt, existing_truth_binding, naming_intent)
    }

    fn attach_optional_continuity_bundle(
        &self,
        receipt: WorthQueryMutationReceipt,
        existing_truth_binding: Option<&WorthQueryExistingTruthTargetBinding>,
        continuity_intent: Option<&WorthQueryContinuityMutationIntent>,
    ) -> WorthQueryMutationReceipt {
        let Some(intent) = continuity_intent else {
            return receipt;
        };
        let (_, target_collection, target_entity_identity) =
            classify_receipt_mutation_summary(&receipt);
        let basis_binding_digest = existing_truth_binding.map(|binding| binding.binding_digest());
        match bridge_continuity_mutation_bundle(
            intent,
            basis_binding_digest.as_deref(),
            target_entity_identity.as_ref(),
            target_collection.as_ref(),
        ) {
            Some(bundle) => attach_continuity_mutation_to_receipt(receipt, bundle),
            None => receipt,
        }
    }

    fn attach_optional_naming_bundle(
        &self,
        receipt: WorthQueryMutationReceipt,
        existing_truth_binding: Option<&WorthQueryExistingTruthTargetBinding>,
        naming_intent: Option<&WorthQueryNamingMutationIntent>,
    ) -> WorthQueryMutationReceipt {
        let Some(intent) = naming_intent else {
            return receipt;
        };
        let (_, mut target_collection, mut target_entity_identity) =
            classify_receipt_mutation_summary(&receipt);
        if let Some(binding) = existing_truth_binding {
            target_collection = binding.target_collection_identity().cloned();
            target_entity_identity = Some(binding.resolved_entity_artifact_identity());
        }
        match bridge_naming_mutation_bundle(
            intent,
            target_entity_identity.as_ref(),
            target_collection.as_ref(),
        ) {
            Some(bundle) => attach_naming_mutation_to_receipt(receipt, bundle),
            None => receipt,
        }
    }

    fn shared_write_execution_provenance(
        &self,
        shared_admission: Option<&WorthQueryWriteAdmissionExecutionRecord>,
        receipt: &WorthQueryMutationReceipt,
    ) -> Option<WorthQueryIntentExecutionProvenance> {
        shared_admission.map(|record| {
            let commit_label = receipt
                .commit_identity
                .evidence_identity()
                .reporting_projection()
                .to_string();
            let snapshot_evidence_identity = receipt.snapshot_identity.evidence_identity();
            WorthQueryIntentExecutionProvenance::for_shared_execution_typed_parts(
                record.family,
                record.entrypoint,
                record.execution_seam,
                &record.decision_digest,
                &record.handoff_digest,
                &record.binding_digest,
                &commit_label,
                &snapshot_evidence_identity,
            )
        })
    }

    fn shared_write_decision_trace_envelope(
        &self,
        shared_admission: Option<&WorthQueryWriteAdmissionExecutionRecord>,
        mutation_family: WorthQueryMutationFamily,
        receipt: &WorthQueryMutationReceipt,
    ) -> Option<WorthQueryIntentDecisionTraceEnvelope> {
        shared_admission.map(|record| {
            let commit_label = receipt
                .commit_identity
                .evidence_identity()
                .reporting_projection()
                .to_string();
            WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution_parts(
                record.family,
                record.entrypoint,
                &record.request_detail,
                &record.request_digest,
                record.eligibility_trace.clone(),
                &record.decision_digest,
                &record.handoff_digest,
                record.execution_seam,
                mutation_family.as_str(),
                &commit_label,
                "mutation-write",
            )
        })
    }

    pub fn write(
        &mut self,
        command: WorthQueryWriteCommand,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.write_intent(command).execute()
    }
}
