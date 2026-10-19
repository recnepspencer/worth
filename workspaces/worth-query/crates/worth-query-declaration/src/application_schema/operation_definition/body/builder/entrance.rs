//! Scope-bound entrance to the typed body builder.
use super::*;
impl<Schema, Operation, Input> ApplicationOperationRef<Schema, Operation, Input> {
    /// Begins scope-bound source expectations and assignments. Membership is
    /// derived when the completed body is attached to this operation definition.
    pub fn body(self) -> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
        Self::body_empty()
    }
    pub(in crate::application_schema::operation_definition::body) fn body_empty(
    ) -> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
        ApplicationOperationBodyBuilder {
            scope: None,
            steps: Vec::new(),
            values: Vec::new(),
            retirements: Vec::new(),
            path_input_expectations: Vec::new(),
            path_value_assignments: Vec::new(),
            reference_inputs: Vec::new(),
            requirements: Vec::new(),
            outputs: Vec::new(),
            succession: None,
            external_emission: None,
            external_emission_recipe: None,
            marker: PhantomData,
        }
    }
}

impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    pub(in crate::application_schema::operation_definition::body) fn empty_in_scope(
        scope: &str,
    ) -> Self {
        let mut builder = ApplicationOperationRef::<S, O, I>::body_empty();
        builder.scope = Some(scope.into());
        builder
    }
}
