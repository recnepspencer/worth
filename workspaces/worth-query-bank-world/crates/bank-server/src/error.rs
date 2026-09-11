use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapterAdmissionDenial, WorthQueryAuthenticationDenial,
};
use worth_query_host::facade::declaration::application_schema::ApplicationSchemaDeclarationDenial;
use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionDenial, WorthQueryInstalledApplicationSchemaDenial,
    WorthQueryInstalledPackageIndexDenial, WorthQueryPortablePackageValidationDenial,
    WorthQueryPrincipalBindingInstallationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationPrincipalKeyDenial, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrincipalResolutionDenial, WorthQueryProductBranchAdmissionDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankWorldSeedDenial {
    PrincipalSetMismatch,
}

impl std::fmt::Display for BankWorldSeedDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bank world seed does not map every and only snapshot principal")
    }
}

impl std::error::Error for BankWorldSeedDenial {}

#[derive(Debug)]
pub enum BankIdentityRuntimeBuildError {
    SchemaDeclaration(ApplicationSchemaDeclarationDenial),
    PrincipalKey(WorthQueryApplicationPrincipalKeyDenial),
    PackageValidation(WorthQueryPortablePackageValidationDenial),
    PackageAdmission(WorthQueryInstallationAdmissionDenial),
    RuntimeInstallation(WorthQueryInstalledPackageIndexDenial),
    PrimaryGraph(WorthQueryPrimaryGraphInstallationDenial),
    WorldSeed(BankWorldSeedDenial),
    InstalledSchema(WorthQueryInstalledApplicationSchemaDenial),
    InstalledBinding(WorthQueryPrincipalBindingInstallationDenial),
}

impl std::fmt::Display for BankIdentityRuntimeBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SchemaDeclaration(error) => error.fmt(formatter),
            Self::PrincipalKey(error) => error.fmt(formatter),
            Self::PackageValidation(error) => error.fmt(formatter),
            Self::PackageAdmission(error) => write!(formatter, "{error:?}"),
            Self::RuntimeInstallation(error) => write!(formatter, "{error:?}"),
            Self::PrimaryGraph(error) => error.fmt(formatter),
            Self::WorldSeed(error) => error.fmt(formatter),
            Self::InstalledSchema(error) => error.fmt(formatter),
            Self::InstalledBinding(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for BankIdentityRuntimeBuildError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankAuthenticationBoundaryBuildError(pub WorthQueryAuthenticationAdapterAdmissionDenial);

impl std::fmt::Display for BankAuthenticationBoundaryBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "bank authentication boundary admission denied: {:?}",
            self.0
        )
    }
}

impl std::error::Error for BankAuthenticationBoundaryBuildError {}

#[derive(Debug)]
pub enum BankPrincipalAdmissionError {
    Authentication(WorthQueryAuthenticationDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    Resolution(WorthQueryPrincipalResolutionDenial),
}

impl std::fmt::Display for BankPrincipalAdmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authentication(error) => error.fmt(formatter),
            Self::ProductSelection(error) => write!(formatter, "{error:?}"),
            Self::Resolution(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for BankPrincipalAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Authentication(error) => Some(error),
            Self::ProductSelection(_) => None,
            Self::Resolution(error) => Some(error),
        }
    }
}
