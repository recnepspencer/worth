use super::runtime_batching::{
    deny_scaffold_multi_command_batch_without_atomic_authority, should_use_backend_atomic_batch,
    BatchCommandSummary,
};
use super::*;
use crate::runtime::runtime_writes::WorthQueryWriteAdmissionExecutionRecord;
#[path = "runtime_batch_write_bridge_refs.rs"]
mod runtime_batch_write_bridge_refs;
#[path = "runtime_batch_write_receipt_context.rs"]
mod runtime_batch_write_receipt_context;
#[path = "runtime_batch_write_receipt_rows.rs"]
mod runtime_batch_write_receipt_rows;
#[path = "runtime_batch_write_symbolics.rs"]
mod runtime_batch_write_symbolics;

use runtime_batch_write_bridge_refs::bridge_symbolic_target_reference;
use runtime_batch_write_receipt_context::{
    batch_decision_trace_envelope, batch_execution_provenance,
};
use runtime_batch_write_symbolics::{
    admit_atomic_batch_symbolic_references, record_planned_same_batch_symbolic_target,
    symbolic_aspect_resolution_evidence_for_command,
    symbolic_aspect_resolution_evidence_for_references,
};

impl WorthQueryRuntime {
    pub(crate) fn execute_authoritative_write_batch_direct(
        &mut self,
        commands: Vec<WorthQueryWriteCommand>,
        graph_composition_breadth: WorthQueryGraphCompositionBreadth,
        graph_composition_program: WorthQueryGraphCompositionProgram,
        shared_admission: Option<WorthQueryWriteAdmissionExecutionRecord>,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        self.reap_abandoned_managed_live_resources()?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Write)?;
        if commands.is_empty() {
            return Err(WorthQueryRuntimeError::Workspace(
                WorthQueryWorkspaceError::new("mutation batch must declare at least one operation"),
            ));
        }
        self.preflight_native_mutation_contracts(&commands)?;
        let support_profile = self.backend.support_profile();
        deny_scaffold_multi_command_batch_without_atomic_authority(&support_profile, &commands)?;
        let use_backend_atomic_batch = should_use_backend_atomic_batch(&support_profile, &commands);
        let mut command_summaries: Vec<BatchCommandSummary> = Vec::with_capacity(commands.len());
        let mut resolved_receipts = Vec::with_capacity(commands.len());
        let mut symbolic_targets = BTreeMap::<
            WorthQuerySameBatchSymbolicTargetKey,
            WorthQuerySameBatchSymbolicTarget,
        >::new();
        let mut planned_symbolic_targets = BTreeMap::<
            WorthQuerySameBatchSymbolicTargetKey,
            WorthQuerySameBatchSymbolicTarget,
        >::new();
        let mut deferred_backend_commands = Vec::new();
        for command in commands {
            if let Some(binding) = command.existing_truth_binding() {
                self.backend
                    .admit_existing_truth_binding(binding)
                    .map_err(WorthQueryRuntimeError::MutationBindingDenied)?;
            }
            admit_continuity_intent(&command)
                .map_err(WorthQueryRuntimeError::MutationContinuityDenied)?;
            admit_naming_intent(&command).map_err(WorthQueryRuntimeError::MutationNamingDenied)?;
            let mutation_family = command.mutation_family();
            let declared_collection_identity = command.declared_collection_identity();
            let declared_entity_identity = command.declared_entity_identity();
            let existing_truth_binding = command.existing_truth_binding().cloned();
            let verified_existing_truth_assertion =
                self.verified_existing_assertion_for_command(&command)?;
            let declared_aspect_operations = command.declared_aspect_operations();
            let declared_aspect_value_digest = command_declared_aspect_value_identity(&command);
            let mutation_metadata = command.mutation_metadata();
            let symbolic_target_reference = command.symbolic_target_reference().cloned();
            let naming_intent = command.naming_intent().cloned();
            let continuity_intent = command.continuity_intent().cloned();
            let symbolic_aspect_resolution_evidence = if use_backend_atomic_batch {
                admit_atomic_batch_symbolic_references(&planned_symbolic_targets, &command)?;
                Vec::new()
            } else {
                symbolic_aspect_resolution_evidence_for_command(&symbolic_targets, &command)?
            };
            let summary = BatchCommandSummary::new(
                mutation_family,
                declared_collection_identity,
                declared_entity_identity,
                existing_truth_binding,
                verified_existing_truth_assertion,
                symbolic_target_reference,
                naming_intent,
                continuity_intent,
                declared_aspect_operations,
                declared_aspect_value_digest.clone(),
                command.symbolic_aspect_references().to_vec(),
                symbolic_aspect_resolution_evidence,
                mutation_metadata,
            );
            if use_backend_atomic_batch
                && !matches!(
                    command,
                    WorthQueryWriteCommand::AssertExistingAspects { .. }
                        | WorthQueryWriteCommand::VerifyExistingAspects { .. }
                )
            {
                record_planned_same_batch_symbolic_target(&mut planned_symbolic_targets, &command);
                deferred_backend_commands.push(self.admit_backend_write_command(command)?);
                command_summaries.push(summary);
                resolved_receipts.push(None);
                continue;
            }
            let mut receipt = match command {
                WorthQueryWriteCommand::AssertExistingAspects { binding, .. } => {
                    synthetic_existing_assertion_receipt(
                        &binding,
                        &self.current_snapshot_identity()?,
                        declared_aspect_value_digest.as_ref(),
                    )
                }
                WorthQueryWriteCommand::VerifyExistingAspects { binding, .. } => {
                    synthetic_existing_assertion_receipt(
                        &binding,
                        &self.current_snapshot_identity()?,
                        declared_aspect_value_digest.as_ref(),
                    )
                }
                WorthQueryWriteCommand::InsertAspects {
                    collection,
                    aspects,
                    symbolic_aspect_references,
                    metadata,
                    naming_intent,
                    continuity_intent,
                    symbolic_target_reference,
                } => {
                    let admitted =
                        self.admit_backend_write_command(WorthQueryWriteCommand::InsertAspects {
                            collection,
                            aspects: resolve_symbolic_aspect_references(
                                &symbolic_targets,
                                aspects,
                                &symbolic_aspect_references,
                            )?,
                            symbolic_aspect_references: Vec::new(),
                            metadata,
                            naming_intent,
                            continuity_intent,
                            symbolic_target_reference,
                        })?;
                    self.backend.write(admitted)?
                }
                WorthQueryWriteCommand::UpdateSymbolicAspects {
                    reference,
                    aspects,
                    metadata,
                    naming_intent,
                    continuity_intent,
                } => {
                    let resolved_target =
                        resolve_same_batch_symbolic_target(&symbolic_targets, &reference)?;
                    let symbolic_target_reference = bridge_symbolic_target_reference(
                        &reference,
                        resolved_target.entity_identity(),
                        resolved_target.target_collection_identity(),
                    )?;
                    let admitted =
                        self.admit_backend_write_command(WorthQueryWriteCommand::UpdateAspects {
                            entity_identity: resolved_target.entity_identity().clone(),
                            aspects,
                            metadata,
                            naming_intent,
                            continuity_intent,
                        })?;
                    attach_symbolic_target_reference_to_receipt(
                        self.backend.write(admitted)?,
                        symbolic_target_reference,
                    )
                }
                WorthQueryWriteCommand::VerifyThenUpdateExistingAspects {
                    binding,
                    asserted_aspects,
                    aspects,
                    symbolic_aspect_references,
                    metadata,
                    naming_intent,
                    continuity_intent,
                } => {
                    let command = Self::lower_backend_write_command(
                        WorthQueryWriteCommand::VerifyThenUpdateExistingAspects {
                            binding,
                            asserted_aspects,
                            aspects: resolve_symbolic_aspect_references(
                                &symbolic_targets,
                                aspects,
                                &symbolic_aspect_references,
                            )?,
                            symbolic_aspect_references: Vec::new(),
                            metadata,
                            naming_intent,
                            continuity_intent,
                        },
                    );
                    let admitted = self.admit_backend_write_command(command)?;
                    self.backend.write(admitted)?
                }
                WorthQueryWriteCommand::DeleteSymbolicAspects {
                    reference,
                    touched_aspects,
                    metadata,
                    naming_intent,
                } => {
                    let resolved_target =
                        resolve_same_batch_symbolic_target(&symbolic_targets, &reference)?;
                    let symbolic_target_reference = bridge_symbolic_target_reference(
                        &reference,
                        resolved_target.entity_identity(),
                        resolved_target.target_collection_identity(),
                    )?;
                    let admitted =
                        self.admit_backend_write_command(WorthQueryWriteCommand::DeleteAspects {
                            entity_identity: resolved_target.entity_identity().clone(),
                            declared_collection: resolved_target
                                .target_collection_identity()
                                .cloned(),
                            touched_aspects,
                            metadata,
                            naming_intent,
                        })?;
                    attach_symbolic_target_reference_to_receipt(
                        self.backend.write(admitted)?,
                        symbolic_target_reference,
                    )
                }
                other => {
                    let admitted =
                        self.admit_backend_write_command(Self::lower_backend_write_command(other))?;
                    self.backend.write(admitted)?
                }
            };
            if let Some(intent) = summary.continuity_intent().as_ref() {
                let (_, target_collection, target_entity_identity) =
                    classify_receipt_mutation_summary(&receipt);
                let existing_truth_binding = summary.existing_truth_binding();
                let basis_binding_digest = existing_truth_binding
                    .as_ref()
                    .map(|binding| binding.binding_digest());
                if let Some(bundle) = bridge_continuity_mutation_bundle(
                    intent,
                    basis_binding_digest.as_deref(),
                    target_entity_identity.as_ref(),
                    target_collection.as_ref(),
                ) {
                    receipt = attach_continuity_mutation_to_receipt(receipt, bundle);
                }
            }
            if let Some(intent) = summary.naming_intent().as_ref() {
                let (_, target_collection, target_entity_identity) =
                    classify_receipt_mutation_summary(&receipt);
                if let Some(bundle) = bridge_naming_mutation_bundle(
                    intent,
                    target_entity_identity.as_ref(),
                    target_collection.as_ref(),
                ) {
                    receipt = attach_naming_mutation_to_receipt(receipt, bundle);
                }
            }
            record_same_batch_symbolic_target(
                &mut symbolic_targets,
                summary.symbolic_target_reference().as_ref(),
                summary.declared_collection_identity().as_ref(),
                &receipt,
            );
            command_summaries.push(summary);
            resolved_receipts.push(Some(receipt));
        }
        if use_backend_atomic_batch {
            let deferred_receipts = self
                .backend
                .write_batch(deferred_backend_commands)
                .map_err(WorthQueryRuntimeError::Workspace)?;
            let deferred_slot_count = resolved_receipts
                .iter()
                .filter(|receipt| receipt.is_none())
                .count();
            if deferred_receipts.len() != deferred_slot_count {
                return Err(WorthQueryRuntimeError::Workspace(
                    WorthQueryWorkspaceError::new(format!(
                        "backend atomic batch returned {} receipts for {} commands",
                        deferred_receipts.len(),
                        deferred_slot_count
                    )),
                ));
            }
            let mut deferred_iter = deferred_receipts.into_iter();
            for receipt in &mut resolved_receipts {
                if receipt.is_none() {
                    *receipt = Some(
                        deferred_iter
                            .next()
                            .expect("deferred backend receipt should exist for each empty slot"),
                    );
                }
            }
            let mut deferred_symbolic_targets = BTreeMap::<
                WorthQuerySameBatchSymbolicTargetKey,
                WorthQuerySameBatchSymbolicTarget,
            >::new();
            let mut rebuilt_summaries = Vec::with_capacity(command_summaries.len());
            for (summary, receipt) in command_summaries.into_iter().zip(resolved_receipts.iter()) {
                let receipt = receipt
                    .as_ref()
                    .expect("deferred backend receipts should fill every empty slot");
                let symbolic_aspect_references = summary.symbolic_aspect_references();
                let rebuilt_summary = summary.with_symbolic_aspect_resolution_evidence(
                    symbolic_aspect_resolution_evidence_for_references(
                        &deferred_symbolic_targets,
                        &symbolic_aspect_references,
                    )?,
                );
                record_same_batch_symbolic_target(
                    &mut deferred_symbolic_targets,
                    rebuilt_summary.symbolic_target_reference().as_ref(),
                    rebuilt_summary.declared_collection_identity().as_ref(),
                    receipt,
                );
                rebuilt_summaries.push(rebuilt_summary);
            }
            command_summaries = rebuilt_summaries;
        }
        let receipts = resolved_receipts
            .into_iter()
            .map(|receipt| {
                receipt.expect("every batch command should resolve to one concrete receipt")
            })
            .collect::<Vec<_>>();
        let combined_receipt = combined_batch_mutation_receipt(&receipts)?;
        let summary = self.route_authoritative_mutation_summary(
            &combined_receipt,
            &WorthQueryMutationMetadata::default(),
        )?;
        let write_receipts = self.build_batch_component_write_receipts(receipts, command_summaries);
        let mut touched_aspects = std::collections::BTreeSet::new();
        for touch in combined_receipt
            .deltas
            .iter()
            .flat_map(|delta| delta.admitted_touched_aspects())
        {
            touched_aspects.insert(touch.clone());
        }
        let batch_request_detail = format!("batch-write:{}", write_receipts.len());
        let execution_provenance =
            batch_execution_provenance(shared_admission.as_ref(), &combined_receipt);
        let decision_trace_envelope = batch_decision_trace_envelope(
            shared_admission.as_ref(),
            &combined_receipt,
            &batch_request_detail,
        );
        let receipt = WorthQueryBatchWriteReceipt::new(
            write_receipts,
            WorthQueryAuthorityLane::AuthoritativeTruth,
            WorthQueryAuthorityLane::AuthoritativeTruth,
            graph_composition_breadth,
            graph_composition_program,
            touched_aspects.into_iter().collect(),
            summary.affected_live_view_targets,
            summary.affected_derived_view_targets,
            summary.considered_computed_view_count,
            summary.considered_effect_count,
            summary.delivered_effect_count,
            summary.pending_write_intent_count,
            summary.suppressed_effect_count,
            summary.meaningful_effect_suppression_count,
            summary.effect_expression_failure_count,
            summary.refresh_fallback,
            decision_trace_envelope,
            execution_provenance,
        )?;
        self.journal_replay.record_batch_receipt(&receipt);
        Ok(receipt)
    }
}
