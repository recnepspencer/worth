//! Authoritative delivery over the actual admitted branch execution state.
use super::{BridgeConditionalDenial, BridgeConditionalDenialKind};
use worth_proof::TransitionOutcome;
impl super::execution_view::BridgeConditionalExecutionView<'_> {
    pub fn deliver_authoritative_change(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        dependency_ordinal: usize,
        request: crate::adapter::RelationalCommittedPatchRequest,
    ) -> Result<crate::correspondence::CorrespondenceDeliveryOutcome, BridgeConditionalDenial> {
        let lowering = &signal_basis.lowering;
        if lowering.signal_contract().graph_instance_id() != self.signal_graph_instance_id {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::StaleLowering,
                "conditional lowering belongs to another Signal graph",
            ));
        }
        self.require_live_installed_lowering(lowering)?;
        let correspondence = lowering
            .correspondences
            .iter()
            .find(|item| item.dependency().dependency_ordinal() == dependency_ordinal)
            .ok_or_else(dependency_ordinal_denial)?;
        crate::execution_contact::admit(execution).map_err(|cause| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::ExecutionDenied(cause),
                cause.label(),
            )
        })?;
        let requested_commit = request.commit_identity().clone();
        let envelope =
            match self
                .bridge
                .committed_patch_source
                .load_committed_patch(request, execution)
            {
                Ok(envelope) => envelope,
                Err(cause) => return Ok(TransitionOutcome::Failed(
                    crate::correspondence::BridgeCorrespondenceAdmissionFailure::SourceLoadFailed(
                        cause,
                    ),
                )),
            };
        let mut counters = crate::correspondence::CorrespondenceDeliveryCounters::zero();
        counters.source_load_attempts = 1;
        counters.source_envelopes_loaded = 1;
        if envelope.commit_identity() != &requested_commit {
            counters.failed_deliveries = 1;
            return Ok(TransitionOutcome::Denied(
                crate::correspondence::BridgeCorrespondenceDeliveryDenial::new(
                    crate::correspondence::BridgeCorrespondenceDenialKind::CommittedPatchRequestMismatch,
                    counters,
                ),
            ));
        }
        if envelope.producer_metadata().authority_kind()
            != crate::facade::BridgeProducerAuthorityKind::RegisteredAuthoritativeSource
        {
            counters.failed_deliveries = 1;
            return Ok(TransitionOutcome::Denied(
                crate::correspondence::BridgeCorrespondenceDeliveryDenial::new(
                    crate::correspondence::BridgeCorrespondenceDenialKind::AuthoritativeSourceMismatch,
                    counters,
                ),
            ));
        }
        Ok(self.deliver_prepared_correspondence(
            execution,
            signal_basis,
            correspondence,
            &correspondence.targets,
            &envelope,
            counters,
        ))
    }

    pub(super) fn deliver_prepared_correspondence(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        correspondence: &crate::correspondence::BridgeInstalledSemanticCorrespondence,
        targets: &crate::correspondence::ProvenCorrespondenceTargets,
        envelope: &crate::facade::BridgeCommittedPatchEnvelope,
        counters: crate::correspondence::CorrespondenceDeliveryCounters,
    ) -> crate::correspondence::CorrespondenceDeliveryOutcome {
        let prepared = match self
            .bridge
            .prepare_installed_correspondence_envelope_to_targets(
                self.allocations,
                correspondence,
                targets,
                self.signal_graph_instance_id,
                envelope,
                counters,
            ) {
            TransitionOutcome::Success(prepared) => prepared,
            TransitionOutcome::Denied(denial) => return TransitionOutcome::Denied(denial),
            TransitionOutcome::Deferred(deferred) => return TransitionOutcome::Deferred(deferred),
            TransitionOutcome::Stale(stale) => return TransitionOutcome::Stale(stale),
            TransitionOutcome::RebindRequired(rebind) => {
                return TransitionOutcome::RebindRequired(rebind)
            }
            TransitionOutcome::Failed(failure) => return TransitionOutcome::Failed(failure),
        };
        self.perform_prepared_correspondence(execution, signal_basis, prepared)
    }

    fn perform_prepared_correspondence(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        prepared: crate::correspondence::BridgePreparedCorrespondenceDelivery,
    ) -> crate::correspondence::CorrespondenceDeliveryOutcome {
        let crate::correspondence::BridgePreparedCorrespondenceDelivery {
            mut counters,
            change_set,
            signal,
            target_count,
            node_fan_out,
        } = prepared;
        let Some(prepared_signal) = signal else {
            return TransitionOutcome::Success(
                crate::correspondence::BridgeCorrespondenceDeliveryReceipt::new(
                    counters, change_set, None,
                ),
            );
        };
        let completion = match signal_basis.signal_port.deliver_committed_patch(
            execution,
            signal_basis.lowering.signal_contract(),
            prepared_signal.signal_delivery_request(),
        ) {
            Ok(completion) => completion,
            Err(error) => return map_signal_delivery_denial(error),
        };
        counters.signal_capability_admissions = completion.target_count();
        counters.signal_seeds_emitted = completion.target_count();
        counters.node_fan_out = node_fan_out;
        counters.slots_touched = target_count;
        let transition = completion.successor_transition().clone();
        TransitionOutcome::Success(
            crate::correspondence::BridgeCorrespondenceDeliveryReceipt::new(
                counters,
                change_set,
                Some(prepared_signal),
            )
            .with_conditional_transition(transition),
        )
    }
}

fn map_signal_delivery_denial(
    denial: worth_signal::facade::branch::SignalCommittedPatchDeliveryDenial,
) -> crate::correspondence::CorrespondenceDeliveryOutcome {
    use worth_signal::facade::branch::SignalCommittedPatchDeliveryDenial as Denial;
    match denial {
        native @ (Denial::StaleBasisAdmission
        | Denial::DefinitionReadmissionRequired
        | Denial::DefinitionMismatch
        | Denial::ForeignGraph
        | Denial::MissingOrStaleTarget) => TransitionOutcome::RebindRequired(
            crate::correspondence::BridgeCorrespondenceRebindRequired::ConditionalService(native),
        ),
        native @ (Denial::OwnerUnavailable(_)
        | Denial::OwnerAdmission(_)
        | Denial::EmptyChangeSet
        | Denial::ForeignContractTarget
        | Denial::DuplicateTarget
        | Denial::SuccessorCaptureCapacityExhausted
        | Denial::SuccessorCaptureWorkExhausted { .. }
        | Denial::SuccessorCaptureUnavailable
        | Denial::SignalMutation(_)) => TransitionOutcome::Failed(
            crate::correspondence::BridgeCorrespondenceAdmissionFailure::SignalMutationFailed(
                crate::correspondence::BridgeCorrespondenceSignalFailure::CommittedPatch(native),
            ),
        ),
    }
}

pub(super) fn dependency_ordinal_denial() -> BridgeConditionalDenial {
    BridgeConditionalDenial::new(
        BridgeConditionalDenialKind::DependencyOrdinalMismatch,
        "conditional declaration does not retain that dependency ordinal",
    )
}
