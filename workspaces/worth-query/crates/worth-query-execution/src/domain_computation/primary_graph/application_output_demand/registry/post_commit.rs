//! Completed-effect entries own infallible work; request entries cannot mint its payer.
use super::*;
mod advancement;
pub(super) mod custody_reader;
pub(super) mod discovery;
mod product_retirement;
mod publication;
mod retirement_observation;
pub(super) mod source_retirement;
mod supersession;
#[cfg(test)]
pub(super) fn sweep_contacts() -> u64 {
    retirement_observation::sweep_contacts()
}
pub(super) use advancement::release_completed_member;
pub(super) use advancement::release_completed_prerequisites;
pub(super) use publication::release_published_member;
pub(super) use publication::release_settled_member;
