//! An all-optional row still owns a distinct Native parent occurrence.
use super::RecordFieldDefinition;
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe, ApplicationFieldPresence,
    BoolApplicationValueBinding,
};

pub(crate) const COLUMN: &str = "__record_presence";
pub(super) fn definition(entity: &str, aspect: &str, path: &str) -> RecordFieldDefinition {
    RecordFieldDefinition {
        content: None,
        path: format!("{path}.{COLUMN}"),
        recipe: ApplicationFieldBindingRecipe::of::<BoolApplicationValueBinding>(
            ApplicationFieldBindingLocus::new(entity, aspect, COLUMN),
        ),
        presence: ApplicationFieldPresence::Required,
        internal_presence: true,
        writable: true,
        equality: false,
        unique: false,
    }
}

struct PresenceBindings;
impl super::BindingSet for PresenceBindings {
    type Work<'a> = crate::application_schema::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut crate::application_schema::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        use_admission(work)
    }
}
impl super::Scalar<bool> for PresenceBindings {
    type Binding = BoolApplicationValueBinding;
}
pub(super) fn binding() -> super::RecordBinding {
    super::RecordBinding::scalar::<PresenceBindings, bool>()
}
