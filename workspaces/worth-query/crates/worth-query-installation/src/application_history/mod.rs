//! Installed correspondence for model-history READ and ordinary operation membership.
mod catalog;
mod denial;
mod installed_binding;

pub(crate) use catalog::WorthQueryInstalledApplicationHistoryCatalog;
pub use denial::{
    WorthQueryDocumentHistoryInstallationDenial, WorthQueryDocumentHistoryInstallationDenialKind,
};
pub use installed_binding::WorthQueryInstalledDocumentHistory;
