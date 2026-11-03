//! Completed-effect entries own infallible work; request entries cannot mint its payer.
use super::*;
mod advancement;
pub(crate) mod custody_reader;
pub(crate) mod discovery;
mod product_retirement;
mod publication;
pub(crate) mod retirement_observation;
pub(crate) mod source_retirement;
mod supersession;
pub fn sweep_contacts() -> u64 {
    retirement_observation::sweep_contacts()
}
/// A registry-wide retirement pass records each row it contacts, wherever the
/// pass body lives, so the sweep bound courts see every pass.
pub(super) fn record_sweep_contact() {
    retirement_observation::record_sweep_contact();
}
pub(super) use advancement::release_completed_member;
pub(super) use advancement::release_completed_prerequisites;
pub(super) use publication::release_published_member;
pub(super) use publication::release_settled_member;
