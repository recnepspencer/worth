//! Original captured absence never comes from a present-state reread.
use super::*;
use worth_query_declaration::facade::application_aftermath::ApplicationCorrectionFieldPreImage as State;

impl CorrectionOccurrence {
    pub(in crate::domain_computation::primary_graph::product_operation::correction_source) fn preimage_state_key(
        &self,
        entity: worth_relational::facade::identity::EntityId,
        name: &str,
        locator: &AspectFieldLocator,
    ) -> State<'_> {
        match self {
            Self::Memory(_) => self.memory_preimage().map_or(State::Uncaptured, |image| {
                image.original_field_state(entity, name, locator)
            }),
            Self::Home(value) => {
                if let Some(value) = self.preimage_key(entity, name, locator) {
                    return State::Value(value);
                }
                let Some(changed) = &value.material.capture.changed_fields else {
                    return State::Uncaptured;
                };
                use crate::domain_computation::primary_graph::retained_correction::record::RecordedAbsenceKind as Kind;
                changed
                    .absences
                    .iter()
                    .find(|field| {
                        field.target
                            == worth_relational::facade::transactions::RecordRef::Entity(entity)
                            && field.entity == name
                            && field.location.aspect == locator.aspect().aspect_key().as_str()
                            && field.location.fields.iter().map(String::as_str).eq(locator
                                .field_path()
                                .fields()
                                .iter()
                                .map(|field| field.as_str()))
                    })
                    .map_or(State::Uncaptured, |field| match field.absence {
                        Kind::Field => State::AbsentField,
                        Kind::Aspect => State::AbsentAspect,
                    })
            }
        }
    }
}
