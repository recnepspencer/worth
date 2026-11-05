//! Refusals retain the actual foreign source and its original schema contracts.
use super::*;
use crate::{
    branch::AdmittedRelationalBranchBasis, identity::data::RelationId,
    schema::data::EndpointDeletionIntegrityDeclaration, validation::data::StructuralRelationRecord,
};

#[derive(Debug)]
pub enum RelationalOwnedModelSelectionDenial {
    Ownership(RelationalModelOwnershipDenial),
    ForeignIncomingRelation(RelationalOwnedModelForeignRelation),
    ForeignRelationSource {
        relation: RelationId,
        owner: EntityId,
    },
    SelectedEndpointUnavailable {
        relation: RelationId,
        endpoint: EntityId,
    },
    TransferEvidenceUnavailable {
        model: EntityId,
        commit: crate::history::data::CommitId,
    },
    SelectedOwnerChanged {
        entity: EntityId,
        owner: EntityId,
        source: EntityId,
        actual_owner: Option<EntityId>,
        relation: RelationId,
    },
    SelectedOwnerUnavailable {
        entity: EntityId,
        owner: EntityId,
        source: EntityId,
        relation: RelationId,
    },
    RelationKindUnavailable {
        relation: RelationId,
        kind: KindId,
    },
}
impl RelationalOwnedModelSelectionDenial {
    /// Borrow the actual foreign-edge attribution without requiring a consumer
    /// to name Native's enum variants through another dependency.
    pub fn foreign_incoming_relation(&self) -> Option<&RelationalOwnedModelForeignRelation> {
        match self {
            Self::ForeignIncomingRelation(foreign) => Some(foreign),
            _ => None,
        }
    }
}
impl From<RelationalModelOwnershipDenial> for RelationalOwnedModelSelectionDenial {
    fn from(value: RelationalModelOwnershipDenial) -> Self {
        Self::Ownership(value)
    }
}

/// Source ownership is the actual full-generation source endpoint (D1117).
/// The diagnostic borrows real installed contracts from its retained basis.
#[derive(Debug)]
pub struct RelationalOwnedModelForeignRelation {
    entity: EntityId,
    relation: RelationId,
    owner: EntityId,
    kind: KindId,
    basis: AdmittedRelationalBranchBasis,
}
impl RelationalOwnedModelForeignRelation {
    pub fn entity(&self) -> EntityId {
        self.entity
    }
    pub fn relation(&self) -> RelationId {
        self.relation
    }
    pub fn owner(&self) -> EntityId {
        self.owner
    }
    pub fn kind(&self) -> KindId {
        self.kind
    }

    pub(super) fn new(
        basis: &AdmittedRelationalBranchBasis,
        entity: EntityId,
        edge: StructuralRelationRecord,
    ) -> Result<Self, RelationalOwnedModelSelectionDenial> {
        basis
            .inner
            .root
            .schema_authority()
            .registry()
            .relation_registration(edge.kind_id)
            .map_err(
                |_| RelationalOwnedModelSelectionDenial::RelationKindUnavailable {
                    relation: edge.relation_id,
                    kind: edge.kind_id,
                },
            )?;
        Ok(Self {
            entity,
            relation: edge.relation_id,
            owner: edge.source,
            kind: edge.kind_id,
            basis: basis.clone(),
        })
    }
    pub fn deletion_contracts(&self) -> &[EndpointDeletionIntegrityDeclaration] {
        &self
            .basis
            .inner
            .root
            .schema_authority()
            .registry()
            .relation_registration(self.kind)
            .expect("constructed from this immutable schema registration")
            .relation_integrity
            .endpoint_deletion_integrity_contracts
    }
}
