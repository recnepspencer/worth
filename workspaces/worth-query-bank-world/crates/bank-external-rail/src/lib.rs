//! Bank external rail: a real, separate-process, TCP-reachable external
//! service boundary with controllable faults, built to close Gate 8.2's
//! exit proof.
//!
//! This crate is a pure Bank boundary. It carries no Query dependency and no
//! Query truth: correlation here is opaque, diagnostic-grade Bank identity,
//! never the typed Query correlation identity that names a dispatch attempt
//! inside the runtime. A Query-side classifier consumes the outcomes this
//! crate produces; it never reaches back into this process's state, and this
//! process is reachable only over real TCP, in a real separate OS process.

#![forbid(unsafe_code)]

mod client;
mod protocol;
mod server;
pub mod test_control;

pub use client::connection::{
    dispatch, inquire_admission_count, inquire_completed_effect_count, inquire_completed_notice,
    inquire_dispatch_contact_count, inquire_notice, inquire_status, RailTransportFailure,
};
pub use client::outcome::RailExchangeOutcome;
pub use client::process_handle::{RailProcessHandle, RailSpawnError};
pub use protocol::correlation::RailCorrelation;
pub use protocol::notice::{EstateDeathNotice, RailRejection};
pub use protocol::payload::RailEffectPayload;
pub use protocol::request::RailDispatch;
pub use protocol::response::LedgerStatus;
pub use protocol::support_profile::RailProtocolSupportProfile;
pub use server::RailServer;
