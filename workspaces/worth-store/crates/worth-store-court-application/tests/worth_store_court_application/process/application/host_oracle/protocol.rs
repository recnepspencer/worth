//! Untrusted descriptive child observations confer no owner authority.
use super::*;
use host_observation::{projection::StateProjection, receipts::MutationProjection};
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum Request {
    Step(usize),
    Observe,
    ObserveReleased {
        key: u64,
        value: u64,
    },
    AbandonOriginal {
        key: u64,
        value: u64,
    },
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
    ReleasedOriginal(ReleasedOriginalProjection),
    Continued(MutationProjection),
    Released(usize),
    ContinuedReleaseRefused,
}

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) struct ReleasedOriginalProjection {
    pub binding_copy_scale: u64,
    pub identity: u64,
    pub attempt: u64,
    pub expected: host_observation::projection::HeadProjection,
    pub association: Option<ReleasedAssociationProjection>,
    pub release_position: Option<u64>,
    pub release_digest: Option<worth_foundational::facade::CanonicalDigestId>,
    pub native_position: u64,
    pub native_digest: worth_foundational::facade::CanonicalDigestId,
    pub semantic: serde_json::Value,
    pub key: worth_foundational::facade::CanonicalDigestId,
    pub intent: String,
    pub affinity: String,
    pub outcome: u64,
    pub emitted: usize,
    pub settlement_code: u64,
}

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) struct ReleasedAssociationProjection {
    pub owner: worth_foundational::facade::WorldOwnerCoordinate,
    pub branch: String,
    pub incarnation: u64,
    pub attempt: u64,
}
