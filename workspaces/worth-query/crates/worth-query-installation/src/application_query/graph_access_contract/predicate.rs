use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, ScalarAspectType};
use worth_query_declaration::facade::application_query::ApplicationQueryPredicateExpected;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryInstalledGraphPredicate {
    pub(super) entity: String,
    pub(super) aspect: AspectKey,
    pub(super) field: FieldKey,
    pub(super) expected: ApplicationQueryPredicateExpected,
    pub(super) scalar_family: ScalarAspectType,
}

impl WorthQueryInstalledGraphPredicate {
    pub fn field(&self) -> (&str, &str, &str) {
        (&self.entity, self.aspect.as_str(), self.field.as_str())
    }

    pub fn aspect_key(&self) -> &AspectKey {
        &self.aspect
    }

    pub fn field_key(&self) -> &FieldKey {
        &self.field
    }

    pub fn parameter(&self) -> Option<&str> {
        match &self.expected {
            ApplicationQueryPredicateExpected::Parameter(parameter) => Some(parameter),
            ApplicationQueryPredicateExpected::Value(_) => None,
        }
    }

    pub fn literal_value(&self) -> Option<&AspectValue> {
        match &self.expected {
            ApplicationQueryPredicateExpected::Parameter(_) => None,
            ApplicationQueryPredicateExpected::Value(value) => Some(value),
        }
    }

    pub const fn expected(&self) -> &ApplicationQueryPredicateExpected {
        &self.expected
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }
}
