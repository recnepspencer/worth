//! Finite all-account roster mechanics; Query owns membership and READ truth.
mod observation;
mod provider;
mod storage;
pub use observation::AccountIndexWork;
pub(super) use provider::Provider;
