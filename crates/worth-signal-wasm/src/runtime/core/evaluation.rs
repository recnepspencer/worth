use std::collections::BTreeMap;

mod recipe_output;
use recipe_output::finish_recipe_output;

#[cfg(test)]
#[path = "evaluation/commit_baseline_tests.rs"]
mod commit_baseline_tests;

#[cfg(target_arch = "wasm32")]
use js_sys::Function;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

use worth_signal::facade::specialist::EvaluationOutput;
use worth_signal::facade::{
    DependencyEdge, EvaluationContext, NodeEvaluationResult, NodeId, OutputChange, SignalError,
};

#[cfg(target_arch = "wasm32")]
use crate::boundary::errors::WorthSignalJsError;
#[cfg(target_arch = "wasm32")]
use crate::boundary::serde::to_js;
use crate::expression::evaluation::ExprEnvironment;
use crate::expression::model::{IdentitySpec, SignalValue};
use crate::recipe::model::RecipeReadSpec;
use crate::runtime::compute_callbacks;
use crate::runtime::summaries::CallbackFailureSummary;

use super::aspects::{bump_aspects, defaulted_produced_aspects, resolve_selected_aspects};
use super::state::{
    PendingCallbackDependencyPatch, SharedCallbackDiagnostics, SharedStore, StoredRecipeDefinition,
};
use super::DEFAULT_ASPECT;

#[cfg(target_arch = "wasm32")]
fn build_runtime_callback_reader(
    store: SharedStore,
    current_callback_id: String,
) -> Closure<dyn FnMut(String) -> Result<JsValue, JsValue>> {
    Closure::wrap(Box::new(move |id: String| {
        let value = {
            let locked = store.lock().map_err(|_| {
                JsValue::from(WorthSignalJsError::internal("runtime store mutex poisoned"))
            })?;
            if let Some(source) = locked.sources.get(&id) {
                source.value.clone()
            } else if locked.recipes.contains_key(&id) {
                let (denial_code, denial_message) = if id == current_callback_id {
                    (
                        "computeCallbackSelfReadDenied",
                        format!(
                            "callback computed `{}` attempted to read itself through `{}`",
                            current_callback_id, id
                        ),
                    )
                } else {
                    (
                        "computeCallbackDynamicCycleDenied",
                        format!(
                            "callback computed `{}` attempted to lazily read derived signal `{}` outside its declared dependency frontier",
                            current_callback_id, id
                        ),
                    )
                };
                return Err(JsValue::from(WorthSignalJsError::callback_failure(
                    denial_code,
                    denial_message,
                    Some(id.clone()),
                )));
            } else {
                return Err(JsValue::from(WorthSignalJsError::invalid_input(format!(
                    "callback computed attempted to read `{id}` outside the active runtime callback value map"
                ))));
            }
        };
        to_js(&value).map_err(JsValue::from)
    }) as Box<dyn FnMut(String) -> Result<JsValue, JsValue>>)
}

pub(super) fn canonicalize_callback_reads(mut read_ids: Vec<String>) -> Vec<RecipeReadSpec> {
    read_ids.sort();
    read_ids.dedup();
    read_ids.into_iter().map(RecipeReadSpec::LegacyId).collect()
}

pub(super) fn evaluate_node(
    view: &mut EvaluationContext<'_, SharedStore>,
    store: &SharedStore,
    callback_diagnostics: &SharedCallbackDiagnostics,
    nodes_by_id: &BTreeMap<NodeId, String>,
) -> Result<EvaluationOutput, SignalError> {
    let Some(id) = nodes_by_id.get(&view.node()) else {
        return Err(SignalError::invalid_input(
            "missing runtime node id mapping",
        ));
    };

    let mut locked = store
        .lock()
        .map_err(|_| SignalError::internal("runtime store mutex poisoned"))?;

    if let Some(source) = locked.sources.get(id) {
        return Ok(view.finish(NodeEvaluationResult::from_version(source.version)));
    }

    let reads = locked
        .recipes
        .get(id)
        .map(|recipe| recipe.definition.reads().to_vec())
        .ok_or_else(|| SignalError::invalid_input(format!("unknown runtime recipe `{id}`")))?;

    let mut read_values = BTreeMap::new();
    for read in &reads {
        let Some(read_node) = nodes_by_id.iter().find_map(|(node, candidate)| {
            if candidate == read.id() {
                Some(*node)
            } else {
                None
            }
        }) else {
            return Err(SignalError::invalid_input(format!(
                "recipe `{id}` references unknown read `{}`",
                read.id()
            )));
        };
        let aspects = resolve_selected_aspects(read.aspect_spec())
            .map_err(|err| SignalError::invalid_input(err.message))?;
        for aspect in aspects {
            match read.scope() {
                Some(scope) => {
                    let _ =
                        view.read_partitioned_aspect_version(read_node, aspect, scope.clone())?;
                }
                None => {
                    let _ = view.read_aspect_version(read_node, aspect)?;
                }
            }
        }
        let value = locked.read_value(read.id()).ok_or_else(|| {
            SignalError::invalid_input(format!(
                "recipe `{id}` could not read current value for `{}`",
                read.id()
            ))
        })?;
        read_values.insert(read.id().to_owned(), value);
    }

    let definition = locked
        .recipes
        .get(id)
        .map(|recipe| recipe.definition.clone())
        .ok_or_else(|| SignalError::invalid_input(format!("unknown runtime recipe `{id}`")))?;
    let (next_value, next_identity, produced_aspects) = match &definition {
        StoredRecipeDefinition::Expr(spec) => {
            let recipe = locked.recipes.get_mut(id).ok_or_else(|| {
                SignalError::invalid_input(format!("unknown runtime recipe `{id}`"))
            })?;
            let env = ExprEnvironment::new(&read_values);
            if let Some(condition) = &spec.when {
                match env.evaluate(&condition.expr) {
                    Ok(SignalValue::Bool(false)) if recipe.initialized => {
                        return finish_recipe_output(view, recipe);
                    }
                    Ok(SignalValue::Bool(_)) => {}
                    Ok(_) => {
                        return Err(SignalError::invalid_input(
                            "recipe condition must evaluate to a boolean",
                        ));
                    }
                    Err(err) => return Err(SignalError::invalid_input(err.message)),
                }
            }
            let next_value = env
                .evaluate(&spec.expr)
                .map_err(|err| SignalError::invalid_input(err.message))?;
            let next_identity = resolve_identity(&spec.identity, &env, &next_value)
                .map_err(|err| SignalError::invalid_input(err.message))?;
            (
                next_value,
                next_identity,
                defaulted_produced_aspects(spec.produces_aspects.as_deref()),
            )
        }
        StoredRecipeDefinition::Callback(callback) => {
            let previous_dependency_count = callback.reads.len();
            let previous_reads = callback.reads.clone();
            #[cfg(target_arch = "wasm32")]
            let runtime_callback_reader =
                build_runtime_callback_reader(store.clone(), callback.id.clone());
            drop(locked);
            let invocation = compute_callbacks::invoke_compute_with_reads(
                callback.token,
                Some(&read_values),
                #[cfg(target_arch = "wasm32")]
                Some(runtime_callback_reader.as_ref().unchecked_ref::<Function>()),
                #[cfg(not(target_arch = "wasm32"))]
                None,
            )
            .map_err(|failure| {
                if let Ok(mut diagnostics) = callback_diagnostics.lock() {
                    let state = diagnostics.entry(callback.id.clone()).or_default();
                    state.last_failure = Some(CallbackFailureSummary {
                        class: format!("{:?}", failure.class),
                        message: failure.message.clone(),
                        code: failure.code.clone(),
                    });
                }
                SignalError::invalid_input(format!(
                    "callback recipe `{}` failed: {}",
                    callback.id, failure.message
                ))
            })?;
            let next_reads = canonicalize_callback_reads(invocation.captured_read_ids);
            locked = store
                .lock()
                .map_err(|_| SignalError::internal("runtime store mutex poisoned"))?;
            locked.pending_callback_runtime_read_breadth = locked
                .pending_callback_runtime_read_breadth
                .saturating_add(invocation.runtime_read_breadth);
            if let Ok(mut diagnostics) = callback_diagnostics.lock() {
                let state = diagnostics.entry(callback.id.clone()).or_default();
                state.current_reads = next_reads.iter().map(|read| read.id().to_owned()).collect();
                state.host_capability_reads = invocation.captured_host_capability_reads.clone();
                state.last_runtime_read_breadth = invocation.runtime_read_breadth;
                state.last_failure = None;
            }
            if next_reads != callback.reads {
                let mut next_dependencies = Vec::with_capacity(next_reads.len());
                for read in &next_reads {
                    let Some(read_node) = nodes_by_id.iter().find_map(|(node, candidate)| {
                        if candidate == read.id() {
                            Some(*node)
                        } else {
                            None
                        }
                    }) else {
                        return Err(SignalError::invalid_input(format!(
                            "callback recipe `{}` references unknown dynamic read `{}`",
                            callback.id,
                            read.id()
                        )));
                    };
                    next_dependencies.push(DependencyEdge::new(read_node, DEFAULT_ASPECT));
                }
                let recipe = locked.recipes.get_mut(id).ok_or_else(|| {
                    SignalError::invalid_input(format!("unknown runtime recipe `{id}`"))
                })?;
                if let StoredRecipeDefinition::Callback(stored_callback) = &mut recipe.definition {
                    stored_callback.reads = next_reads.clone();
                    stored_callback.host_capability_reads =
                        invocation.captured_host_capability_reads.clone();
                }
                locked
                    .pending_callback_dependency_patches
                    .push(PendingCallbackDependencyPatch {
                        node: view.node(),
                        id: callback.id.clone(),
                        previous_reads,
                        reads: next_reads,
                        host_capability_reads: invocation.captured_host_capability_reads.clone(),
                        dependencies: next_dependencies,
                        previous_dependency_count,
                        runtime_read_breadth: invocation.runtime_read_breadth as usize,
                    });
            } else {
                let recipe = locked.recipes.get_mut(id).ok_or_else(|| {
                    SignalError::invalid_input(format!("unknown runtime recipe `{id}`"))
                })?;
                if let StoredRecipeDefinition::Callback(stored_callback) = &mut recipe.definition {
                    stored_callback.host_capability_reads =
                        invocation.captured_host_capability_reads.clone();
                }
            }
            (
                invocation.value,
                None,
                defaulted_produced_aspects(callback.produces_aspects.as_deref()),
            )
        }
    };

    let recipe = locked
        .recipes
        .get_mut(id)
        .ok_or_else(|| SignalError::invalid_input(format!("unknown runtime recipe `{id}`")))?;

    let output_change = if !recipe.initialized {
        OutputChange::Replaced
    } else if recipe.output_identity == next_identity && recipe.value == next_value {
        OutputChange::Unchanged
    } else if recipe.output_identity == next_identity {
        OutputChange::Refreshed
    } else {
        OutputChange::Replaced
    };

    if !recipe.initialized || !matches!(output_change, OutputChange::Unchanged) {
        recipe.version = bump_aspects(recipe.version, &produced_aspects);
        recipe.value = next_value;
        recipe.initialized = true;
        recipe.output_identity = next_identity.clone();
    }

    finish_recipe_output(view, recipe)
}

pub(super) fn resolve_identity(
    spec: &Option<IdentitySpec>,
    env: &ExprEnvironment<'_>,
    value: &SignalValue,
) -> Result<Option<String>, crate::boundary::errors::WorthSignalJsError> {
    match spec {
        Some(IdentitySpec::Exact) => Ok(Some(canonical_value_string(value)?)),
        Some(IdentitySpec::Expr { expr }) => {
            Ok(Some(canonical_value_string(&env.evaluate(expr)?)?))
        }
        None => Ok(None),
    }
}

pub(super) fn canonical_value_string(
    value: &SignalValue,
) -> Result<String, crate::boundary::errors::WorthSignalJsError> {
    serde_json::to_string(value).map_err(|err| {
        crate::boundary::errors::WorthSignalJsError::internal(format!(
            "failed to canonicalize signal value: {err}"
        ))
    })
}

pub(super) fn signal_value_breadth(value: &SignalValue) -> u64 {
    match value {
        SignalValue::Null
        | SignalValue::Bool(_)
        | SignalValue::Number(_)
        | SignalValue::String(_) => 1,
        SignalValue::Array(items) => 1 + items.iter().map(signal_value_breadth).sum::<u64>(),
        SignalValue::Object(fields) => {
            1 + fields
                .iter()
                .map(|(_, value)| signal_value_breadth(value))
                .sum::<u64>()
        }
    }
}
