use serde::{Deserialize, Serialize};

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};

use crate::schema::data::{
    LoweredAcyclicityContract, LoweredCardinalityMaximumContract,
    LoweredCardinalityMinimumContract, LoweredConnectivityMinimumContract,
    LoweredEndpointDeletionIntegrityContract, LoweredEndpointKindContract,
    LoweredPartitionIsolationContract, LoweredSymmetryContract, LoweredUniquenessContract,
    MinimumCardinalityEnforcement,
};

use super::descriptor::{
    InvariantRuleDescriptor, InvariantSemanticsClass, SupportedExecutionPoints,
};
use super::execution::InvariantExecutionPoint;
use super::groups::{InvariantCostClass, InvariantGroup, InvariantGroupSet};
use super::rule_id::{InvariantRuleId, NativeInvariantRuleId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecordKindTag {
    Entity,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvariantRule {
    LiveRecordRequiresSidecar(RecordKindTag),
    MaxMergedIntents(usize),
    MaxSnapshotEntities(usize),
    UniqueEntityAspectField {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
        field_locator: AspectFieldLocator,
    },
    EndpointKindContract(LoweredEndpointKindContract),
    CardinalityMaximumContract(LoweredCardinalityMaximumContract),
    CardinalityMinimumContract(LoweredCardinalityMinimumContract),
    UniquenessContract(LoweredUniquenessContract),
    SymmetryContract(LoweredSymmetryContract),
    EndpointDeletionIntegrityContract(LoweredEndpointDeletionIntegrityContract),
    AcyclicityContract(LoweredAcyclicityContract),
    PartitionIsolationContract(LoweredPartitionIsolationContract),
    ConnectivityMinimumContract(LoweredConnectivityMinimumContract),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InvariantRuleMetadata {
    pub groups: InvariantGroupSet,
    pub cost: InvariantCostClass,
}

impl InvariantRule {
    pub fn unique_entity_aspect_field(aspect_key: AspectKey, field: FieldKey) -> Self {
        Self::unique_entity_aspect_field_locator(AspectFieldLocator::new(
            LocatorAuthority::Planned,
            aspect_key,
            CanonicalFieldPath::single(field),
        ))
    }

    pub fn unique_entity_aspect_field_locator(field_locator: AspectFieldLocator) -> Self {
        Self::UniqueEntityAspectField { field_locator }
    }

    pub fn rule_id(&self) -> InvariantRuleId {
        InvariantRuleId::Native(match self {
            Self::LiveRecordRequiresSidecar(RecordKindTag::Entity) => {
                NativeInvariantRuleId::LiveRecordRequiresSidecarEntity
            }
            Self::LiveRecordRequiresSidecar(RecordKindTag::Relation) => {
                NativeInvariantRuleId::LiveRecordRequiresSidecarRelation
            }
            Self::MaxMergedIntents(_) => NativeInvariantRuleId::MaxMergedIntents,
            Self::MaxSnapshotEntities(_) => NativeInvariantRuleId::MaxSnapshotEntities,
            Self::UniqueEntityAspectField { .. } => NativeInvariantRuleId::UniqueEntityField,
            Self::EndpointKindContract(_) => NativeInvariantRuleId::EndpointKindContract,
            Self::CardinalityMaximumContract(_) => {
                NativeInvariantRuleId::CardinalityMaximumContract
            }
            Self::CardinalityMinimumContract(_) => {
                NativeInvariantRuleId::CardinalityMinimumContract
            }
            Self::UniquenessContract(_) => NativeInvariantRuleId::UniquenessContract,
            Self::SymmetryContract(_) => NativeInvariantRuleId::SymmetryContract,
            Self::EndpointDeletionIntegrityContract(_) => {
                NativeInvariantRuleId::EndpointDeletionIntegrityContract
            }
            Self::AcyclicityContract(_) => NativeInvariantRuleId::AcyclicityContract,
            Self::PartitionIsolationContract(_) => {
                NativeInvariantRuleId::PartitionIsolationContract
            }
            Self::ConnectivityMinimumContract(_) => {
                NativeInvariantRuleId::ConnectivityMinimumContract
            }
        })
    }

    pub fn descriptor_for(
        &self,
        failure_effect: super::execution::InvariantFailureEffect,
    ) -> InvariantRuleDescriptor {
        let execution_points = [
            InvariantExecutionPoint::MutationSensitive,
            InvariantExecutionPoint::CommitBoundary,
            InvariantExecutionPoint::SnapshotPublication,
            InvariantExecutionPoint::CertificationBoundary,
            InvariantExecutionPoint::HarnessAudit,
            InvariantExecutionPoint::GraphComposition,
        ]
        .into_iter()
        .filter(|point| self.supports_execution_point(*point))
        .fold(SupportedExecutionPoints::empty(), |supported, point| {
            supported.union(SupportedExecutionPoints::only(point))
        });
        InvariantRuleDescriptor {
            id: self.rule_id(),
            execution_points,
            groups: self.groups(),
            cost_class: self.cost_class(),
            failure_effect,
            semantics: match self {
                Self::LiveRecordRequiresSidecar(_)
                | Self::MaxMergedIntents(_)
                | Self::MaxSnapshotEntities(_)
                | Self::UniqueEntityAspectField { .. } => InvariantSemanticsClass::NativeAlwaysOn,
                Self::EndpointKindContract(_)
                | Self::CardinalityMaximumContract(_)
                | Self::CardinalityMinimumContract(_)
                | Self::UniquenessContract(_)
                | Self::SymmetryContract(_)
                | Self::EndpointDeletionIntegrityContract(_)
                | Self::AcyclicityContract(_)
                | Self::PartitionIsolationContract(_)
                | Self::ConnectivityMinimumContract(_) => {
                    InvariantSemanticsClass::NativeSchemaLowered
                }
            },
        }
    }

    pub(crate) fn metadata(&self) -> InvariantRuleMetadata {
        match self {
            Self::LiveRecordRequiresSidecar(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::StorageCoherence)
                    .union(InvariantGroupSet::of(InvariantGroup::IdentityCoherence)),
                cost: InvariantCostClass::Touched,
            },
            Self::MaxMergedIntents(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::PublicationCoherence),
                cost: InvariantCostClass::Touched,
            },
            Self::MaxSnapshotEntities(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::VersionVisibility)
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence)),
                cost: InvariantCostClass::Global,
            },
            Self::UniqueEntityAspectField { .. } => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance)
                    .union(InvariantGroupSet::of(InvariantGroup::IdentityCoherence)),
                cost: InvariantCostClass::Global,
            },
            Self::EndpointKindContract(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance)
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity)),
                cost: InvariantCostClass::Touched,
            },
            Self::CardinalityMaximumContract(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::RelationIntegrity),
                cost: InvariantCostClass::Touched,
            },
            Self::CardinalityMinimumContract(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::RelationIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::VersionVisibility)),
                cost: InvariantCostClass::Global,
            },
            Self::UniquenessContract(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::IdentityCoherence)
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity)),
                cost: InvariantCostClass::Touched,
            },
            Self::SymmetryContract(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::AdjacencyIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity)),
                cost: InvariantCostClass::Touched,
            },
            Self::EndpointDeletionIntegrityContract(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::RelationIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::SchemaCompliance)),
                cost: InvariantCostClass::Touched,
            },
            Self::AcyclicityContract(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::AdjacencyIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::RelationIntegrity)),
                cost: InvariantCostClass::Global,
            },
            Self::PartitionIsolationContract(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::RelationIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence)),
                cost: InvariantCostClass::Touched,
            },
            Self::ConnectivityMinimumContract(_) => InvariantRuleMetadata {
                groups: InvariantGroupSet::of(InvariantGroup::RelationIntegrity)
                    .union(InvariantGroupSet::of(InvariantGroup::VersionVisibility))
                    .union(InvariantGroupSet::of(InvariantGroup::PublicationCoherence)),
                cost: InvariantCostClass::Global,
            },
        }
    }

    pub(crate) fn cost_class(&self) -> InvariantCostClass {
        self.metadata().cost
    }

    pub(crate) fn groups(&self) -> InvariantGroupSet {
        self.metadata().groups
    }

    pub(crate) fn supports_execution_point(
        &self,
        execution_point: InvariantExecutionPoint,
    ) -> bool {
        match self {
            Self::LiveRecordRequiresSidecar(_) => {
                execution_point == InvariantExecutionPoint::MutationSensitive
            }
            Self::MaxMergedIntents(_) => {
                execution_point == InvariantExecutionPoint::CommitBoundary
                    || execution_point == InvariantExecutionPoint::HarnessAudit
            }
            Self::UniqueEntityAspectField { .. } => {
                execution_point == InvariantExecutionPoint::MutationSensitive
                    || execution_point == InvariantExecutionPoint::CommitBoundary
                    || execution_point == InvariantExecutionPoint::HarnessAudit
            }
            Self::MaxSnapshotEntities(_) => {
                execution_point == InvariantExecutionPoint::SnapshotPublication
            }
            Self::EndpointKindContract(_)
            | Self::UniquenessContract(_)
            | Self::SymmetryContract(_)
            | Self::EndpointDeletionIntegrityContract(_) => {
                execution_point == InvariantExecutionPoint::CommitBoundary
            }
            Self::AcyclicityContract(_) | Self::PartitionIsolationContract(_) => {
                execution_point == InvariantExecutionPoint::CommitBoundary
            }
            Self::CardinalityMaximumContract(_) => {
                execution_point == InvariantExecutionPoint::CommitBoundary
            }
            Self::CardinalityMinimumContract(contract) => match contract.minimum_enforcement {
                MinimumCardinalityEnforcement::CommitBoundary => {
                    execution_point == InvariantExecutionPoint::CommitBoundary
                }
                MinimumCardinalityEnforcement::CertificationBoundary => {
                    execution_point == InvariantExecutionPoint::CertificationBoundary
                }
            },
            Self::ConnectivityMinimumContract(_) => {
                execution_point == InvariantExecutionPoint::SnapshotPublication
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn same_registration_kind(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LiveRecordRequiresSidecar(left), Self::LiveRecordRequiresSidecar(right)) => {
                left == right
            }
            (Self::MaxMergedIntents(_), Self::MaxMergedIntents(_))
            | (Self::MaxSnapshotEntities(_), Self::MaxSnapshotEntities(_))
            | (Self::UniqueEntityAspectField { .. }, Self::UniqueEntityAspectField { .. }) => true,
            (Self::EndpointKindContract(left), Self::EndpointKindContract(right)) => {
                left.contract_id == right.contract_id
                    && left.relation_kind_id == right.relation_kind_id
            }
            (Self::CardinalityMaximumContract(left), Self::CardinalityMaximumContract(right)) => {
                left.contract_id == right.contract_id
                    && left.relation_kind_id == right.relation_kind_id
            }
            (Self::CardinalityMinimumContract(left), Self::CardinalityMinimumContract(right)) => {
                left.contract_id == right.contract_id
                    && left.relation_kind_id == right.relation_kind_id
            }
            (Self::UniquenessContract(left), Self::UniquenessContract(right)) => {
                left.contract_id == right.contract_id
                    && left.relation_kind_id == right.relation_kind_id
            }
            (Self::SymmetryContract(left), Self::SymmetryContract(right)) => {
                left.contract_id == right.contract_id
                    && left.relation_kind_id == right.relation_kind_id
            }
            (
                Self::EndpointDeletionIntegrityContract(left),
                Self::EndpointDeletionIntegrityContract(right),
            ) => {
                left.contract_id == right.contract_id
                    && left.relation_kind_id == right.relation_kind_id
            }
            (Self::AcyclicityContract(left), Self::AcyclicityContract(right)) => {
                left.contract_id == right.contract_id
                    && left.relation_kind_id == right.relation_kind_id
            }
            (Self::PartitionIsolationContract(left), Self::PartitionIsolationContract(right)) => {
                left.contract_id == right.contract_id
                    && left.relation_kind_id == right.relation_kind_id
            }
            (Self::ConnectivityMinimumContract(left), Self::ConnectivityMinimumContract(right)) => {
                left.contract_id == right.contract_id
                    && left.relation_kind_id == right.relation_kind_id
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::data::{
        InvariantFailureEffect, InvariantRuleId, InvariantSemanticsClass, NativeInvariantRuleId,
    };

    #[test]
    fn native_rules_report_stable_rule_ids_and_descriptors() {
        let rule = InvariantRule::LiveRecordRequiresSidecar(RecordKindTag::Entity);
        assert_eq!(
            rule.rule_id(),
            InvariantRuleId::Native(NativeInvariantRuleId::LiveRecordRequiresSidecarEntity)
        );

        let descriptor = rule.descriptor_for(InvariantFailureEffect::BlockCommit);
        assert_eq!(
            descriptor.id,
            InvariantRuleId::Native(NativeInvariantRuleId::LiveRecordRequiresSidecarEntity)
        );
        assert_eq!(
            descriptor.semantics,
            InvariantSemanticsClass::NativeAlwaysOn
        );
        assert!(descriptor
            .execution_points
            .supports(InvariantExecutionPoint::MutationSensitive));
        assert!(!descriptor
            .execution_points
            .supports(InvariantExecutionPoint::SnapshotPublication));
    }
}
