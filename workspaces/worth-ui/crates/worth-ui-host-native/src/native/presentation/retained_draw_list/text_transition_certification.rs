//! Simulated before-effects refusal and retry through the production delta planner.
use super::{
    UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial, UiNativeTextReplayOperation,
};
use crate::native::presentation::{
    delta::{prepare_delta_plan, settle_staged_delta},
    UiNativePresentationFailure,
};
use crate::native::text_atlas::UiNativeTextAtlas;
use worth_ui_host_contract::*;

impl UiNativeRetainedDrawList {
    pub(crate) fn certify_text_sample_retry(
        &mut self,
        sample: &UiMountedPresentationSample,
        atlas: &UiNativeTextAtlas,
    ) -> Result<
        (
            Box<[UiNativeTextReplayOperation]>,
            UiHostPresentationCostReport,
        ),
        Denial,
    > {
        let coverage = self
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let basis = coverage.basis;
        let previous = sample
            .changes()
            .iter()
            .map(|change| {
                let id = change.command();
                (id, coverage.get(id).cloned(), self.sample_override(id))
            })
            .collect::<Vec<_>>();
        let (plan, undo) = super::super::sample::prepare_sample_plan(basis, sample, atlas, self)
            .map_err(|_| Denial::CommandMismatch)?;
        let operations =
            super::foreground_replay_certification::observe_plan(&plan, basis.extent());
        self.rollback_sample(undo)?;
        for (id, images, sample) in previous {
            assert_eq!(
                self.physical_coverage.as_ref().unwrap().get(id),
                images.as_ref()
            );
            assert_eq!(self.sample_override(id), sample);
        }
        let (retry, _undo) = super::super::sample::prepare_sample_plan(basis, sample, atlas, self)
            .map_err(|_| Denial::CommandMismatch)?;
        assert_eq!(
            super::foreground_replay_certification::observe_plan(&retry, basis.extent()),
            operations
        );
        Ok((operations, retry.cost))
    }

    pub(crate) fn certify_text_delta_retry(
        &mut self,
        delta: &UiMountedPresentationDelta,
        runs: &[UiGlyphRunView],
        atlas: &UiNativeTextAtlas,
    ) -> Result<Box<[UiNativeTextReplayOperation]>, Denial> {
        let coverage = self
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let basis = coverage.basis;
        let commands = self.commands.as_map().clone();
        let order = self.order.ordered().collect::<Vec<_>>();
        let runs_before = self.glyph_runs.clone();
        let samples_before = self.sample_overrides.clone();
        let images = commands
            .keys()
            .copied()
            .chain(super::delta_transaction::changed_identities(
                delta.changes(),
            ))
            .map(|id| (id, coverage.get(id).cloned()))
            .collect::<Vec<_>>();
        let (plan, undo, effects) = prepare_delta_plan(basis, delta, runs, atlas, self)
            .map_err(|_| Denial::CommandMismatch)?;
        let refused = super::foreground_replay_certification::observe_plan(&plan, basis.extent());
        let refusal = settle_staged_delta(
            self,
            undo,
            effects,
            Err(UiNativePresentationFailure::BeforeEffects(
                UiHostSurfacePresentationDenial::AdapterDeclined,
            )),
        );
        assert!(matches!(
            refusal,
            Err(UiNativePresentationFailure::BeforeEffects(_))
        ));
        assert_eq!(Some(self.frame()), delta.affinity().predecessor());
        assert_eq!(self.commands.as_map(), &commands);
        assert_eq!(self.glyph_runs, runs_before);
        assert_eq!(self.sample_overrides, samples_before);
        assert_eq!(self.order.ordered().collect::<Vec<_>>(), order);
        let coverage = self.physical_coverage.as_ref().unwrap();
        for (id, previous) in images {
            assert_eq!(coverage.get(id), previous.as_ref());
        }
        let (retry, _undo, _) = prepare_delta_plan(basis, delta, runs, atlas, self)
            .map_err(|_| Denial::CommandMismatch)?;
        let retried = super::foreground_replay_certification::observe_plan(&retry, basis.extent());
        assert_eq!(
            retried, refused,
            "refusal preserves identical retry raster work"
        );
        // This model retains prepared successor state; it makes no GPU acceptance claim.
        Ok(retried)
    }
}
