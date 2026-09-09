//! Real finalized coverage through existing delta and pending settlement machinery.
//! The external port observations are simulated; native pixel acceptance is excluded.
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::appearance::text_foreground::UiNativeFinalizedTextForeground;
use crate::native::presentation::{
    self, UiNativePendingSurfaceSettlement, UiNativePresentationEffects,
    UiNativePresentationFailure, UiNativePresentationPortFailure,
};
use crate::native::text_atlas::UiNativeTextAtlas;
use worth_ui_host_contract::*;

impl UiNativeRetainedDrawList {
    pub(crate) fn from_text_coverage_for_certification(
        candidate: UiNativeFinalizedTextForeground,
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
        atlas: &UiNativeTextAtlas,
    ) -> Result<Self, Denial> {
        Self::from_text_coverages_for_certification(vec![candidate], view, extent, atlas)
    }

    pub(crate) fn from_text_coverages_for_certification(
        candidates: Vec<UiNativeFinalizedTextForeground>,
        view: &UiMountedFrameConsumptionView<'_>,
        extent: [u32; 2],
        atlas: &UiNativeTextAtlas,
    ) -> Result<Self, Denial> {
        let mut seen = std::collections::HashSet::new();
        let commands = candidates
            .iter()
            .flat_map(UiNativeFinalizedTextForeground::qualified_candidates)
            .filter_map(|mechanic| {
                let identity = UiMountedPaintCommandIdentity::semantic_text(mechanic);
                seen.insert(identity)
                    .then(|| UiMountedPaintCommand::SemanticText {
                        identity,
                        mechanic: mechanic.clone(),
                    })
            })
            .collect::<Vec<_>>();
        let order = commands
            .iter()
            .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
            .collect::<Vec<_>>();
        let mut retained =
            Self::from_text_view_for_certification(&commands, &order, view, extent, atlas)?;
        retained.initialize_text_coverage(candidates, atlas)?;
        Ok(retained)
    }

    pub(crate) fn prepare_text_coverage_changes_for_certification(
        &mut self,
        candidates: Vec<UiNativeFinalizedTextForeground>,
        atlas: &UiNativeTextAtlas,
        fragment: &UiUnpublishedAppearanceFragment,
        delta: &UiMountedPresentationDelta,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> Result<Box<[[i64; 4]]>, Denial> {
        let runs = view
            .text_raster_work()
            .map(UiMountedTextRasterWork::glyph_runs)
            .unwrap_or_default();
        self.stage_text_coverage_replacements(
            delta,
            fragment,
            view.attempt(),
            runs,
            candidates,
            atlas,
        )?;
        Ok(coverage(self)?
            .iter()
            .flat_map(|value| value.coverage())
            .map(|region| [region.left, region.top, region.right, region.bottom])
            .collect())
    }

    pub(crate) fn certify_text_coverage_settlement(
        &mut self,
        candidate: UiNativeFinalizedTextForeground,
        atlas: &UiNativeTextAtlas,
        fragment: &UiUnpublishedAppearanceFragment,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> Result<Box<[[i64; 4]]>, Denial> {
        let affinity = candidate.affinity();
        let runs = view
            .text_raster_work()
            .ok_or(Denial::CommandMismatch)?
            .glyph_runs();
        let changes = fragment
            .text_candidates()
            .iter()
            .map(|mechanic| {
                let identity = UiMountedPaintCommandIdentity::semantic_text(mechanic);
                UiMountedPaintCommandChange::replacement(
                    identity,
                    UiMountedPaintCommand::SemanticText {
                        identity,
                        mechanic: mechanic.clone(),
                    },
                )
            })
            .collect();
        let order = self.order.ordered().collect::<Vec<_>>();
        let delta =
            UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
                predecessor: affinity.predecessor().ok_or(Denial::AffinityMismatch)?,
                successor: affinity.successor(),
                surface: affinity.surface(),
                binding: affinity.binding(),
                content: affinity.content(),
                baseline: affinity.baseline(),
                changes,
                nodes: vec![],
                order: vec![],
                order_integrity: UiMountedPaintOrderIntegrity::for_order(&order),
                damage: vec![],
                auxiliary: None,
                production_cost: Default::default(),
            })
            .with_successor_receipt_affinity(affinity.receipt_affinity());
        if view.presentation_work().affinity() != affinity
            || view.attempt() != candidate.presentation_attempt()
        {
            return Err(Denial::AffinityMismatch);
        }
        let predecessor = self.frame;
        let before = coverage(self)?;
        let foreign = UiMountedPresentationAttemptIdentity::mint_unbound()
            .map_err(|_| Denial::AffinityMismatch)?;
        if !matches!(
            self.stage_text_coverage_replacements(
                &delta,
                fragment,
                foreign,
                runs,
                vec![candidate.clone()],
                atlas
            ),
            Err(Denial::AffinityMismatch)
        ) || coverage(self)? != before
            || self.frame != predecessor
        {
            return Err(Denial::AffinityMismatch);
        }
        for invalid in [vec![], vec![candidate.clone(), candidate.clone()]] {
            if !matches!(
                self.stage_text_coverage_replacements(
                    &delta,
                    fragment,
                    view.attempt(),
                    runs,
                    invalid,
                    atlas
                ),
                Err(Denial::CommandMismatch)
            ) || coverage(self)? != before
                || self.frame != predecessor
            {
                return Err(Denial::CommandMismatch);
            }
        }
        // A coherent finalized/fragment successor cannot stand in for an omitted
        // ordinary geometry replacement. This denial occurs after stage_delta.
        let omitted =
            UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
                predecessor,
                successor: affinity.successor(),
                surface: affinity.surface(),
                binding: affinity.binding(),
                content: affinity.content(),
                baseline: affinity.baseline(),
                changes: vec![],
                nodes: vec![],
                order: vec![],
                order_integrity: UiMountedPaintOrderIntegrity::for_order(&order),
                damage: vec![],
                auxiliary: None,
                production_cost: Default::default(),
            })
            .with_successor_receipt_affinity(affinity.receipt_affinity());
        if !matches!(
            self.stage_text_coverage_replacements(
                &omitted,
                fragment,
                view.attempt(),
                runs,
                vec![candidate.clone()],
                atlas
            ),
            Err(Denial::CommandMismatch)
        ) || coverage(self)? != before
            || self.frame != predecessor
        {
            return Err(Denial::CommandMismatch);
        }
        let mut resources = crate::native::UiNativeResourceRegistry::new();
        let mut signal = crate::native::physical_work_signal::UiNativePhysicalSignalOwner::new();
        let basis =
            crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::from_view(view);
        let (refused_replay, undo) = self.stage_text_coverage_replacements(
            &delta,
            fragment,
            view.attempt(),
            runs,
            vec![candidate.clone()],
            atlas,
        )?;
        let owners = presentation::reserve_presentation_owners(&mut resources, &mut signal, basis)
            .map_err(|_| Denial::CommandMismatch)?;
        let refused = presentation::settle_port_result(
            &mut resources,
            &mut signal,
            owners,
            Err(UiNativePresentationPortFailure::Surface(
                presentation::port::UiNativeSurfaceAcquireFailure::Timeout,
            )),
        );
        if !matches!(
            presentation::delta::settle_staged_delta(
                self,
                undo,
                UiNativePresentationEffects::default(),
                refused
            ),
            Err(UiNativePresentationFailure::BeforeEffects(_))
        ) || coverage(self)? != before
            || self.frame != predecessor
        {
            return Err(Denial::CommandMismatch);
        }
        let (pending_replay, undo) = self.stage_text_coverage_replacements(
            &delta,
            fragment,
            view.attempt(),
            runs,
            vec![candidate.clone()],
            atlas,
        )?;
        if pending_replay.staged_appearance_regions != refused_replay.staged_appearance_regions {
            return Err(Denial::CommandMismatch);
        }
        let owners = presentation::reserve_presentation_owners(&mut resources, &mut signal, basis)
            .map_err(|_| Denial::CommandMismatch)?;
        let pending = presentation::settle_port_result(
            &mut resources,
            &mut signal,
            owners,
            Err(UiNativePresentationPortFailure::ReadbackUnsettled(
                Box::new(UnsettledPort),
            )),
        );
        let Err(UiNativePresentationFailure::Pending(mut pending)) =
            presentation::delta::settle_staged_delta(
                self,
                undo,
                UiNativePresentationEffects::default(),
                pending,
            )
        else {
            return Err(Denial::CommandMismatch);
        };
        if coverage(self)? == before || self.frame != affinity.successor() {
            return Err(Denial::CommandMismatch);
        }
        let due = signal.next_due_tick().ok_or(Denial::CommandMismatch)?;
        signal
            .advance_clock_to(due)
            .map_err(|_| Denial::CommandMismatch)?;
        let token = signal
            .take_ready_presentation(pending.physical_work(), pending.physical_token())
            .map_err(|_| Denial::CommandMismatch)?
            .current();
        let observed =
            signal.reconcile(token.observe(
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed,
            ));
        if !matches!(
            observed,
            crate::native::physical_work_signal::UiNativePhysicalSignalSettlement::Completed
        ) {
            return Err(Denial::CommandMismatch);
        }
        let Some(UiNativePendingSurfaceSettlement::Delta(lineage)) = pending.take_settlement()
        else {
            return Err(Denial::CommandMismatch);
        };
        // Signal completion without a presented observation cannot commit coverage.
        lineage
            .rollback(self)
            .map_err(|_| Denial::CommandMismatch)?;
        pending.release(&mut resources);
        if coverage(self)? != before || self.frame != predecessor || !resources.current().is_zero()
        {
            return Err(Denial::CommandMismatch);
        }
        let expected = candidate.clone();
        let (accepted_replay, undo) = self.stage_text_coverage_replacements(
            &delta,
            fragment,
            view.attempt(),
            runs,
            vec![candidate],
            atlas,
        )?;
        if accepted_replay.staged_appearance_regions != refused_replay.staged_appearance_regions {
            return Err(Denial::CommandMismatch);
        }
        let owners = presentation::reserve_presentation_owners(&mut resources, &mut signal, basis)
            .map_err(|_| Denial::CommandMismatch)?;
        let accepted = presentation::settle_port_result(
            &mut resources,
            &mut signal,
            owners,
            Ok(
                presentation::port::UiNativePresentationPortObservation::from_async_readback(
                    [[0; 4]; 2],
                    Default::default(),
                ),
            ),
        );
        presentation::delta::settle_staged_delta(
            self,
            undo,
            UiNativePresentationEffects::default(),
            accepted,
        )
        .map_err(|_| Denial::CommandMismatch)?;
        let current = coverage(self)?;
        if current.as_slice() != std::slice::from_ref(&expected)
            || self.frame != affinity.successor()
            || !resources.current().is_zero()
        {
            return Err(Denial::CommandMismatch);
        }
        let (_, appearance) = self
            .staged_appearance
            .as_mut()
            .ok_or(Denial::CommandMismatch)?;
        if !appearance.take_damage().is_empty() {
            return Err(Denial::CommandMismatch);
        }
        let keys = appearance.ordered_keys();
        for region in &accepted_replay.staged_appearance_regions {
            let intersects = expected
                .coverage()
                .iter()
                .any(|r| r.intersects(region.damage));
            let selected = if intersects { keys.as_ref() } else { &[] };
            if region.replay.as_ref() != selected {
                return Err(Denial::CommandMismatch);
            }
        }
        let _ = signal.shutdown();
        Ok(accepted_replay
            .staged_appearance_regions
            .iter()
            .map(|region| {
                let r = region.damage;
                [r.left, r.top, r.right, r.bottom]
            })
            .collect())
    }
}

fn coverage(
    retained: &UiNativeRetainedDrawList,
) -> Result<Vec<UiNativeFinalizedTextForeground>, Denial> {
    let (_, appearance) = retained
        .staged_appearance
        .as_ref()
        .ok_or(Denial::CommandMismatch)?;
    appearance
        .ordered_keys()
        .iter()
        .map(|key| match appearance.command(*key) {
            Some(
                crate::native::presentation::appearance::UiNativeAppearanceCommand::TextForeground(
                    value,
                ),
            ) => Ok(value.clone()),
            _ => Err(Denial::CommandMismatch),
        })
        .collect()
}

struct UnsettledPort;
impl presentation::UiNativePendingExternalObligation for UnsettledPort {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending)
    }
}
