//! Managed Source installation, persistent occurrences and Native preparation.
pub(super) mod failure;
pub(crate) mod initialization;
pub(crate) mod installation;
pub(super) mod occurrences;
pub(super) mod projection;
pub(super) mod service;
pub(in crate::domain_computation::primary_graph) use installation::ManagedSourceInstallation;

mod content;
