use super::{
    denial, map_structural_error, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantReadView,
    WorthQueryInvariantAccessDenial, WorthQueryInvariantAccessDenialKind,
};
use worth_foundational::facade::{
    AbsenceLaw, AspectValue, ContractValidatedAspectValueView, FieldKey,
};
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;

impl<Schema> WorthQueryApplicationInvariantReadView<'_, Schema> {
    pub fn field<Entity, Value>(
        &self,
        field: &WorthQueryApplicationInvariantFieldBinding<Schema, Entity, Value>,
        entity: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
    ) -> Result<Option<Value>, WorthQueryInvariantAccessDenial> {
        self.check_binding(&field.binding_identity)?;
        self.check_entity(entity, field.entity_kind)?;
        let state = self
            .aspects
            .entity_aspect_state(entity.entity_id)
            .map_err(|error| map_structural_error(error, "entity aspect state"))?;
        self.reads.field();
        field_material(
            state
                .get(field.locator.aspect().aspect_key())
                .map(|value| value.view()),
            field.aspect_absence,
            field.presence,
            field.locator.field_path().fields().first(),
        )?
        .map(|raw| {
            (field.decode)(raw)
                .map_err(|error| denial(WorthQueryInvariantAccessDenialKind::InvalidValue, error))
        })
        .transpose()
    }
}

// Both prior and proposed invariant views consume this same material decision.
// A child's requirement cannot describe whole-aspect absence.
pub(super) fn field_material<'a>(
    aspect: Option<ContractValidatedAspectValueView<'a>>,
    aspect_absence: AbsenceLaw,
    presence: ApplicationFieldPresence,
    key: Option<&FieldKey>,
) -> Result<Option<&'a AspectValue>, WorthQueryInvariantAccessDenial> {
    let Some(aspect) = aspect else {
        return match aspect_absence {
            AbsenceLaw::Optional => Ok(None),
            AbsenceLaw::Required | AbsenceLaw::Defaulted => Err(missing_required()),
        };
    };
    let ContractValidatedAspectValueView::Struct(fields) = aspect else {
        return Err(denial(
            WorthQueryInvariantAccessDenialKind::InvalidValue,
            "field aspect shape",
        ));
    };
    match key.and_then(|key| fields.get(key)) {
        Some(raw) => Ok(Some(raw)),
        None => match presence {
            ApplicationFieldPresence::Optional => Ok(None),
            ApplicationFieldPresence::Required => Err(missing_required()),
        },
    }
}
fn missing_required() -> WorthQueryInvariantAccessDenial {
    denial(
        WorthQueryInvariantAccessDenialKind::MissingRequiredField,
        "required field",
    )
}
#[cfg(test)]
mod tests;
