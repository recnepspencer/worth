//! Native schema installation and recovery remain exclusive until Query finishes.
mod attachment;
mod close;
pub use close::{UnrecoveredApplicationHomeCloseRefused, UnsharedApplicationHomeCloseRefused};
mod initialization;
mod refusal;
mod unrecovered;
mod unshared;

pub use attachment::attach_relational;
pub(crate) use initialization::HomeInitializationOrigin;
pub use initialization::{
    ApplicationHomeInitializationDenial, ApplicationHomeInitializationRefused,
};
pub use refusal::{
    RelationalAttachmentFailedHome, RelationalAttachmentRefused, RelationalHomeReadmissionRefused,
};
pub use unrecovered::UnrecoveredApplicationHome;
pub use unshared::UnsharedReadmittedApplicationHome;
