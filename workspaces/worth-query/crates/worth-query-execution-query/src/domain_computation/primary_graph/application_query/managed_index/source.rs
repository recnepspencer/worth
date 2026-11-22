//! Managed Source installation, persistent occurrences and Native preparation.
pub mod failure;
pub mod initialization;
pub mod installation;
pub mod occurrences;
pub mod projection;
pub mod projection_support;
mod read_work;
pub mod service;
pub use installation::ManagedSourceInstallation;

mod content;
