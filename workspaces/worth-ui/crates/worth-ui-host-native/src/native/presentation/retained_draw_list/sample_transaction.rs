use std::collections::HashSet;

use worth_ui_host_contract::{
    UiMountedAppearancePresentationWork, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedPaintCommand, UiMountedPaintCommandIdentity, UiMountedPresentationSample,
    UiMountedPresentationSampleChange, UiMountedPresentationTransform,
};

use super::mutation::{update_damage, visible_bounds};
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial, UiNativeRetainedReplayPlan};

pub(crate) struct UiNativeRetainedSampleUndo {
    appearance_coverage: super::sampled_appearance_coverage::CoverageUndo,
    pub(super) physical_coverage: Vec<(
        UiMountedPaintCommandIdentity,
        Option<super::physical_coverage::UiNativeCommandImageCoverage>,
    )>,
    overrides: Vec<(
        UiMountedPaintCommandIdentity,
        Option<UiMountedPresentationSampleChange>,
    )>,
}

pub(crate) struct UiNativeAppearanceSampleUndo {
    appearance_coverage: super::sampled_appearance_coverage::CoverageUndo,
    overrides: Vec<(
        UiMountedPaintCommandIdentity,
        Option<UiMountedPresentationSampleChange>,
    )>,
}

impl UiNativeRetainedSampleUndo {
    pub(in crate::native::presentation) fn changed_identities(
        &self,
    ) -> impl Iterator<Item = UiMountedPaintCommandIdentity> + '_ {
        self.overrides.iter().map(|(identity, _)| *identity)
    }
}

impl UiNativeRetainedDrawList {
    /// Re-issued samples replace their predecessors; surface samples the work
    /// replaces or removes without re-issuing retire in the same undo.
    pub(crate) fn stage_appearance_work_sample_overrides(
        &mut self,
        work: &UiMountedAppearancePresentationWork,
    ) -> Result<UiNativeAppearanceSampleUndo, UiNativeRetainedDrawListDenial> {
        let changes = work.sample_overrides();
        let unique = changes
            .iter()
            .map(|change| change.command())
            .collect::<HashSet<_>>();
        if unique.len() != changes.len()
            || changes
                .iter()
                .any(|change| !self.admits_sample_target(change.command()))
        {
            return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
        }
        let retired = self.retired_appearance_surface_samples(work);
        let undo = UiNativeAppearanceSampleUndo {
            appearance_coverage: self.sampled_appearance.snapshot(
                retired
                    .iter()
                    .copied()
                    .chain(changes.iter().map(|change| change.command())),
            ),
            overrides: retired
                .iter()
                .copied()
                .chain(changes.iter().map(|change| change.command()))
                .map(|identity| (identity, self.sample_overrides.get(&identity).copied()))
                .collect(),
        };
        for identity in &retired {
            self.sample_overrides.remove(identity);
            self.sampled_appearance.replace(*identity, None)?;
        }
        for (applied, change) in changes.iter().enumerate() {
            if let Err(denial) = self.apply_sample_change(change) {
                self.rollback_appearance_sample_overrides(UiNativeAppearanceSampleUndo {
                    appearance_coverage: undo.appearance_coverage,
                    overrides: undo.overrides[..retired.len() + applied].to_vec(),
                })
                .expect("already-refreshed appearance samples roll back exactly");
                return Err(denial);
            }
        }
        Ok(undo)
    }

    /// Both paint and appearance coverage follow the staged sample; their
    /// separate indexes preserve the two retained command families.
    fn apply_sample_change(
        &mut self,
        change: &UiMountedPresentationSampleChange,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        let identity = change.command();
        if identity.is_appearance_sample() {
            let bounds = self.sampled_target_bounds(identity, Some(*change))?;
            self.sampled_appearance.replace(identity, bounds)?;
        } else {
            let old = self
                .sampled_target_bounds(identity, self.sample_overrides.get(&identity).copied())?;
            let new = self.sampled_target_bounds(identity, Some(*change))?;
            update_damage(&mut self.damage, identity, old, new)?;
        }
        self.sample_overrides.insert(identity, *change);
        Ok(())
    }

    pub(crate) fn rollback_appearance_sample_overrides(
        &mut self,
        undo: UiNativeAppearanceSampleUndo,
    ) -> Result<(), UiNativeRetainedDrawListDenial> {
        self.rollback_sample(UiNativeRetainedSampleUndo {
            appearance_coverage: undo.appearance_coverage,
            physical_coverage: Vec::new(),
            overrides: undo.overrides,
        })
    }

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
                    appearance_coverage: Vec::new(),
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
        let mut damage = sample.damage().to_vec();
        damage.extend(self.backdrop_sample_damage(sample));
        // Appearance-only children can extend beyond the Portal body (for
        // example its shadow). Replay their actual old and new paint coverage.
        for change in sample
            .changes()
            .iter()
            .filter(|change| change.command().is_appearance_sample())
        {
            for sampled in [
                self.sample_overrides.get(&change.command()).copied(),
                Some(*change),
            ] {
                if let Some(bounds) = self.sampled_target_bounds(change.command(), sampled)? {
                    damage.push(
                        worth_ui_host_contract::UiMountedLogicalDamage::from_runtime_mounting(
                            bounds,
                        ),
                    );
                }
            }
        }
        let undo = UiNativeRetainedSampleUndo {
            appearance_coverage: self
                .sampled_appearance
                .snapshot(sample.changes().iter().map(|change| change.command())),
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
            if let Err(denial) = self.apply_sample_change(change) {
                let applied_undo = UiNativeRetainedSampleUndo {
                    appearance_coverage: undo.appearance_coverage,
                    physical_coverage: Vec::new(),
                    overrides: undo.overrides[..applied].to_vec(),
                };
                self.rollback_sample(applied_undo)
                    .expect("already-applied sample changes roll back exactly");
                return Err(denial);
            }
        }
        match self.replay_plan(&damage, sample.changes().len(), 0) {
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
        self.sampled_appearance.restore(undo.appearance_coverage)?;
        for (identity, previous) in undo.overrides {
            if !identity.is_appearance_sample() {
                let current = self.sampled_target_bounds(
                    identity,
                    self.sample_overrides.get(&identity).copied(),
                )?;
                let restored = self.sampled_target_bounds(identity, previous)?;
                update_damage(&mut self.damage, identity, current, restored)?;
            }
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
            || (sample.changes().is_empty() && !sample.damage().is_empty())
            || sample
                .changes()
                .iter()
                .any(|change| !self.admits_sample_target(change.command()))
        {
            return Err(UiNativeRetainedDrawListDenial::CommandMismatch);
        }
        for change in sample.changes() {
            if let Some(bounds) = self.sampled_target_bounds(change.command(), Some(*change))? {
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
    let Some(bounds) = change
        .filter(|change| change.clip().is_some())
        .map_or_else(|| visible_bounds(command), |_| Some(command.bounds()))
    else {
        return Ok(None);
    };
    sampled_bounds(bounds, change)
}

pub(super) fn sampled_bounds(
    bounds: UiMountedCanonicalBox,
    change: Option<UiMountedPresentationSampleChange>,
) -> Result<Option<UiMountedCanonicalBox>, UiNativeRetainedDrawListDenial> {
    let Some(change) = change else {
        return Ok(Some(bounds));
    };
    if change.opacity().factor() == 0.0 {
        return Ok(None);
    }
    let bounds = change
        .transform()
        .map_or(Ok(bounds), |transform| transform_box(bounds, transform))?;
    Ok(change
        .clip()
        .map_or(Some(bounds), |clip| bounds.intersection(clip)))
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
