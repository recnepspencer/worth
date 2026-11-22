//! Completed-effect entries own infallible work; request entries cannot mint its payer.
use super::*;
pub mod advancement;
pub mod custody_reader;
pub mod discovery;
pub mod product_retirement;
pub mod publication;
pub mod retirement_observation;
pub mod source_retirement;
pub mod supersession;
pub fn sweep_contacts() -> u64 {
    retirement_observation::sweep_contacts()
}
/// A registry-wide retirement pass records each row it contacts, wherever the
/// pass body lives, so the sweep bound courts see every pass.
pub fn record_sweep_contact() {
    retirement_observation::record_sweep_contact();
}
pub use advancement::release_completed_member;
pub use advancement::release_completed_prerequisites;
pub use publication::release_published_member;
pub use publication::release_settled_member;
