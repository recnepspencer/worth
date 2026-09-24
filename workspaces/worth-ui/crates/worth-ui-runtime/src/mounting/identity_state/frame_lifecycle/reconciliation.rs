use super::*;

type UiReconciledBindingView = (UiSurfaceBindingGeneration, UiSurfaceBindingIdentityView);

impl super::super::UiAuthorityAdmittedMountedFrame {
    pub(in crate::mounting::identity_state) fn new(
        frame: crate::mounting::UiPreparedMountedFrame,
    ) -> Self {
        Self { frame }
    }

    pub(in crate::mounting) fn into_frame(self) -> crate::mounting::UiPreparedMountedFrame {
        self.frame
    }
}

impl UiMountedIdentityState {
    pub(crate) fn prepare_current_reconciliation_frame(
        &self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        protocol: worth_ui_host_contract::UiHostProtocolAgreement,
        capability_report: &worth_ui_host_contract::WorthUiHostCapabilityReport,
    ) -> Result<super::super::UiAuthorityAdmittedMountedFrame, UiMountedIdentityDenial> {
        crate::mounting::UiPreparedMountedFrame::reconcile_current(
            self,
            replacements,
            protocol,
            capability_report,
        )
        .map(super::super::UiAuthorityAdmittedMountedFrame::new)
    }

    pub(in crate::mounting) fn assemble_current_reconciliation_frame(
        &self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        protocol: worth_ui_host_contract::UiHostProtocolAgreement,
        capability_report: &worth_ui_host_contract::WorthUiHostCapabilityReport,
    ) -> Result<crate::mounting::UiAssembledMountedFrame, UiMountedIdentityDenial> {
        if replacements.is_empty() {
            return Err(UiMountedIdentityDenial::ReconciliationBasisMismatch);
        }
        let replacement_views = self.resolve_reconciliation_bindings(replacements)?;
        let published = self
            .frame
            .published()
            .ok_or(UiMountedIdentityDenial::NoPublishedMountedFrame)?;
        let current_core = published.core;
        let admission = crate::mounting::UiPreparedMountedFrameAdmission {
            text_publication: None,
            text_publication_work: 0,
            candidate: self.reconciled_projection_candidate(&replacement_views)?,
            generation: published.publication.generation().clone(),
            manifest: self.reconciled_manifest(&replacement_views)?,
            graph_world: current_core.graph_world(),
            allocation_truth_revision: current_core.allocation_truth_revision(),
            trace_source: published.trace_source.clone(),
            reuse_contract: published.reuse_contract.reconciled(
                self.binding_revision,
                protocol,
                capability_report.observation_generation(),
                capability_report.profile_identity_digest(),
            ),
        };
        crate::mounting::UiAssembledMountedFrame::admit(admission)
            .map_err(|_| UiMountedIdentityDenial::ReconciliationBasisMismatch)
    }

    pub(in crate::mounting) fn resolve_reconciliation_bindings(
        &self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<Vec<UiReconciledBindingView>, UiMountedIdentityDenial> {
        let distinct = replacements
            .iter()
            .map(|replacement| replacement.affected())
            .collect::<std::collections::BTreeSet<_>>();
        if distinct.len() != replacements.len() {
            return Err(UiMountedIdentityDenial::ReconciliationBasisMismatch);
        }
        replacements
            .iter()
            .map(|replacement| {
                self.bindings
                    .values()
                    .find(|record| record.view.binding_generation() == replacement.replacement())
                    .map(|record| (replacement.affected(), record.view))
                    .ok_or(UiMountedIdentityDenial::UnknownSurfaceBinding)
            })
            .collect()
    }

    fn reconciled_projection_candidate(
        &self,
        replacement_views: &[UiReconciledBindingView],
    ) -> Result<crate::mounting::UiProjectedMountedFrameCandidate, UiMountedIdentityDenial> {
        let published = self
            .frame
            .published()
            .ok_or(UiMountedIdentityDenial::NoPublishedMountedFrame)?;
        let current_owner = &*published.projection;
        let current_projection = current_owner.projection();
        let current_instances = current_projection.mounted_instances().collect::<Vec<_>>();
        let identity_candidate = super::super::UiMountedIdentityFrameCandidate {
            receipt_basis: published.receipts.clone(),
        };
        let reconciled_surfaces = replacement_views
            .iter()
            .map(|(_, replacement)| replacement.semantic_surface_identity())
            .collect::<Vec<_>>();
        let projection_changes = self
            .projection_change_snapshot()
            .for_reconciliation(&current_instances, &reconciled_surfaces)
            .ok_or(UiMountedIdentityDenial::ReconciliationBasisMismatch)?;
        let projection = current_projection
            .rebound(identity_candidate.frame(), replacement_views)
            .map_err(|_| UiMountedIdentityDenial::ReconciliationBasisMismatch)?;
        Ok(crate::mounting::UiProjectedMountedFrameCandidate {
            owner: crate::mounting::UiMountedProjectionFrameOwner::new(
                std::rc::Rc::new(projection),
                crate::mounting::UiMountedAppearanceFrameState::fork(
                    Some(current_owner.appearance()),
                    std::rc::Rc::new(
                        crate::mounting::UiMountedAppearanceProjectionSelection::empty(),
                    ),
                ),
                current_owner.pointer.clone(),
            ),
            identity_candidate,
            projection_changes,
            presentation_predecessor: Some(published.receipts.frame()),
            presentation_changed_instances: current_instances.clone().into(),
            presentation_node_changed_instances: current_instances.into(),
        })
    }

    fn reconciled_manifest(
        &self,
        replacement_views: &[UiReconciledBindingView],
    ) -> Result<worth_ui_host_contract::UiMountedFrameManifest, UiMountedIdentityDenial> {
        let current_manifest = &self
            .frame
            .published()
            .ok_or(UiMountedIdentityDenial::NoPublishedMountedFrame)?
            .manifest;
        let requirements = current_manifest
            .surfaces()
            .iter()
            .map(|requirement| {
                replacement_views
                    .iter()
                    .find(|(affected, _)| requirement.binding() == *affected)
                    .map(|(_, replacement)| crate::mounting::binding_requirement(*replacement))
                    .unwrap_or(*requirement)
            })
            .collect();
        let manifest = worth_ui_host_contract::UiMountedFrameManifest::new(
            requirements,
            current_manifest.lane_contributions().to_vec(),
        );
        Ok(manifest)
    }
}
