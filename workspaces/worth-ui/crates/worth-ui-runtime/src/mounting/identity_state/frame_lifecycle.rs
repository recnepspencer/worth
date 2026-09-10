use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
    UiSurfaceBindingGeneration,
};

use super::{UiMountedIdentityFrameCandidate, UiMountedIdentityState};
use crate::mounting::{
    UiMountedFrameIdentityView, UiMountedFramePublicationReceipt, UiMountedFrameReuseContract,
    UiMountedFrameReuseWitness, UiMountedGraphNodeHandle, UiMountedIdentityDenial,
    UiMountedIdentityView, UiMountedInstanceIdentityView, UiPreparedMountedFrame,
    UiSurfaceBindingIdentityView,
};

#[path = "frame_lifecycle/reconciliation.rs"]
mod reconciliation;

impl UiMountedIdentityState {
    pub(crate) fn admit_prepared_frame_authority(
        &self,
        frame: UiPreparedMountedFrame,
    ) -> Result<
        super::UiAuthorityAdmittedMountedFrame,
        crate::mounting::UiMountedPresentationAdmissionRejection,
    > {
        let contract = frame.reuse_contract();
        let basis_is_current = contract.host_session() == self.host_session_identity.as_u64()
            && contract.graph_world() == self.world_identity.diagnostic_value()
            && contract.mounted_semantic_revision() == self.semantic_revision
            && contract.surface_binding_revision() == self.binding_revision;
        if basis_is_current {
            Ok(super::UiAuthorityAdmittedMountedFrame::new(frame))
        } else {
            Err(crate::mounting::UiMountedPresentationAdmissionRejection::new(
                frame,
                crate::mounting::UiMountedPresentationAdmissionDenial::PreparedFrameBasisChanged,
            ))
        }
    }

    pub(in crate::mounting) fn has_published_frame(&self) -> bool {
        self.current_publication.is_some()
    }

    pub(in crate::mounting) fn current_frame_identity(&self) -> Option<UiMountedFrameIdentity> {
        self.current_frame
    }

    pub(crate) fn advance_frame(
        &mut self,
    ) -> Result<UiMountedFrameIdentity, UiMountedIdentityDenial> {
        let candidate = self.prepare_frame_candidate()?;
        let frame = candidate.frame();
        self.publish_frame_candidate(candidate);
        Ok(frame)
    }

    pub(in crate::mounting) fn prepare_frame_candidate(
        &self,
    ) -> Result<UiMountedIdentityFrameCandidate, UiMountedIdentityDenial> {
        self.prepare_frame_candidate_for(self.mounted_instance_membership.clone())
    }

    pub(in crate::mounting) fn prepare_frame_candidate_for(
        &self,
        presented_instances: crate::runtime::persistent_index::UiPersistentOrdSet<
            UiMountedInstanceIdentity,
        >,
    ) -> Result<UiMountedIdentityFrameCandidate, UiMountedIdentityDenial> {
        let frame = UiMountedFrameIdentity::mint_unbound()
            .map_err(|_| UiMountedIdentityDenial::IdentityExhausted)?;
        let receipt_basis =
            super::super::UiMountedNodeReceiptBasis::mint(frame, presented_instances)
                .map_err(|_| UiMountedIdentityDenial::IdentityExhausted)?;
        Ok(UiMountedIdentityFrameCandidate { receipt_basis })
    }

    pub(in crate::mounting) fn publish_frame_candidate(
        &mut self,
        candidate: UiMountedIdentityFrameCandidate,
    ) {
        let frame = candidate.frame();
        self.current_frame = Some(frame);
        self.current_receipt_basis = Some(candidate.receipt_basis);
        self.unprojected_semantic_predecessor = self.semantic_predecessor().cloned();
        self.unprojected_appearance_predecessor = self.appearance_predecessor().cloned();
        self.unprojected_pointer_predecessor = self.pointer_predecessor().cloned();
        self.current_projection = None;
        self.current_manifest = None;
        self.current_core = None;
        self.current_publication = None;
        self.current_trace_source = None;
        self.current_reuse_contract = None;
    }

    pub(crate) fn publish_presented_frame(
        &mut self,
        frame: UiPreparedMountedFrame,
        receipt: UiMountedFramePublicationReceipt,
    ) {
        let trace_source = frame.identity_trace_basis().authored_source().clone();
        let (candidate, manifest, core, reuse_contract) = frame.into_publication_parts();
        self.current_manifest = Some(manifest);
        self.current_core = Some(core);
        let (owner, identity_candidate, projection_changes) = candidate.into_parts();
        self.peak_qualified_layouts = self
            .peak_qualified_layouts
            .max(owner.projection().qualified_layout_count());
        let frame = identity_candidate.frame();
        let committed = self.commit_projection_changes(&projection_changes);
        debug_assert!(committed);
        self.current_frame = Some(frame);
        self.current_receipt_basis = Some(identity_candidate.receipt_basis);
        self.current_projection = Some(std::rc::Rc::new(owner));
        self.unprojected_semantic_predecessor = None;
        self.unprojected_appearance_predecessor = None;
        self.unprojected_pointer_predecessor = None;
        self.current_publication = Some(receipt);
        self.current_trace_source = Some(trace_source);
        self.current_reuse_contract = Some(reuse_contract);
    }

    pub(crate) fn publish_reconciled_frame(
        &mut self,
        frame: UiPreparedMountedFrame,
        receipt: UiMountedFramePublicationReceipt,
    ) {
        debug_assert_eq!(
            self.current_frame,
            frame.presentation_delta_source().predecessor()
        );
        let trace_source = frame.identity_trace_basis().authored_source().clone();
        let (candidate, manifest, core, reuse_contract) = frame.into_publication_parts();
        self.current_manifest = Some(manifest);
        self.current_core = Some(core);
        let (owner, identity_candidate, projection_changes) = candidate.into_parts();
        self.peak_qualified_layouts = self
            .peak_qualified_layouts
            .max(owner.projection().qualified_layout_count());
        let frame = identity_candidate.frame();
        let committed = self.commit_projection_changes(&projection_changes);
        debug_assert!(committed);
        self.current_frame = Some(frame);
        self.current_receipt_basis = Some(identity_candidate.receipt_basis);
        self.current_projection = Some(std::rc::Rc::new(owner));
        self.unprojected_semantic_predecessor = None;
        self.unprojected_appearance_predecessor = None;
        self.unprojected_pointer_predecessor = None;
        self.current_publication = Some(receipt);
        self.current_trace_source = Some(trace_source);
        self.current_reuse_contract = Some(reuse_contract);
    }

    pub(crate) fn publication_receipt(&self) -> Option<&UiMountedFramePublicationReceipt> {
        self.current_publication.as_ref()
    }

    pub(crate) fn current_requires_binding(&self, binding: UiSurfaceBindingGeneration) -> bool {
        self.current_manifest.as_ref().is_some_and(|manifest| {
            manifest
                .surfaces()
                .iter()
                .any(|requirement| requirement.binding() == binding)
        })
    }

    pub(crate) fn current_binding_requirement(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> Option<worth_ui_host_contract::UiMountedSurfaceBindingRequirement> {
        self.current_manifest.as_ref().and_then(|manifest| {
            manifest
                .surfaces()
                .iter()
                .copied()
                .find(|requirement| requirement.binding() == binding)
        })
    }

    pub(crate) fn classify_reuse(
        &self,
        contract: UiMountedFrameReuseContract,
    ) -> super::super::UiMountedFrameReuse {
        match (&self.current_reuse_contract, &self.current_publication) {
            (Some(current), Some(publication)) if current == &contract => {
                super::super::UiMountedFrameReuse::Exact(UiMountedFrameReuseWitness::mint(
                    contract,
                    publication.clone(),
                ))
            }
            _ => super::super::UiMountedFrameReuse::ComparisonRequired(contract),
        }
    }

    pub(crate) fn instances_for(
        &self,
        handle: UiMountedGraphNodeHandle,
    ) -> Result<Box<[UiMountedInstanceIdentity]>, UiMountedIdentityDenial> {
        self.require_handle(handle)?;
        Ok(self
            .by_graph
            .get(&handle.graph_node_identity())
            .into_iter()
            .flat_map(|instances| instances.iter().copied())
            .collect::<Vec<_>>()
            .into_boxed_slice())
    }

    pub(crate) fn validate_binding(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> Result<(), UiMountedIdentityDenial> {
        self.bindings
            .values()
            .any(|record| record.view.binding_generation() == binding)
            .then_some(())
            .ok_or(UiMountedIdentityDenial::UnknownSurfaceBinding)
    }

    pub(crate) fn validate_current_frame(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        (self.current_frame == Some(frame))
            .then_some(())
            .ok_or(UiMountedIdentityDenial::FrameNotCurrent)
    }

    pub(crate) fn validate_current_receipt(
        &self,
        instance: UiMountedInstanceIdentity,
        receipt: UiMountedNodeReceiptIdentity,
    ) -> Result<(), UiMountedIdentityDenial> {
        let current = self
            .current_receipt_basis
            .as_ref()
            .and_then(|basis| basis.receipt_for(instance))
            .ok_or(UiMountedIdentityDenial::NodeReceiptNotCurrent)?;
        (current == receipt)
            .then_some(())
            .ok_or(UiMountedIdentityDenial::NodeReceiptNotCurrent)
    }

    pub(crate) fn view(&self) -> UiMountedIdentityView {
        let mounted_instances = self
            .visible_order
            .iter()
            .filter_map(|identity| {
                self.instances.get(identity).map(|record| {
                    UiMountedInstanceIdentityView::new(*identity, record.basis.clone())
                })
            })
            .collect();
        let surface_bindings = self.bindings.values().map(|record| record.view).collect();
        let frame_receipts = self
            .current_receipt_basis
            .as_ref()
            .into_iter()
            .flat_map(|basis| {
                let frame = basis.frame();
                basis.receipts().map(move |(instance, receipt)| {
                    UiMountedFrameIdentityView::new(frame, instance, receipt)
                })
            })
            .collect();
        UiMountedIdentityView::new(
            mounted_instances,
            surface_bindings,
            self.current_frame,
            frame_receipts,
        )
    }
}
