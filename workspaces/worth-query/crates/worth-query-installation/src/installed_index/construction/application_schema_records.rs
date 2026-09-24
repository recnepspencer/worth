use std::collections::BTreeMap;
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaBindingIdentity, ErasedApplicationSchemaDeclaration,
};

use crate::application_schema::{
    compile_native_contract_catalog, derive_installed_schema_identity,
    WorthQueryInstalledApplicationSchemaContractCatalog,
};
use crate::generation::{WorthQueryInstallationGeneration, WorthQueryInstallationRuntimeIdentity};
use crate::package::{
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableNativeAspectContractRecord,
};

use super::super::application_schema::denial_mapping::{
    map_catalog_denial_to_index_denial, map_schema_digest_denial_to_index_denial,
};
use super::super::application_schema_record::WorthQueryInstalledApplicationSchemaRecord;
use super::super::{
    WorthQueryInstalledPackageIndexCounters, WorthQueryInstalledPackageIndexDenial,
    WorthQueryInstalledPackageRecord,
};

pub(super) struct ApplicationSchemaRecordCompilationInput<'a> {
    pub runtime: &'a WorthQueryInstallationRuntimeIdentity,
    pub generation: WorthQueryInstallationGeneration,
    pub packages: &'a BTreeMap<String, WorthQueryInstalledPackageRecord>,
    pub declarations: BTreeMap<(String, String), PortableApplicationSchemaInstallationSeed>,
    pub counters: &'a mut WorthQueryInstalledPackageIndexCounters,
}

#[derive(Clone)]
pub(super) struct PortableApplicationSchemaInstallationSeed {
    pub declaration: ErasedApplicationSchemaDeclaration,
    pub native_contracts: Vec<WorthQueryPortableNativeAspectContractRecord>,
    pub operation_contracts: Vec<WorthQueryPortableApplicationOperationContractRecord>,
}

pub(super) fn compile_application_schema_records(
    input: ApplicationSchemaRecordCompilationInput<'_>,
) -> Result<
    BTreeMap<(String, String), WorthQueryInstalledApplicationSchemaRecord>,
    WorthQueryInstalledPackageIndexDenial,
> {
    let mut records = BTreeMap::new();
    for ((owner, name), seed) in input.declarations {
        let package = input
            .packages
            .get(&owner)
            .expect("an admitted application schema retains its owning package");
        let (schema_identity, schema_work) =
            derive_installed_schema_identity(seed.declaration.identity())
                .map_err(|denial| map_schema_digest_denial_to_index_denial(&name, denial))?;
        let binding = ApplicationSchemaBindingIdentity::from_installed_parts(
            input.runtime.ordinal(),
            input.generation.ordinal(),
            *package.package.package().identity().digest(),
            schema_identity,
        );
        let catalog =
            compile_native_contract_catalog(&binding, &seed.native_contracts, schema_work)
                .map_err(map_catalog_denial_to_index_denial)?;
        accumulate_catalog_counters(input.counters, &catalog);
        records.insert(
            (owner, name),
            WorthQueryInstalledApplicationSchemaRecord::new(
                seed.declaration,
                schema_identity,
                schema_work,
                Arc::new(catalog),
                Arc::new(seed.native_contracts),
                Arc::new(seed.operation_contracts),
            ),
        );
    }
    Ok(records)
}

fn accumulate_catalog_counters(
    counters: &mut WorthQueryInstalledPackageIndexCounters,
    catalog: &WorthQueryInstalledApplicationSchemaContractCatalog,
) {
    let compiled = catalog.counters();
    counters.application_schema_catalogs_compiled += compiled.catalogs_compiled();
    counters.application_aspect_contracts_compiled += compiled.contracts_compiled();
    counters.application_aspect_fields_compiled += compiled.fields_compiled();
    counters.application_aspect_canonical_bases_prepared +=
        compiled.canonical_contract_bases_prepared();
}
