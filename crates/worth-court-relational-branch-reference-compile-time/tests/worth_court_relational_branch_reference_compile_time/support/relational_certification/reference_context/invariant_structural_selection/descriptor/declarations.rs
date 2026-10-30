pub(crate) use {std::sync::Arc, worth_relational::facade::runtime::CustomInvariantDescriptor, worth_relational::facade::runtime::CustomInvariantOperationalMetadata, worth_relational::facade::runtime::CustomInvariantSemanticIdentity, worth_relational::facade::runtime::CustomInvariantSemanticVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantFailureEffect, worth_relational::facade::runtime::InvariantGroupSet, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, crate::relational_certification::reference_context::invariant_structural_selection::StructuralSelectionProbe, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id};

pub(crate) fn selection_descriptor() -> CustomInvariantDescriptor {
    CustomInvariantDescriptor {
        identity: CustomInvariantSemanticIdentity {
            rule_id: worth_relational::facade::runtime::CustomInvariantRuleId::new(
                crate::relational_certification::reference_context::invariant_structural_selection::StructuralSelectionProbe::RULE_ID,
            ),
            semantic_version: worth_relational::facade::runtime::CustomInvariantSemanticVersion::new(1, 0),
        },
        display_name: std::sync::Arc::from("Supply Chain selected structural adjacency probe"),
        operational: CustomInvariantOperationalMetadata {
            maximum_work_units: Some(std::num::NonZeroU64::new(1_000_000).unwrap()),
            access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                read_entity_kinds: vec![
                    entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot),
                    entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage),
                ],
                read_relation_kinds: vec![relation_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage)],
                affected_entity_kinds: vec![
                    entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot),
                    entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage),
                ],
                affected_relation_kinds: vec![relation_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage)],
                include_relation_endpoint_entity_touches: true,
            }
            .canonicalize(),
            execution_point: worth_relational::facade::runtime::InvariantExecutionPoint::CommitBoundary,
            groups: worth_relational::facade::runtime::InvariantGroupSet::all(),
            cost_class: worth_relational::facade::runtime::InvariantCostClass::Touched,
            failure_effect: worth_relational::facade::runtime::InvariantFailureEffect::BlockCommit,
        },
    }
}
