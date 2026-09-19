use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration,
};

use crate::application_schema::{
    compile_application_schema, ApplicationSchemaCompilationInput,
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledApplicationSchemaDenial,
    WorthQueryInstalledApplicationSchemaDenialKind,
};

use super::super::WorthQueryInstalledPackageIndex;
use super::denial_mapping::{map_compilation_denial, map_index_denial_to_schema_denial};

impl WorthQueryInstalledPackageIndex {
    pub fn bind_application_schema<Schema>(
        &self,
        declaration: ApplicationSchemaDeclaration<Schema>,
    ) -> Result<
        WorthQueryInstalledApplicationSchema<Schema>,
        WorthQueryInstalledApplicationSchemaDenial,
    >
    where
        Schema: ApplicationSchema,
    {
        let schema = declaration.erased();
        if !declaration.member_provenance().is_empty() {
            let owner_declaration = Schema::declaration().map_err(|_| {
                WorthQueryInstalledApplicationSchemaDenial::new(
                    WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged,
                    schema.name(),
                )
            })?;
            if owner_declaration.member_provenance() != declaration.member_provenance() {
                return Err(WorthQueryInstalledApplicationSchemaDenial::new(
                    WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged,
                    schema.name(),
                ));
            }
        }
        let installed = self
            .application_schemas
            .get(&(schema.owner().to_string(), schema.name().to_string()))
            .ok_or_else(|| {
                WorthQueryInstalledApplicationSchemaDenial::new(
                    WorthQueryInstalledApplicationSchemaDenialKind::SchemaNotInstalled,
                    schema.name(),
                )
            })?;
        if installed.declaration() != schema {
            return Err(WorthQueryInstalledApplicationSchemaDenial::new(
                WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged,
                schema.name(),
            ));
        }
        let authority = self
            .domain(schema.owner())
            .map_err(map_index_denial_to_schema_denial)?;
        let compiled = compile_application_schema(ApplicationSchemaCompilationInput {
            package_authority: authority,
            declaration: &declaration,
            schema_identity: installed.schema_identity(),
            native_contract_catalog: installed.catalog().clone(),
            portable_native_contracts: installed.native_contracts().clone(),
            portable_operation_contracts: installed.operation_contracts().clone(),
            upstream_installation_work: self.installation_canonical_work(),
        })
        .map_err(|denial| map_compilation_denial(schema.name(), denial))?;
        WorthQueryInstalledApplicationSchema::from_compilation(compiled)
            .map_err(|denial| map_compilation_denial(schema.name(), denial))
    }

    pub fn validate_application_schema<Schema>(
        &self,
        installed: &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryInstalledApplicationSchemaDenial>
    where
        Schema: ApplicationSchema,
    {
        self.validate(&installed.package_authority)
            .map_err(map_index_denial_to_schema_denial)?;
        let current = self
            .application_schemas
            .get(&(
                installed.package_authority.owner.clone(),
                installed.schema_name.clone(),
            ))
            .ok_or_else(|| {
                WorthQueryInstalledApplicationSchemaDenial::new(
                    WorthQueryInstalledApplicationSchemaDenialKind::SchemaNotInstalled,
                    &installed.schema_name,
                )
            })?;
        if current.declaration() != &installed.schema {
            return Err(WorthQueryInstalledApplicationSchemaDenial::new(
                WorthQueryInstalledApplicationSchemaDenialKind::SchemaMeaningChanged,
                &installed.schema_name,
            ));
        }
        Ok(())
    }
}
