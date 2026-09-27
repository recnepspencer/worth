use super::*;
use crate::mounting::presented_hit_index::UiPresentedHitQueryDenial;
use crate::mounting::spatial_index::UiMountedSpatialBudget;
use crate::mounting::UiHitTestSpatialWork;

pub(crate) enum UiPresentedPointLookupDenial {
    Presentation(UiPresentedFrameBasisDenial),
    Query(UiPresentedHitQueryDenial),
}

impl UiMountedFrameRetentionCoordinator {
    pub(in crate::mounting) fn current_retained_node_receipt(
        &self,
        instance: UiMountedInstanceIdentity,
    ) -> Option<UiMountedNodeReceiptIdentity> {
        self.authority
            .borrow()
            .frames
            .current
            .as_ref()?
            .receipt_for_with_probes(instance)
            .0
    }

    pub(in crate::mounting) fn current_node_receipt(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        instance: UiMountedInstanceIdentity,
    ) -> Result<UiMountedNodeReceiptIdentity, UiPresentedFrameBasisDenial> {
        let authority = self.authority.borrow();
        let surface = authority.surface_for_current_presentation(presentation)?;
        let evidence = authority
            .surface_evidence(surface)
            .ok_or(UiPresentedFrameBasisDenial::Unknown)?;
        evidence
            .receipt_for_with_probes(instance)
            .0
            .ok_or(UiPresentedFrameBasisDenial::InstanceNotPresented)
    }
    pub(in crate::mounting) fn current_presented_hit_row(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        work: &mut UiHitTestSpatialWork,
    ) -> Result<crate::mounting::UiPresentedHitTestRow, UiPresentedFrameBasisDenial> {
        let authority = self.authority.borrow();
        let surface = authority.surface_for_current_presentation(presentation)?;
        let evidence = authority
            .surface_evidence(surface)
            .ok_or(UiPresentedFrameBasisDenial::Unknown)?;
        evidence.classify(presentation, None, None)?;
        let basis = evidence.visual_region_basis(presentation.binding());
        let floor = basis.modal_input_floor(presentation.binding(), work);
        if !basis.admits_modal_input(instance, floor) {
            return Err(UiPresentedFrameBasisDenial::InstanceNotPresented);
        }
        let (row, probes) = basis
            .presented_hits
            .for_instance(presentation.binding(), instance);
        work.map_key_probes += probes;
        let row = row.ok_or(UiPresentedFrameBasisDenial::InstanceNotPresented)?;
        let (row, probes) = row.reattributed(evidence.receipts());
        work.map_key_probes += probes;
        Ok(row)
    }

    /// Every row the presented hit index holds for `presentation`, whether
    /// or not a modal shields it, attributed as interaction reads it.
    pub(in crate::mounting) fn indexed_hit_rows(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<Vec<crate::mounting::UiPresentedHitTestRow>, UiPresentedFrameBasisDenial> {
        let authority = self.authority.borrow();
        let surface = authority.surface_for_current_presentation(presentation)?;
        let evidence = authority
            .surface_evidence(surface)
            .ok_or(UiPresentedFrameBasisDenial::Unknown)?;
        evidence.classify(presentation, None, None)?;
        let basis = evidence.visual_region_basis(presentation.binding());
        Ok(basis
            .presented_hits
            .rows_on(presentation.binding())
            .map(|row| row.reattributed(evidence.receipts()).0)
            .collect())
    }

    pub(in crate::mounting) fn current_presentation_for_surface(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<crate::mounting::presentation::UiDisplayedSurfaceBasis> {
        let authority = self.authority.borrow();
        let current = authority.surface_evidence(surface)?;
        current.displayed_for_surface(surface)
    }

    pub(in crate::mounting) fn current_semantic_surface_for_presentation(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<worth_ui_host_contract::UiSemanticSurfaceIdentity, UiPresentedFrameBasisDenial>
    {
        let authority = self.authority.borrow();
        authority.surface_for_current_presentation(presentation)
    }

    pub(in crate::mounting) fn current_hit_evidence(
        &self,
    ) -> Option<Rc<super::super::UiRetainedPresentedFrame>> {
        self.authority.borrow().frames.current.clone()
    }

    pub(in crate::mounting) fn committed_hit_transition(
        &self,
        previous: Option<Rc<super::super::UiRetainedPresentedFrame>>,
        frame: UiMountedFrameIdentity,
    ) -> Option<super::super::UiCommittedPresentedHitTransition> {
        Some(super::super::UiCommittedPresentedHitTransition::new(
            previous,
            self.hit_evidence(frame)?,
        ))
    }

    pub(in crate::mounting) fn hit_evidence(
        &self,
        frame: UiMountedFrameIdentity,
    ) -> Option<Rc<super::super::UiRetainedPresentedFrame>> {
        self.authority.borrow().evidence_rc(frame)
    }

    pub(in crate::mounting) fn presented_hit_candidates(
        &self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        point: crate::mounting::presentation::UiPlatformPoint,
        budget: UiMountedSpatialBudget,
    ) -> Result<super::super::UiPresentedHitTestBasis, UiPresentedPointLookupDenial> {
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
        let displayed = evidence
            .classify(presentation, None, None)
            .map_err(UiPresentedPointLookupDenial::Presentation)?;
        let basis = evidence.visual_region_basis(presentation.binding());
        let mut query = basis
            .presented_hits
            .at_point(presentation.binding(), point, budget)
            .map_err(UiPresentedPointLookupDenial::Query)?;
        let floor = basis.modal_input_floor(presentation.binding(), &mut query.work);
        query
            .rows
            .retain(|row| basis.admits_modal_input(row.mounted_instance(), floor));
        if let Some(surface) = query.rows.first().map(|row| row.mounted().surface()) {
            let current = authority.surface_evidence(surface).ok_or(
                UiPresentedPointLookupDenial::Presentation(UiPresentedFrameBasisDenial::Expired),
            )?;
            if current.frame() != evidence.frame() {
                // A retained event keeps its observed geometry, but cannot
                // regain input authority behind a newly accepted modal.
                let current_basis = current.visual_region_basis(presentation.binding());
                let current_floor =
                    current_basis.modal_input_floor(presentation.binding(), &mut query.work);
                query.rows.retain(|row| {
                    current_basis.admits_modal_input(row.mounted_instance(), current_floor)
                });
            }
        }
        for row in &mut query.rows {
            let (reattributed, probes) = row.reattributed(evidence.receipts());
            *row = reattributed;
            query.work.map_key_probes += probes;
        }
        Ok(super::super::UiPresentedHitTestBasis::from_candidates(
            displayed, relation, query,
        ))
    }

    /// Apply one committed scroll pose's translations to the current
    /// retained frame for its surface, and report the hit transition that
    /// crossing leaves behind. A pose that moves nothing still retires the
    /// leads an earlier displayed pose left.
    pub(in crate::mounting) fn refresh_presented_hit_scroll(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        translations: &[(
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::mounting::UiHitScrollMove,
        )],
    ) -> (
        Option<super::super::UiCommittedPresentedHitTransition>,
        UiHitTestSpatialWork,
    ) {
        let frame = {
            let authority = self.authority.borrow();
            authority.frames.surface_frames.get(&surface).copied()
        };
        self.move_presented_hit_scroll(
            frame,
            surface,
            translations,
            crate::mounting::presented_hit_index::UiHitScrollStanding::Committed,
        )
    }

    /// Lead the frame a witness displays to a pose mounted geometry has yet
    /// to commit, and report the hit transition that crossing leaves behind.
    /// While a presentation is in flight the surface's current frame is the
    /// one it presents, not the one the host shows, so the lead names the
    /// displayed frame itself.
    pub(in crate::mounting) fn lead_presented_hit_scroll(
        &mut self,
        displayed: crate::mounting::presentation::UiDisplayedSurfaceBasis,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        translations: &[(
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::mounting::UiHitScrollMove,
        )],
    ) -> (
        Option<super::super::UiCommittedPresentedHitTransition>,
        UiHitTestSpatialWork,
    ) {
        self.move_presented_hit_scroll(
            Some(displayed.basis().frame()),
            surface,
            translations,
            crate::mounting::presented_hit_index::UiHitScrollStanding::Leading,
        )
    }

    /// Retire every lead the rows of `surface` hold, in its current frame and
    /// in `displayed`, the frame its witness shows, and report the hit
    /// transitions that leaves behind. A frame holding no lead is left
    /// untouched, so retiring costs nothing while no settle is owed.
    pub(in crate::mounting) fn retire_presented_hit_scroll_leads(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        displayed: UiMountedFrameIdentity,
    ) -> Vec<super::super::UiCommittedPresentedHitTransition> {
        let frames = {
            let authority = self.authority.borrow();
            authority
                .frames
                .surface_frames
                .get(&surface)
                .copied()
                .into_iter()
                .chain(std::iter::once(displayed))
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .filter(|frame| {
                    authority
                        .evidence_rc(*frame)
                        .is_some_and(|evidence| evidence.holds_hit_scroll_leads(surface))
                })
                .collect::<Vec<_>>()
        };
        frames
            .into_iter()
            .filter_map(|frame| {
                self.move_presented_hit_scroll(
                    Some(frame),
                    surface,
                    &[],
                    crate::mounting::presented_hit_index::UiHitScrollStanding::Leading,
                )
                .0
            })
            .collect()
    }

    /// The predecessor is taken before the move and the successor after it,
    /// so the two name the same frame with the rows in the positions they held
    /// on either side of the pose. That pair is what tells interaction which
    /// rows moved under a pointer that did not.
    fn move_presented_hit_scroll(
        &mut self,
        frame: Option<UiMountedFrameIdentity>,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        translations: &[(
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::mounting::UiHitScrollMove,
        )],
        standing: crate::mounting::presented_hit_index::UiHitScrollStanding,
    ) -> (
        Option<super::super::UiCommittedPresentedHitTransition>,
        UiHitTestSpatialWork,
    ) {
        let mut work = UiHitTestSpatialWork::default();
        let Some(frame) = frame else {
            return (None, work);
        };
        let predecessor = self.hit_evidence(frame);
        {
            let mut authority = self.authority.borrow_mut();
            let Some(mut evidence) = authority.evidence_rc(frame) else {
                return (None, work);
            };
            work.merge(Rc::make_mut(&mut evidence).refresh_hit_scroll(
                surface,
                translations,
                standing,
            ));
            authority.replace_evidence(evidence);
        }
        (self.committed_hit_transition(predecessor, frame), work)
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
        let frames = targets
            .iter()
            .filter_map(|target| {
                authority
                    .frames
                    .surface_frames
                    .get(&target.semantic_surface())
                    .copied()
            })
            .collect::<std::collections::BTreeSet<_>>();
        for frame in frames {
            let local_targets = targets
                .iter()
                .copied()
                .filter(|target| {
                    authority
                        .frames
                        .surface_frames
                        .get(&target.semantic_surface())
                        == Some(&frame)
                })
                .collect::<Vec<_>>();
            if let Some(mut evidence) = authority.evidence_rc(frame) {
                work.merge(Rc::make_mut(&mut evidence).refresh_hit_motion(sampler, &local_targets));
                authority.replace_evidence(evidence);
            }
        }
        work
    }
}
