//! Native earned selection and once-prepared payload of one installed external effect.
mod installation;
use crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind as DenialKind;
pub(super) use installation::install;
use std::sync::Arc;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyExternalEffectRecipe as Recipe,
    ApplicationOperationBodyExternalEmission as Meaning,
};

/// Minted only by the native body evaluator. It is not caller-supplied
/// availability, an effect count, or an alternate publication permit.
pub(in crate::domain_computation::primary_graph) struct BodyExternalEffectSelection {
    recipe: Arc<Recipe>,
    emit: bool,
    payload: Option<worth_query_declaration::facade::application_schema::ApplicationExternalEffectPayloadProjection>,
    backing: Option<ObservedQueryExpressionContext>,
    values: Option<super::value::PreparedBodyValues>,
}
impl BodyExternalEffectSelection {
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
    pub(in crate::domain_computation::primary_graph) fn take_payload(&mut self) -> Option<worth_query_declaration::facade::application_schema::ApplicationExternalEffectPayloadProjection>{
        self.payload.take()
    }
    pub(in crate::domain_computation::primary_graph) fn backing(
        &self,
    ) -> Option<ObservedQueryExpressionContext> {
        self.backing.clone()
    }
    pub(in crate::domain_computation::primary_graph) fn values(
        &self,
    ) -> Option<super::value::PreparedBodyValues> {
        self.values.clone()
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
    values: Option<&super::value::PreparedBodyValues>,
    execution: &crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution<'_>,
) -> Result<
    BodyExternalEffectSelection,
    crate::domain_computation::primary_graph::HandlerExecutionDenial,
> {
    let emit = recipe
        .meaning()
        .condition
        .as_ref()
        .map(|condition| {
            super::requirement::evaluate_requirement(reader, scope, condition, execution)
        })
        .transpose()?
        .is_none_or(|status| status == super::WorthQueryOperationBodyRequirementStatus::Met);
    let backing = if emit {
        match &recipe.meaning().source {
            worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::OperationInput => None,
            worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::QueryContext { query, .. }
            | worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::Evaluation { query, .. } => Some(reader.body_query_context(query.name())?),
        }
    } else {
        None
    };
    let values = if emit && matches!(recipe.meaning().source, worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::Evaluation { .. }) { Some(values.cloned().ok_or_else(|| invalid_payload(&recipe, DenialKind::MissingAuthoritativeFact))?) } else { None };
    let evaluation = backing.as_ref().zip(values.as_ref()).map(|(context, values)| worth_query_declaration::facade::application_schema::ApplicationOperationBodyEvaluationSnapshot::from_untrusted_parts(context.snapshot().clone(), values.snapshot().clone()));
    let source: &dyn std::any::Any = if let Some(evaluation) = &evaluation {
        evaluation
    } else {
        backing.as_ref().map_or(input, |context| context.snapshot())
    };
    let payload = if emit {
        // The exact native decision owns the source. Candidate construction
        // consumes this projection without rereading or encoding.
        let payload = recipe.project(source).map_err(|_| {
            invalid_payload(&recipe, DenialKind::ExternalEffectPayloadProjectionRejected)
        })?;
        Some(payload)
    } else {
        None
    };
    Ok(BodyExternalEffectSelection {
        recipe,
        emit,
        payload,
        backing,
        values,
    })
}

fn invalid_payload(
    recipe: &Recipe,
    kind: DenialKind,
) -> crate::domain_computation::primary_graph::HandlerExecutionDenial {
    use crate::domain_computation::primary_graph::{
        HandlerExecutionDenial, WorthQueryApplicationAttemptDenial,
    };
    HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(
        kind,
        &recipe.meaning().effect,
    ))
}
