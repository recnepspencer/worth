use std::collections::HashSet;

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedPaintCommand,
    UiMountedPaintCommandIdentity, UiMountedPresentationSample, UiMountedPresentationSampleChange,
    UiMountedPresentationTransform,
};

use super::mutation::{update_damage, visible_bounds};
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial, UiNativeRetainedReplayPlan};

pub(crate) struct UiNativeRetainedSampleUndo {
    pub(super) physical_coverage: Vec<(
        UiMountedPaintCommandIdentity,
        Option<super::physical_coverage::UiNativeCommandImageCoverage>,
    )>,
    overrides: Vec<(
        UiMountedPaintCommandIdentity,
        Option<UiMountedPresentationSampleChange>,
    )>,
}

impl UiNativeRetainedDrawList {
    pub(super) fn retire_sample_overrides_for_semantic_delta(
        &mut self,
        identities: &[UiMountedPaintCommandIdentity],
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        let retirements = identities
            .iter()
            .filter_map(|identity| {
                self.sample_overrides
                    .get(identity)
                    .copied()
                    .map(|change| (*identity, change))
            })
            .map(|(identity, change)| {
                let command = self
                    .commands
                    .get(&identity)
                    .ok_or(UiNativeRetainedDrawListDenial::CommandMismatch)?;
                Ok((
                    identity,
                    change,
                    sampled_visible_bounds(command, Some(change))?,
                    sampled_visible_bounds(command, None)?,
                ))
            })
            .collect::<Result<Vec<_>, UiNativeRetainedDrawListDenial>>()?;
        let mut retired = Vec::with_capacity(retirements.len());
        for (identity, change, sampled, semantic) in retirements {
            if let Err(denial) = update_damage(&mut self.damage, identity, sampled, semantic) {
                self.rollback_sample(UiNativeRetainedSampleUndo {
                    overrides: retired,
                    physical_coverage: Vec::new(),
                })
                .expect("already-retired sample overrides roll back exactly");
                return Err(denial);
            }
            self.sample_overrides.remove(&identity);
            retired.push((identity, Some(change)));
        }
        Ok(())
    }

    pub(crate) fn stage_sample(
        &mut self,
        sample: &UiMountedPresentationSample,
    ) -> Result<
        (UiNativeRetainedReplayPlan, UiNativeRetainedSampleUndo),
        UiNativeRetainedDrawListDenial,
    > {
        self.validate_sample(sample)?;
        let undo = UiNativeRetainedSampleUndo {
            physical_coverage: Vec::new(),
            overrides: sample
                .changes()
                .iter()
                .map(|change| {
                    (
                        change.command(),
                        self.sample_overrides.get(&change.command()).copied(),
                    )
                })
                .collect(),
        };
        for (applied, change) in sample.changes().iter().enumerate() {
            let command = self
                .commands
                .get(&change.command())
                .ok_or(UiNativeRetainedDrawListDenial::CommandMismatch)?;
            let old = sampled_visible_bounds(
                command,
                self.sample_overrides.get(&change.command()).copied(),
            )?;
            let new = sampled_visible_bounds(command, Some(*change))?;
            if let Err(denial) = update_damage(&mut self.damage, change.command(), old, new) {
                let applied_undo = UiNativeRetainedSampleUndo {
                    physical_coverage: Vec::new(),
                    overrides: undo.overrides[..applied].to_vec(),
                };
                self.rollback_sample(applied_undo)
                    .expect("already-applied sample changes roll back exactly");
                return Err(denial);
            }
            self.sample_overrides.insert(change.command(), *change);
        }
        match self.replay_plan(sample.damage(), sample.changes().len(), 0) {
            Ok(plan) => Ok((plan, undo)),
            Err(denial) => {
                self.rollback_sample(undo)
                    .expect("a prevalidated sample rolls back exactly");
                Err(denial)
            }
        }
    }

    pub(crate) fn rollback_sample(
        &mut self,
        undo: UiNativeRetainedSampleUndo,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        self.restore_physical_coverage(undo.physical_coverage)?;
        for (identity, previous) in undo.overrides {
            let command = self
                .commands
                .get(&identity)
                .ok_or(UiNativeRetainedDrawListDenial::CommandMismatch)?;
            let current =
                sampled_visible_bounds(command, self.sample_overrides.get(&identity).copied())?;
            let restored = sampled_visible_bounds(command, previous)?;
            update_damage(&mut self.damage, identity, current, restored)?;
            match previous {
                Some(previous) => {
                    self.sample_overrides.insert(identity, previous);
                }
                None => {
                    self.sample_overrides.remove(&identity);
                }
            }
        }
        Ok(())
    }

    fn validate_sample(
        &self,
        sample: &UiMountedPresentationSample,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        let affinity = sample.affinity();
        if affinity.predecessor() != Some(self.frame)
            || affinity.successor() != self.frame
            || affinity.surface() != self.surface
            || affinity.binding() != self.binding
            || affinity.content() != self.content
            || affinity.baseline() != self.baseline
        {
            return Err(UiNativeRetainedDrawListDenial::AffinityMismatch);
        }
        let unique = sample
            .changes()
            .iter()
            .map(|change| change.command())
            .collect::<HashSet<_>>();
        if unique.len() != sample.changes().len()
            || sample.changes().is_empty()
            || sample
                .changes()
                .iter()
                .any(|change| !self.commands.contains(&change.command()))
        {
            return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
        }
        for change in sample.changes() {
            let command = self
                .commands
                .get(&change.command())
                .ok_or(UiNativeRetainedDrawListDenial::CommandMismatch)?;
            if let Some(bounds) = sampled_visible_bounds(command, Some(*change))? {
                self.damage.validate_bounds(bounds)?;
            }
        }
        for damage in sample.damage() {
            self.damage.validate_bounds(damage.bounds())?;
        }
        Ok(())
    }
}

pub(in crate::native::presentation) fn sampled_visible_bounds(
    command: &UiMountedPaintCommand,
    change: Option<UiMountedPresentationSampleChange>,
) -> Result<Option<UiMountedCanonicalBox>, UiNativeRetainedDrawListDenial> {
    let Some(bounds) = visible_bounds(command) else {
        return Ok(None);
    };
    let Some(change) = change else {
        return Ok(Some(bounds));
    };
    if change.opacity().factor() == 0.0 {
        return Ok(None);
    }
    change
        .transform()
        .map_or(Ok(bounds), |transform| transform_box(bounds, transform))
        .map(Some)
}

pub(super) fn transform_box(
    bounds: UiMountedCanonicalBox,
    transform: UiMountedPresentationTransform,
) -> Result<UiMountedCanonicalBox, UiNativeRetainedDrawListDenial> {
    let source = transform.source();
    let sampled = transform.sampled();
    if bounds.coordinate_space() != source.coordinate_space() {
        return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
    }
    let scale_x = sampled.width() / source.width();
    let scale_y = sampled.height() / source.height();
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: sampled.x() + (bounds.x() - source.x()) * scale_x,
        y: sampled.y() + (bounds.y() - source.y()) * scale_y,
        width: bounds.width() * scale_x,
        height: bounds.height() * scale_y,
        coordinate_space: bounds.coordinate_space(),
    })
    .map_err(|_| UiNativeRetainedDrawListDenial::CommandMismatch)
}
