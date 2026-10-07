use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};

impl From<crate::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryApplicationCommitDenial
{
    fn from(denial: crate::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::Handle(denial),
            stage: WorthQueryApplicationCommitDenialStage::BasisAdmission,
            detail: None,
            cause: None,
        }
    }
}
