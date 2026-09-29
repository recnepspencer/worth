use super::{
    UiResolvedIntentApplicationSource, UiResolvedIntentPayloadBinding,
    UiResolvedIntentPayloadSource,
};
use crate::capability::{
    UiIntentPayloadFieldDescriptor, UiIntentPayloadFieldKind, UiIntentPayloadFieldSet,
};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(crate) fn resolve_payload_sources(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    fields: UiIntentPayloadFieldSet,
    sources: &super::UiIntentSourcePlans<'_>,
) -> Result<Box<[UiResolvedIntentPayloadBinding]>, super::UiIntentCatalogPreparationDenial> {
    let mut authored = BTreeMap::new();
    for source in declaration.payload_sources() {
        if authored.insert(source.field(), source).is_some() {
            return Err(
                super::UiIntentCatalogPreparationDenial::DuplicatePayloadField {
                    declaration: declaration.identity().into(),
                    field: source.field().into(),
                },
            );
        }
    }
    let mut resolved = Vec::with_capacity(fields.len());
    for field in fields.fields() {
        let source = authored.remove(field.stable_name()).ok_or_else(|| {
            super::UiIntentCatalogPreparationDenial::MissingPayloadField {
                declaration: declaration.identity().into(),
                field: field.stable_name().into(),
            }
        })?;
        resolved.push(resolve_source(
            declaration,
            *field,
            source.source(),
            sources,
        )?);
    }
    if let Some((field, _)) = authored.into_iter().next() {
        return Err(
            super::UiIntentCatalogPreparationDenial::UnknownPayloadField {
                declaration: declaration.identity().into(),
                field: field.into(),
            },
        );
    }
    validate_interaction_sources(declaration, &resolved)?;
    Ok(resolved.into_boxed_slice())
}

fn validate_interaction_sources(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    resolved: &[UiResolvedIntentPayloadBinding],
) -> Result<(), super::UiIntentCatalogPreparationDenial> {
    use super::UiIntentInteractionPayloadSourceKind as Shape;
    let draft_count = resolved
        .iter()
        .filter(|binding| {
            matches!(
                binding.source(),
                UiResolvedIntentPayloadSource::CommittedDraft
            )
        })
        .count();
    let selection_count = resolved
        .iter()
        .filter(|binding| {
            matches!(
                binding.source(),
                UiResolvedIntentPayloadSource::ProjectionSelection(_)
            )
        })
        .count();
    require_unique_shape_source(declaration, draft_count, Shape::CommittedDraft)?;
    require_unique_shape_source(declaration, selection_count, Shape::ProjectionSelection)?;
    let interaction = runtime_interaction(declaration.interaction());
    require_shape_affinity(
        declaration,
        interaction,
        draft_count,
        crate::capability::UiSemanticInteractionFamily::EditCommit,
        Shape::CommittedDraft,
    )?;
    require_shape_affinity(
        declaration,
        interaction,
        selection_count,
        crate::capability::UiSemanticInteractionFamily::SelectionCommit,
        Shape::ProjectionSelection,
    )
}

fn require_unique_shape_source(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    count: usize,
    source: super::UiIntentInteractionPayloadSourceKind,
) -> Result<(), super::UiIntentCatalogPreparationDenial> {
    if count > 1 {
        return Err(
            super::UiIntentCatalogPreparationDenial::DuplicateInteractionPayloadSource {
                declaration: declaration.identity().into(),
                source,
            },
        );
    }
    Ok(())
}

fn require_shape_affinity(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    interaction: crate::capability::UiSemanticInteractionFamily,
    count: usize,
    owner: crate::capability::UiSemanticInteractionFamily,
    source: super::UiIntentInteractionPayloadSourceKind,
) -> Result<(), super::UiIntentCatalogPreparationDenial> {
    match (interaction == owner, count) {
        (true, 0) => Err(
            super::UiIntentCatalogPreparationDenial::MissingInteractionPayloadSource {
                declaration: declaration.identity().into(),
                interaction,
                source,
            },
        ),
        (false, 1) => Err(
            super::UiIntentCatalogPreparationDenial::InteractionPayloadSourceMismatch {
                declaration: declaration.identity().into(),
                interaction,
                source,
            },
        ),
        _ => Ok(()),
    }
}

fn runtime_interaction(
    interaction: worth_ui_dsl::WorthUiIntentInteractionFamily,
) -> crate::capability::UiSemanticInteractionFamily {
    match interaction {
        worth_ui_dsl::WorthUiIntentInteractionFamily::Activate => {
            crate::capability::UiSemanticInteractionFamily::Activate
        }
        worth_ui_dsl::WorthUiIntentInteractionFamily::EditCommit => {
            crate::capability::UiSemanticInteractionFamily::EditCommit
        }
        worth_ui_dsl::WorthUiIntentInteractionFamily::SelectionCommit => {
            crate::capability::UiSemanticInteractionFamily::SelectionCommit
        }
        worth_ui_dsl::WorthUiIntentInteractionFamily::Submit => {
            crate::capability::UiSemanticInteractionFamily::Submit
        }
    }
}

fn resolve_source(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    authored: &worth_ui_dsl::WorthUiIntentPayloadSource,
    sources: &super::UiIntentSourcePlans<'_>,
) -> Result<UiResolvedIntentPayloadBinding, super::UiIntentCatalogPreparationDenial> {
    use super::projection_resolution::{resolve_collection_text, resolve_scalar_text};
    use worth_ui_dsl::WorthUiIntentPayloadSource as Source;
    use UiIntentPayloadFieldKind as Kind;
    use UiResolvedIntentPayloadSource as Resolved;
    let application = |kind, fact: &str| {
        require_kind(declaration, field, kind)?;
        resolve_application_fact(declaration, field, fact, sources.application_facts)
    };
    let source = match authored {
        Source::ProjectionText { projection } => {
            require_kind(declaration, field, Kind::Text)?;
            Resolved::ProjectionText(resolve_scalar_text(
                declaration,
                field,
                projection,
                sources.query,
            )?)
        }
        Source::ProjectionSelection { projection } => {
            require_kind(declaration, field, Kind::Selection)?;
            Resolved::ProjectionSelection(resolve_collection_text(
                declaration,
                field,
                projection,
                sources.query,
            )?)
        }
        Source::CommittedDraft => {
            require_kind(declaration, field, Kind::Text)?;
            Resolved::CommittedDraft
        }
        Source::ConstantText { value } => resolve_constant_text(declaration, field, value)?,
        Source::ConstantBoolean { value } => {
            require_kind(declaration, field, Kind::Boolean)?;
            Resolved::ConstantBoolean(*value)
        }
        Source::ConstantUnsigned64 { value } => {
            require_kind(declaration, field, Kind::Unsigned64)?;
            Resolved::ConstantUnsigned64(*value)
        }
        Source::ApplicationText { fact } => {
            Resolved::ApplicationText(application(Kind::Text, fact)?)
        }
        Source::ApplicationBoolean { fact } => {
            Resolved::ApplicationBoolean(application(Kind::Boolean, fact)?)
        }
        Source::ApplicationUnsigned64 { fact } => {
            Resolved::ApplicationUnsigned64(application(Kind::Unsigned64, fact)?)
        }
        Source::Derived { expression } => super::expression_resolution::resolve_derived(
            declaration.identity(),
            field,
            expression,
            sources.expressions,
        )?,
        Source::Condition { expression } => super::expression_resolution::resolve_condition(
            declaration.identity(),
            field,
            expression,
            sources.expressions,
        )?,
    };
    Ok(UiResolvedIntentPayloadBinding { field, source })
}

fn resolve_constant_text(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    value: &str,
) -> Result<UiResolvedIntentPayloadSource, super::UiIntentCatalogPreparationDenial> {
    require_kind(declaration, field, UiIntentPayloadFieldKind::Text)?;
    if value.len() > field.byte_budget() {
        return Err(
            super::UiIntentCatalogPreparationDenial::PayloadConstantBudgetExceeded {
                declaration: declaration.identity().into(),
                field: field.stable_name().into(),
                observed: value.len(),
                maximum: field.byte_budget(),
            },
        );
    }
    Ok(UiResolvedIntentPayloadSource::ConstantText(Arc::from(
        value,
    )))
}

fn resolve_application_fact(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    identity: &str,
    facts: &super::UiIntentApplicationFactPlan,
) -> Result<UiResolvedIntentApplicationSource, super::UiIntentCatalogPreparationDenial> {
    let definition = facts.get(identity).ok_or_else(|| {
        super::UiIntentCatalogPreparationDenial::UnknownApplicationPayloadFact {
            declaration: declaration.identity().into(),
            field: field.stable_name().into(),
            fact: identity.into(),
        }
    })?;
    if definition.kind() != field.kind() {
        return Err(
            super::UiIntentCatalogPreparationDenial::ApplicationPayloadFactKindMismatch {
                declaration: declaration.identity().into(),
                field: field.stable_name().into(),
                fact: identity.into(),
                field_kind: field.kind(),
                fact_kind: definition.kind(),
            },
        );
    }
    Ok(UiResolvedIntentApplicationSource {
        identity: identity.into(),
        slot: definition.slot(),
    })
}

fn require_kind(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    expected: UiIntentPayloadFieldKind,
) -> Result<(), super::UiIntentCatalogPreparationDenial> {
    if field.kind() != expected {
        return Err(
            super::UiIntentCatalogPreparationDenial::PayloadSourceKindMismatch {
                declaration: declaration.identity().into(),
                field: field.stable_name().into(),
                field_kind: field.kind(),
                source_kind: expected,
            },
        );
    }
    Ok(())
}
