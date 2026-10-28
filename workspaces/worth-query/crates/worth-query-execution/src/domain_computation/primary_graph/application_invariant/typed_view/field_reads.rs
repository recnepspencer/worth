use worth_foundational::facade::ContractValidatedAspectValueView;

use super::{
    denial, map_structural_error, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantReadView,
    WorthQueryInvariantAccessDenial, WorthQueryInvariantAccessDenialKind,
};

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
        let Some(value) = state.get(field.locator.aspect().aspect_key()) else {
            return self.absent_field(field.presence);
        };
        let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
            return Err(denial(
                WorthQueryInvariantAccessDenialKind::InvalidValue,
                "field aspect shape",
            ));
        };
        let Some(raw) = field
            .locator
            .field_path()
            .fields()
            .first()
            .and_then(|key| fields.get(key))
        else {
            return self.absent_field(field.presence);
        };
        (field.decode)(raw)
            .map(Some)
            .map_err(|error| denial(WorthQueryInvariantAccessDenialKind::InvalidValue, error))
    }

    fn absent_field<Value>(
        &self,
        presence: worth_query_declaration::facade::application_schema::ApplicationFieldPresence,
    ) -> Result<Option<Value>, WorthQueryInvariantAccessDenial> {
        use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;
        match presence {
            ApplicationFieldPresence::Optional => Ok(None),
            ApplicationFieldPresence::Required => Err(denial(
                WorthQueryInvariantAccessDenialKind::MissingRequiredField,
                "required field",
            )),
        }
    }
}
