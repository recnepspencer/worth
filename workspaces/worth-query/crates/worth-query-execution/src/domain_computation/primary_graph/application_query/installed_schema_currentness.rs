use worth_query_declaration::facade::application_schema::ApplicationSchema;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn installed_schema_is_current(&self) -> bool {
        // Publication validates the complete installed declaration. The index is
        // owned by this runtime and can subsequently change only to an exact
        // successor generation or an equivalent-meaning rebuild. A successor
        // invalidates this retained schema; an equivalent rebuild preserves it.
        let installed = self.runtime.installed_packages();
        let binding = self.installed_schema.binding_identity();
        installed.runtime_ordinal() == binding.runtime_ordinal()
            && installed.generation().ordinal() == binding.generation()
    }
}
