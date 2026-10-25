use crate::authority::mutation::outcomes::RecordMutation;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::runtime::RuntimeSymbolTable;
use crate::schema::data::{AspectContractPlanCatalog, RelationalSchemaRegistry};

use super::super::{canonical_delta_for_mutation, CanonicalDeltaError};
use super::support::{empty_working_state, empty_workspace, mutation_config};

#[test]
fn missing_entity_aspect_plan_returns_typed_error() {
    let config = mutation_config();
    let mut state = empty_working_state(&config);
    let mut symbols = RuntimeSymbolTable::default().begin_pending_meanings();
    let catalog = AspectContractPlanCatalog::empty();
    let schema = RelationalSchemaRegistry::new();
    let mutation = RecordMutation::EntityCreated {
        entity_id: EntityId::new(PartitionId(1), 0, 1),
        kind_id: KindId(999),
        authoritative_patch: None,
    };

    let error = canonical_delta_for_mutation(
        &mutation,
        &empty_workspace(&mut state, &mut symbols, &catalog, &config, &schema),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        CanonicalDeltaError::MissingEntityAspectPlan {
            kind_id: KindId(999)
        }
    ));
}

#[test]
fn missing_relation_aspect_plan_returns_typed_error() {
    let config = mutation_config();
    let mut state = empty_working_state(&config);
    let mut symbols = RuntimeSymbolTable::default().begin_pending_meanings();
    let catalog = AspectContractPlanCatalog::empty();
    let schema = RelationalSchemaRegistry::new();
    let source = EntityId::new(PartitionId(1), 0, 1);
    let target = EntityId::new(PartitionId(1), 1, 1);
    let mutation = RecordMutation::RelationCreated {
        relation_id: RelationId::new(PartitionId(2), 0, 1),
        kind_id: KindId(777),
        source,
        target,
        authoritative_patch: None,
    };

    let error = canonical_delta_for_mutation(
        &mutation,
        &empty_workspace(&mut state, &mut symbols, &catalog, &config, &schema),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        CanonicalDeltaError::MissingRelationAspectPlan {
            kind_id: KindId(777)
        }
    ));
}
