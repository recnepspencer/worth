//! Installed family head selection for native producer succession, never currentness.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;

mod head;
use head::SelectedRequiredPublicationHead;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn required_publication_head(
        &self,
        family: &str,
        binding: TypeId,
        scope: &WorthQueryOperationScopeBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        partition: [u8; 32],
    ) -> Result<
        (String, Option<SelectedRequiredPublicationHead>),
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
        let bindings = self
            .output_families
            .get(family)
            .ok_or(Stop::SelectedSourceMismatch)?;

        if !bindings.iter().any(|(candidate, _)| *candidate == binding) {
            return Err(Stop::SelectedSourceMismatch);
        }
        let mut selected: Option<(usize, u64, &Arc<OnceLock<RecordedOutput>>)> = None;
        for (binding, _) in bindings {
            let source = SemanticSource {
                runtime_authority: scope.runtime_authority(),
                schema: scope.binding_identity().clone(),
                scope: scope.scope(),
                output_binding: self
                    .binding_identity(*binding)
                    .ok_or(Stop::SelectedSourceMismatch)?,
            };
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            let mut depth = 0;
            loop {
                let cell = self.latest_cell_in_partition(&source, coordinate, partition);

                if let Some(cell) = cell {
                    let row = cell.get().ok_or(Stop::SelectedSourceMismatch)?;
                    let row_generation = row.settlement_identity.coordinate().generation;
                    if selected.is_none_or(|(old_depth, old_generation, _)| {
                        depth < old_depth || (depth == old_depth && row_generation > old_generation)
                    }) {
                        selected = Some((depth, row_generation, cell));
                    } else if selected.is_some_and(|(old_depth, old_generation, old)| {
                        depth == old_depth
                            && row_generation == old_generation
                            && old.get().map(|old| &old.settlement_identity)
                                != Some(&row.settlement_identity)
                    }) {
                        return Err(Stop::SelectedSourceMismatch);
                    }
                    break;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
                depth += 1;
            }
        }

        let Some((_, _, cell)) = selected else {
            return Ok((family.to_owned(), None));
        };
        Ok((
            family.to_owned(),
            Some(SelectedRequiredPublicationHead::retain(cell)?),
        ))
    }
}
