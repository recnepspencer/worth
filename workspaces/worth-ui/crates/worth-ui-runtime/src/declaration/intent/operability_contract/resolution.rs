use crate::capability::UiIntentPayloadFieldKind;

use worth_ui_dsl::{
    WorthUiExpressionRole, WorthUiIntentMutabilitySourceSpec, WorthUiIntentPolicySourceSpec,
    WorthUiIntentReadinessSourceSpec,
};

use super::{
    UiIntentOperabilityDependencyAxis, UiResolvedIntentConditionSource,
    UiResolvedIntentMutabilitySource, UiResolvedIntentOperabilityContract,
    UiResolvedIntentPolicySource, UiResolvedIntentReadinessSource,
};

/// The prepared owners an operability contract resolves its sources against.
pub(crate) struct UiIntentOperabilitySourcePlans<'plan> {
    pub(crate) query: &'plan worth_ui_query_binding::WorthUiQueryBindingPlan,
    pub(crate) application_facts: &'plan crate::declaration::UiIntentApplicationFactPlan,
    pub(crate) expressions: &'plan crate::runtime::expression::UiExpressionCatalog,
}

pub(crate) fn resolve_operability_contract(
    declaration: &str,
    spec: &worth_ui_dsl::WorthUiIntentOperabilityContractSpec,
    interaction: crate::capability::UiSemanticInteractionFamily,
    plans: &UiIntentOperabilitySourcePlans<'_>,
) -> Result<UiResolvedIntentOperabilityContract, crate::declaration::UiIntentCatalogPreparationDenial>
{
    let mutability = resolve_mutability(declaration, spec.mutability(), interaction, plans)?;
    let readiness = resolve_readiness(declaration, spec.readiness(), interaction, plans)?;
    let axis = UiIntentOperabilityDependencyAxis::Policy;
    let policy = match spec.policy() {
        WorthUiIntentPolicySourceSpec::ApplicationBoolean { fact } => {
            UiResolvedIntentPolicySource::ApplicationBoolean(resolve_boolean_fact(
                declaration,
                axis,
                fact,
                plans.application_facts,
            )?)
        }
        WorthUiIntentPolicySourceSpec::Condition { condition } => {
            UiResolvedIntentPolicySource::Condition(resolve_condition(
                declaration,
                axis,
                condition,
                plans.expressions,
            )?)
        }
    };
    Ok(UiResolvedIntentOperabilityContract {
        identity: spec.identity().into(),
        mutability,
        readiness,
        policy,
    })
}

fn resolve_mutability(
    declaration: &str,
    source: &WorthUiIntentMutabilitySourceSpec,
    interaction: crate::capability::UiSemanticInteractionFamily,
    plans: &UiIntentOperabilitySourcePlans<'_>,
) -> Result<UiResolvedIntentMutabilitySource, crate::declaration::UiIntentCatalogPreparationDenial>
{
    let axis = UiIntentOperabilityDependencyAxis::Mutability;
    Ok(match source {
        WorthUiIntentMutabilitySourceSpec::ApplicationBoolean { fact } => {
            UiResolvedIntentMutabilitySource::ApplicationBoolean(resolve_boolean_fact(
                declaration,
                axis,
                fact,
                plans.application_facts,
            )?)
        }
        WorthUiIntentMutabilitySourceSpec::ProjectionReadonly { projection } => {
            let (identity, slot) = resolve_projection(declaration, axis, projection, plans.query)?;
            UiResolvedIntentMutabilitySource::ProjectionReadonly { identity, slot }
        }
        WorthUiIntentMutabilitySourceSpec::Condition { condition } => {
            UiResolvedIntentMutabilitySource::Condition(resolve_condition(
                declaration,
                axis,
                condition,
                plans.expressions,
            )?)
        }
        WorthUiIntentMutabilitySourceSpec::CommittedDraft => {
            require_draft_source(declaration, axis, interaction)?;
            UiResolvedIntentMutabilitySource::CommittedDraft
        }
    })
}

fn resolve_readiness(
    declaration: &str,
    source: &WorthUiIntentReadinessSourceSpec,
    interaction: crate::capability::UiSemanticInteractionFamily,
    plans: &UiIntentOperabilitySourcePlans<'_>,
) -> Result<UiResolvedIntentReadinessSource, crate::declaration::UiIntentCatalogPreparationDenial> {
    let axis = UiIntentOperabilityDependencyAxis::Readiness;
    Ok(match source {
        WorthUiIntentReadinessSourceSpec::ApplicationBoolean { fact } => {
            UiResolvedIntentReadinessSource::ApplicationBoolean(resolve_boolean_fact(
                declaration,
                axis,
                fact,
                plans.application_facts,
            )?)
        }
        WorthUiIntentReadinessSourceSpec::Projection { projection } => {
            let (identity, slot) = resolve_projection(declaration, axis, projection, plans.query)?;
            UiResolvedIntentReadinessSource::Projection { identity, slot }
        }
        WorthUiIntentReadinessSourceSpec::Condition { condition } => {
            UiResolvedIntentReadinessSource::Condition(resolve_condition(
                declaration,
                axis,
                condition,
                plans.expressions,
            )?)
        }
        WorthUiIntentReadinessSourceSpec::CommittedDraft => {
            require_draft_source(declaration, axis, interaction)?;
            UiResolvedIntentReadinessSource::CommittedDraft
        }
    })
}

/// Resolves a condition use site against the prepared expression catalog. A
/// sealed package already refuses both denials; a programmatically fed
/// catalog meets them here.
fn resolve_condition(
    declaration: &str,
    axis: UiIntentOperabilityDependencyAxis,
    identity: &str,
    expressions: &crate::runtime::expression::UiExpressionCatalog,
) -> Result<UiResolvedIntentConditionSource, crate::declaration::UiIntentCatalogPreparationDenial> {
    let installed = expressions
        .slot_of(identity)
        .and_then(|slot| Some((slot, expressions.expression(slot)?)));
    let Some((slot, expression)) = installed else {
        return Err(
            crate::declaration::UiIntentCatalogPreparationDenial::UnknownOperabilityCondition {
                declaration: declaration.into(),
                axis,
                condition: identity.into(),
            },
        );
    };
    if expression.role() != WorthUiExpressionRole::Condition {
        return Err(
            crate::declaration::UiIntentCatalogPreparationDenial::OperabilityConditionRoleMismatch {
                declaration: declaration.into(),
                axis,
                condition: identity.into(),
            },
        );
    }
    Ok(UiResolvedIntentConditionSource {
        identity: identity.into(),
        slot,
    })
}

fn resolve_boolean_fact(
    declaration: &str,
    axis: UiIntentOperabilityDependencyAxis,
    identity: &str,
    application_facts: &crate::declaration::UiIntentApplicationFactPlan,
) -> Result<
    crate::declaration::UiIntentApplicationFactSlot,
    crate::declaration::UiIntentCatalogPreparationDenial,
> {
    let fact = application_facts.get(identity).ok_or_else(|| {
        crate::declaration::UiIntentCatalogPreparationDenial::UnknownOperabilityApplicationFact {
            declaration: declaration.into(),
            axis,
            fact: identity.into(),
        }
    })?;
    if fact.kind() != UiIntentPayloadFieldKind::Boolean {
        return Err(
            crate::declaration::UiIntentCatalogPreparationDenial::
                OperabilityApplicationFactKindMismatch {
                    declaration: declaration.into(),
                    axis,
                    fact: identity.into(),
                    observed: fact.kind(),
                },
        );
    }
    Ok(fact.slot())
}

fn resolve_projection(
    declaration: &str,
    axis: UiIntentOperabilityDependencyAxis,
    authored: &str,
    query: &worth_ui_query_binding::WorthUiQueryBindingPlan,
) -> Result<
    (
        worth_ui_query_binding::WorthUiQueryViewIdentity,
        worth_ui_query_binding::UiProjectionInputSlot,
    ),
    crate::declaration::UiIntentCatalogPreparationDenial,
> {
    let identity =
        worth_ui_query_binding::WorthUiQueryViewIdentity::new(authored).map_err(|_| {
            crate::declaration::UiIntentCatalogPreparationDenial::
            InvalidOperabilityProjectionIdentity {
                declaration: declaration.into(),
                axis,
                projection: authored.into(),
            }
        })?;
    let slot = query.projection_input_slot(&identity).ok_or_else(|| {
        crate::declaration::UiIntentCatalogPreparationDenial::UnknownOperabilityProjection {
            declaration: declaration.into(),
            axis,
            projection: authored.into(),
        }
    })?;
    Ok((identity, slot))
}

fn require_draft_source(
    declaration: &str,
    axis: UiIntentOperabilityDependencyAxis,
    interaction: crate::capability::UiSemanticInteractionFamily,
) -> Result<(), crate::declaration::UiIntentCatalogPreparationDenial> {
    if interaction != crate::capability::UiSemanticInteractionFamily::EditCommit {
        return Err(
            crate::declaration::UiIntentCatalogPreparationDenial::OperabilityDraftSourceMismatch {
                declaration: declaration.into(),
                axis,
                interaction,
            },
        );
    }
    Ok(())
}
