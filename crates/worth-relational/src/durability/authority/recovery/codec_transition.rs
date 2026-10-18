//! Restore historical source meaning while retaining a distinct native target.
mod publication_custody;
pub(crate) use publication_custody::{
    AdmittedRecoveredCodecPublication, RecoveredCodecPublicationCustody,
};
mod activation_commitment;
mod schema_proposal;
mod source_schema;
mod target_publication;
#[cfg(test)]
mod tests;
mod transition_admission;

use super::super::{DurabilityRecoveryAuthority, RecoveredRelationalRuntimeAuthority};
use crate::durability::data::{RelationalNativeCheckpoint, RelationalRecoveryStop};
use crate::runtime::{RecoveredCheckpointProgramActivationUpdate, RelationalRuntime};
use crate::schema::data::RelationalSchemaRegistry;

#[derive(Debug)]
pub(in crate::durability::authority) struct RecoveredCodecTransitionState {
    pub(super) target_registry: RelationalSchemaRegistry,
    pub(super) activation: Option<RecoveredCheckpointProgramActivationUpdate>,
    pub(in crate::durability::authority) retirement:
        Option<crate::history::data::OwnedCodecRetirementClosure>,
    pub(super) publication_custody: Option<std::sync::Arc<RecoveredCodecPublicationCustody>>,
    pub(super) transaction_id: Option<crate::transactions::data::TransactionId>,
}

impl DurabilityRecoveryAuthority<'_> {
    /// Restore an unpublished candidate under its exact historical source
    /// contracts. A codec target remains sealed inside native recovery custody;
    /// ordinary restoration and initial-schema readmission remain exact.
    pub fn restore_native_checkpoint_for_codec_transition_with_authority(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalNativeCheckpoint,
    ) -> Result<
        (
            crate::runtime::RecoveryOutcome,
            RecoveredRelationalRuntimeAuthority,
        ),
        RelationalRecoveryStop,
    > {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let destination = self.runtime.admit_codec_recovery_destination();
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        destination.map_err(|detail| {
            crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::ReplayFailure,
                detail,
            )
        })?;
        let checkpoint = crate::durability::log::native_file_codec::decode_checkpoint(
            execution,
            native.bytes(),
        )?
        .checkpoint;
        let target_registry = self.runtime.config().schema.registry;
        let source_registry = source_schema::readmit_current_source_schema(
            &checkpoint,
            self.runtime.main_branch_identity().branch_id(),
        )?;
        let changes =
            crate::runtime::codec_field_changes(execution, &source_registry, &target_registry)
                .map_err(|denial| match denial.request_cause() {
                    Some(cause) => RelationalRecoveryStop::Request(*cause),
                    None => RelationalRecoveryStop::Durability(source_schema::schema_denial(
                        denial.detail().to_owned(),
                    )),
                })?;
        if changes.is_empty() {
            return self.restore_decoded_native_checkpoint_with_authority(
                execution,
                checkpoint,
                native.bytes().len(),
            );
        }
        source_schema::admit_single_codec_head(&checkpoint)
            .map_err(source_schema::schema_denial)?;
        source_schema::admit_supported_consequences(
            self.runtime,
            &checkpoint.index_definitions,
            &changes,
        )
        .map_err(source_schema::schema_denial)?;
        let mut config = self.runtime.config();
        config.schema.registry = source_registry;
        let initial_authority = self.runtime.initial_schema_authority_snapshot();
        let mut source = RelationalRuntime::new(config);
        source
            .unshared_state_mut()
            .expect("new recovery admission runtime is exclusive")
            .commit_strategies
            .executors = self.runtime.commit_strategy_executor_registry().clone();
        source.restore_initial_schema_authority_after_recovery(initial_authority);
        let (outcome, mut recovered) = source
            .durability_recovery()
            .restore_decoded_native_checkpoint_with_authority(
                execution,
                checkpoint,
                native.bytes().len(),
            )?;
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        recovered.codec_transition = Some(RecoveredCodecTransitionState {
            target_registry,
            activation: None,
            retirement: None,
            publication_custody: None,
            transaction_id: None,
        });
        *self.runtime = source;
        Ok((outcome, recovered))
    }
}
