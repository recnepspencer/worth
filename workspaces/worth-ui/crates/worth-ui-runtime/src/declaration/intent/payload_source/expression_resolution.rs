//! Payload fields that read an admitted expression. The sealed package
//! already refuses an unknown expression, a use site in the wrong role, and a
//! derived token; only the runtime knows the payload schema, so the field
//! kind each expression must fit is checked here, before any generation that
//! carries the declaration can activate.

use worth_ui_dsl::{WorthUiExpressionResultType, WorthUiExpressionRole};

use super::{
    UiIntentCatalogPreparationDenial, UiResolvedIntentExpressionSource,
    UiResolvedIntentPayloadSource,
};
use crate::capability::{UiIntentPayloadFieldDescriptor, UiIntentPayloadFieldKind};
use crate::runtime::expression::UiExpressionCatalog;

#[cfg(test)]
mod tests;

/// A `payload <field> derived <identity>` source: a text field reads a
/// `derived text`, and an unsigned 64-bit field reads a `derived integer`,
/// whose sign and range the payload checks against the value it reads. A
/// token or decimal result fits no payload field kind, and a selection field
/// reads only the Query-issued option its selection owner holds. Nothing
/// narrows or converts a value into another kind.
pub(super) fn resolve_derived(
    declaration: &str,
    field: UiIntentPayloadFieldDescriptor,
    identity: &str,
    expressions: &UiExpressionCatalog,
) -> Result<UiResolvedIntentPayloadSource, UiIntentCatalogPreparationDenial> {
    let (source, role) = installed(declaration, field, identity, expressions)?;
    match (role, field.kind()) {
        (
            WorthUiExpressionRole::Derived(WorthUiExpressionResultType::Text),
            UiIntentPayloadFieldKind::Text,
        ) => Ok(UiResolvedIntentPayloadSource::DerivedText(source)),
        (
            WorthUiExpressionRole::Derived(WorthUiExpressionResultType::Integer),
            UiIntentPayloadFieldKind::Unsigned64,
        ) => Ok(UiResolvedIntentPayloadSource::DerivedInteger(source)),
        _ => Err(kind_mismatch(declaration, field, identity, role)),
    }
}

/// A `payload <field> condition <identity>` source: a condition is the one
/// computed Boolean, so only a Boolean field reads it.
pub(super) fn resolve_condition(
    declaration: &str,
    field: UiIntentPayloadFieldDescriptor,
    identity: &str,
    expressions: &UiExpressionCatalog,
) -> Result<UiResolvedIntentPayloadSource, UiIntentCatalogPreparationDenial> {
    let (source, role) = installed(declaration, field, identity, expressions)?;
    match (role, field.kind()) {
        (WorthUiExpressionRole::Condition, UiIntentPayloadFieldKind::Boolean) => {
            Ok(UiResolvedIntentPayloadSource::Condition(source))
        }
        _ => Err(kind_mismatch(declaration, field, identity, role)),
    }
}

fn installed(
    declaration: &str,
    field: UiIntentPayloadFieldDescriptor,
    identity: &str,
    expressions: &UiExpressionCatalog,
) -> Result<
    (UiResolvedIntentExpressionSource, WorthUiExpressionRole),
    UiIntentCatalogPreparationDenial,
> {
    UiResolvedIntentExpressionSource::resolve(identity, expressions).ok_or_else(|| {
        UiIntentCatalogPreparationDenial::UnknownPayloadExpression {
            declaration: declaration.into(),
            field: field.stable_name().into(),
            expression: identity.into(),
        }
    })
}

fn kind_mismatch(
    declaration: &str,
    field: UiIntentPayloadFieldDescriptor,
    identity: &str,
    role: WorthUiExpressionRole,
) -> UiIntentCatalogPreparationDenial {
    UiIntentCatalogPreparationDenial::PayloadExpressionKindMismatch {
        declaration: declaration.into(),
        field: field.stable_name().into(),
        expression: identity.into(),
        field_kind: field.kind(),
        role,
    }
}
