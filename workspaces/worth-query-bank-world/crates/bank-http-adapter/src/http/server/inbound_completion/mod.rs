mod authentication;
mod configuration;
mod endpoint;
mod maintenance;
mod route;

pub(super) use authentication::BankRailCompletionVerifier;
pub use configuration::BankRailCompletionServerInstallation;
pub(in crate::http::server) use endpoint::rail_completion;
pub(in crate::http::server) use maintenance::start_maintenance;
pub(in crate::http::server) use route::{install, BankRailCompletionRoute};
