//! Higher-level desktop temporal contribution, caller-pumped monotonic clock,
//! and bounded reopen catch-up loop helper.
//!
//! Derives the standard domain operation definition, conditional binding,
//! named-clock registration, reconstruction access, and recurring execution
//! wiring from an authored timed-intent model while keeping authoritative
//! lifecycle, bounded projection, operation effects, and fresh execution
//! authority strictly caller-owned.
//!
//! OS background wakes while the desktop application is closed remain outside
//! WORTH: callers install paired clock bindings via
//! [`install_desktop_temporal_clock_operation`] (returning
//! [`WorthQueryInstalledDesktopTemporalClock`], the sole implementor of sealed
//! [`WorthQueryDesktopTemporalHomeBinding`]) and advance missed occurrences
//! explicitly on reopen or foreground ticks through
//! `WorthQueryApplicationHome::catch_up_temporal` /
//! `catch_up_temporal_observing` / `pump_temporal` / `pump_temporal_observing`
//! (or low-level [`pump_temporal_catch_up_until`] /
//! [`pump_temporal_catch_up_until_observing`] / [`pump_temporal_catch_up`] /
//! [`pump_temporal_catch_up_observing`]).

mod catch_up;
mod clock;
mod contract;
mod home_binding;
mod installer;

pub use catch_up::{
    pump_temporal_catch_up, pump_temporal_catch_up_observing, pump_temporal_catch_up_until,
    pump_temporal_catch_up_until_observing, WorthQueryTemporalCatchUpClockStop,
    WorthQueryTemporalCatchUpDenial, WorthQueryTemporalCatchUpPolicy,
    WorthQueryTemporalCatchUpReport, WorthQueryTemporalCatchUpStatus, MAX_TEMPORAL_CATCH_UP_STEPS,
};
pub use clock::{
    WorthQueryDesktopClockAdvanceDenial, WorthQueryDesktopClockHandle, WorthQueryDesktopClockSource,
};
pub use contract::{
    WorthQueryAlwaysReadyTemporalPredicate, WorthQueryDesktopTemporalNodeSpec,
    DEFAULT_DESKTOP_TEMPORAL_MAX_MEMORY_BYTES, DEFAULT_DESKTOP_TEMPORAL_MAX_WORK,
};
pub use home_binding::{
    install_desktop_temporal_clock_operation, WorthQueryDesktopTemporalHomeBinding,
    WorthQueryInstalledDesktopTemporalClock,
};
pub use installer::{
    install_desktop_temporal_operation, WorthQueryDesktopTemporalBindingSpec,
    WorthQueryDesktopTemporalInstallationDenial, WorthQueryInstalledDesktopTemporal,
};
