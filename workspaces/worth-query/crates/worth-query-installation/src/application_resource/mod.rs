//! Installation resolves portable resource profiles against genuine typed recipes.
mod access_compilation;
mod access_contract;
mod access_purpose;
mod catalog;
mod denial;
mod installed_binding;
pub use access_contract::WorthQueryInstalledResourceAccessContract;
pub use access_purpose::WorthQueryResourceAccessPurpose;
pub(crate) use catalog::WorthQueryInstalledApplicationResourceCatalog;
pub use denial::{
    WorthQueryApplicationResourceInstallationDenial,
    WorthQueryApplicationResourceInstallationDenialKind,
};
pub use installed_binding::WorthQueryInstalledApplicationResource;
