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
        let admitted_ids = execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(lease,None, |context| {
                self.runtime.indexes.with_definitions(|definitions| {
                    let bytes = definitions.len().checked_mul(std::mem::size_of::<crate::indexes::data::DerivedIndexId>())
                        .and_then(|n| u64::try_from(n).ok()).ok_or(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)?;
                    context.checkpoint(u64::try_from(definitions.len()).map_err(|_| crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow)?).map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                    let memory = worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease,bytes)
                        .map_err(crate::execution::denial::lease_denial)?;
                    let mut ids = Vec::with_capacity(definitions.len());
                    ids.extend(definitions.keys().copied());
                    Ok::<_,crate::execution::RelationalExecutionDenialCause>((ids,memory))
                })
            })
        }).map_err(|stop| Denial::CodecRequest(stop.into()))
            .and_then(|result| result.map_err(|stop| Denial::CodecRequest(stop.into())))
            .and_then(|result| result.0.map_err(Denial::CodecRequest));
        crate::execution::check_request(execution).map_err(Denial::CodecRequest)?;
        let (index_ids, _index_id_memory) = admitted_ids?;
        if !index_ids.is_empty() {
            // Codec changes invalidate representation-sensitive postings. Build
            // from the already fenced target root, never old index equality.
            let maximum = usize::try_from(execution.work_ceiling()).unwrap_or(usize::MAX);
            self.runtime
                .index_authority()
                .prepare_for_candidate(
                    candidate,
                    &index_ids,
                    None,
                    crate::indexes::data::DerivedIndexMaintenanceBudget {
                        maximum_work_units: maximum,
                        maximum_cold_record_slots: maximum,
                        maximum_derived_rows: maximum,
                    },
                    execution,
                )
                .map_err(Denial::CodecIndexPreparation)?;
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
