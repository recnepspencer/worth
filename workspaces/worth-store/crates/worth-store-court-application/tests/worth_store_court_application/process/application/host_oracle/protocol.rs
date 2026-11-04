//! Untrusted descriptive child observations confer no owner authority.
use super::*;
use host_observation::{projection::StateProjection, receipts::MutationProjection};
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum Request {
    Step(usize),
    Observe,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct Response {
    pub state: Option<StateProjection>,
    pub mutation: Option<MutationProjection>,
    pub initialized: usize,
    pub contacts: usize,
    pub closed: bool,
    pub disposition: Option<Disposition>,
}
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) enum Disposition {
    Continued(Vec<Vec<Action>>),
    Released(usize),
}
pub(super) use host_observation::custody::CustodyAction as Action;
