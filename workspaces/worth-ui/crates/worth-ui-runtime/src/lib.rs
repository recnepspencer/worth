// A cast that can drop a value's high bits, its sign, or its fraction reads
// some other number without saying so. Counts convert with `try_from` against
// the capacity that bounds them, floats cross `whole_number`, and distances
// narrow through `units`.
#![deny(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

mod admission;
mod capability;
#[cfg(worth_ui_compile_probe)]
mod compile_probes;
mod declaration;
mod evidence;
pub mod facade;
mod fact_contract;
mod graph;
pub(crate) mod host;
mod host_exchange;
mod inspection;
mod lifecycle;
mod mounting;
pub mod native_platform;
mod obligations;
mod runtime;
mod source;
mod units;
mod whole_number;

#[cfg(feature = "certification-support")]
#[doc(hidden)]
pub mod certification_support;

#[cfg(all(test, not(feature = "certification-support")))]
pub mod certification_support;
