use super::*;

pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<FinancialHostSchema, Self, ExternalMapping, Principal> {
        relation(FinancialModel, ExternalMapping, Self)
    }
}
