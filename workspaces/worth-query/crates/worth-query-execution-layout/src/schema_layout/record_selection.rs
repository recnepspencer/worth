//! Typed selectors reuse the same immutable catalog installed with the layout.
use super::WorthQueryPrimaryGraphLayout;
use worth_query_declaration::facade::application_schema::fragment::{
    RecordFieldDefinition, RecordSelection,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryUndeclaredRecordSelection(String);

impl WorthQueryUndeclaredRecordSelection {
    pub fn into_subject(self) -> String {
        self.0
    }
}

impl WorthQueryPrimaryGraphLayout {
    pub fn aspect_selection(
        &self,
        name: &str,
        select: fn(
            &worth_query_declaration::facade::application_schema::fragment::FragmentCatalog,
        ) -> Option<
            &worth_query_declaration::facade::application_schema::fragment::AspectDefinition,
        >,
    ) -> Result<
        &worth_query_declaration::facade::application_schema::fragment::AspectDefinition,
        WorthQueryUndeclaredRecordSelection,
    > {
        select(self.value_bindings.fragments())
            .ok_or_else(|| WorthQueryUndeclaredRecordSelection(name.to_owned()))
    }

    pub fn record_selection<Field: RecordSelection>(
        &self,
        field: Field,
    ) -> Result<&RecordFieldDefinition, WorthQueryUndeclaredRecordSelection> {
        self.record_column(field.identity())
    }
    pub fn record_column(
        &self,
        field: worth_query_declaration::facade::application_schema::fragment::ColumnIdentity,
    ) -> Result<&RecordFieldDefinition, WorthQueryUndeclaredRecordSelection> {
        field
            .installed(self.value_bindings.fragments())
            .map_err(|_| WorthQueryUndeclaredRecordSelection(field.field().to_owned()))
    }
}

impl WorthQueryPrimaryGraphLayout {
    pub fn record_presence_locator(
        &self,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    ) -> Option<&worth_foundational::facade::AspectFieldLocator> {
        self.field(field.entity(), field.aspect(), field.field())?
            .record_presence
            .as_ref()
    }
}
