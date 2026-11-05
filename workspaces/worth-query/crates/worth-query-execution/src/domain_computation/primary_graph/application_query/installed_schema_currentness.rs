use worth_query_declaration::facade::application_schema::ApplicationSchema;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(super) trait InstalledSchemaCurrentnessParentOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn installed_schema_is_current(&self) -> bool {
        // Publication validates the complete installed declaration. The index is
        // owned by this runtime and can subsequently change only to an exact
        // successor generation or an equivalent-meaning rebuild. A successor
        // invalidates this retained schema; an equivalent rebuild preserves it.
        let installed = self.installed_packages();
        let binding = self.installed_schema().binding_identity();
        installed.runtime_ordinal() == binding.runtime_ordinal()
            && installed.generation().ordinal() == binding.generation()
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > InstalledSchemaCurrentnessParentOps<Schema> for HostRuntime
{
}
