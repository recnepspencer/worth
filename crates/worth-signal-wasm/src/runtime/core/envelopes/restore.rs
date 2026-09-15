use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use crate::boundary::errors::WorthSignalJsError;
use crate::recipe::model::{RecipeSpec, SetValue, TransactionOp};
use crate::runtime::adapters::{RuntimeDefinitionEnvelope, RuntimeEnvelope};
use crate::runtime::summaries::{RuntimeSnapshotEnvelope, RuntimeStoreSnapshot};

use super::{
    ExactRuntimeRestoreArtifact, RuntimeCore, CALLBACK_UNAVAILABLE_FOR_RUNTIME_ENVELOPE_IMPORT,
};

impl RuntimeCore {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn replace_runtime_envelope(
        &mut self,
        envelope: RuntimeEnvelope,
    ) -> Result<(), WorthSignalJsError> {
        reject_unavailable_callbacks(self, &envelope.definitions.unavailable_callbacks)?;

        let mut rebuilt = RuntimeCore::new(envelope.definitions.policy.clone())?;
        for family in envelope.definitions.source_families {
            rebuilt.define_source_family(family)?;
        }
        for family in envelope.definitions.recipe_families {
            rebuilt.define_keyed_recipe_family(family)?;
        }
        let source_ids = envelope
            .definitions
            .sources
            .iter()
            .map(|source| source.id.clone())
            .collect::<Vec<_>>();
        for source in envelope.definitions.sources {
            rebuilt.define_source(source)?;
        }
        let worker_public_output_ids = envelope.definitions.worker_public_output_ids;
        define_recipes_in_dependency_order(&mut rebuilt, envelope.definitions.recipes, source_ids)?;
        rebuilt.reconstruct_imported_runtime_snapshot(envelope.snapshot)?;
        rebuilt.mark_worker_public_outputs(worker_public_output_ids)?;
        *self = rebuilt;
        Ok(())
    }

    pub(crate) fn replace_runtime_envelope_portable_artifact(
        &mut self,
        definitions: RuntimeDefinitionEnvelope,
        state: RuntimeStoreSnapshot,
    ) -> Result<(), WorthSignalJsError> {
        reject_unavailable_callbacks(self, &definitions.unavailable_callbacks)?;

        let RuntimeDefinitionEnvelope {
            policy,
            sources,
            recipes,
            source_families,
            recipe_families,
            worker_public_output_ids,
            unavailable_callbacks: _,
        } = definitions;

        let mut rebuilt = RuntimeCore::new(policy)?;
        for family in source_families {
            rebuilt.define_source_family(family)?;
        }
        for family in recipe_families {
            rebuilt.define_keyed_recipe_family(family)?;
        }
        let source_ids = sources
            .iter()
            .map(|source| source.id.clone())
            .collect::<Vec<_>>();
        for source in sources {
            rebuilt.define_source(source)?;
        }
        define_recipes_in_dependency_order(&mut rebuilt, recipes, source_ids)?;

        // Public outputs are standing demand: marking them before the source
        // truth lands makes that transaction evaluate them, so the admitted
        // runtime holds evaluated truth (and diagnostics that describe it)
        // exactly as a compatibility import does, rather than a dirty graph
        // whose diagnostics change on the first read.
        rebuilt.mark_worker_public_outputs(worker_public_output_ids)?;
        apply_imported_source_truth(&mut rebuilt, &state)?;
        // The source-truth transaction leaves history_now() describing that
        // one commit. Refresh the retained views from the finished graph so
        // history_now() describes the imported graph, as it does after an
        // exact import (which carries the snapshot-time views).
        worth_signal::facade::core::refresh_retained_diagnostics_views(rebuilt.runtime.graph_mut());
        *self = rebuilt;
        Ok(())
    }

    pub(crate) fn replace_runtime_envelope_exact(
        &mut self,
        artifact: ExactRuntimeRestoreArtifact,
    ) -> Result<(), WorthSignalJsError> {
        let mut rebuilt = RuntimeCore::new(artifact.policy.clone())?;
        rebuilt.catalog = artifact.catalog;
        rebuilt.web_signals = artifact.web_signals;
        rebuilt.nodes_by_id = artifact.nodes_by_id;
        rebuilt.dense_grids = artifact.dense_grids;
        rebuilt.web_metrics = artifact.web_metrics;
        rebuilt.store = Arc::new(Mutex::new(artifact.store));
        rebuilt.callback_diagnostics = Arc::new(Mutex::new(artifact.callback_diagnostics));
        rebuilt.reconstruct_imported_runtime_snapshot(artifact.snapshot)?;
        *self = rebuilt;
        Ok(())
    }

    pub(in crate::runtime::core) fn reconstruct_imported_runtime_snapshot(
        &mut self,
        envelope: RuntimeSnapshotEnvelope,
    ) -> Result<(), WorthSignalJsError> {
        let RuntimeSnapshotEnvelope { snapshot, state } = envelope;
        let branch = self.runtime.current_branch();
        let basis = self.native_branch_basis(branch)?;
        let (admitted_snapshot, _) = self
            .runtime
            .reconstruct_signal_branch_snapshot(&basis, &snapshot)
            .map_err(|error| {
                WorthSignalJsError::invalid_input(format!(
                    "Signal snapshot reconstruction denied: {error:?}"
                ))
            })?
            .into_parts();
        self.restore_runtime_store_snapshot(state)?;
        let snapshot_key = (snapshot.meta.branch_id.0, snapshot.meta.snapshot_id.0);
        self.runtime_snapshots
            .insert(snapshot_key, snapshot.clone());
        self.admitted_runtime_snapshots
            .insert(snapshot_key, admitted_snapshot);
        self.snapshot_states
            .insert(snapshot_key, self.snapshot_branch_state());
        self.branch_states
            .insert(snapshot.meta.branch_id.0, self.snapshot_branch_state());
        Ok(())
    }
}

fn apply_imported_source_truth(
    rebuilt: &mut RuntimeCore,
    state: &RuntimeStoreSnapshot,
) -> Result<(), WorthSignalJsError> {
    if state.sources.is_empty() {
        return Ok(());
    }
    rebuilt.apply_transaction(vec![TransactionOp::SetMany {
        values: state
            .sources
            .iter()
            .map(|source| SetValue {
                id: source.id.clone(),
                value: source.value.clone(),
                aspect: None,
                aspects: source.produces_aspects.clone(),
            })
            .collect(),
    }])?;
    Ok(())
}

fn reject_unavailable_callbacks(
    runtime: &mut RuntimeCore,
    unavailable_callbacks: &[crate::runtime::adapters::UnavailableCallbackArtifact],
) -> Result<(), WorthSignalJsError> {
    if unavailable_callbacks.is_empty() {
        return Ok(());
    }

    runtime
        .web_metrics
        .compute_callback_missing_unavailability_count = runtime
        .web_metrics
        .compute_callback_missing_unavailability_count
        .saturating_add(unavailable_callbacks.len() as u64);
    let ids = unavailable_callbacks
        .iter()
        .map(|artifact| artifact.id.clone())
        .collect::<Vec<_>>()
        .join(", ");
    Err(WorthSignalJsError::callback_failure(
        CALLBACK_UNAVAILABLE_FOR_RUNTIME_ENVELOPE_IMPORT,
        format!(
            "runtime envelope import cannot restore callback-backed nodes without live callback registrations: {ids}"
        ),
        Some(ids),
    ))
}

fn define_recipes_in_dependency_order(
    rebuilt: &mut RuntimeCore,
    recipes: Vec<RecipeSpec>,
    source_ids: Vec<String>,
) -> Result<(), WorthSignalJsError> {
    let mut known_ids = source_ids.into_iter().collect::<BTreeSet<_>>();
    let mut pending = recipes;
    while !pending.is_empty() {
        let mut next_pending = Vec::new();
        let mut progressed = false;
        for recipe in pending {
            if recipe
                .reads
                .iter()
                .all(|read| known_ids.contains(read.id()))
            {
                known_ids.insert(recipe.id.clone());
                rebuilt.define_recipe(recipe)?;
                progressed = true;
            } else {
                next_pending.push(recipe);
            }
        }
        if !progressed {
            let unresolved = next_pending
                .iter()
                .map(|recipe| {
                    let missing = recipe
                        .reads
                        .iter()
                        .filter(|read| !known_ids.contains(read.id()))
                        .map(|read| read.id().to_owned())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{} -> [{missing}]", recipe.id)
                })
                .collect::<Vec<_>>()
                .join("; ");
            return Err(WorthSignalJsError::invalid_input(format!(
                "runtime envelope definitions contained unresolved recipe reads: {unresolved}"
            )));
        }
        pending = next_pending;
    }
    Ok(())
}
