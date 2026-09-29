mod authentication;
mod closure;
mod configuration;
mod endpoint;
mod host;
mod maintenance;
mod route;

pub(super) use authentication::BankRailCompletionVerifier;
pub use closure::{
    BankRailCloseAssessment, BankRailCompletionClose, BankRailCompletionCloseFailure,
    BankRailCompletionContinuation,
};
pub use configuration::BankRailCompletionServerInstallation;
pub(in crate::http::server) use endpoint::{
    rail_completion, receive, BankRailCompletionEndpointState,
};
pub use host::{BankRailCallbackServer, BankRailCallbackServerBinding};
pub(in crate::http::server) use maintenance::start_maintenance;
#[cfg(test)]
pub(in crate::http::server) use route::BankRailMaintenanceBatch;
pub(in crate::http::server) use route::{
    install, BankRailCompletionRoute, BankRailCompletionRuntime,
};
