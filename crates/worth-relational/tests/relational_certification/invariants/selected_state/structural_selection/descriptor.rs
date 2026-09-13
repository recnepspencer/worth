use super::*;

pub(super) fn selection_descriptor() -> CustomInvariantDescriptor {
    CustomInvariantDescriptor {
        identity: CustomInvariantSemanticIdentity {
            rule_id: worth_relational::facade::runtime::CustomInvariantRuleId::new(
                StructuralSelectionProbe::RULE_ID,
            ),
            semantic_version: CustomInvariantSemanticVersion::new(1, 0),
        },
        display_name: Arc::from("Supply Chain selected structural adjacency probe"),
        operational: CustomInvariantOperationalMetadata {
            maximum_work_units: std::num::NonZeroU64::new(1_000_000).unwrap(),
            access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                read_entity_kinds: vec![
                    entity_kind_id(EntityKind::CargoLot),
                    entity_kind_id(EntityKind::Voyage),
                ],
                read_relation_kinds: vec![relation_kind_id(RelationKind::CargoBookedOnVoyage)],
                affected_entity_kinds: vec![
                    entity_kind_id(EntityKind::CargoLot),
                    entity_kind_id(EntityKind::Voyage),
                ],
                affected_relation_kinds: vec![relation_kind_id(RelationKind::CargoBookedOnVoyage)],
            }
            .canonicalize(),
            execution_point: InvariantExecutionPoint::CommitBoundary,
            groups: InvariantGroupSet::all(),
            cost_class: InvariantCostClass::Touched,
            failure_effect: InvariantFailureEffect::BlockCommit,
        },
    }
}
