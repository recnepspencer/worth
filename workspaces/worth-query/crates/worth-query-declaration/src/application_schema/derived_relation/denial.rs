//! Refusal of symbolic derived meaning, before it can reach installed readers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A malformed recipe, missing declared dependency or invalid consumer association.
pub enum ApplicationDerivedRelationDefinitionDenial {
    InvalidIdentifier,
    EmptyExpression,
    InvalidNodeReference,
    UnusedExpressionNode,
    DiscontinuousEndpoints,
    HeterogeneousSelfExclusion,
    WrongDefinitionEndpoints,
    InvalidGraphLaws,
    MissingSource,
    DuplicateDefinition,
    CyclicDefinitions,
    InvalidCollectionAssociation,
    MissingOperation,
}

impl std::fmt::Display for ApplicationDerivedRelationDefinitionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "derived relation definition denied: {self:?}")
    }
}

impl std::error::Error for ApplicationDerivedRelationDefinitionDenial {}
