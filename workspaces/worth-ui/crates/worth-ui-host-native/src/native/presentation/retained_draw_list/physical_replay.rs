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

    /// Record that a complete presentation painted this list into `target`.
    pub(in crate::native::presentation) fn paint_target(&mut self, target: u64) {
        self.painted_target = target;
        self.repaint_target = None;
    }

    pub(crate) const fn owes_repaint(&self, target: u64) -> bool {
        self.painted_target != target
    }

    pub(in crate::native::presentation) const fn repaint_owed(&self) -> bool {
        self.repaint_target.is_some()
    }

    /// Carry retained coverage onto the surface's current target. A successor
    /// target of the same scale starts empty: coverage is clipped anew to its
    /// extent and the list owes it a whole repaint until a presentation into
    /// it is submitted. A refusal (a scale change) leaves the list to
    /// reconstruction, which discards it.
    pub(in crate::native::presentation) fn prepare_target(
        &mut self,
        basis: UiNativeRasterBasis,
        target: u64,
    ) -> Result<(), Denial> {
        if !self.owes_repaint(target) {
            self.repaint_target = None;
            return Ok(());
        }
        self.rebase_physical_coverage(basis)?;
        self.repaint_target = Some(target);
        Ok(())
    }

    /// A presentation that repainted a successor target was submitted,
    /// completed or pending. An abandoned pending presentation requires
    /// recovery, which discards the list.
    pub(in crate::native::presentation) fn settle_target(&mut self) {
        if let Some(target) = self.repaint_target.take() {
            self.painted_target = target;
        }
    }

    fn rebase_physical_coverage(&mut self, basis: UiNativeRasterBasis) -> Result<(), Denial> {
        let coverage = self
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        if coverage.basis == basis {
            return Ok(());
        }
        if coverage.basis.scale_factor() != basis.scale_factor() {
            return Err(Denial::AffinityMismatch);
        }
        self.clip_physical_coverage(basis)
    }

    /// Clip coverage anew to the extent it is measured in. A superseded
    /// predecessor staged before the list carried onto a successor target
    /// rolls back coverage clipped to the earlier target's extent.
    pub(in crate::native::presentation) fn reclip_physical_coverage(
        &mut self,
    ) -> Result<(), Denial> {
        let basis = self
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?
            .basis;
        self.clip_physical_coverage(basis)
    }

    fn clip_physical_coverage(&mut self, basis: UiNativeRasterBasis) -> Result<(), Denial> {
        let coverage = self
            .physical_coverage
            .as_ref()
            .ok_or(Denial::CommandMismatch)?;
        let mut rebased = UiNativePhysicalCoverage::new(basis);
        for command in self.commands.as_map().values() {
            let identity = command.identity();
            // Glyph images are placed by the atlas, not the target; every
            // other command's images are its visible bounds in the target.
            let base = match command {
                UiMountedPaintCommand::SemanticText { .. } => coverage
                    .get(identity)
                    .ok_or(Denial::CommandMismatch)?
                    .base
                    .clone(),
                _ => sampled_images(command, &[], None, basis)?,
            };
            let current = sampled_images(command, &base, self.sample_override(identity), basis)?;
            rebased.replace(
                identity,
                Some(UiNativeCommandImageCoverage { base, current }),
            )?;
        }
        if let Some((_, appearance)) = self.staged_appearance.as_mut() {
            appearance
                .rebase_text_coverage(basis.extent())
                .map_err(|_| Denial::CommandMismatch)?;
        }
        self.physical_coverage = Some(rebased);
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
            // Appearance surfaces carry analytic coverage in their own replay
            // index; only paint commands own retained text/image coverage here.
            .filter(|change| !change.command().is_appearance_sample())
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
                super::super::text::plan_raw_glyph_commands(self.glyph_runs(*identity), atlas)
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
    if let UiMountedPaintCommand::SemanticText { mechanic, .. } = command {
        let images: Vec<[f32; 4]> = base
            .iter()
            .map(|image| {
                super::super::text::sampled_image(
                    *image,
                    command.clip_bounds(),
                    mechanic.intrinsic_clip_bounds(),
                    sample,
                    basis,
                )
                .map_err(|_| Denial::CommandMismatch)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
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
