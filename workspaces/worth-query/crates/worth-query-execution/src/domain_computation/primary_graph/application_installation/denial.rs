use super::super::WorthQueryPrimaryGraphInstallationDenial;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionDenial, WorthQueryInstalledApplicationSchemaDenial,
    WorthQueryInstalledPackageIndexDenial, WorthQueryPortablePackageValidationDenial,
};

/// The exact phase that prevented an application from being published.
#[derive(Debug)]
pub enum WorthQueryInMemoryApplicationDenial {
    Package(WorthQueryPortablePackageValidationDenial),
    Admission(WorthQueryInstallationAdmissionDenial),
    Runtime(WorthQueryInstalledPackageIndexDenial),
    Schema(WorthQueryInstalledApplicationSchemaDenial),
    Contributions(WorthQueryPrimaryGraphInstallationDenial),
    Graph(WorthQueryPrimaryGraphInstallationDenial),
    InitialState(WorthQueryPrimaryGraphInstallationDenial),
    Publication(WorthQueryPrimaryGraphInstallationDenial),
}

impl std::fmt::Display for WorthQueryInMemoryApplicationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "in-memory application installation denied: {self:?}"
        )
    }
}

impl std::error::Error for WorthQueryInMemoryApplicationDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Package(error) => Some(error),
            Self::Admission(_) | Self::Runtime(_) => None,
            Self::Schema(error) => Some(error),
            Self::Contributions(error)
            | Self::Graph(error)
            | Self::InitialState(error)
            | Self::Publication(error) => Some(error),
        }
    }
}
