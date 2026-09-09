//! Admission and ordered selection of actual retained physical image coverage.
use super::super::raster::{raster_damage_for_basis, UiNativeRasterBasis};
use super::physical_coverage::{UiNativeCommandImageCoverage, UiNativePhysicalCoverage};
use super::{
    UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial,
    UiNativeRetainedMutationCounters,
};
use crate::native::text_atlas::UiNativeTextAtlas;
use worth_ui_host_contract::{
    UiMountedPaintCommand, UiMountedPaintCommandIdentity, UiMountedPaintOrderIdentity,
};

impl UiNativeRetainedDrawList {
    pub(in crate::native::presentation) fn initialize_physical_coverage(
        &mut self,
        basis: UiNativeRasterBasis,
        atlas: &UiNativeTextAtlas,
    ) -> Result<(), Denial> {
        if self.physical_coverage.is_some() {
            return Err(Denial::CommandMismatch);
        }
        let mut coverage = UiNativePhysicalCoverage::new(basis);
        for command in self.commands.as_map().values() {
            let record = self.command_image_coverage(command, basis, atlas)?;
            coverage.replace(command.identity(), Some(record))?;
        }
        self.physical_coverage = Some(coverage);
        Ok(())
    }

    pub(in crate::native::presentation) fn refresh_physical_delta(
        &mut self,
        delta: &worth_ui_host_contract::UiMountedPresentationDelta,
        undo: &mut super::UiNativeRetainedDeltaUndo,
        basis: UiNativeRasterBasis,
        atlas: &UiNativeTextAtlas,
        replay: &mut super::UiNativeRetainedReplayPlan,
    ) -> Result<(), Denial> {
        self.validate_physical_basis(basis)?;
        let coverage = self
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        for change in delta.changes() {
            use worth_ui_host_contract::UiMountedPaintCommandChange as Change;
            let predecessor = match change {
                Change::Insert(_) => continue,
                Change::Replace { predecessor, .. } | Change::Remove(predecessor) => *predecessor,
            };
            coverage.get(predecessor).ok_or(Denial::CommandMismatch)?;
        }
        let identities = super::delta_transaction::changed_identities(delta.changes());
        let mut replacements = identities
            .iter()
            .map(|identity| {
                self.command(*identity)
                    .map(|command| self.command_image_coverage(command, basis, atlas))
                    .transpose()
                    .map(|record| (*identity, record))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (identity, replacement) in &replacements {
            super::physical_coverage::damage::append_text_transition(
                &mut replay.physical_text_regions,
                *identity,
                coverage.get(*identity),
                replacement.as_ref(),
                basis,
            )?;
        }
        super::physical_coverage::damage::append_text_order_damage(
            coverage,
            delta.order(),
            &identities,
            &mut replay.physical_text_regions,
        )?;
        undo.physical_coverage.reserve(replacements.len());
        replacements.sort_by_key(|(_, record)| record.is_some());
        let coverage = self
            .physical_coverage
            .as_mut()
            .ok_or(Denial::CommandMismatch)?;
        for (identity, replacement) in replacements {
            let previous = coverage.get(identity).cloned();
            coverage.replace(identity, replacement)?;
            undo.physical_coverage.push((identity, previous));
        }
        Ok(())
    }

    pub(in crate::native::presentation) fn refresh_physical_sample(
        &mut self,
        sample: &worth_ui_host_contract::UiMountedPresentationSample,
        undo: &mut super::UiNativeRetainedSampleUndo,
        basis: UiNativeRasterBasis,
        replay: &mut super::UiNativeRetainedReplayPlan,
    ) -> Result<(), Denial> {
        self.validate_physical_basis(basis)?;
        let coverage = self
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let replacements = sample
            .changes()
            .iter()
            .map(|change| {
                let identity = change.command();
                let command = self.command(identity).ok_or(Denial::CommandMismatch)?;
                let base = coverage
                    .get(identity)
                    .ok_or(Denial::CommandMismatch)?
                    .base
                    .clone();
                let current = sampled_images(command, &base, Some(*change), basis)?;
                Ok((identity, UiNativeCommandImageCoverage { base, current }))
            })
            .collect::<Result<Vec<_>, Denial>>()?;
        for (identity, replacement) in &replacements {
            super::physical_coverage::damage::append_text_transition(
                &mut replay.physical_text_regions,
                *identity,
                coverage.get(*identity),
                Some(replacement),
                basis,
            )?;
        }
        undo.physical_coverage.reserve(replacements.len());
        let coverage = self
            .physical_coverage
            .as_mut()
            .ok_or(Denial::CommandMismatch)?;
        for (identity, replacement) in replacements {
            let previous = coverage.get(identity).cloned();
            coverage.replace(identity, Some(replacement))?;
            undo.physical_coverage.push((identity, previous));
        }
        Ok(())
    }

    pub(in crate::native::presentation) fn physical_replay_for_damage(
        &self,
        basis: UiNativeRasterBasis,
        damage: [f32; 4],
        counters: &mut UiNativeRetainedMutationCounters,
    ) -> Result<Box<[UiMountedPaintCommandIdentity]>, Denial> {
        self.validate_physical_basis(basis)?;
        let query = self
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?
            .intersecting(damage)?;
        let ordered = self.order.ordered_subset(
            query
                .identities
                .into_iter()
                .map(UiMountedPaintOrderIdentity::for_command),
        )?;
        counters.damage_index_branch_aabb_probes += query.branch_aabb_probes as u64;
        counters.damage_index_leaf_command_bounds_probes += query.leaf_command_bounds_probes as u64;
        counters.damage_index_stored_records = query.stored_records as u64;
        counters.damage_index_high_water = query.high_water_records as u64;
        counters.damage_region_command_checks += ordered.len() as u64;
        counters.replayed_commands += ordered.len() as u64;
        let order_cost = self.order.take_cost();
        counters.order_index_lookups += order_cost.identity_lookups();
        counters.order_index_node_touches += order_cost.node_touches();
        counters.order_index_rotations += order_cost.rotations();
        counters.order_index_high_water = order_cost.high_water_entries();
        Ok(ordered
            .into_iter()
            .map(UiMountedPaintOrderIdentity::command)
            .collect())
    }

    fn validate_physical_basis(&self, basis: UiNativeRasterBasis) -> Result<(), Denial> {
        if self
            .physical_coverage
            .as_ref()
            .is_some_and(|coverage| coverage.basis == basis)
        {
            Ok(())
        } else {
            Err(Denial::AffinityMismatch)
        }
    }

    pub(super) fn restore_physical_coverage(
        &mut self,
        previous: Vec<(
            UiMountedPaintCommandIdentity,
            Option<UiNativeCommandImageCoverage>,
        )>,
    ) -> Result<(), Denial> {
        if previous.is_empty() {
            return Ok(());
        }
        let coverage = self
            .physical_coverage
            .as_mut()
            .ok_or(Denial::CommandMismatch)?;
        // Release all successor membership before restoring a full-capacity predecessor.
        for (identity, _) in &previous {
            coverage.replace(*identity, None)?;
        }
        for (identity, record) in previous {
            coverage.replace(identity, record)?;
        }
        Ok(())
    }

    fn command_image_coverage(
        &self,
        command: &UiMountedPaintCommand,
        basis: UiNativeRasterBasis,
        atlas: &UiNativeTextAtlas,
    ) -> Result<UiNativeCommandImageCoverage, Denial> {
        let base = match command {
            UiMountedPaintCommand::SemanticText { identity, .. } => {
                super::super::text::plan_glyph_commands(
                    self.glyph_runs(*identity),
                    atlas,
                    basis.extent(),
                )
                .map_err(|_| Denial::CommandMismatch)?
                .iter()
                .map(|glyph| glyph.target)
                .collect()
            }
            _ => sampled_images(command, &[], None, basis)?,
        };
        let current = sampled_images(
            command,
            &base,
            self.sample_override(command.identity()),
            basis,
        )?;
        Ok(UiNativeCommandImageCoverage { base, current })
    }
}

fn sampled_images(
    command: &UiMountedPaintCommand,
    base: &[[f32; 4]],
    sample: Option<worth_ui_host_contract::UiMountedPresentationSampleChange>,
    basis: UiNativeRasterBasis,
) -> Result<Box<[[f32; 4]]>, Denial> {
    if matches!(command, UiMountedPaintCommand::SemanticText { .. }) {
        let images: Vec<[f32; 4]> = base
            .iter()
            .map(|image| {
                sample
                    .and_then(|s| s.transform())
                    .map_or(Ok(*image), |transform| {
                        super::super::sample::transform_physical_box(*image, transform, basis)
                            .map_err(|_| Denial::CommandMismatch)
                    })
            })
            .collect::<Result<_, _>>()?;
        if images.iter().flatten().any(|value| !value.is_finite()) {
            return Err(Denial::CommandMismatch);
        }
        return Ok(images
            .into_iter()
            .filter(|image| image[2] > 0.0 && image[3] > 0.0)
            .collect());
    }
    super::sample_transaction::sampled_visible_bounds(command, sample)?
        .map(|bounds| raster_damage_for_basis(bounds, basis).map_err(|_| Denial::CommandMismatch))
        .transpose()
        .map(|rect| {
            rect.flatten()
                .into_iter()
                .map(|rect| rect.physical_bounds())
                .collect()
        })
}
