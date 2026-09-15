use std::collections::{BTreeMap, BTreeSet};

use crate::boundary::errors::WorthSignalJsError;
use crate::runtime::adapters::RuntimeDefinitionEnvelope;
use crate::runtime::compute_callbacks;
use crate::runtime::compute_callbacks::{ComputeCallbackInvocationResult, ComputeCallbackToken};

use super::worker_definition_publication_plan::{
    preflight_definition_envelope_publication, DefinitionPublicationPlan, DefinitionPublicationStep,
};
use super::RuntimeCore;

#[derive(Debug, Clone)]
pub(crate) struct DefinitionEnvelopeCallbackReattachment {
    pub callback_id: String,
    pub token: ComputeCallbackToken,
    pub invocation: ComputeCallbackInvocationResult,
}

impl RuntimeCore {
    pub(crate) fn publish_callback_free_definition_envelope(
        &mut self,
        envelope: RuntimeDefinitionEnvelope,
    ) -> Result<(u64, u64), WorthSignalJsError> {
        if !envelope.unavailable_callbacks.is_empty() {
            return Err(WorthSignalJsError::invalid_input(
                "callback-free definition publication received callback artifacts",
            ));
        }
        let plan = preflight_definition_envelope_publication(self, &envelope)?;
        let published_source_count = envelope.sources.len() as u64;
        let published_recipe_count = envelope.recipes.len() as u64;
        publish_planned_callback_free_definition_envelope_parts(self, envelope, plan)?;
        Ok((published_source_count, published_recipe_count))
    }

    pub(crate) fn publish_definition_envelope_with_callback_reattachments(
        &mut self,
        envelope: RuntimeDefinitionEnvelope,
        reattachments: Vec<DefinitionEnvelopeCallbackReattachment>,
    ) -> Result<u64, WorthSignalJsError> {
        let required_callback_ids = envelope
            .unavailable_callbacks
            .iter()
            .map(|artifact| artifact.id.clone())
            .collect::<BTreeSet<_>>();
        if required_callback_ids.is_empty() {
            reject_unexpected_callback_free_publication_reattachments(reattachments)?;
            publish_callback_free_definition_envelope(self, envelope)?;
            return Ok(0);
        }

        let mut reattachments_by_id = collect_unique_callback_reattachments(reattachments)?;
        reject_missing_callback_reattachments(&required_callback_ids, &reattachments_by_id)?;
        reject_unexpected_callback_reattachments(&required_callback_ids, &reattachments_by_id)?;
        validate_callback_reattachment_frontiers(&envelope, &reattachments_by_id)?;
        let publication_plan = match preflight_definition_envelope_publication(self, &envelope) {
            Ok(plan) => plan,
            Err(error) => {
                dispose_callback_reattachment_map(reattachments_by_id);
                return Err(error);
            }
        };

        if let Err(error) = publish_definition_envelope_parts_with_callback_reattachments(
            self,
            envelope,
            publication_plan,
            &required_callback_ids,
            &mut reattachments_by_id,
        ) {
            dispose_callback_reattachment_map(reattachments_by_id);
            return Err(error);
        }
        Ok(required_callback_ids.len() as u64)
    }
}

fn publish_definition_envelope_parts_with_callback_reattachments(
    runtime: &mut RuntimeCore,
    envelope: RuntimeDefinitionEnvelope,
    publication_plan: DefinitionPublicationPlan,
    required_callback_ids: &BTreeSet<String>,
    reattachments_by_id: &mut BTreeMap<String, DefinitionEnvelopeCallbackReattachment>,
) -> Result<(), WorthSignalJsError> {
    let worker_public_output_ids = envelope.worker_public_output_ids;
    for family in envelope.source_families {
        runtime.define_source_family(family)?;
    }
    for source in envelope.sources {
        runtime.define_source(source)?;
    }
    for family in envelope.recipe_families {
        runtime.define_keyed_recipe_family(family)?;
    }
    let mut recipes_by_id = envelope
        .recipes
        .into_iter()
        .map(|recipe| (recipe.id.clone(), recipe))
        .collect::<BTreeMap<_, _>>();
    for step in publication_plan.dynamic_steps {
        match step {
            DefinitionPublicationStep::Recipe(recipe_id) => {
                let recipe = recipes_by_id.remove(&recipe_id).ok_or_else(|| {
                    WorthSignalJsError::invalid_input(format!(
                        "definition publication plan referenced missing recipe `{recipe_id}`"
                    ))
                })?;
                runtime.define_recipe(recipe)?;
            }
            DefinitionPublicationStep::Callback(callback_id) => {
                if !required_callback_ids.contains(&callback_id) {
                    return Err(WorthSignalJsError::invalid_input(format!(
                        "definition publication plan referenced unexpected callback `{callback_id}`"
                    )));
                }
                let reattachment = reattachments_by_id.remove(&callback_id).ok_or_else(|| {
                    WorthSignalJsError::invalid_input(format!(
                        "definition envelope publication requires callback reattachment for `{callback_id}`"
                    ))
                })?;
                let reattachment_token = reattachment.token;
                if let Err(error) = runtime.install_web_computed_callback_recipe(
                    reattachment.callback_id,
                    reattachment_token,
                    reattachment.invocation,
                ) {
                    let _ = compute_callbacks::dispose_compute(reattachment_token);
                    return Err(error);
                }
            }
        }
    }
    runtime.mark_worker_public_outputs(worker_public_output_ids)?;
    Ok(())
}

fn publish_callback_free_definition_envelope(
    runtime: &mut RuntimeCore,
    envelope: RuntimeDefinitionEnvelope,
) -> Result<(), WorthSignalJsError> {
    let plan = preflight_definition_envelope_publication(runtime, &envelope)?;
    if plan
        .dynamic_steps
        .iter()
        .any(|step| matches!(step, DefinitionPublicationStep::Callback(_)))
    {
        return Err(WorthSignalJsError::invalid_input(
            "callback-free definition publication received callback plan steps",
        ));
    }
    publish_planned_callback_free_definition_envelope_parts(runtime, envelope, plan)
}

/// Publishes every part of a callback-free definition envelope, including
/// its public output marks. `worker_public_output_ids` is part of the
/// definition: a runtime built from an envelope must treat the same recipes
/// as published outputs (standing demand at commit) as the runtime that
/// exported it, or the two commit different truths for the same transaction.
fn publish_planned_callback_free_definition_envelope_parts(
    runtime: &mut RuntimeCore,
    envelope: RuntimeDefinitionEnvelope,
    publication_plan: DefinitionPublicationPlan,
) -> Result<(), WorthSignalJsError> {
    let worker_public_output_ids = envelope.worker_public_output_ids;
    for family in envelope.source_families {
        runtime.define_source_family(family)?;
    }
    for source in envelope.sources {
        runtime.define_source(source)?;
    }
    for family in envelope.recipe_families {
        runtime.define_keyed_recipe_family(family)?;
    }
    let mut recipes_by_id = envelope
        .recipes
        .into_iter()
        .map(|recipe| (recipe.id.clone(), recipe))
        .collect::<BTreeMap<_, _>>();
    for step in publication_plan.dynamic_steps {
        let DefinitionPublicationStep::Recipe(recipe_id) = step else {
            return Err(WorthSignalJsError::invalid_input(
                "callback-free definition publication received callback plan steps",
            ));
        };
        let recipe = recipes_by_id.remove(&recipe_id).ok_or_else(|| {
            WorthSignalJsError::invalid_input(format!(
                "definition publication plan referenced missing recipe `{recipe_id}`"
            ))
        })?;
        runtime.define_recipe(recipe)?;
    }
    runtime.mark_worker_public_outputs(worker_public_output_ids)?;
    Ok(())
}

fn reject_unexpected_callback_free_publication_reattachments(
    reattachments: Vec<DefinitionEnvelopeCallbackReattachment>,
) -> Result<(), WorthSignalJsError> {
    if reattachments.is_empty() {
        return Ok(());
    }
    let ids = reattachments
        .iter()
        .map(|reattachment| reattachment.callback_id.clone())
        .collect::<Vec<_>>()
        .join(", ");
    dispose_callback_reattachments(reattachments);
    Err(WorthSignalJsError::invalid_input(format!(
        "definition envelope publication received unexpected callback reattachments: {ids}"
    )))
}

fn collect_unique_callback_reattachments(
    reattachments: Vec<DefinitionEnvelopeCallbackReattachment>,
) -> Result<BTreeMap<String, DefinitionEnvelopeCallbackReattachment>, WorthSignalJsError> {
    let mut reattachments_by_id = BTreeMap::new();
    let mut duplicate_ids = Vec::new();
    for reattachment in reattachments {
        let callback_id = reattachment.callback_id.clone();
        if reattachments_by_id.contains_key(&callback_id) {
            let _ = compute_callbacks::dispose_compute(reattachment.token);
            duplicate_ids.push(callback_id);
        } else {
            reattachments_by_id.insert(callback_id, reattachment);
        }
    }
    if duplicate_ids.is_empty() {
        return Ok(reattachments_by_id);
    }
    dispose_callback_reattachment_map(reattachments_by_id);
    Err(WorthSignalJsError::invalid_input(format!(
        "definition envelope publication received duplicate callback reattachments: {}",
        duplicate_ids.join(", ")
    )))
}

fn reject_missing_callback_reattachments(
    required_callback_ids: &BTreeSet<String>,
    reattachments_by_id: &BTreeMap<String, DefinitionEnvelopeCallbackReattachment>,
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
        "definition envelope publication is missing callback reattachments: {}",
        missing.join(", ")
    )))
}

fn reject_unexpected_callback_reattachments(
    required_callback_ids: &BTreeSet<String>,
    reattachments_by_id: &BTreeMap<String, DefinitionEnvelopeCallbackReattachment>,
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
        "definition envelope publication received unexpected callback reattachments: {}",
        unexpected.join(", ")
    )))
}

fn validate_callback_reattachment_frontiers(
    envelope: &RuntimeDefinitionEnvelope,
    reattachments_by_id: &BTreeMap<String, DefinitionEnvelopeCallbackReattachment>,
) -> Result<(), WorthSignalJsError> {
    for artifact in &envelope.unavailable_callbacks {
        let reattachment = reattachments_by_id.get(&artifact.id).ok_or_else(|| {
            WorthSignalJsError::invalid_input(format!(
                "definition envelope publication requires callback reattachment for `{}`",
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
                "definition envelope publication reattachment for `{}` must preserve exported callback read frontier",
                artifact.id
            )));
        }
    }
    Ok(())
}

fn dispose_callback_reattachments(reattachments: Vec<DefinitionEnvelopeCallbackReattachment>) {
    for reattachment in reattachments {
        let _ = compute_callbacks::dispose_compute(reattachment.token);
    }
}

fn dispose_callback_reattachment_map(
    reattachments_by_id: BTreeMap<String, DefinitionEnvelopeCallbackReattachment>,
) {
    dispose_callback_reattachments(reattachments_by_id.into_values().collect());
}
