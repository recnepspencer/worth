//! Stable cross-process observation meaning for the permanent Platform Pulse.

mod application_readiness;

pub const PLATFORM_PULSE_STATUS_QUERY_VIEW: &str = "platform.pulse.status";

pub mod intent;
mod native_seed_application;
pub mod observation_contract;
#[doc(hidden)]
pub mod product_world;
pub mod visual_identity_pulse;
mod watched_file_change;

#[doc(hidden)]
pub use application_readiness::PlatformPulseApplicationReadinessSignal;
pub use native_seed_application::PlatformPulseNativeSeedApplication;
#[doc(hidden)]
pub use watched_file_change::bears_on_watched_file;
