use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::history_recovery_denial::RelationalHistoryRecoveryDenial;
use std::sync::Arc;
use contracts::branch::target::RelationalBranchTargetOwnerPort as _;

use worth_foundational::FoundationalBranchTarget;

use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;

use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;

#[path = "history_catalog_recovery/checkpoint_root.rs"]
pub mod checkpoint_root;
#[path = "history_catalog_recovery/inventory.rs"]
pub mod inventory;
#[path = "history_catalog_recovery/source_binding.rs"]
pub mod source_binding;

/// The canonical envelope of a live root's commit. A historical root a
/// retained fork selected may be checkpoint-covered and cold; its envelope is
/// demanded from the same authenticated durable source.
fn live_root_envelope(
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    commit_id: contracts::history::data::coordinates::CommitId,
) -> Result<Arc<CanonicalCommitEnvelope>, RelationalHistoryRecoveryDenial> {
    if let Some(envelope) = history.recorded_commit_envelope(commit_id) {
        return Ok(envelope);
    }
    let missing = || format!("live recovery root `{}` has no envelope", commit_id.0);
    let artifact = history.commit_artifact(commit_id).ok_or_else(missing)?;
    let positioned = artifact
        .envelope_residency()
        .demand_positioned(execution, 1 << 30)
        .map_err(RelationalHistoryRecoveryDenial::Source)?;
    Ok(positioned.canonical_arc().clone())
}

fn rebuild_catalog_with_live_roots(
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    symbols: &contracts::symbols::data::string_interner::StringInterner,
) -> Result<(), RelationalHistoryRecoveryDenial> {
    rebuild_catalog_with_live_roots_and_descriptors(
        history,
        execution,
        &std::collections::BTreeMap::new(),
        &std::collections::BTreeMap::new(),
        &std::collections::BTreeMap::new(),
        symbols,
    )
}

fn rebuild_catalog_with_live_roots_and_descriptors(
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    descriptors: &std::collections::BTreeMap<
        contracts::history::data::coordinates::CommitId,
        contracts::branch::target::RelationalBranchRootDescriptor,
    >,
    additional_roots: &std::collections::BTreeMap<
        contracts::history::data::coordinates::CommitId,
        Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
    >,
    admitted: &std::collections::BTreeMap<
        contracts::history::data::coordinates::CommitId,
        Arc<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>,
    >,
    symbols: &contracts::symbols::data::string_interner::StringInterner,
) -> Result<(), RelationalHistoryRecoveryDenial> {
    let mut roots = additional_roots.clone();
    let mut cells = history.branch_cells.take_all();
    let replacements = (|| -> Result<_, RelationalHistoryRecoveryDenial> {
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
                    return Err("empty recovery branch carries a committed root"
                        .to_owned()
                        .into());
                }
                continue;
            }
            let commit_id = root
                .commit_id()
                .ok_or_else(|| "live recovery root has no commit identity".to_owned())?;
            if checkpoint_root::is_readmitted_checkpoint_root(history, &root)? {
                if roots
                    .get(&commit_id)
                    .is_some_and(|actual| !Arc::ptr_eq(actual, &root))
                {
                    return Err("recovery carries competing checkpoint root owners"
                        .to_owned()
                        .into());
                }
                roots.insert(commit_id, root);
                continue;
            }
            let canonical = live_root_envelope(history, execution, commit_id)?;
            let canonical_root = if let Some(existing) = roots.get(&commit_id) {
                Arc::clone(existing)
            } else if root.links_envelope(&canonical) {
                Arc::clone(&root)
            } else {
                let original =
                    source_binding::original_for_envelope(history, execution, &canonical)?;
                if let Some(source) = admitted.get(&commit_id) {
                    root.readmit_canonical_envelope(execution, source, original, symbols)
                } else {
                    root.relink_canonical_envelope(execution, original, symbols)
                }
                .map_err(RelationalHistoryRecoveryDenial::Root)?
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
        Ok::<_, RelationalHistoryRecoveryDenial>(replacements)
    })();
    let replacements = match replacements {
        Ok(replacements) => replacements,
        Err(error) => {
            history.branch_cells.restore_all(cells);
            return Err(error);
        }
    };
    let reservations = replacements
        .into_iter()
        .map(|(branch_id, previous_root, next_root)| {
            let cell = cells
                .get(&branch_id)
                .expect(/* Proof: rebuild_catalog_with_live_roots_and_descriptors/readmit_replayed_root_descriptor selects these branch ids directly from cells.values(), then retains the detached BTreeMap in this local variable. No key insertion or removal occurs before this lookup; installing roots changes only the selected cell values. */ "planned recovery branch remains in detached registry");
            history.reserve_branch_head_retirement(
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
            history.branch_cells.restore_all(cells);
            return Err(error.into());
        }
    };
    for (branch_id, reservation, _previous_head_custody, next_root) in reservations {
        cells
            .get_mut(&branch_id)
            .expect(/* Proof: rebuild_catalog_with_live_roots_and_descriptors/readmit_replayed_root_descriptor selects these branch ids directly from cells.values(), then retains the detached BTreeMap in this local variable. No key insertion or removal occurs before this lookup; installing roots changes only the selected cell values. */ "reserved recovery branch remains in detached registry")
            .install_root(Arc::clone(&next_root));
        reservation.transfer_head(&next_root).replace_head();
    }
    history.branch_cells.restore_all(cells);
    let envelopes = history.recorded_commit_envelopes();
    let replaced = envelopes
        .iter()
        .map(|envelope| envelope.commit.commit_id)
        .collect();
    let mut catalog = history
        .ledger
        .read()
        .commit_catalog
        .retained_entries_except(&replaced);
    for envelope in envelopes {
        let commit_id = envelope.commit.commit_id;
        let preencoded = history.commit_artifact(commit_id).ok_or_else(|| {
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
        let artifact = artifact.prepare_catalog_owner(execution).map_err(|cause| {
            RelationalHistoryRecoveryDenial::Root(
                contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Request(
                    cause,
                ),
            )
        })?;
        catalog
            .append_preencoded_recovery(artifact)
            .map_err(|denial| format!("recovered catalog root linkage denied: {denial:?}"))?;
    }
    history.install_commit_catalog(catalog);
    Ok(())
}

pub(crate) fn readmit_replayed_root_descriptor(
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    commit_id: contracts::history::data::coordinates::CommitId,
    descriptor: contracts::branch::target::RelationalBranchRootDescriptor,
    replayed_root: Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
    symbols: &contracts::symbols::data::string_interner::StringInterner,
) -> Result<
    Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
    RelationalHistoryRecoveryDenial,
> {
    if replayed_root.commit_id() != Some(commit_id) {
        return Err(format!(
            "recovery root `{}` was supplied for a different commit",
            commit_id.0
        )
        .into());
    }
    let canonical_root = if replayed_root.descriptor() == Some(&descriptor) {
        replayed_root
    } else {
        replayed_root
            .readmit_descriptor(execution, descriptor, symbols)
            .map_err(|denial| format!("recovery root descriptor denied: {denial:?}"))?
    };
    let mut cells = history.branch_cells.take_all();
    let reservations = cells
        .values()
        .filter_map(|cell| {
            let previous_root = cell.root()?;
            (previous_root.commit_id() == Some(commit_id)
                && !Arc::ptr_eq(&previous_root, &canonical_root))
            .then_some((cell, previous_root))
        })
        .map(|(cell, previous_root)| {
            history
                .reserve_branch_head_retirement(
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
            history.branch_cells.restore_all(cells);
            return Err(error.into());
        }
    };
    for (branch_id, reservation, _previous_head_custody) in reservations {
        cells
            .get_mut(&branch_id)
            .expect(/* Proof: rebuild_catalog_with_live_roots_and_descriptors/readmit_replayed_root_descriptor selects these branch ids directly from cells.values(), then retains the detached BTreeMap in this local variable. No key insertion or removal occurs before this lookup; installing roots changes only the selected cell values. */ "reserved recovery branch remains in detached registry")
            .install_root(Arc::clone(&canonical_root));
        reservation.transfer_head(&canonical_root).replace_head();
    }
    history.branch_cells.restore_all(cells);
    let descriptors = history
        .commit_artifacts()
        .into_iter()
        .map(|artifact| (artifact.commit_id(), artifact.roots().clone()))
        .collect();
    let mut recovered_roots = std::collections::BTreeMap::new();
    recovered_roots.insert(commit_id, Arc::clone(&canonical_root));
    rebuild_catalog_with_live_roots_and_descriptors(
        history,
        execution,
        &descriptors,
        &recovered_roots,
        &std::collections::BTreeMap::new(),
        symbols,
    )?;
    Ok(canonical_root)
}

pub fn record_recovered_commit(
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    envelope: &Arc<CanonicalCommitEnvelope>,
    allow_reconstructed_replacement: bool,
    advance_branch_currentness: bool,
    symbols: &contracts::symbols::data::string_interner::StringInterner,
) -> Result<(), RelationalHistoryRecoveryDenial> {
    let catalog_artifact = history.commit_artifact(envelope.commit.commit_id);
    let conflicts = catalog_artifact
        .as_ref()
        .map(|artifact| artifact.matches_envelope(envelope))
        .transpose()
        .map_err(|denial| format!("recovered catalog comparison denied: {denial:?}"))?
        .is_some_and(|matches| !matches);
    if conflicts {
        if !allow_reconstructed_replacement {
            return Err(format!(
                "recovery commit artifact conflicts for commit {}",
                envelope.commit.commit_id.0
            )
            .into());
        }
        rebuild_catalog_with_live_roots(history, execution, symbols)?;
    } else if catalog_artifact.is_none() {
        let artifact = source_binding::prepare_catalog_artifact_for_envelope(
            history,
            execution,
            Arc::clone(envelope),
            None,
        )?;
        history
            .with_ledger_mut(|ledger| ledger.install_source_recovery(None, artifact))
            .map_err(RelationalHistoryRecoveryDenial::Catalog)?;
    }
    history.require_recovered_branch(&envelope.branch_context)?;
    if !advance_branch_currentness {
        return Ok(());
    }
    let roots = history
        .commit_artifact(envelope.commit.commit_id)
        .map(|artifact| artifact.roots().clone())
        .ok_or_else(|| "recovered commit must have a catalog artifact".to_owned())?;
    let target = contracts::branch::target::RelationalBranchTarget::from_commit_receipt(
        history.runtime_instance_id,
        &envelope.commit,
        roots,
    );
    let already_current = history
        .branch_cell(&envelope.branch_context)
        .is_some_and(|cell| match cell.observation().target() {
            FoundationalBranchTarget::Basis(current) => {
                current.selected_commit_id() == envelope.commit.commit_id.0
                    && current.version_id() == envelope.commit.version_id.0
            }
            FoundationalBranchTarget::Empty => false,
        });
    if !already_current {
        history
            .branch_cell_mut(&envelope.branch_context)
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
