use super::*;
use crate::mounting::presented_hit_index::{UiPresentedHitQuery, UiPresentedHitQueryDenial};
use crate::mounting::spatial_index::UiMountedSpatialBudget;
use crate::mounting::UiHitTestSpatialWork;

pub(crate) enum UiPresentedPointLookupDenial {
    Presentation(UiPresentedFrameBasisDenial),
    Query(UiPresentedHitQueryDenial),
}

impl UiMountedFrameRetentionCoordinator {
    pub(in crate::mounting) fn current_presented_hit_row(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        work: &mut UiHitTestSpatialWork,
    ) -> Result<crate::mounting::UiPresentedHitTestRow, UiPresentedFrameBasisDenial> {
        let authority = self.authority.borrow();
        let evidence = authority
            .frames
            .current
            .as_ref()
            .ok_or(UiPresentedFrameBasisDenial::Unknown)?;
        if evidence.frame() != presentation.frame() {
            return Err(UiPresentedFrameBasisDenial::Expired);
        }
        evidence.classify(presentation, None, None)?;
        let basis = evidence.visual_region_basis(presentation.binding());
        let (row, probes) = basis
            .presented_hits
            .for_instance(presentation.binding(), instance);
        work.map_key_probes += probes;
        let row = row.ok_or(UiPresentedFrameBasisDenial::InstanceNotPresented)?;
        let (row, probes) = row.reattributed(evidence.receipts());
        work.map_key_probes += probes;
        Ok(row)
    }

    pub(in crate::mounting) fn current_presentation_for_surface(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        let authority = self.authority.borrow();
        let current = authority.frames.current.as_ref()?;
        let presentation = current
            .current_presentations()
            .find_map(|(candidate, basis)| (candidate == surface).then_some(basis));
        presentation
    }

    pub(in crate::mounting) fn current_semantic_surface_for_presentation(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<worth_ui_host_contract::UiSemanticSurfaceIdentity, UiPresentedFrameBasisDenial>
    {
        let authority = self.authority.borrow();
        let current = authority
            .frames
            .current
            .as_ref()
            .ok_or(UiPresentedFrameBasisDenial::Unknown)?;
        if current.frame() != presentation.frame() {
            return Err(UiPresentedFrameBasisDenial::Expired);
        }
        current.classify(presentation, None, None)?;
        current
            .presentation_receipt()
            .into_iter()
            .flat_map(|receipt| receipt.surfaces())
            .find(|surface| {
                surface.binding() == presentation.binding()
                    && surface.host_surface() == presentation.host_surface()
            })
            .map(|surface| surface.semantic_surface())
            .ok_or(UiPresentedFrameBasisDenial::Unknown)
    }

    pub(in crate::mounting) fn current_hit_evidence(
        &self,
    ) -> Option<Rc<super::super::UiRetainedPresentedFrame>> {
        self.authority.borrow().frames.current.clone()
    }

    pub(in crate::mounting) fn committed_hit_transition(
        &self,
        previous: Option<Rc<super::super::UiRetainedPresentedFrame>>,
    ) -> Option<super::super::UiCommittedPresentedHitTransition> {
        Some(super::super::UiCommittedPresentedHitTransition::new(
            previous,
            self.current_hit_evidence()?,
        ))
    }

    pub(in crate::mounting) fn presented_hit_candidates(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        point: [f64; 2],
        budget: UiMountedSpatialBudget,
    ) -> Result<(UiPresentedFrameBasisRelation, UiPresentedHitQuery), UiPresentedPointLookupDenial>
    {
        let authority = self.authority.borrow();
        let (evidence, relation) = match authority.frame(presentation.frame()) {
            UiMountedRetainedFrameLookup::Found {
                evidence, relation, ..
            } => (evidence, relation),
            UiMountedRetainedFrameLookup::Expired { .. } => {
                return Err(UiPresentedPointLookupDenial::Presentation(
                    UiPresentedFrameBasisDenial::Expired,
                ))
            }
            UiMountedRetainedFrameLookup::Unknown { .. } => {
                return Err(UiPresentedPointLookupDenial::Presentation(
                    UiPresentedFrameBasisDenial::Unknown,
                ))
            }
        };
        evidence
            .classify(presentation, None, None)
            .map_err(UiPresentedPointLookupDenial::Presentation)?;
        let basis = evidence.visual_region_basis(presentation.binding());
        let mut query = basis
            .presented_hits
            .at_point(presentation.binding(), point, budget)
            .map_err(UiPresentedPointLookupDenial::Query)?;
        for row in &mut query.rows {
            let (reattributed, probes) = row.reattributed(evidence.receipts());
            *row = reattributed;
            query.work.map_key_probes += probes;
        }
        Ok((relation, query))
    }

    pub(in crate::mounting) fn refresh_presented_hit_motion(
        &mut self,
        sampler: &crate::mounting::presentation::motion_sampling::UiMountedMotionSampler,
        targets: &[crate::runtime::motion::UiMotionTargetIdentity],
    ) -> UiHitTestSpatialWork {
        let mut work = UiHitTestSpatialWork::default();
        if targets.is_empty() {
            return work;
        }
        let mut authority = self.authority.borrow_mut();
        if let Some(evidence) = authority.frames.current.as_mut() {
            work.merge(Rc::make_mut(evidence).refresh_hit_motion(sampler, targets));
        }
        work
    }
}
