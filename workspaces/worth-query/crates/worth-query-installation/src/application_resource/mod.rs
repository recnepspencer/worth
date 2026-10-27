//! Installation resolves portable resource profiles against genuine typed recipes.
mod catalog;
mod denial;
mod installed_binding;
pub(crate) use catalog::WorthQueryInstalledApplicationResourceCatalog;
pub use denial::{
    WorthQueryApplicationResourceInstallationDenial,
    WorthQueryApplicationResourceInstallationDenialKind,
};
pub use installed_binding::WorthQueryInstalledApplicationResource;
