//! Typed selectors reuse the same immutable catalog installed with the layout.
use super::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_query_declaration::facade::application_schema::fragment::{
    RecordFieldDefinition, RecordSelection,
};
impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn aspect_selection(
        &self,
        name: &str,
        select: fn(
            &worth_query_declaration::facade::application_schema::fragment::FragmentCatalog,
        ) -> Option<
            &worth_query_declaration::facade::application_schema::fragment::AspectDefinition,
        >,
    ) -> Result<
        &worth_query_declaration::facade::application_schema::fragment::AspectDefinition,
        Denial,
    > {
        select(self.value_bindings.fragments())
            .ok_or_else(|| Denial::new(Kind::UndeclaredEffect, name))
    }

    pub(in crate::domain_computation::primary_graph) fn record_selection<Field: RecordSelection>(
        &self,
        field: Field,
    ) -> Result<&RecordFieldDefinition, Denial> {
        self.record_column(field.identity())
    }
    pub(in crate::domain_computation::primary_graph) fn record_column(
        &self,
        field: worth_query_declaration::facade::application_schema::fragment::ColumnIdentity,
    ) -> Result<&RecordFieldDefinition, Denial> {
        field
            .installed(self.value_bindings.fragments())
            .map_err(|_| Denial::new(Kind::UndeclaredEffect, field.field()))
    }
}

impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn record_presence_locator(
        &self,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    ) -> Option<&worth_foundational::facade::AspectFieldLocator> {
        self.field(field.entity(), field.aspect(), field.field())?
            .record_presence
            .as_ref()
    }
}
