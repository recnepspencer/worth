//! Native admission of a recovered codec retirement and exact program update.
use super::super::super::{DurabilityRecoveryAuthority, RecoveredRelationalRuntimeAuthority};
use crate::branch::AdmittedRelationalBranchBasis;
use crate::mvcc::{
    BranchBoundRelationalTransaction, RelationalInstalledApplicationPublicationAuthority,
    RelationalOperationControl, RelationalTransactionIntent,
};
use crate::runtime::{
    RecoveredCheckpointProgramActivationUpdate,
    RelationalSchemaTransitionAdmissionDenial as Denial,
    RelationalSchemaTransitionAdmissionDenialKind as Kind,
};
use crate::transactions::data::{
    AspectFieldPatch, DeleteEntityIntent, EntityMutationIntent, MutationIntent,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};

impl DurabilityRecoveryAuthority<'_> {
    /// Bind a source publisher occurrence to the exact recovered image. Target
    /// installation receipts cannot stand in for the actual source authority.
    pub fn issue_recovered_codec_source_publication_owner(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovered: &RecoveredRelationalRuntimeAuthority,
    ) -> Result<
        (
            worth_proof::InstalledApplicationPublicationOwner,
            RelationalInstalledApplicationPublicationAuthority,
        ),
        Denial,
    > {
        crate::execution::check_request(execution).map_err(Denial::request)?;
        if recovered.codec_transition.is_none()
            || recovered.runtime_instance_id != self.runtime.runtime_instance_id()
            || self
                .runtime
                .history()
                .branch_cells_snapshot()
                .iter()
                .any(|cell| {
                    recovered.recovered_branch_images.get(&cell.branch_id)
                        != Some(&(cell.observation.clone(), cell.truth_version))
                })
        {
            return Err(deny(
                Kind::RecoveredAuthorityMismatch,
                "source publisher requires the exact unpublished recovered image",
            ));
        }
        Ok(self
            .runtime
            .issue_recovered_codec_source_publication_owner())
    }

    /// Stage only native-proven whole output retirement and the separately
    /// authenticated exact program activation patch. Deletion plans use source
    /// contracts; the prepared successor owns the target schema and indexes.
    #[allow(clippy::too_many_arguments)]
    pub fn begin_recovered_codec_transition(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovered: &mut RecoveredRelationalRuntimeAuthority,
        basis: &AdmittedRelationalBranchBasis,
        control: RelationalOperationControl,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        activation_proof: worth_proof::AdmittedCheckpointProgramActivation,
        activation: RecoveredCheckpointProgramActivationUpdate,
    ) -> Result<BranchBoundRelationalTransaction, Denial> {
        crate::execution::check_request(execution).map_err(Denial::request)?;
        if !recovered.owns_image(basis.descriptor())
            || recovered.runtime_instance_id != self.runtime.runtime_instance_id()
        {
            return Err(deny(
                Kind::RecoveredAuthorityMismatch,
                "codec transition source is not the exact recovered image",
            ));
        }
        let activation_commitment = self.checkpoint_program_activation_commitment(
            execution,
            recovered,
            basis,
            &activation,
        )?;
        let state = recovered.codec_transition.as_mut().ok_or_else(|| {
            deny(
                Kind::RecoveredAuthorityMismatch,
                "recovery retained no codec target",
            )
        })?;
        let source_authority = basis.inner.root.schema_authority();
        let source = source_authority.registry();
        let target = &state.target_registry;
        if installed.runtime_instance_id != self.runtime.runtime_instance_id()
            || installed.custom_invariant_generation
                != self
                    .runtime
                    .schema_contract_runtime
                    .custom_invariant_generation
            || installed.schema_digest
                != crate::schema::data::schema_authority_snapshot_digest_bytes(
                    &source.authority_snapshot(),
                )
            || !installed.admits_checkpoint_program_activation(&activation_proof)
        {
            return Err(deny(
                Kind::RecoveredAuthorityMismatch,
                "checkpoint activation has a foreign installed publisher occurrence",
            ));
        }
        if activation_proof.activation_commitment() != &activation_commitment {
            return Err(deny(
                Kind::RecoveredAuthorityMismatch,
                "checkpoint activation proof belongs to another exact update",
            ));
        }
        let changes = crate::runtime::codec_field_changes(execution, source, target)?;
        let (transition, transition_memory) =
            super::schema_proposal::derive_codec_transition(execution, source, target, &changes)?;
        self.runtime
            .indexes
            .with_definitions(|indexes| {
                super::source_schema::admit_supported_consequences(
                    self.runtime,
                    indexes.values().map(std::sync::Arc::as_ref),
                    &changes,
                )
            })
            .map_err(|detail| deny(Kind::UnsupportedCodecConsequence, detail))?;
        let retirement = crate::history::data::derive_codec_retirement_closure(
            execution,
            self.runtime,
            basis,
            &changes,
        )
        .map_err(|stop| match stop {
            crate::history::data::CodecRetirementClosureStop::Request(cause) => {
                Denial::request(cause)
            }
            crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
                deny(Kind::CodecClosureRejected, detail)
            }
        })?;
        let intent =
            RelationalTransactionIntent::ordinary().with_schema_transition(transition, None);
        let mut transaction = self
            .runtime
            .begin_branch_transaction_with_control(basis, intent, control)
            .map_err(|error| deny(Kind::TransactionAdmission, format!("{error:?}")))?;
        transaction.schema_authority_input = Some(
            crate::schema::SchemaContinuityAuthorityInput::from_registry(
                target.clone(),
                source_authority.descriptor_semantics_version(),
                self.runtime
                    .config()
                    .schema
                    .descriptor_canonical_basis_policy
                    .current_write_version(),
            ),
        );
        let mut batch = WorkerIntentBatch::new("recovered-derived-codec-retirement");
        for entity_id in &retirement.entities {
            batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent {
                    entity_id: *entity_id,
                },
            )));
        }
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: activation.entity_id,
                fields: AspectFieldPatch::from(std::collections::BTreeMap::from([(
                    activation.field_locator.clone(),
                    activation.replacement.clone(),
                )])),
            },
        )));
        transaction
            .push_batch(batch, execution.allocation_policy())
            .map_err(Denial::staging)?;
        crate::execution::check_request(execution).map_err(Denial::request)?;
        let custody = super::RecoveredCodecPublicationCustody::new(transition_memory);
        transaction.recovered_codec_publication_custody = Some(std::sync::Arc::clone(&custody));
        state.publication_custody = Some(custody);
        state.activation = Some(activation);
        state.retirement = Some(retirement);
        state.transaction_id = Some(transaction.transaction_id());
        Ok(transaction)
    }
}

fn deny(kind: Kind, detail: impl Into<String>) -> Denial {
    Denial::new(kind, detail)
}
