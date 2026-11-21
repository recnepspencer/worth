//! Host access to the installed index's read-only package export.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledPackageExportDenial,
    WorthQueryPortablePackageExportLimits, WorthQueryPortablePackageRecordSet,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Export the actually installed owner's records, including composed contributions.
    /// Use SDK reconstruction separately when fresh readmission is required.
    pub fn export_domain_package(
        &self,
        owner: &str,
        limits: WorthQueryPortablePackageExportLimits,
    ) -> Result<WorthQueryPortablePackageRecordSet, WorthQueryInstalledPackageExportDenial> {
        self.runtime
            .installed_packages()
            .export_domain_package(owner, limits)
    }
}
