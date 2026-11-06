//! Live and frozen selectors share the installed equality lookup owner.
use super::*;
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
pub(super) enum SourceScopeSelector<Field: RecordSelection> {
    Live {
        field: Field,
        value: Field::Value,
    },
    Recorded {
        field: Field,
        recipe: Arc<super::super::request_recipe::CapturedSourceQueryRequest>,
    },
}
impl<Field: RecordSelection> SourceScopeSelector<Field> {
    pub(super) fn resolve<'selected, 'runtime, S: ApplicationSchema>(
        &self,
        selected: &'selected crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'runtime, S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &WorthQueryRequestScope,
        native: &mut ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryIssuedSelectedScope<
            'selected,
            'runtime,
            S,
            Field::Entity,
        >,
        crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial,
    > {
        match self {
            Self::Live { field, value } => selected
                .resolve_query_scope_selector_issued_with_native_access(
                    execution, *field, value, request, native,
                ),
            Self::Recorded { field, recipe } => selected
                .resolve_query_scope_value_issued_with_native_access(
                    execution,
                    *field,
                    recipe.scope().expect("recorded field selector"),
                    request,
                    native,
                ),
        }
    }
}
