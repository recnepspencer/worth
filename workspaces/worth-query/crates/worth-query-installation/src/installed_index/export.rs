//! Read-only export of the exact admitted package retained by this installation.
use super::WorthQueryInstalledPackageIndex;
use crate::package::{
    WorthQueryPortablePackageExportDenial, WorthQueryPortablePackageExportLimits,
    WorthQueryPortablePackageRecordSet,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledPackageExportDenial {
    DomainNotInstalled { owner: String },
    Export(WorthQueryPortablePackageExportDenial),
}

impl std::fmt::Display for WorthQueryInstalledPackageExportDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "installed package export refused: {self:?}")
    }
}
impl std::error::Error for WorthQueryInstalledPackageExportDenial {}

impl WorthQueryInstalledPackageIndex {
    /// Export retained installed meaning through the existing typed record owner.
    /// Records describe a package; they carry no installation or runtime authority.
    pub fn export_domain_package(
        &self,
        owner: &str,
        limits: WorthQueryPortablePackageExportLimits,
    ) -> Result<WorthQueryPortablePackageRecordSet, WorthQueryInstalledPackageExportDenial> {
        let record = self.packages.get(owner).ok_or_else(|| {
            WorthQueryInstalledPackageExportDenial::DomainNotInstalled {
                owner: owner.to_owned(),
            }
        })?;
        record
            .package
            .package()
            .export_typed_records_with_limits(limits)
            .map_err(WorthQueryInstalledPackageExportDenial::Export)
    }
}
