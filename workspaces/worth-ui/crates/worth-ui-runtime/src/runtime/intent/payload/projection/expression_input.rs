//! Payload fields read from expressions: a text field from a `derived text`
//! declaration, an unsigned 64-bit field from a `derived integer` one, and a
//! Boolean field from a condition. Each read is pull-only:
//! it takes the result the expression owner retains for the observed
//! generation, records its outcome revision as a payload owner revision, and
//! never evaluates or subscribes.

use std::sync::Arc;

use crate::capability::{UiIntentPayloadFieldDescriptor, UiIntentProjectedValue};
use crate::declaration::UiResolvedIntentExpressionSource;
use crate::runtime::expression::{UiExpressionResultReference, UiExpressionWithholding};

use super::super::{UiIntentInputOwnerRevision, UiIntentPayloadStop};

impl super::PayloadProjection<'_> {
    /// The text a `derived text` expression holds now. Any other outcome
    /// withholds the field with the owner's own posture.
    pub(super) fn derived_text(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        source: &UiResolvedIntentExpressionSource,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        let result = self.expression_result(field, source)?;
        let text = result
            .outcome()
            .derived_text()
            .map_err(|posture| withheld(field, source, posture))?;
        self.admit_text_bytes(field, text.len())?;
        let value = Arc::from(text);
        self.record_expression_input(field, result);
        Ok(UiIntentProjectedValue::text(value))
    }

    /// The unsigned integer a `derived integer` expression holds now. A
    /// negative or wider value is refused with the value observed.
    pub(super) fn derived_unsigned64(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        source: &UiResolvedIntentExpressionSource,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        let result = self.expression_result(field, source)?;
        let integer = result
            .outcome()
            .derived_integer()
            .map_err(|posture| withheld(field, source, posture))?;
        let value =
            u64::try_from(integer).map_err(|_| UiIntentPayloadStop::DerivedIntegerOutOfRange {
                field: field.stable_name(),
                expression: source.identity().into(),
                observed: integer,
            })?;
        self.record_expression_input(field, result);
        Ok(UiIntentProjectedValue::unsigned64(value))
    }

    /// The truth value a condition holds now. A denied, unavailable or stale
    /// condition withholds the field; it is never read as `false`.
    pub(super) fn condition(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        source: &UiResolvedIntentExpressionSource,
    ) -> Result<UiIntentProjectedValue, UiIntentPayloadStop> {
        let result = self.expression_result(field, source)?;
        let value = result
            .condition()
            .map_err(|posture| withheld(field, source, posture))?;
        self.record_expression_input(field, result);
        Ok(UiIntentProjectedValue::boolean(value))
    }

    /// The retained result of `source` in the observed generation. An owner
    /// that does not follow that generation holds no result for it: the
    /// field is stale, never defaulted.
    fn expression_result(
        &self,
        field: UiIntentPayloadFieldDescriptor,
        source: &UiResolvedIntentExpressionSource,
    ) -> Result<UiExpressionResultReference, UiIntentPayloadStop> {
        self.basis
            .expression(source.slot())
            .ok_or_else(|| withheld(field, source, UiExpressionWithholding::Stale))
    }

    fn record_expression_input(
        &mut self,
        field: UiIntentPayloadFieldDescriptor,
        result: UiExpressionResultReference,
    ) {
        self.cost.record_expression_input();
        self.owner_revisions
            .push(UiIntentInputOwnerRevision::expression(field, &result));
        self.expression_inputs.push(result);
    }
}

fn withheld(
    field: UiIntentPayloadFieldDescriptor,
    source: &UiResolvedIntentExpressionSource,
    posture: UiExpressionWithholding,
) -> UiIntentPayloadStop {
    UiIntentPayloadStop::ExpressionWithheld {
        field: field.stable_name(),
        expression: source.identity().into(),
        posture,
    }
}
