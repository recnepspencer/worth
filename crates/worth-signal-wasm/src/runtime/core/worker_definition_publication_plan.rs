use std::collections::{BTreeMap, BTreeSet};

use crate::boundary::errors::WorthSignalJsError;
use crate::recipe::model::{RecipeFamilyReadSpec, RecipeReadSpec};
use crate::runtime::adapters::RuntimeDefinitionEnvelope;

use super::aspects::resolve_selected_aspects;
use super::RuntimeCore;

#[derive(Debug, Clone)]
pub(super) enum DefinitionPublicationStep {
    Recipe(String),
    Callback(String),
}

#[derive(Debug, Clone)]
pub(super) struct DefinitionPublicationPlan {
    pub(super) dynamic_steps: Vec<DefinitionPublicationStep>,
}

pub(super) fn preflight_definition_envelope_publication(
    runtime: &RuntimeCore,
    envelope: &RuntimeDefinitionEnvelope,
) -> Result<DefinitionPublicationPlan, WorthSignalJsError> {
    let mut available_signal_ids = runtime.catalog.keys().cloned().collect::<BTreeSet<_>>();
    let mut available_family_ids = runtime.lock_store().map(|store| {
        store
            .source_families
            .keys()
            .chain(store.recipe_families.keys())
            .cloned()
            .collect::<BTreeSet<_>>()
    })?;
    let mut incoming_signal_ids = BTreeSet::new();
    let mut incoming_family_ids = BTreeSet::new();

    reject_duplicate_family_ids(
        envelope,
        &mut available_family_ids,
        &mut incoming_family_ids,
    )?;
    reject_duplicate_signal_ids(
        envelope,
        &mut available_signal_ids,
        &mut incoming_signal_ids,
    )?;

    let known_signal_ids = available_signal_ids
        .union(&incoming_signal_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    let incoming_source_family_ids = envelope
        .source_families
        .iter()
        .map(|family| family.family_id.clone())
        .collect::<BTreeSet<_>>();
    let recipe_family_read_ids = available_family_ids
        .union(&incoming_source_family_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    reject_unplannable_recipe_family_reads(envelope, &known_signal_ids, &recipe_family_read_ids)?;
    reject_invalid_recipe_read_aspects(envelope)?;
    plan_dynamic_definition_publication(envelope, available_signal_ids)
}

fn reject_duplicate_family_ids(
    envelope: &RuntimeDefinitionEnvelope,
    available_family_ids: &mut BTreeSet<String>,
    incoming_family_ids: &mut BTreeSet<String>,
) -> Result<(), WorthSignalJsError> {
    for family in &envelope.source_families {
        reject_duplicate_publication_id(
            available_family_ids,
            incoming_family_ids,
            &family.family_id,
        )?;
    }
    for family in &envelope.recipe_families {
        reject_duplicate_publication_id(
            available_family_ids,
            incoming_family_ids,
            &family.family_id,
        )?;
    }
    Ok(())
}

fn reject_duplicate_signal_ids(
    envelope: &RuntimeDefinitionEnvelope,
    available_signal_ids: &mut BTreeSet<String>,
    incoming_signal_ids: &mut BTreeSet<String>,
) -> Result<(), WorthSignalJsError> {
    for source in &envelope.sources {
        reject_duplicate_publication_id(available_signal_ids, incoming_signal_ids, &source.id)?;
    }
    for recipe in &envelope.recipes {
        reject_duplicate_publication_id(available_signal_ids, incoming_signal_ids, &recipe.id)?;
    }
    for artifact in &envelope.unavailable_callbacks {
        reject_duplicate_publication_id(available_signal_ids, incoming_signal_ids, &artifact.id)?;
    }
    Ok(())
}

fn plan_dynamic_definition_publication(
    envelope: &RuntimeDefinitionEnvelope,
    mut available_signal_ids: BTreeSet<String>,
) -> Result<DefinitionPublicationPlan, WorthSignalJsError> {
    available_signal_ids.extend(envelope.sources.iter().map(|source| source.id.clone()));
    let mut pending_recipes = envelope
        .recipes
        .iter()
        .map(|recipe| (recipe.id.clone(), recipe.reads.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut pending_callbacks = envelope
        .unavailable_callbacks
        .iter()
        .map(|artifact| (artifact.id.clone(), artifact.current_reads.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut dynamic_steps = Vec::new();

    while !pending_recipes.is_empty() || !pending_callbacks.is_empty() {
        let ready_recipe_ids = ready_definition_ids(&pending_recipes, &available_signal_ids);
        let ready_callback_ids = ready_definition_ids(&pending_callbacks, &available_signal_ids);
        if ready_recipe_ids.is_empty() && ready_callback_ids.is_empty() {
            let blocked_reads =
                blocked_dynamic_definition_reads(&pending_recipes, &pending_callbacks);
            return Err(WorthSignalJsError::invalid_input(format!(
                "definition envelope publication contains unresolved recipe or callback dependencies: {}",
                blocked_reads.join(", ")
            )));
        }
        for callback_id in ready_callback_ids {
            pending_callbacks.remove(&callback_id);
            available_signal_ids.insert(callback_id.clone());
            dynamic_steps.push(DefinitionPublicationStep::Callback(callback_id));
        }
        for recipe_id in ready_recipe_ids {
            pending_recipes.remove(&recipe_id);
            available_signal_ids.insert(recipe_id.clone());
            dynamic_steps.push(DefinitionPublicationStep::Recipe(recipe_id));
        }
    }
    Ok(DefinitionPublicationPlan { dynamic_steps })
}

fn blocked_dynamic_definition_reads(
    pending_recipes: &BTreeMap<String, Vec<RecipeReadSpec>>,
    pending_callbacks: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut blocked_reads = Vec::new();
    for (recipe_id, reads) in pending_recipes {
        blocked_reads.extend(
            reads
                .iter()
                .map(|read| format!("{recipe_id}->{}", read.id())),
        );
    }
    for (callback_id, reads) in pending_callbacks {
        blocked_reads.extend(reads.iter().map(|read| format!("{callback_id}->{read}")));
    }
    blocked_reads
}

fn reject_unplannable_recipe_family_reads(
    envelope: &RuntimeDefinitionEnvelope,
    known_signal_ids: &BTreeSet<String>,
    known_family_ids: &BTreeSet<String>,
) -> Result<(), WorthSignalJsError> {
    for family in &envelope.recipe_families {
        for read in &family.reads {
            let scope = match read {
                RecipeFamilyReadSpec::Signal { scope, .. }
                | RecipeFamilyReadSpec::Keyed { scope, .. } => scope,
            };
            if let Some(scope) = scope {
                scope.compile_key_requirement()?;
            }
            match read {
                RecipeFamilyReadSpec::Signal { id, .. } => {
                    reject_unknown_publication_read(known_signal_ids, id, "keyed family")?;
                }
                RecipeFamilyReadSpec::Keyed { family_id, .. } => {
                    reject_unknown_publication_read(known_family_ids, family_id, "keyed family")?;
                }
            }
        }
    }
    Ok(())
}

fn reject_invalid_recipe_read_aspects(
    envelope: &RuntimeDefinitionEnvelope,
) -> Result<(), WorthSignalJsError> {
    for recipe in &envelope.recipes {
        for read in &recipe.reads {
            resolve_selected_aspects(read.aspect_spec())?;
        }
    }
    Ok(())
}

fn reject_duplicate_publication_id(
    existing_ids: &mut BTreeSet<String>,
    incoming_ids: &mut BTreeSet<String>,
    id: &str,
) -> Result<(), WorthSignalJsError> {
    if existing_ids.contains(id) || !incoming_ids.insert(id.to_owned()) {
        return Err(WorthSignalJsError::invalid_input(format!(
            "definition envelope publication cannot redefine `{id}`"
        )));
    }
    Ok(())
}

fn ready_definition_ids<T>(
    pending_reads_by_id: &BTreeMap<String, Vec<T>>,
    available_signal_ids: &BTreeSet<String>,
) -> Vec<String>
where
    T: PublicationReadId,
{
    pending_reads_by_id
        .iter()
        .filter(|(_, reads)| {
            reads
                .iter()
                .all(|read| available_signal_ids.contains(read.publication_read_id()))
        })
        .map(|(id, _)| id.clone())
        .collect()
}

trait PublicationReadId {
    fn publication_read_id(&self) -> &str;
}

impl PublicationReadId for RecipeReadSpec {
    fn publication_read_id(&self) -> &str {
        self.id()
    }
}

impl PublicationReadId for String {
    fn publication_read_id(&self) -> &str {
        self
    }
}

fn reject_unknown_publication_read(
    known_ids: &BTreeSet<String>,
    id: &str,
    owner: &str,
) -> Result<(), WorthSignalJsError> {
    if known_ids.contains(id) {
        return Ok(());
    }
    Err(WorthSignalJsError::invalid_input(format!(
        "definition envelope publication `{owner}` reads unknown `{id}`"
    )))
}
