use worth_foundational::facade::{AspectKey, FieldKey};

use crate::validation::data::InvariantRule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InvariantRegistrationContract {
    DefaultAlwaysOnStructural,
    OptInUserCatalog,
}

impl InvariantRule {
    pub(crate) fn registration_examples() -> Vec<Self> {
        vec![
            Self::LiveRecordRequiresSidecar(super::RecordKindTag::Entity),
            Self::LiveRecordRequiresSidecar(super::RecordKindTag::Relation),
            Self::MaxMergedIntents(1),
            Self::MaxSnapshotEntities(1),
            Self::unique_entity_aspect_field(
                AspectKey::new("__registration_probe__")
                    .expect("valid registration probe aspect key"),
                FieldKey::new("__registration_probe__").expect("valid registration probe field"),
            ),
            Self::EndpointKindContract(crate::schema::data::LoweredEndpointKindContract {
                contract_id: "__registration_probe__".into(),
                relation_kind_id: crate::identity::data::KindId(999),
                allowed_source_kinds: vec![crate::identity::data::KindId(1)],
                allowed_target_kinds: vec![crate::identity::data::KindId(1)],
                self_edges_allowed: true,
                cross_context_policy: crate::config::data::CrossContextPolicy::AllowExplicit,
                plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
            }),
            Self::CardinalityMaximumContract(
                crate::schema::data::LoweredCardinalityMaximumContract {
                    contract_id: "__registration_probe__".into(),
                    relation_kind_id: crate::identity::data::KindId(999),
                    source_max: Some(1),
                    target_max: None,
                    pair_max: None,
                    plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
                },
            ),
            Self::CardinalityMinimumContract(
                crate::schema::data::LoweredCardinalityMinimumContract {
                    contract_id: "__registration_probe__".into(),
                    relation_kind_id: crate::identity::data::KindId(999),
                    source_min: Some(1),
                    target_min: None,
                    pair_min: None,
                    pair_min_semantics: crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                    candidate_source_kinds: vec![crate::identity::data::KindId(1)],
                    candidate_target_kinds: vec![crate::identity::data::KindId(1)],
                    minimum_enforcement: crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
                    plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
                },
            ),
            Self::UniquenessContract(crate::schema::data::LoweredUniquenessContract {
                contract_id: "__registration_probe__".into(),
                relation_kind_id: crate::identity::data::KindId(999),
                scope: crate::schema::data::UniquenessScope::DirectedSemanticEdge,
                plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
            }),
            Self::SymmetryContract(crate::schema::data::LoweredSymmetryContract {
                contract_id: "__registration_probe__".into(),
                relation_kind_id: crate::identity::data::KindId(999),
                mode: crate::schema::data::SymmetryMode::InverseProhibited,
                plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
            }),
            Self::EndpointDeletionIntegrityContract(
                crate::schema::data::LoweredEndpointDeletionIntegrityContract {
                    contract_id: "__registration_probe__".into(),
                    relation_kind_id: crate::identity::data::KindId(999),
                    mode: crate::schema::data::EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
                    cascade_delete_policy: crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
                    plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
                },
            ),
            Self::AcyclicityContract(crate::schema::data::LoweredAcyclicityContract {
                contract_id: "__registration_probe__".into(),
                relation_kind_id: crate::identity::data::KindId(999),
                traversal_direction: crate::schema::data::DirectedTraversalKind::SourceToTarget,
                allowed_cycle_class: crate::schema::data::AllowedCycleClass::NoCycles,
                plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
            }),
            Self::PartitionIsolationContract(
                crate::schema::data::LoweredPartitionIsolationContract {
                    contract_id: "__registration_probe__".into(),
                    relation_kind_id: crate::identity::data::KindId(999),
                    isolation_mode: crate::schema::data::PartitionIsolationMode::SamePartitionEndpoints,
                    plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
                },
            ),
            Self::ConnectivityMinimumContract(
                crate::schema::data::LoweredConnectivityMinimumContract {
                    contract_id: "__registration_probe__".into(),
                    source_kind_ids: vec![crate::identity::data::KindId(1)],
                    relation_kind_id: crate::identity::data::KindId(999),
                    target_kind_ids: vec![crate::identity::data::KindId(2)],
                    minimum_reachable_targets: 1,
                    enforcement_boundary: crate::schema::data::ConnectivityMinimumEnforcement::SnapshotPublication,
                    plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
                },
            ),
        ]
    }

    pub(crate) fn registration_contract(&self) -> InvariantRegistrationContract {
        match self {
            Self::LiveRecordRequiresSidecar(_) => {
                InvariantRegistrationContract::DefaultAlwaysOnStructural
            }
            Self::MaxMergedIntents(_)
            | Self::MaxSnapshotEntities(_)
            | Self::UniqueEntityAspectField { .. }
            | Self::EndpointKindContract(_)
            | Self::CardinalityMaximumContract(_)
            | Self::CardinalityMinimumContract(_)
            | Self::UniquenessContract(_)
            | Self::SymmetryContract(_)
            | Self::EndpointDeletionIntegrityContract(_)
            | Self::AcyclicityContract(_)
            | Self::PartitionIsolationContract(_)
            | Self::ConnectivityMinimumContract(_) => {
                InvariantRegistrationContract::OptInUserCatalog
            }
        }
    }
}
