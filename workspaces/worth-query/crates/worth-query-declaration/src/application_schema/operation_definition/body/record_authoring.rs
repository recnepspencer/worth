//! Existing body consequences consume fields selected from the owning record.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationOperationBodyCreatedField as CreatedField, ApplicationOperationBodyCreationBuilder,
    ApplicationOperationBodyScopeField as ScopeField,
};
use crate::application_schema::fragment::{FieldEncodeRefusal, RecordField};

impl From<FieldEncodeRefusal> for Denial {
    fn from(cause: FieldEncodeRefusal) -> Self {
        Self::RecordField(cause)
    }
}
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    pub fn expect_record<E, A, T>(
        mut self,
        field: RecordField<E, A, T>,
        value: T,
    ) -> Result<Self, Denial> {
        let encoded = field.encode(&value)?;
        self.steps.push(ScopeField::Expect {
            field: field.locus().clone(),
            binding: field.recipe().binding_identity().clone(),
            value: encoded,
        });
        Ok(self)
    }
    pub fn assign_record<E, A, T>(
        mut self,
        field: RecordField<E, A, T>,
        value: T,
    ) -> Result<Self, Denial> {
        if !field.writable() {
            return Err(Denial::IncompatibleSourceBinding);
        }
        let encoded = field.encode(&value)?;
        self.steps.push(ScopeField::Assign {
            field: field.locus().clone(),
            binding: field.recipe().binding_identity().clone(),
            value: encoded,
        });
        Ok(self)
    }
}
impl<S, O, I, E> ApplicationOperationBodyCreationBuilder<S, O, I, E> {
    pub fn initialize_record<A, T>(
        mut self,
        field: RecordField<E, A, T>,
        value: T,
    ) -> Result<Self, Denial> {
        let encoded = field.encode(&value)?;
        self.fields.push(CreatedField::Literal {
            field: field.locus().clone(),
            binding: field.recipe().binding_identity().clone(),
            value: encoded,
        });
        Ok(self)
    }
}

pub(super) fn resolve<Field: crate::application_schema::fragment::RecordSelection>(
    field: Field,
) -> Result<RecordField<Field::Entity, Field::Aspect, Field::Value>, Denial> {
    field.checked().map_err(|_| Denial::InvalidRecordField {
        field: field.expected_recipe().locus().clone(),
    })
}
