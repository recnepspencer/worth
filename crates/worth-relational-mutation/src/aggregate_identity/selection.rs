//! Explicit installed definition selection, distinct from orbit relation sugar.
use worth_foundational::facade::AggregateDefinitionIdentity;
use worth_relational_contracts::internal_port::identity::data::primitives::KindId;
use worth_relational_contracts::internal_port::schema::data::relation_integrity::aggregate::AggregateIdentityContractDeclaration;

#[derive(Clone, Copy)]
pub(super) enum AggregateSelection {
    Orbit {
        relation: KindId,
        definition: Option<AggregateDefinitionIdentity>,
    },
    Definition(AggregateDefinitionIdentity),
}
impl AggregateSelection {
    pub(super) fn matches(self, contract: &AggregateIdentityContractDeclaration) -> bool {
        match self {
            Self::Orbit {
                relation,
                definition,
            } => {
                contract.source.orbit_relation() == Some(relation)
                    && definition.is_none_or(|id| id == contract.definition)
            }
            Self::Definition(id) => id == contract.definition,
        }
    }
    pub(super) fn is_definition(self) -> bool {
        matches!(
            self,
            Self::Definition(_)
                | Self::Orbit {
                    definition: Some(_),
                    ..
                }
        )
    }
}
