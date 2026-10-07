use super::WorthQueryHandleDenial;
use worth_relational::facade::branch::RelationalBranchBasisDenial;

/// A source entry preserves both lifecycle and Relational admission refusals.
#[derive(Debug)]
pub enum WorthQueryRelationalSourceDenial {
    Handle(WorthQueryHandleDenial),
    Basis(RelationalBranchBasisDenial),
    InvalidBranchName(String),
}

impl From<WorthQueryHandleDenial> for WorthQueryRelationalSourceDenial {
    fn from(denial: WorthQueryHandleDenial) -> Self {
        Self::Handle(denial)
    }
}
impl From<RelationalBranchBasisDenial> for WorthQueryRelationalSourceDenial {
    fn from(denial: RelationalBranchBasisDenial) -> Self {
        Self::Basis(denial)
    }
}
