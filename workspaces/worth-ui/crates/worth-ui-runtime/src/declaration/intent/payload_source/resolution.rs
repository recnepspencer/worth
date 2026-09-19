use super::{
    UiResolvedIntentApplicationSource, UiResolvedIntentPayloadBinding,
    UiResolvedIntentPayloadSource, UiResolvedIntentProjectionSource,
};
use crate::capability::{
    UiIntentPayloadFieldDescriptor, UiIntentPayloadFieldKind, UiIntentPayloadFieldSet,
};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(crate) fn resolve_payload_sources(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    fields: UiIntentPayloadFieldSet,
    query: &worth_ui_query_binding::WorthUiQueryBindingPlan,
    application_facts: &super::UiIntentApplicationFactPlan,
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
            query,
            application_facts,
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
    query: &worth_ui_query_binding::WorthUiQueryBindingPlan,
    application_facts: &super::UiIntentApplicationFactPlan,
) -> Result<UiResolvedIntentPayloadBinding, super::UiIntentCatalogPreparationDenial> {
    use worth_ui_dsl::WorthUiIntentPayloadSource as Source;
    let source = match authored {
        Source::ProjectionText { projection } => {
            require_kind(declaration, field, UiIntentPayloadFieldKind::Text)?;
            let identity = projection_identity(declaration, field, projection)?;
            let native_family = query
                .scalar_projection_registration(&identity)
                .map(|registration| registration.requirement().native_family())
                .or_else(|| {
                    query
                        .application_scalar_projection_registration(&identity)
                        .map(|registration| registration.requirement().native_family())
                })
                .ok_or_else(|| unknown_projection(declaration, field, projection, "scalar-text"))?;
            if native_family != worth_ui_query_binding::UiProjectionNativeFamily::Text {
                return Err(source_mismatch(declaration, field, "scalar-text"));
            }
            UiResolvedIntentPayloadSource::ProjectionText(resolve_projection_slot(
                declaration,
                field,
                query,
                identity,
            )?)
        }
        Source::ProjectionSelection { projection } => {
            require_kind(declaration, field, UiIntentPayloadFieldKind::Selection)?;
            let identity = projection_identity(declaration, field, projection)?;
            let registration = query
                .collection_projection_registration(&identity)
                .ok_or_else(|| unknown_projection(declaration, field, projection, "collection"))?;
            if registration.requirement().native_family()
                != worth_ui_query_binding::UiProjectionNativeFamily::Text
            {
                return Err(source_mismatch(declaration, field, "collection-text"));
            }
            UiResolvedIntentPayloadSource::ProjectionSelection(resolve_projection_slot(
                declaration,
                field,
                query,
                identity,
            )?)
        }
        Source::CommittedDraft => {
            require_kind(declaration, field, UiIntentPayloadFieldKind::Text)?;
            UiResolvedIntentPayloadSource::CommittedDraft
        }
        Source::ConstantText { value } => {
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
            UiResolvedIntentPayloadSource::ConstantText(Arc::from(value.as_ref()))
        }
        Source::ConstantBoolean { value } => {
            require_kind(declaration, field, UiIntentPayloadFieldKind::Boolean)?;
            UiResolvedIntentPayloadSource::ConstantBoolean(*value)
        }
        Source::ConstantUnsigned64 { value } => {
            require_kind(declaration, field, UiIntentPayloadFieldKind::Unsigned64)?;
            UiResolvedIntentPayloadSource::ConstantUnsigned64(*value)
        }
        Source::ApplicationText { fact } => {
            require_kind(declaration, field, UiIntentPayloadFieldKind::Text)?;
            UiResolvedIntentPayloadSource::ApplicationText(resolve_application_fact(
                declaration,
                field,
                fact,
                application_facts,
            )?)
        }
        Source::ApplicationBoolean { fact } => {
            require_kind(declaration, field, UiIntentPayloadFieldKind::Boolean)?;
            UiResolvedIntentPayloadSource::ApplicationBoolean(resolve_application_fact(
                declaration,
                field,
                fact,
                application_facts,
            )?)
        }
        Source::ApplicationUnsigned64 { fact } => {
            require_kind(declaration, field, UiIntentPayloadFieldKind::Unsigned64)?;
            UiResolvedIntentPayloadSource::ApplicationUnsigned64(resolve_application_fact(
                declaration,
                field,
                fact,
                application_facts,
            )?)
        }
    };
    Ok(UiResolvedIntentPayloadBinding { field, source })
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

fn projection_identity(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    authored: &str,
) -> Result<worth_ui_query_binding::WorthUiQueryViewIdentity, super::UiIntentCatalogPreparationDenial>
{
    worth_ui_query_binding::WorthUiQueryViewIdentity::new(authored).map_err(|_| {
        super::UiIntentCatalogPreparationDenial::InvalidPayloadProjectionIdentity {
            declaration: declaration.identity().into(),
            field: field.stable_name().into(),
            projection: authored.into(),
        }
    })
}

fn resolve_projection_slot(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    query: &worth_ui_query_binding::WorthUiQueryBindingPlan,
    identity: worth_ui_query_binding::WorthUiQueryViewIdentity,
) -> Result<UiResolvedIntentProjectionSource, super::UiIntentCatalogPreparationDenial> {
    let slot = query.projection_input_slot(&identity).ok_or_else(|| {
        unknown_projection(
            declaration,
            field,
            identity.as_str(),
            "registered-input-slot",
        )
    })?;
    Ok(UiResolvedIntentProjectionSource { identity, slot })
}

fn unknown_projection(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    projection: &str,
    required_shape: &'static str,
) -> super::UiIntentCatalogPreparationDenial {
    super::UiIntentCatalogPreparationDenial::UnknownPayloadProjection {
        declaration: declaration.identity().into(),
        field: field.stable_name().into(),
        projection: projection.into(),
        required_shape,
    }
}

fn source_mismatch(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    required_source: &'static str,
) -> super::UiIntentCatalogPreparationDenial {
    super::UiIntentCatalogPreparationDenial::PayloadProjectionShapeMismatch {
        declaration: declaration.identity().into(),
        field: field.stable_name().into(),
        required_source,
    }
}
