use super::definition::DerivedRelationDenial;
use crate::execution::denial_cause::RelationalExecutionDenialCause;
use crate::validation::data::structural_walk_denial::StructuralWalkDenialKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DerivedRelationReadDenial {
    Definition(DerivedRelationDenial),
    Structural(StructuralWalkDenialKind),
    Request(RelationalExecutionDenialCause),
}

impl From<DerivedRelationDenial> for DerivedRelationReadDenial {
    fn from(denial: DerivedRelationDenial) -> Self {
        Self::Definition(denial)
    }
}
