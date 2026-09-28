//! Text work evidence the mounted coordinator keeps for certification, and
//! the resize trace every build writes.

use super::super::preparation::{EMITTED_LINES, POSITIONED_GLYPHS, SHAPED_RUNS, SHAPED_SCALARS};
use super::super::rasterization::UiNativeTextRasterWorkReport;
use super::super::{
    UiNativeTextPresentationMechanicObservation, UiNativeTextPresentationPrepared,
    UiNativeTextPresentationWorkObservation,
};
use super::{UiNativeMountedTextCoordinator, TEXT_WORK_OBSERVATION_CAPACITY};

impl UiNativeMountedTextCoordinator {
    pub(crate) fn take_work_observations(
        &mut self,
    ) -> (Box<[UiNativeTextPresentationWorkObservation]>, bool) {
        (
            std::mem::take(&mut self.work_observations).into_boxed_slice(),
            !std::mem::take(&mut self.work_observation_overflowed),
        )
    }

    /// Traces this turn's text work for resize qualification. Only
    /// certification builds keep the turn's evidence, and only while the
    /// history has room: its digests would otherwise cost every frame.
    pub(super) fn observe_work(
        &mut self,
        basis: &worth_ui_query_binding::WorthUiPresentationRequestBasis,
        prepared: &UiNativeTextPresentationPrepared,
        raster_work: UiNativeTextRasterWorkReport,
    ) {
        let key = [
            basis.mounted_frame().diagnostic_value(),
            basis.binding().diagnostic_value(),
        ];
        let layout_work = if self.admit_layout_work(key) {
            prepared.performed_layout_work()
        } else {
            [0; 17]
        };
        trace_resize_work(
            basis.attempt().diagnostic_value(),
            &layout_work,
            raster_work.rasterized_glyphs(),
        );
        if !cfg!(feature = "certification-support") {
            return;
        }
        // Removed mechanics are reported against every earlier turn, so the
        // retained set advances even when this turn's evidence is dropped.
        let (active_mechanics, removed_mechanics) = self.advance_mechanic_evidence(basis);
        if self.work_observations.len() == TEXT_WORK_OBSERVATION_CAPACITY {
            self.work_observation_overflowed = true;
            return;
        }
        self.work_observations
            .push(UiNativeTextPresentationWorkObservation::after_mounted_work(
                basis,
                prepared,
                raster_work,
                layout_work,
                active_mechanics,
                removed_mechanics,
            ));
    }

    fn admit_layout_work(&mut self, key: [u64; 2]) -> bool {
        if self.reported_layout_work.contains(&key) {
            return false;
        }
        if self.reported_layout_work.len() == TEXT_WORK_OBSERVATION_CAPACITY {
            self.reported_layout_work.pop_front();
        }
        self.reported_layout_work.push_back(key);
        true
    }

    fn advance_mechanic_evidence(
        &mut self,
        basis: &worth_ui_query_binding::WorthUiPresentationRequestBasis,
    ) -> (
        Box<[UiNativeTextPresentationMechanicObservation]>,
        Box<[UiNativeTextPresentationMechanicObservation]>,
    ) {
        let mut removed = if basis.complete() {
            std::mem::take(&mut self.retained_mechanics)
                .into_values()
                .collect::<Vec<_>>()
        } else {
            basis
                .removed_mechanics()
                .iter()
                .filter_map(|identity| self.retained_mechanics.remove(identity))
                .collect::<Vec<_>>()
        };
        let active = basis
            .mechanics()
            .iter()
            .map(UiNativeTextPresentationMechanicObservation::from_basis)
            .collect::<Vec<_>>();
        for mechanic in &active {
            self.retained_mechanics
                .insert(mechanic.mechanic(), *mechanic);
        }
        removed.sort_by_key(|mechanic| {
            let identity = mechanic.mechanic();
            let (slot, row) = identity
                .semantic_text_identity_parts()
                .expect("retained text mechanic preserves semantic-text identity");
            (identity.mounted_instance().diagnostic_value(), slot, row)
        });
        (active.into_boxed_slice(), removed.into_boxed_slice())
    }
}

/// Writes one turn's text work to the resize qualification trace, when the
/// host writes one.
fn trace_resize_work(attempt: u64, layout: &[u64; 17], rasterized_glyphs: u32) {
    worth_ui_host_native::trace_resize_text_work(
        attempt,
        worth_ui_host_native::UiNativeResizeTraceTextWork {
            shaped_runs: layout[SHAPED_RUNS],
            shaped_scalars: layout[SHAPED_SCALARS],
            positioned_glyphs: layout[POSITIONED_GLYPHS],
            emitted_lines: layout[EMITTED_LINES],
            rasterized_glyphs: u64::from(rasterized_glyphs),
        },
    );
}
