//! Validate the exact native successor and prepare its target index image.
use crate::durability::authority::checkpoint_transition::RecoveredCheckpointTransitionDenial as Denial;
use crate::durability::authority::{
    DurabilityRecoveryAuthority, RecoveredRelationalRuntimeAuthority,
};
use crate::mvcc::PreparedRelationalCommitCandidate;

impl DurabilityRecoveryAuthority<'_> {
    pub(in crate::durability::authority) fn prepare_recovered_codec_target(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovered: &RecoveredRelationalRuntimeAuthority,
        candidate: &mut PreparedRelationalCommitCandidate,
    ) -> Result<Option<super::AdmittedRecoveredCodecPublication>, Denial> {
        let Some(state) = &recovered.codec_transition else {
            return Ok(None);
        };
        crate::execution::check_request(execution).map_err(Denial::CodecRequest)?;
        if state.transaction_id != Some(candidate.transaction_id()) {
            return Err(Denial::CodecTransition(
                "candidate was not staged by native codec recovery".into(),
            ));
        }
        let custody = state.publication_custody.as_ref().ok_or_else(|| {
            Denial::CodecTransition("codec transition has no native publication custody".into())
        })?;
        if !candidate
            .recovered_codec_publication_custody()
            .is_some_and(|candidate| std::sync::Arc::ptr_eq(custody, &candidate))
        {
            return Err(Denial::CodecTransition(
                "candidate has a foreign codec publication occurrence".into(),
            ));
        }
        let activation = state.activation.as_ref().ok_or_else(|| {
            Denial::CodecTransition("codec recovery has no admitted activation patch".into())
        })?;
        let retirement = state.retirement.as_ref().ok_or_else(|| {
            Denial::CodecTransition("codec recovery has no native retirement closure".into())
        })?;
        let (before, after) = candidate
            .index_preparation_roots()
            .ok_or(Denial::CandidateUnavailable)?;
        if after.schema_authority().registry() != &state.target_registry {
            return Err(Denial::CodecTransition(
                "prepared codec successor has a different target schema".into(),
            ));
        }
        crate::history::data::validate_retired_codec_closure(
            execution, &before, &after, retirement, activation,
        )
        .map_err(|stop| match stop {
            crate::history::data::CodecRetirementClosureStop::Request(cause) => {
                Denial::CodecRequest(cause)
            }
            crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
                Denial::CodecTransition(detail)
            }
        })?;
        if !candidate.indexes_already_prepared() {
            // The codec trust boundary uses the same rooted native owner as
            // ordinary publication, after validating its exact retirement closure.
            let change = after.commit_change().ok_or(Denial::CandidateUnavailable)?;
            if let Some(prepared) = self
                .runtime
                .indexes
                .definition_read_binding()
                .prepare_publication(execution, &before, &after, change)
                .map_err(Denial::CodecIndexPreparation)?
            {
                let work = prepared.work();
                candidate
                    .attach_prepared_indexes(prepared)
                    .map_err(|kind| {
                        Denial::CodecIndexPreparation(
                            crate::indexes::data::DerivedIndexMaintenanceDenial { kind, work },
                        )
                    })?;
            }
        }
        crate::execution::check_request(execution).map_err(Denial::CodecRequest)?;
        Ok(Some(super::AdmittedRecoveredCodecPublication::new(
            std::sync::Arc::clone(custody),
        )))
    }

    pub(in crate::durability::authority) fn install_acknowledged_codec_target(
        &mut self,
        recovered: &mut RecoveredRelationalRuntimeAuthority,
    ) {
        if let Some(state) = recovered.codec_transition.take() {
            recovered.performed_codec_retirement = state.retirement;
            self.runtime
                .install_recovered_codec_target(state.target_registry);
        }
    }
}
