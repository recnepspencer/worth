use super::retained_draw_list::UiNativeRetainedDeltaUndo;
use super::UiNativeRetainedDrawList;

pub(crate) enum UiNativePendingSurfaceSettlement {
    Initial(Box<UiNativeRetainedDrawList>),
    Delta(UiNativePendingDeltaSettlement),
    Unchanged(UiNativePendingUnchangedSettlement),
    Sample(super::retained_draw_list::UiNativeRetainedSampleUndo),
    Reconstruction {
        retained: Box<UiNativeRetainedDrawList>,
        recovery: Option<crate::native::UiNativeRecoveryRequirement>,
    },
    SupersededDeltaResolved,
}

pub(crate) struct UiNativePendingDeltaSettlement {
    rollback_lineage: Vec<UiNativeRetainedDeltaUndo>,
    effects: super::UiNativePresentationEffects,
}

pub(crate) struct UiNativePendingUnchangedSettlement {
    undo: super::retained_draw_list::UiNativeRetainedUnchangedUndo,
    effects: super::UiNativePresentationEffects,
}

impl UiNativePendingUnchangedSettlement {
    pub(super) fn new(
        undo: super::retained_draw_list::UiNativeRetainedUnchangedUndo,
        effects: super::UiNativePresentationEffects,
    ) -> Self {
        Self { undo, effects }
    }

    fn rollback(self, retained: &mut UiNativeRetainedDrawList) -> Result<(), ()> {
        retained.rollback_unchanged(self.undo).map_err(|_| ())
    }
}

impl UiNativePendingDeltaSettlement {
    pub(crate) fn new(
        undo: UiNativeRetainedDeltaUndo,
        effects: super::UiNativePresentationEffects,
    ) -> Self {
        Self {
            rollback_lineage: vec![undo],
            effects,
        }
    }

    fn inherit(&mut self, mut predecessor: Self) {
        self.rollback_lineage
            .append(&mut predecessor.rollback_lineage);
        self.effects.inherit(predecessor.effects);
    }

    pub(crate) fn rollback(self, retained: &mut UiNativeRetainedDrawList) -> Result<(), ()> {
        for undo in self.rollback_lineage {
            retained.rollback_delta(undo).map_err(|_| ())?;
        }
        Ok(())
    }
}

impl UiNativePendingSurfaceSettlement {
    pub(crate) const fn work_kind(&self) -> crate::native::UiNativePresentationWorkKind {
        match self {
            Self::Initial(_) => crate::native::UiNativePresentationWorkKind::Initial,
            Self::Delta(_) => crate::native::UiNativePresentationWorkKind::Delta,
            Self::Unchanged(_) => crate::native::UiNativePresentationWorkKind::Unchanged,
            Self::Sample(_) => crate::native::UiNativePresentationWorkKind::Sample,
            Self::Reconstruction { .. } => {
                crate::native::UiNativePresentationWorkKind::Reconstruction
            }
            Self::SupersededDeltaResolved => crate::native::UiNativePresentationWorkKind::Delta,
        }
    }

    pub(crate) fn inherit_predecessor(&mut self, predecessor: Self) -> Result<(), Self> {
        match (self, predecessor) {
            (Self::Delta(successor), Self::Delta(predecessor)) => {
                successor.inherit(predecessor);
                Ok(())
            }
            (_, predecessor) => Err(predecessor),
        }
    }

    pub(crate) const fn is_resolved_supersession(&self) -> bool {
        matches!(self, Self::SupersededDeltaResolved)
    }

    pub(crate) fn commit_superseded_predecessor(self) {
        match self {
            Self::Initial(_)
            | Self::Delta(_)
            | Self::Unchanged(_)
            | Self::Sample(_)
            | Self::Reconstruction { .. }
            | Self::SupersededDeltaResolved => {}
        }
    }

    pub(crate) fn rollback_superseded_predecessor(
        self,
        state: &mut crate::native::UiNativeHostState,
        basis: crate::native::physical_work_signal::UiNativePhysicalPresentationBasis,
    ) {
        let key = basis.binding().diagnostic_value();
        match self {
            Self::Delta(lineage) => {
                let restored = state
                    .retained_draw_lists
                    .get_mut(&key)
                    .is_some_and(|retained| lineage.rollback(retained).is_ok());
                if !restored {
                    state.retained_draw_lists.remove(&key);
                    state.lifecycle.require_recovery(
                        key,
                        crate::native::UiNativeRecoveryCause::PresentationIndeterminate,
                    );
                }
            }
            Self::Unchanged(settlement) => {
                let restored = state
                    .retained_draw_lists
                    .get_mut(&key)
                    .is_some_and(|retained| settlement.rollback(retained).is_ok());
                if !restored {
                    state.retained_draw_lists.remove(&key);
                    state.lifecycle.require_recovery(
                        key,
                        crate::native::UiNativeRecoveryCause::PresentationIndeterminate,
                    );
                }
            }
            Self::Sample(undo) => {
                let restored = state
                    .retained_draw_lists
                    .get_mut(&key)
                    .is_some_and(|retained| retained.rollback_sample(undo).is_ok());
                if !restored {
                    state.retained_draw_lists.remove(&key);
                    state.lifecycle.require_recovery(
                        key,
                        crate::native::UiNativeRecoveryCause::PresentationIndeterminate,
                    );
                }
            }
            Self::Initial(_) | Self::Reconstruction { .. } => {}
            Self::SupersededDeltaResolved => {}
        }
    }

    pub(crate) fn abandon(
        self,
        state: &mut crate::native::UiNativeHostState,
        basis: crate::native::physical_work_signal::UiNativePhysicalPresentationBasis,
        completion_identity: Option<u64>,
    ) {
        let key = basis.binding().diagnostic_value();
        state
            .lifecycle
            .abandon_pending_presentation(basis.binding(), completion_identity);
        match self {
            Self::Delta(lineage) => {
                if let Some(retained) = state.retained_draw_lists.get_mut(&key) {
                    lineage
                        .rollback(retained)
                        .expect("pending delta rollback must restore exact predecessor truth");
                }
            }
            Self::Unchanged(settlement) => {
                if let Some(retained) = state.retained_draw_lists.get_mut(&key) {
                    settlement
                        .rollback(retained)
                        .expect("pending unchanged appearance rollback must restore exact predecessor truth");
                }
            }
            Self::Sample(undo) => {
                if let Some(retained) = state.retained_draw_lists.get_mut(&key) {
                    retained
                        .rollback_sample(undo)
                        .expect("pending sample rollback must restore exact predecessor truth");
                }
            }
            Self::Initial(_) | Self::Reconstruction { .. } | Self::SupersededDeltaResolved => {}
        }
        state.lifecycle.require_recovery(
            key,
            crate::native::UiNativeRecoveryCause::PresentationIndeterminate,
        );
        state.lifecycle.record_presentation_indeterminate();
    }

    pub(crate) fn complete(
        self,
        state: &mut crate::native::UiNativeHostState,
        basis: crate::native::physical_work_signal::UiNativePhysicalPresentationBasis,
        completion_identity: u64,
        observation: super::port::UiNativePresentationPortObservation,
    ) -> Option<worth_ui_host_contract::UiMountedSurfacePresentationCompletion> {
        let kind = self.work_kind();
        let effects = match &self {
            Self::Initial(retained) | Self::Reconstruction { retained, .. } => {
                super::UiNativePresentationEffects::new(true, retained.identity_overlay_active())
            }
            Self::Delta(delta) => delta.effects,
            Self::Unchanged(unchanged) => unchanged.effects,
            Self::Sample(_) => super::UiNativePresentationEffects::new(true, false),
            Self::SupersededDeltaResolved => super::UiNativePresentationEffects::default(),
        };
        let key = basis.binding().diagnostic_value();
        let resolve_required = match self {
            Self::Initial(retained) => {
                state.retained_draw_lists.insert(key, *retained);
                true
            }
            Self::Reconstruction { retained, recovery } => {
                debug_assert!(recovery
                    .as_ref()
                    .is_none_or(|recovery| recovery.binding() == key));
                state.retained_draw_lists.insert(key, *retained);
                if let Some(recovery) = recovery {
                    let _current_recovery = state.lifecycle.settle_recovery(recovery);
                }
                false
            }
            Self::Delta(_) => true,
            Self::Unchanged(_) => true,
            Self::Sample(_) => true,
            Self::SupersededDeltaResolved => return None,
        };
        let Some(retained) = state.retained_draw_lists.get(&key) else {
            state
                .lifecycle
                .abandon_pending_presentation(basis.binding(), Some(completion_identity));
            return None;
        };
        let frame = retained.frame();
        let (pixels, cost, port_crossings) = observation.into_parts();
        let epoch = worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(
            basis.attempt().diagnostic_value(),
        );
        let access = state.presentation_access();
        let last_presentation = access.as_ref().and_then(|graphics| {
            observation_for_physical_basis(
                basis,
                graphics,
                &state.text_atlas,
                retained,
                pixels,
                cost,
                port_crossings,
            )
        });
        state.record_retained_frame_observation(
            crate::native::UiNativeRetainedFrameObservation::observed(
                frame.diagnostic_value(),
                kind,
                (kind == crate::native::UiNativePresentationWorkKind::Sample).then_some(epoch),
                pixels,
                cost,
                last_presentation.clone(),
            ),
        );
        state.last_presentation = last_presentation;
        if resolve_required {
            state.lifecycle.resolve_recovery(key);
        }
        state.presentation_epochs.insert(key, epoch);
        let _input_settlement = state.lifecycle.complete_pending_presentation(
            frame,
            basis.binding(),
            epoch,
            completion_identity,
        );
        crate::native::capture::record_completed_basis(state, basis, frame, epoch);
        state.lifecycle.record_presented();
        let completion = worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
            worth_ui_host_contract::UiHostSurfacePresentationMode::NativeDisplay,
            epoch,
            effects.completion(),
            cost,
        );
        #[cfg(feature = "certification-support")]
        state.apply_completed_qualified_derived_state_loss(key);
        #[cfg(feature = "certification-support")]
        if kind == crate::native::UiNativePresentationWorkKind::Reconstruction {
            state.record_qualified_derived_state_reconstruction(key);
        }
        Some(completion)
    }
}

fn observation_for_physical_basis(
    basis: crate::native::physical_work_signal::UiNativePhysicalPresentationBasis,
    graphics: &crate::native::UiNativePresentationAccess,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    retained: &UiNativeRetainedDrawList,
    pixels: [[u8; 4]; 2],
    cost: worth_ui_host_contract::UiHostPresentationCostReport,
    port_crossings: u8,
) -> Option<crate::native::UiNativePresentationObservation> {
    let (order_ordinal, attribution) = retained.top_paint_attribution()?;
    let [retained_baseline_rgba8, retained_center_rgba8] = pixels;
    let bounds = attribution.bounds;
    Some(crate::native::UiNativePresentationObservation::new(
        crate::native::UiNativePresentationInput {
            client_physical_size: graphics.extent(),
            scale_factor_milli: (graphics.scale_factor() * 1_000.0).round() as u32,
            source_rgba8: attribution.color.channels(),
            retained_center_rgba8,
            retained_baseline_rgba8,
            presented_frame: retained.frame().diagnostic_value(),
            semantic_surface: basis.surface().diagnostic_value(),
            host_surface: basis.host_surface().diagnostic_value(),
            binding_generation: basis.binding().diagnostic_value(),
            mounted_instance: attribution.mounted_instance.diagnostic_value(),
            node_receipt: attribution.node_receipt.diagnostic_value(),
            presentation_attempt: basis.attempt().diagnostic_value(),
            logical_bounds_milli: [
                milli(bounds.x()),
                milli(bounds.y()),
                milli(bounds.width()),
                milli(bounds.height()),
            ],
            order_ordinal: u16::try_from(order_ordinal).expect("native profile bounds paint order"),
            port_crossings,
            production_cost: basis.production_cost(),
            cost,
            alpha_glyphs: super::glyph_observation::alpha(retained, atlas, graphics.extent()),
            intrinsic_glyphs: super::glyph_observation::intrinsic(
                retained,
                atlas,
                graphics.extent(),
            ),
        },
    ))
}

fn milli(value: f32) -> i64 {
    (f64::from(value) * 1_000.0).round() as i64
}
