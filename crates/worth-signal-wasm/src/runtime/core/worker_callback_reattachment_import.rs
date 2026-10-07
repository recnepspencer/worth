use std::collections::{BTreeMap, BTreeSet};

use crate::boundary::errors::WorthSignalJsError;
use crate::runtime::adapters::RuntimeEnvelope;
use crate::runtime::compute_callbacks;
use crate::runtime::compute_callbacks::{ComputeCallbackInvocationResult, ComputeCallbackToken};

use super::RuntimeCore;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeEnvelopeCallbackReattachment {
    pub callback_id: String,
    pub token: ComputeCallbackToken,
    pub invocation: ComputeCallbackInvocationResult,
}

impl RuntimeCore {
    pub(crate) fn replace_runtime_envelope_with_callback_reattachments(
        &mut self,
        envelope: RuntimeEnvelope,
        reattachments: Vec<RuntimeEnvelopeCallbackReattachment>,
    ) -> Result<u64, WorthSignalJsError> {
        let required_callback_ids = envelope
            .definitions
            .unavailable_callbacks
            .iter()
            .map(|artifact| artifact.id.clone())
            .collect::<BTreeSet<_>>();
        if required_callback_ids.is_empty() {
            self.reject_or_import_callback_free_envelope(envelope, reattachments)?;
            return Ok(0);
        }
        let mut reattachments_by_id = collect_unique_callback_reattachments(reattachments)?;
        reject_missing_callback_reattachments(&required_callback_ids, &reattachments_by_id)?;
        reject_unexpected_callback_reattachments(&required_callback_ids, &reattachments_by_id)?;
        validate_callback_reattachment_frontiers(&envelope, &reattachments_by_id)?;

        let rebuilt = match rebuild_runtime_with_callback_reattachments(
            envelope,
            &required_callback_ids,
            &mut reattachments_by_id,
        ) {
            Ok(rebuilt) => rebuilt,
            Err(error) => {
                dispose_callback_reattachment_map(reattachments_by_id);
                return Err(error);
            }
        };
        *self = rebuilt;
        Ok(required_callback_ids.len() as u64)
    }

    fn reject_or_import_callback_free_envelope(
        &mut self,
        envelope: RuntimeEnvelope,
        reattachments: Vec<RuntimeEnvelopeCallbackReattachment>,
    ) -> Result<(), WorthSignalJsError> {
        if !reattachments.is_empty() {
            let ids = reattachments
                .iter()
                .map(|reattachment| reattachment.callback_id.clone())
                .collect::<Vec<_>>()
                .join(", ");
            dispose_callback_reattachments(reattachments);
            return Err(WorthSignalJsError::invalid_input(format!(
                "runtime envelope import received unexpected callback reattachments: {ids}"
            )));
        }
        self.replace_runtime_envelope(envelope)
    }
}

fn rebuild_runtime_with_callback_reattachments(
    mut envelope: RuntimeEnvelope,
    required_callback_ids: &BTreeSet<String>,
    reattachments_by_id: &mut BTreeMap<String, RuntimeEnvelopeCallbackReattachment>,
) -> Result<RuntimeCore, WorthSignalJsError> {
    let mut rebuilt = RuntimeCore::new(envelope.definitions.policy.clone())?;
    for family in envelope.definitions.source_families.clone() {
        rebuilt.define_source_family(family)?;
    }
    for family in envelope.definitions.recipe_families.clone() {
        rebuilt.define_keyed_recipe_family(family)?;
    }
    for source in envelope.definitions.sources.clone() {
        rebuilt.define_source(source)?;
    }
    for recipe in envelope.definitions.recipes.clone() {
        rebuilt.define_recipe(recipe)?;
    }
    for callback_id in required_callback_ids {
        let reattachment = reattachments_by_id.remove(callback_id).ok_or_else(|| {
            WorthSignalJsError::invalid_input(format!(
                "runtime envelope import requires callback reattachment for `{callback_id}`"
            ))
        })?;
        rewrite_snapshot_callback_token(&mut envelope, callback_id, reattachment.token)?;
        let reattachment_token = reattachment.token;
        if let Err(error) = rebuilt.install_web_computed_callback_recipe(
            reattachment.callback_id,
            reattachment_token,
            reattachment.invocation,
        ) {
            let _ = compute_callbacks::dispose_compute(reattachment_token);
            return Err(error);
        }
    }
    rebuilt.reconstruct_imported_runtime_snapshot(envelope.snapshot)?;
    rebuilt.mark_worker_public_outputs(envelope.definitions.worker_public_output_ids)?;
    Ok(rebuilt)
}

fn collect_unique_callback_reattachments(
    reattachments: Vec<RuntimeEnvelopeCallbackReattachment>,
) -> Result<BTreeMap<String, RuntimeEnvelopeCallbackReattachment>, WorthSignalJsError> {
    let mut reattachments_by_id = BTreeMap::new();
    let mut duplicate_ids = Vec::new();
    for reattachment in reattachments {
        match reattachments_by_id.entry(reattachment.callback_id.clone()) {
            std::collections::btree_map::Entry::Occupied(duplicate) => {
                let _ = compute_callbacks::dispose_compute(reattachment.token);
                duplicate_ids.push(duplicate.key().clone());
            }
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(reattachment);
            }
        }
    }
    if duplicate_ids.is_empty() {
        return Ok(reattachments_by_id);
    }
    dispose_callback_reattachment_map(reattachments_by_id);
    Err(WorthSignalJsError::invalid_input(format!(
        "runtime envelope import received duplicate callback reattachments: {}",
        duplicate_ids.join(", ")
    )))
}

fn reject_missing_callback_reattachments(
    required_callback_ids: &BTreeSet<String>,
    reattachments_by_id: &BTreeMap<String, RuntimeEnvelopeCallbackReattachment>,
) -> Result<(), WorthSignalJsError> {
    let missing = required_callback_ids
        .iter()
        .filter(|id| !reattachments_by_id.contains_key(*id))
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(());
    }
    dispose_callback_reattachment_map(reattachments_by_id.clone());
    Err(WorthSignalJsError::invalid_input(format!(
        "runtime envelope import is missing callback reattachments: {}",
        missing.join(", ")
    )))
}

fn reject_unexpected_callback_reattachments(
    required_callback_ids: &BTreeSet<String>,
    reattachments_by_id: &BTreeMap<String, RuntimeEnvelopeCallbackReattachment>,
) -> Result<(), WorthSignalJsError> {
    let unexpected = reattachments_by_id
        .keys()
        .filter(|id| !required_callback_ids.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    if unexpected.is_empty() {
        return Ok(());
    }
    dispose_callback_reattachment_map(reattachments_by_id.clone());
    Err(WorthSignalJsError::invalid_input(format!(
        "runtime envelope import received unexpected callback reattachments: {}",
        unexpected.join(", ")
    )))
}

fn validate_callback_reattachment_frontiers(
    envelope: &RuntimeEnvelope,
    reattachments_by_id: &BTreeMap<String, RuntimeEnvelopeCallbackReattachment>,
) -> Result<(), WorthSignalJsError> {
    for artifact in &envelope.definitions.unavailable_callbacks {
        let reattachment = reattachments_by_id.get(&artifact.id).ok_or_else(|| {
            WorthSignalJsError::invalid_input(format!(
                "runtime envelope import requires callback reattachment for `{}`",
                artifact.id
            ))
        })?;
        let mut reattached_reads = reattachment.invocation.captured_read_ids.clone();
        reattached_reads.sort();
        reattached_reads.dedup();
        let mut exported_reads = artifact.current_reads.clone();
        exported_reads.sort();
        exported_reads.dedup();
        if reattached_reads != exported_reads {
            dispose_callback_reattachment_map(reattachments_by_id.clone());
            return Err(WorthSignalJsError::invalid_input(format!(
                "runtime envelope import reattachment for `{}` must preserve exported callback read frontier",
                artifact.id
            )));
        }
    }
    Ok(())
}

fn rewrite_snapshot_callback_token(
    envelope: &mut RuntimeEnvelope,
    callback_id: &str,
    token: ComputeCallbackToken,
) -> Result<(), WorthSignalJsError> {
    let recipe = envelope
        .snapshot
        .state
        .recipes
        .iter_mut()
        .find(|recipe| recipe.id == callback_id)
        .ok_or_else(|| {
            WorthSignalJsError::invalid_input(format!(
                "runtime envelope import cannot find callback snapshot `{callback_id}`"
            ))
        })?;
    let callback = recipe.callback.as_mut().ok_or_else(|| {
        WorthSignalJsError::invalid_input(format!(
            "runtime envelope import requires callback snapshot for `{callback_id}`"
        ))
    })?;
    callback.token_slot = token.slot;
    callback.token_generation = token.generation;
    Ok(())
}

fn dispose_callback_reattachments(reattachments: Vec<RuntimeEnvelopeCallbackReattachment>) {
    for reattachment in reattachments {
        let _ = compute_callbacks::dispose_compute(reattachment.token);
    }
}

fn dispose_callback_reattachment_map(
    reattachments_by_id: BTreeMap<String, RuntimeEnvelopeCallbackReattachment>,
) {
    dispose_callback_reattachments(reattachments_by_id.into_values().collect());
}
