//! A requested projection must fit the selected root's declared contracts.
use crate::identity::data::KindId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionRecordDomain {
    Entity,
    Relation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionContractRefusal {
    MissingAspectPlan,
    UndeclaredAspect,
    MaskRejected,
    MissingCanonicalMaskBasis,
}

/// The selected record domain, kind and scope requirement that refused projection.
/// Admission returns this evidence before invoking the caller's projector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalProjectionContractDenial {
    pub domain: ProjectionRecordDomain,
    pub kind_id: KindId,
    /// Index into the caller's projection scope; absent when no plan exists.
    pub requirement: Option<usize>,
    pub refusal: ProjectionContractRefusal,
}
