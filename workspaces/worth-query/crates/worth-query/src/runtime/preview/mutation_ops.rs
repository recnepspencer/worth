use super::symbolic_targets::{
    admit_preview_batch_symbolic_references, record_preview_symbolic_target,
    resolve_preview_symbolic_target,
};
use super::*;
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::mutation::admit_naming_intent;
use crate::runtime::{WorthQueryExistingEntityTarget, WorthQueryExistingRelationTarget};

impl<'a> WorthQueryPreviewSession<'a> {
    pub fn bind_existing_entity(
        &self,
        target: WorthQueryExistingEntityTarget,
    ) -> Result<WorthQueryExistingTruthTargetBinding, WorthQueryRuntimeError> {
        Ok(WorthQueryExistingTruthTargetBinding::from_entity_target(
            target,
        )?)
    }

    pub fn bind_existing_relation(
        &self,
        target: WorthQueryExistingRelationTarget,
    ) -> Result<WorthQueryExistingTruthTargetBinding, WorthQueryRuntimeError> {
        Ok(WorthQueryExistingTruthTargetBinding::from_relation_target(
            target,
        )?)
    }

    pub fn write(
        &mut self,
        command: WorthQueryWriteCommand,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        if let Some(reference) = command.symbolic_target_reference() {
            return Err(WorthQueryRuntimeError::MutationTargetReferenceDenied(
                WorthQuerySymbolicTargetReferenceDenial::new(
                    reference,
                    WorthQuerySymbolicTargetReferenceDenialKind::RequiresBatchContext,
                    "same-batch symbolic target references require batch execution",
                ),
            ));
        }
        deny_preview_assertion(&command)?;
        deny_preview_continuity(&command)?;
        admit_naming_intent(&command).map_err(WorthQueryRuntimeError::MutationNamingDenied)?;
        self.runtime.backend.admit_preview_write_command(&command)?;
        let receipt = WorthQueryWriteReceipt::preview(
            &self.label,
            self.pending_commands.len() + 1,
            &command,
            self.runtime.current_snapshot_identity()?,
        );
        self.pending_commands.push(command);
        self.writes.push(receipt.clone());
        self.route_preview_execution(&receipt);
        Ok(receipt)
    }

    pub fn insert(
        &mut self,
        collection: impl Into<String>,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_insert(collection)?;
        self.write(command)
    }

    pub fn update(
        &mut self,
        entity_identity: WorthQueryEntityIdentity,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_update(entity_identity)?;
        self.write(command)
    }

    pub fn update_existing(
        &mut self,
        binding: WorthQueryExistingTruthTargetBinding,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_update_existing(binding)?;
        self.write(command)
    }

    pub fn assert_existing(
        &mut self,
        binding: WorthQueryExistingTruthTargetBinding,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_assert_existing(binding)?;
        self.write(command)
    }

    pub fn verify_existing(
        &mut self,
        binding: WorthQueryExistingTruthTargetBinding,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_verify_existing(binding)?;
        self.write(command)
    }

    pub fn update_existing_verified(
        &mut self,
        binding: WorthQueryExistingTruthTargetBinding,
        verify: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
        update: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let asserted_aspects = verify(WorthQueryAspectMutationBuilder::new())
            .finish_existing_truth_verification_aspects("backend-verified existing-truth update")?;
        let command = update(WorthQueryAspectMutationBuilder::new())
            .build_update_existing_verified(binding, asserted_aspects)?;
        self.write(command)
    }

    pub fn delete(
        &mut self,
        entity_identity: impl Into<String>,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.write(WorthQueryWriteCommand::Delete {
            entity_identity: crate::memory_workspace::admit_authored_entity_label(
                entity_identity.into(),
            ),
        })
    }

    pub fn delete_with(
        &mut self,
        entity_identity: WorthQueryEntityIdentity,
        declaration: impl FnOnce(WorthQueryDeleteMutationBuilder) -> WorthQueryDeleteMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryDeleteMutationBuilder::new()).build_delete(entity_identity)?;
        self.write(command)
    }

    pub fn delete_existing(
        &mut self,
        binding: WorthQueryExistingTruthTargetBinding,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.write(WorthQueryWriteCommand::DeleteExistingAspects {
            binding,
            touched_aspects: Vec::new(),
            metadata: WorthQueryMutationMetadata::default(),
            naming_intent: None,
        })
    }

    pub fn delete_existing_with(
        &mut self,
        binding: WorthQueryExistingTruthTargetBinding,
        declaration: impl FnOnce(WorthQueryDeleteMutationBuilder) -> WorthQueryDeleteMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryDeleteMutationBuilder::new()).build_delete_existing(binding)?;
        self.write(command)
    }

    pub fn delete_existing_verified(
        &mut self,
        binding: WorthQueryExistingTruthTargetBinding,
        verify: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
        delete: impl FnOnce(WorthQueryDeleteMutationBuilder) -> WorthQueryDeleteMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let asserted_aspects = verify(WorthQueryAspectMutationBuilder::new())
            .finish_existing_truth_verification_aspects("backend-verified existing-truth delete")?;
        let command = delete(WorthQueryDeleteMutationBuilder::new())
            .build_delete_existing_verified(binding, asserted_aspects)?;
        self.write(command)
    }

    pub fn batch(
        &mut self,
        declaration: impl FnOnce(WorthQueryMutationBatchBuilder) -> WorthQueryMutationBatchBuilder,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        let commands = declaration(WorthQueryMutationBatchBuilder::new()).finish()?;
        for command in &commands {
            deny_preview_assertion(command)?;
            deny_preview_continuity(command)?;
            if command.symbolic_target_reference().is_none() {
                admit_naming_intent(command)
                    .map_err(WorthQueryRuntimeError::MutationNamingDenied)?;
            }
        }
        admit_preview_batch_symbolic_references(&commands)?;
        for command in &commands {
            self.runtime.backend.admit_preview_write_command(command)?;
        }
        let mut symbolic_targets = BTreeMap::<
            WorthQuerySameBatchSymbolicTargetKey,
            WorthQuerySameBatchSymbolicTarget,
        >::new();
        let mut receipts = Vec::with_capacity(commands.len());
        for command in commands {
            let symbolic_target_reference = command.symbolic_target_reference().cloned();
            let receipt = match &command {
                WorthQueryWriteCommand::UpdateSymbolicAspects {
                    reference,
                    aspects,
                    metadata,
                    naming_intent,
                    continuity_intent,
                } => {
                    let resolved_target =
                        resolve_preview_symbolic_target(&symbolic_targets, reference)?;
                    let concrete = WorthQueryWriteCommand::UpdateAspects {
                        entity_identity: resolved_target.entity_identity().clone(),
                        aspects: aspects.clone(),
                        metadata: metadata.clone(),
                        naming_intent: naming_intent.clone(),
                        continuity_intent: continuity_intent.clone(),
                    };
                    WorthQueryWriteReceipt::preview(
                        &self.label,
                        self.pending_commands.len() + 1,
                        &concrete,
                        self.runtime.current_snapshot_identity()?,
                    )
                    .with_symbolic_target_reference(
                        reference,
                        resolved_target.entity_identity().clone(),
                        resolved_target.target_collection_identity().cloned(),
                    )
                }
                WorthQueryWriteCommand::DeleteSymbolicAspects {
                    reference,
                    touched_aspects,
                    metadata,
                    naming_intent,
                } => {
                    let resolved_target =
                        resolve_preview_symbolic_target(&symbolic_targets, reference)?;
                    let concrete = WorthQueryWriteCommand::DeleteAspects {
                        entity_identity: resolved_target.entity_identity().clone(),
                        declared_collection: resolved_target.target_collection_identity().cloned(),
                        touched_aspects: touched_aspects.clone(),
                        metadata: metadata.clone(),
                        naming_intent: naming_intent.clone(),
                    };
                    WorthQueryWriteReceipt::preview(
                        &self.label,
                        self.pending_commands.len() + 1,
                        &concrete,
                        self.runtime.current_snapshot_identity()?,
                    )
                    .with_symbolic_target_reference(
                        reference,
                        resolved_target.entity_identity().clone(),
                        resolved_target.target_collection_identity().cloned(),
                    )
                }
                _ => WorthQueryWriteReceipt::preview(
                    &self.label,
                    self.pending_commands.len() + 1,
                    &command,
                    self.runtime.current_snapshot_identity()?,
                ),
            };
            if let Some(reference) = symbolic_target_reference.as_ref() {
                record_preview_symbolic_target(&mut symbolic_targets, reference, &receipt);
            }
            self.pending_commands.push(command);
            self.writes.push(receipt.clone());
            self.route_preview_execution(&receipt);
            receipts.push(receipt);
        }
        WorthQueryBatchWriteReceipt::from_write_receipts(receipts)
    }
}

fn deny_preview_continuity(command: &WorthQueryWriteCommand) -> Result<(), WorthQueryRuntimeError> {
    let Some(intent) = command.continuity_intent() else {
        return Ok(());
    };
    let denial = crate::runtime::WorthQueryContinuityMutationDenial::new(
        intent,
        command.existing_truth_binding(),
        crate::runtime::WorthQueryContinuityMutationDenialKind::RequiresAuthoritativeLane,
        "continuity-aware mutation currently requires the authoritative bridge-backed lane",
    );
    Err(WorthQueryRuntimeError::MutationContinuityDenied(denial))
}

fn deny_preview_assertion(command: &WorthQueryWriteCommand) -> Result<(), WorthQueryRuntimeError> {
    if matches!(
        command,
        WorthQueryWriteCommand::AssertExistingAspects { .. }
            | WorthQueryWriteCommand::VerifyThenUpdateExistingAspects { .. }
            | WorthQueryWriteCommand::VerifyThenDeleteExistingAspects { .. }
            | WorthQueryWriteCommand::VerifyExistingAspects { .. }
    ) {
        return Err(
            WorthQueryRuntimeError::ExistingTruthAssertionRequiresAuthorityLane {
                required_lane: crate::runtime::WorthQueryAuthorityLane::AuthoritativeTruth,
            },
        );
    }
    Ok(())
}
