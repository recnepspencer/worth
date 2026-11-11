use super::authoritative_delivery_view::dependency_ordinal_denial;
use super::{BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeOwnedSignalRuntime};

impl BridgeOwnedSignalRuntime {
    pub fn deliver_owned_authoritative_change(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        dependency_ordinal: usize,
    ) -> Result<crate::correspondence::CorrespondenceDeliveryOutcome, BridgeConditionalDenial> {
        let lowering = &signal_basis.lowering;
        if lowering.signal_contract().graph_instance_id() != self.signal_graph_instance_id {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::StaleLowering,
                "conditional lowering belongs to another owned Signal graph",
            ));
        }
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
        crate::execution_contact::admit(execution).map_err(|cause| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::ExecutionDenied(cause),
                cause.label(),
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
        Ok(self.execution_view().deliver_prepared_correspondence(
            execution,
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
        execution: worth_execution::ExecutionRequest<'_, '_>,
        signal_basis: &super::BridgeConditionalSignalBasisBinding,
        dependency_ordinal: usize,
        request: crate::adapter::RelationalCommittedPatchRequest,
    ) -> Result<crate::correspondence::CorrespondenceDeliveryOutcome, BridgeConditionalDenial> {
        self.execution_view().deliver_authoritative_change(
            execution,
            signal_basis,
            dependency_ordinal,
            request,
        )
    }
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
