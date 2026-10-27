use super::{ApplicationSchema, WorthQueryInstalledApplicationSchema};
use crate::application_operation::binding::{
    compile_application_mutation_catalog, WorthQueryInstalledApplicationMutationCatalog,
};
use crate::application_query::binding::{
    compile_application_query_catalog, WorthQueryInstalledApplicationQueryCatalog,
};
use crate::application_resource::WorthQueryInstalledApplicationResourceCatalog;
use crate::application_schema::{
    compilation::CompiledApplicationSchema,
    invariant::WorthQueryInstalledApplicationInvariantCatalog, ApplicationSchemaCompilationDenial,
};

impl<Schema: ApplicationSchema> WorthQueryInstalledApplicationSchema<Schema> {
    pub(crate) fn from_compilation(
        compiled: CompiledApplicationSchema<Schema>,
    ) -> Result<Self, ApplicationSchemaCompilationDenial> {
        let invariant_catalog =
            WorthQueryInstalledApplicationInvariantCatalog::compile(&compiled.schema);
        let mut installed = Self {
            package_authority: compiled.package_authority,
            schema_name: compiled.schema_name,
            schema_identity: compiled.schema_identity,
            schema: compiled.schema,
            member_provenance: compiled.member_provenance,
            capability_registry: compiled.capability_registry,
            authorization_policy_registry: compiled.authorization_policy_registry,
            value_binding_catalog: compiled.value_binding_catalog,
            native_contract_catalog: compiled.native_contract_catalog,
            portable_native_contracts: compiled.portable_native_contracts,
            portable_operation_contracts: compiled.portable_operation_contracts,
            installation_canonical_work: compiled.installation_canonical_work,
            query_catalog: WorthQueryInstalledApplicationQueryCatalog::default(),
            mutation_catalog: WorthQueryInstalledApplicationMutationCatalog::default(),
            resource_catalog: WorthQueryInstalledApplicationResourceCatalog::default(),
            invariant_catalog,
            _schema: compiled.marker,
        };
        installed.query_catalog = compile_application_query_catalog(&installed)
            .map_err(ApplicationSchemaCompilationDenial::Query)?;
        installed.mutation_catalog = compile_application_mutation_catalog(&installed)
            .map_err(ApplicationSchemaCompilationDenial::Operation)?;
        installed.resource_catalog =
            WorthQueryInstalledApplicationResourceCatalog::compile(&installed)
                .map_err(ApplicationSchemaCompilationDenial::Resource)?;
        Ok(installed)
    }
}
