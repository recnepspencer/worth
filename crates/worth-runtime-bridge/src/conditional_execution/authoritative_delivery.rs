use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
    BridgeOwnedSignalRuntime,
};
use worth_proof::TransitionOutcome;

impl BridgeOwnedSignalRuntime {
    pub fn deliver_owned_authoritative_change(
        &self,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        dependency_ordinal: usize,
    ) -> Result<crate::correspondence::CorrespondenceDeliveryOutcome, BridgeConditionalDenial> {
        let lowering = &signal_basis.lowering;
        self.validate_lowering_graph(lowering)?;
        self.require_live_installed_lowering(lowering)?;
        let correspondence = lowering
            .correspondences
            .iter()
            .find(|item| item.dependency().dependency_ordinal() == dependency_ordinal)
            .ok_or_else(dependency_ordinal_denial)?;
        let record = correspondence
            .dependency()
            .source_record_identity()
            .ok_or_else(|| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                    "owner-published conditional change requires an exact retained source record",
                )
            })?;
        let envelope = self.reserve_owned_change_envelope(correspondence, record)?;
        let targets = self
            .owned_conditional_targets
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .resolve(correspondence.dependency())?;
        let mut counters = crate::correspondence::CorrespondenceDeliveryCounters::zero();
        counters.correspondence_lookups = 1;
        Ok(self.deliver_prepared_correspondence(
            signal_basis,
            correspondence,
            &targets,
            &envelope,
            counters,
        ))
    }

    fn reserve_owned_change_envelope(
        &self,
        correspondence: &crate::correspondence::BridgeInstalledSemanticCorrespondence,
        record: crate::facade::RelationalBridgeRecordIdentityParts,
    ) -> Result<crate::facade::BridgeCommittedPatchEnvelope, BridgeConditionalDenial> {
        let mut observed = self
            .next_owned_semantic_publication
            .load(std::sync::atomic::Ordering::Acquire);
        loop {
            let publication = observed.checked_add(1).ok_or_else(|| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                    "owner semantic publication identity space was exhausted",
                )
            })?;
            let envelope = owned_change_envelope(correspondence, record, publication)?;
            match self.next_owned_semantic_publication.compare_exchange(
                observed,
                publication,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            ) {
                Ok(_) => return Ok(envelope),
                Err(current) => observed = current,
            }
        }
    }

    pub fn deliver_authoritative_change(
        &self,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        dependency_ordinal: usize,
        request: crate::adapter::RelationalCommittedPatchRequest,
    ) -> Result<crate::correspondence::CorrespondenceDeliveryOutcome, BridgeConditionalDenial> {
        let lowering = &signal_basis.lowering;
        self.validate_lowering_graph(lowering)?;
        self.require_live_installed_lowering(lowering)?;
        let correspondence = lowering
            .correspondences
            .iter()
            .find(|item| item.dependency().dependency_ordinal() == dependency_ordinal)
            .ok_or_else(dependency_ordinal_denial)?;
        let requested_commit = request.commit_identity().clone();
        let envelope =
            match self
                .bridge
                .committed_patch_source
                .load_committed_patch(request)
            {
                Ok(envelope) => envelope,
                Err(_) => return Ok(TransitionOutcome::Failed(
                    crate::correspondence::BridgeCorrespondenceAdmissionFailure::SourceLoadFailed,
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
            signal_basis,
            correspondence,
            &correspondence.targets,
            &envelope,
            counters,
        ))
    }

    fn deliver_prepared_correspondence(
        &self,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        correspondence: &crate::correspondence::BridgeInstalledSemanticCorrespondence,
        targets: &crate::correspondence::ProvenCorrespondenceTargets,
        envelope: &crate::facade::BridgeCommittedPatchEnvelope,
        counters: crate::correspondence::CorrespondenceDeliveryCounters,
    ) -> crate::correspondence::CorrespondenceDeliveryOutcome {
        let prepared = match self
            .bridge
            .prepare_installed_correspondence_envelope_to_targets(
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
        self.perform_prepared_correspondence(signal_basis, prepared)
    }

    fn perform_prepared_correspondence(
        &self,
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

    fn validate_lowering_graph(
        &self,
        lowering: &BridgeInstalledConditionalLowering,
    ) -> Result<(), BridgeConditionalDenial> {
        if lowering.signal_contract().graph_instance_id() != self.signal_graph_instance_id {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::StaleLowering,
                "conditional lowering belongs to another owned Signal graph",
            ));
        }
        Ok(())
    }
}

fn map_signal_delivery_denial(
    denial: worth_signal::facade::branch::SignalCommittedPatchDeliveryDenial,
) -> crate::correspondence::CorrespondenceDeliveryOutcome {
    use worth_signal::facade::branch::SignalCommittedPatchDeliveryDenial as Denial;
    match denial {
        Denial::StaleBasisAdmission => TransitionOutcome::RebindRequired(
            crate::correspondence::BridgeCorrespondenceRebindRequired::ConditionalBasis,
        ),
        Denial::DefinitionReadmissionRequired => TransitionOutcome::RebindRequired(
            crate::correspondence::BridgeCorrespondenceRebindRequired::ConditionalDefinitionReadmission,
        ),
        Denial::DefinitionMismatch => TransitionOutcome::RebindRequired(
            crate::correspondence::BridgeCorrespondenceRebindRequired::ConditionalDefinitionMismatch,
        ),
        Denial::ForeignGraph | Denial::MissingOrStaleTarget => {
            TransitionOutcome::RebindRequired(
                crate::correspondence::BridgeCorrespondenceRebindRequired::ConditionalTarget,
            )
        }
        Denial::OwnerUnavailable(_)
        | Denial::OwnerAdmission(_)
        | Denial::EmptyChangeSet
        | Denial::ForeignContractTarget
        | Denial::DuplicateTarget
        | Denial::SuccessorCaptureCapacityExhausted
        | Denial::SuccessorCaptureWorkExhausted { .. }
        | Denial::SuccessorCaptureUnavailable
        | Denial::SignalMutation(_) => TransitionOutcome::Failed(
            crate::correspondence::BridgeCorrespondenceAdmissionFailure::SignalMutationFailed,
        ),
    }
}

fn dependency_ordinal_denial() -> BridgeConditionalDenial {
    BridgeConditionalDenial::new(
        BridgeConditionalDenialKind::DependencyOrdinalMismatch,
        "conditional declaration does not retain that dependency ordinal",
    )
}

fn owned_change_envelope(
    correspondence: &crate::correspondence::BridgeInstalledSemanticCorrespondence,
    record: crate::facade::RelationalBridgeRecordIdentityParts,
    publication: u64,
) -> Result<crate::facade::BridgeCommittedPatchEnvelope, BridgeConditionalDenial> {
    let dependency = correspondence.dependency();
    let source = owner_source_provenance(correspondence)?;
    let identity = owned_envelope_identity(correspondence, source, publication);
    let target = crate::facade::BridgeCommittedPatchTarget::authoritative_aspect(
        worth_foundational::facade::AspectLocator::new(
            worth_foundational::facade::LocatorAuthority::Authoritative,
            dependency.contract().key().clone(),
        ),
    );
    let item = crate::facade::BridgeCommittedPatchItem::with_relational_semantic_change(
        record,
        target,
        owner_semantic_change(dependency),
    );
    crate::facade::BridgeCommittedPatchEnvelope::new(identity, vec![item]).map_err(|error| {
        BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
            format!("owner semantic publication envelope was denied: {error:?}"),
        )
    })
}

fn owner_source_provenance(
    correspondence: &crate::correspondence::BridgeInstalledSemanticCorrespondence,
) -> Result<crate::facade::BridgeAuthoritativeSourceProvenance, BridgeConditionalDenial> {
    let dependency = correspondence.dependency();
    let profile = correspondence
        .basis()
        .authoritative_source_profile
        .as_ref()
        .ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::DeclarationCorrespondenceMismatch,
                "owner semantic publication requires a registered authoritative source profile",
            )
        })?;
    let source = crate::facade::BridgeAuthoritativeSourceProvenance::from_owner_publication(
        profile.runtime_instance_id(),
        dependency.declared_graph_role(),
        profile.adapter_semantic_identity(),
        dependency.source_basis.as_ref(),
    );
    Ok(
        if let crate::correspondence::BridgeSemanticLocality::SourcePartition(role) =
            dependency.locality()
        {
            crate::facade::BridgeAuthoritativeSourceProvenance::from_owner_partition_publication(
                profile.runtime_instance_id(),
                dependency.declared_graph_role(),
                profile.adapter_semantic_identity(),
                dependency.source_basis.as_ref(),
                role.clone(),
            )
        } else {
            source
        },
    )
}

fn owner_semantic_change(
    dependency: &crate::correspondence::BridgeSemanticDependencyCandidate,
) -> crate::facade::BridgeSemanticAspectChange {
    crate::facade::BridgeSemanticAspectChange::from_authoritative_publication(
        dependency.contract().key().clone(),
        dependency.contract().identity(),
        dependency.contract().revision(),
        dependency.binding().clone(),
        worth_foundational::facade::AuthoritativeAspectChangeKind::WholeAspectSet,
        None,
    )
}

fn owned_envelope_identity(
    correspondence: &crate::correspondence::BridgeInstalledSemanticCorrespondence,
    source: crate::facade::BridgeAuthoritativeSourceProvenance,
    publication: u64,
) -> crate::facade::BridgeCommittedPatchEnvelopeIdentity {
    let label = format!(
        "owned-semantic:{}:{publication}",
        correspondence.basis().signal_graph_instance_id
    );
    crate::facade::BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
        crate::facade::BridgeProducerMetadata::registered_authoritative_source()
            .with_authoritative_source(source),
        crate::facade::TruthCommitIdentity::admit_bridge_owned(format!("commit:{label}")),
        crate::facade::TruthPatchIdentity::admit_bridge_owned(format!("patch:{label}")),
        crate::facade::TruthSnapshotIdentity::admit_bridge_owned(format!("snapshot:{label}")),
        crate::facade::TruthBranchIdentity::admit_bridge_owned("branch:worth-ui-presentation"),
    )
}
