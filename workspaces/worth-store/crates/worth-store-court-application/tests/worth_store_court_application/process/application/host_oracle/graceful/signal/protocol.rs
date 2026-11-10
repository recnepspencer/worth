//! Descriptive observations compare owners; only the Home image admits identity.
use super::operations::State;
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum Request {
    Prepare,
    Continue,
    Close,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Response {
    pub states: Vec<State>,
    pub frontiers: Vec<(u64, u64, u64)>,
    pub initialized: usize,
    pub contacts: (usize, usize, usize, usize),
    pub closed: bool,
}
