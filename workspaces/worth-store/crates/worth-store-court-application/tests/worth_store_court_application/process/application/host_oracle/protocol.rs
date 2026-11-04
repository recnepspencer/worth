//! Untrusted descriptive child observations confer no owner authority.
use super::*;
use host_observation::{projection::StateProjection, receipts::MutationProjection};
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum Request {
    Step(usize),
    Observe,
    ContinueOriginal {
        key: u64,
        value: u64,
        pause_after_disposition: bool,
    },
    ReleaseOriginal {
        key: u64,
        value: u64,
        expect_continued: bool,
    },
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
    Continued(MutationProjection),
    Released(usize),
    ContinuedReleaseRefused,
}
