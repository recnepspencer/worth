//! Native earned selection of an installed body's one external input effect.
mod installation;
use crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation;
pub(super) use installation::install;
use std::sync::Arc;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyInputEffectRecipe as Recipe,
    ApplicationOperationBodyInputEmission as Meaning,
};

/// Minted only by the native body evaluator. It is not caller-supplied
/// availability, an effect count, or an alternate publication permit.
pub(in crate::domain_computation::primary_graph) struct BodyInputEffectSelection {
    recipe: Arc<Recipe>,
    emit: bool,
    funding: Option<Arc<QueryMemoryReservation>>,
}
impl BodyInputEffectSelection {
    /// A missing disposition preserves the ordinary mandatory external slot.
    /// Only a matching native selection may suppress it for this invocation.
    pub(in crate::domain_computation::primary_graph) fn selected_external_contract<'a>(
        selection: Option<&Self>,
        original: &'a worth_query_installation::facade::InstalledExternalEffectContract,
    ) -> Result<
        Option<&'a worth_query_installation::facade::InstalledExternalEffectContract>,
        &'static str,
    > {
        let Some(selection) = selection else {
            return Ok(Some(original));
        };
        let worth_query_installation::facade::InstalledExternalEffectContract::Declared {
            effect,
            rust_payload_type,
            protocol,
            maximum_payload_bytes,
            ..
        } = original
        else {
            return Err("body emission has no installed external slot");
        };
        let meaning = selection.meaning();
        if effect != &meaning.effect
            || rust_payload_type != &meaning.payload_binding
            || protocol != &meaning.protocol
            || maximum_payload_bytes != &meaning.maximum_payload_bytes
        {
            return Err("native body emission disposition belongs to a different external slot");
        }
        Ok(selection.emits().then_some(original))
    }
    pub(super) fn new(
        recipe: Arc<Recipe>,
        emit: bool,
        funding: Option<QueryMemoryReservation>,
    ) -> Self {
        Self {
            recipe,
            emit,
            funding: funding.map(Arc::new),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn matches(&self, meaning: &Meaning) -> bool {
        self.recipe.meaning() == meaning
    }
    pub(in crate::domain_computation::primary_graph) fn meaning(&self) -> &Meaning {
        self.recipe.meaning()
    }
    pub(in crate::domain_computation::primary_graph) fn emits(&self) -> bool {
        self.emit
    }
    pub(in crate::domain_computation::primary_graph) fn recipe(&self) -> &Recipe {
        &self.recipe
    }
    pub(in crate::domain_computation::primary_graph) fn funding(
        &self,
    ) -> Option<Arc<QueryMemoryReservation>> {
        self.funding.as_ref().map(Arc::clone)
    }
}

pub(in crate::domain_computation::primary_graph) fn prepare<
    Schema: worth_query_installation::facade::ApplicationSchema,
    Operation,
    Entity,
>(
    reader: &mut crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
    scope: &crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity<
        Schema,
        Entity,
    >,
    recipe: Arc<Recipe>,
    input: &dyn std::any::Any,
    execution: &crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution<'_>,
) -> Result<
    BodyInputEffectSelection,
    crate::domain_computation::primary_graph::HandlerExecutionDenial,
> {
    use crate::domain_computation::primary_graph::{
        HandlerExecutionDenial, WorthQueryAdvancementDenial,
        WorthQueryManagedComputationResourceDenial,
    };
    let emit = recipe
        .meaning()
        .condition
        .as_ref()
        .map(|condition| {
            super::requirement::evaluate_requirement(reader, scope, condition, execution)
        })
        .transpose()?
        .is_none_or(|status| status == super::WorthQueryOperationBodyRequirementStatus::Met);
    let funding = if emit {
        let retained = recipe.quote(input).map_err(|_| HandlerExecutionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::InvalidEffectValue, &recipe.meaning().effect)))?;
        let bytes = retained
            .checked_add(recipe.meaning().maximum_payload_bytes)
            .and_then(|bytes| bytes.checked_add(size_of::<BodyInputEffectSelection>() as u64))
            .ok_or_else(|| {
                HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                ))
            })?;
        Some(execution.reserve(bytes).map_err(|cause| {
            HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
        })?)
    } else {
        None
    };
    Ok(BodyInputEffectSelection::new(recipe, emit, funding))
}
