use std::sync::Arc;

use worth_foundational::FoundationalBranchTarget;

use crate::history::data::CanonicalCommitEnvelope;
use crate::history::RelationalCommitCatalog;

use super::HistorySubsystem;

impl HistorySubsystem {
    pub(crate) fn install_recovered_canonical_inventory(
        &mut self,
        commits: Vec<Arc<crate::history::data::PositionedCanonicalCommit>>,
    ) -> Result<(), String> {
        self.rebuild_recovered_canonical_routes(commits.iter().cloned())
            .map_err(str::to_owned)?;
        self.with_ledger_mut(|ledger| {
            ledger.commit_envelopes = commits
                .iter()
                .map(|commit| {
                    (
                        commit.envelope().commit.commit_id,
                        Arc::clone(commit.canonical_arc()),
                    )
                })
                .collect();
            ledger.patch_stream_index = commits
                .iter()
                .map(|commit| (commit.position(), commit.envelope().commit.commit_id))
                .collect();
            for commit in commits {
                ledger
                    .commit_catalog
                    .synchronize_recovered_envelope(Arc::clone(commit.canonical_arc()))
                    .map_err(|denial| {
                        format!("recovered canonical catalog synchronization denied: {denial:?}")
                    })?;
            }
            Ok(())
        })
    }

    pub(super) fn rebuild_catalog_with_live_roots(
        &self,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<(), String> {
        self.rebuild_catalog_with_live_roots_and_descriptors(
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            symbols,
        )
    }

    fn rebuild_catalog_with_live_roots_and_descriptors(
        &self,
        descriptors: &std::collections::BTreeMap<
            crate::history::data::CommitId,
            crate::branch::RelationalBranchRootDescriptor,
        >,
        additional_roots: &std::collections::BTreeMap<
            crate::history::data::CommitId,
            Arc<crate::branch::RelationalBranchRoot>,
        >,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<(), String> {
        let mut roots = additional_roots.clone();
        let mut cells = self.branch_cells.take_all();
        let replacements = (|| {
            let mut replacements = Vec::new();
            for cell in cells.values() {
                let Some(root) = cell.root() else {
                    continue;
                };
                if matches!(
                    cell.observation().target(),
                    worth_foundational::FoundationalBranchTarget::Empty
                ) {
                    if root.id() != 0 || root.descriptor().is_some() {
                        return Err("empty recovery branch carries a committed root".to_owned());
                    }
                    continue;
                }
                let commit_id = root
                    .commit_id()
                    .ok_or_else(|| "live recovery root has no commit identity".to_owned())?;
                let canonical = self.recorded_commit_envelope(commit_id).ok_or_else(|| {
                    format!("live recovery root `{}` has no envelope", commit_id.0)
                })?;
                let canonical_root = if let Some(existing) = roots.get(&commit_id) {
                    Arc::clone(existing)
                } else if root.links_envelope(&canonical) {
                    Arc::clone(&root)
                } else {
                    root.relink_canonical_envelope(canonical, symbols)
                        .map_err(|denial| {
                            format!("recovery root canonical relink denied: {denial:?}")
                        })?
                };
                if !Arc::ptr_eq(&root, &canonical_root) {
                    replacements.push((
                        cell.identity().branch_id().clone(),
                        root,
                        Arc::clone(&canonical_root),
                    ));
                }
                roots.insert(commit_id, canonical_root);
            }
            Ok::<_, String>(replacements)
        })();
        let replacements = match replacements {
            Ok(replacements) => replacements,
            Err(error) => {
                self.branch_cells.restore_all(cells);
                return Err(error);
            }
        };
        let reservations = replacements
            .into_iter()
            .map(|(branch_id, previous_root, next_root)| {
                let cell = cells
                    .get(&branch_id)
                    .expect("planned recovery branch remains in detached registry");
                self.reserve_branch_head_retirement(
                    cell.identity(),
                    &previous_root,
                    cell.head_retention(),
                )
                .map(|reservation| (branch_id, reservation, previous_root, next_root))
                .map_err(|denial| format!("recovery root replacement retention denied: {denial:?}"))
            })
            .collect::<Result<Vec<_>, _>>();
        let reservations = match reservations {
            Ok(reservations) => reservations,
            Err(error) => {
                self.branch_cells.restore_all(cells);
                return Err(error);
            }
        };
        for (branch_id, mut reservation, previous_root, next_root) in reservations {
            cells
                .get_mut(&branch_id)
                .expect("reserved recovery branch remains in detached registry")
                .install_root(Arc::clone(&next_root));
            reservation.transfer_head(&previous_root, &next_root);
            reservation.replace_head(previous_root);
        }
        self.branch_cells.restore_all(cells);
        let mut catalog = RelationalCommitCatalog::default();
        for envelope in self.recorded_commit_envelopes() {
            let commit_id = envelope.commit.commit_id;
            let preencoded = self.commit_artifact(commit_id).ok_or_else(|| {
                format!(
                    "recovered canonical catalog omitted commit `{}`",
                    commit_id.0
                )
            })?;
            let artifact = preencoded
                .relink_preencoded_recovery(
                    &envelope,
                    roots.get(&commit_id),
                    descriptors.get(&commit_id),
                )
                .map_err(|denial| format!("recovered catalog root linkage denied: {denial:?}"))?;
            catalog
                .append_preencoded_recovery(artifact)
                .map_err(|denial| format!("recovered catalog root linkage denied: {denial:?}"))?;
        }
        self.install_commit_catalog(catalog);
        Ok(())
    }

    pub(super) fn readmit_replayed_root_descriptor(
        &self,
        commit_id: crate::history::data::CommitId,
        descriptor: crate::branch::RelationalBranchRootDescriptor,
        replayed_root: Arc<crate::branch::RelationalBranchRoot>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<crate::branch::RelationalBranchRoot>, String> {
        if replayed_root.commit_id() != Some(commit_id) {
            return Err(format!(
                "recovery root `{}` was supplied for a different commit",
                commit_id.0
            ));
        }
        let canonical_root = if replayed_root.descriptor() == Some(&descriptor) {
            replayed_root
        } else {
            replayed_root
                .readmit_descriptor(descriptor, symbols)
                .map_err(|denial| format!("recovery root descriptor denied: {denial:?}"))?
        };
        let mut cells = self.branch_cells.take_all();
        let reservations = cells
            .values()
            .filter_map(|cell| {
                let previous_root = cell.root()?;
                (previous_root.commit_id() == Some(commit_id)
                    && !Arc::ptr_eq(&previous_root, &canonical_root))
                .then_some((cell, previous_root))
            })
            .map(|(cell, previous_root)| {
                self.reserve_branch_head_retirement(
                    cell.identity(),
                    &previous_root,
                    cell.head_retention(),
                )
                .map(|reservation| {
                    (
                        cell.identity().branch_id().clone(),
                        reservation,
                        previous_root,
                    )
                })
                .map_err(|denial| format!("recovery root replacement retention denied: {denial:?}"))
            })
            .collect::<Result<Vec<_>, _>>();
        let reservations = match reservations {
            Ok(reservations) => reservations,
            Err(error) => {
                self.branch_cells.restore_all(cells);
                return Err(error);
            }
        };
        for (branch_id, mut reservation, previous_root) in reservations {
            cells
                .get_mut(&branch_id)
                .expect("reserved recovery branch remains in detached registry")
                .install_root(Arc::clone(&canonical_root));
            reservation.transfer_head(&previous_root, &canonical_root);
            reservation.replace_head(previous_root);
        }
        self.branch_cells.restore_all(cells);
        let descriptors = self
            .commit_artifacts()
            .into_iter()
            .map(|artifact| (artifact.commit_id(), artifact.roots().clone()))
            .collect();
        let mut recovered_roots = std::collections::BTreeMap::new();
        recovered_roots.insert(commit_id, Arc::clone(&canonical_root));
        self.rebuild_catalog_with_live_roots_and_descriptors(
            &descriptors,
            &recovered_roots,
            symbols,
        )?;
        Ok(canonical_root)
    }

    pub(crate) fn record_recovered_commit(
        &self,
        envelope: &CanonicalCommitEnvelope,
        allow_reconstructed_replacement: bool,
        advance_branch_currentness: bool,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<(), String> {
        let catalog_artifact = self.commit_artifact(envelope.commit.commit_id);
        if catalog_artifact
            .as_ref()
            .is_some_and(|artifact| artifact.envelope().as_ref() != envelope)
        {
            if !allow_reconstructed_replacement {
                return Err(format!(
                    "recovery commit artifact conflicts for commit {}",
                    envelope.commit.commit_id.0
                ));
            }
            self.rebuild_catalog_with_live_roots(symbols)?;
        } else if catalog_artifact.is_none() {
            self.with_ledger_mut(|ledger| {
                ledger
                    .commit_catalog
                    .append_envelope(Arc::new(envelope.clone()))
            })
            .map_err(|denial| {
                format!(
                    "recovery commit artifact could not be admitted for commit {}: {denial:?}",
                    envelope.commit.commit_id.0
                )
            })?;
        }
        self.require_recovered_branch(&envelope.branch_context)?;
        if !advance_branch_currentness {
            return Ok(());
        }
        let roots = self
            .commit_artifact(envelope.commit.commit_id)
            .map(|artifact| artifact.roots().clone())
            .ok_or_else(|| "recovered commit must have a catalog artifact".to_owned())?;
        let target = crate::branch::RelationalBranchTarget::from_commit_receipt(
            self.runtime_instance_id,
            &envelope.commit,
            roots,
        );
        let already_current = self
            .branch_cell(&envelope.branch_context)
            .is_some_and(|cell| match cell.observation().target() {
                FoundationalBranchTarget::Basis(current) => {
                    current.selected_commit_id() == envelope.commit.commit_id.0
                        && current.version_id() == envelope.commit.version_id.0
                }
                FoundationalBranchTarget::Empty => false,
            });
        if !already_current {
            self.branch_cell_mut(&envelope.branch_context)
                .ok_or_else(|| {
                    format!(
                        "recovered branch cell missing for `{}`",
                        envelope.branch_context.0
                    )
                })?
                .advance_truth(FoundationalBranchTarget::basis(target))
                .map_err(|denial| format!("recovered branch reference denied: {denial:?}"))?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn replace_catalog_from_legacy_for_test(&self) {
        let mut catalog = RelationalCommitCatalog::default();
        for envelope in self.recorded_commit_envelopes() {
            catalog
                .append_envelope(envelope)
                .expect("test envelopes retain ordered unique parentage");
        }
        self.install_commit_catalog(catalog);
    }
}
