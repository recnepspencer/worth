//! Resolves a payload field that reads a Query projection: a scalar text
//! projection for a text field, a collection text projection for a selection.

use super::UiResolvedIntentProjectionSource;
use crate::capability::UiIntentPayloadFieldDescriptor;
use worth_ui_query_binding::{
    UiProjectionNativeFamily, WorthUiQueryBindingPlan, WorthUiQueryViewIdentity,
};

/// The scalar text projection a text field reads, whether Query or the
/// application registered it.
pub(super) fn resolve_scalar_text(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    projection: &str,
    query: &WorthUiQueryBindingPlan,
) -> Result<UiResolvedIntentProjectionSource, super::UiIntentCatalogPreparationDenial> {
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
    if native_family != UiProjectionNativeFamily::Text {
        return Err(source_mismatch(declaration, field, "scalar-text"));
    }
    resolve_projection_slot(declaration, field, query, identity)
}

/// The collection text projection whose rows a selection field references.
pub(super) fn resolve_collection_text(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    projection: &str,
    query: &WorthUiQueryBindingPlan,
) -> Result<UiResolvedIntentProjectionSource, super::UiIntentCatalogPreparationDenial> {
    let identity = projection_identity(declaration, field, projection)?;
    let registration = query
        .collection_projection_registration(&identity)
        .ok_or_else(|| unknown_projection(declaration, field, projection, "collection"))?;
    if registration.requirement().native_family() != UiProjectionNativeFamily::Text {
        return Err(source_mismatch(declaration, field, "collection-text"));
    }
    resolve_projection_slot(declaration, field, query, identity)
}

fn projection_identity(
    declaration: &crate::declaration::WorthUiAuthoredIntentDeclaration,
    field: UiIntentPayloadFieldDescriptor,
    authored: &str,
) -> Result<WorthUiQueryViewIdentity, super::UiIntentCatalogPreparationDenial> {
    WorthUiQueryViewIdentity::new(authored).map_err(|_| {
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
    query: &WorthUiQueryBindingPlan,
    identity: WorthUiQueryViewIdentity,
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
