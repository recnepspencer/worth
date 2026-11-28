use worth_foundational::{
    aspects, AspectContractRevision, AspectIdentity, AspectKey as FoundationalAspectKey,
    ScalarAspectType,
};

use crate::authority::mutation::outcomes::RecordMutation;
use crate::contracts::identity::data::primitives::{KindId, PartitionId};
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::schema::data::aspect_contracts::lowered_plan::{
    AspectContractPlanCatalog, LoweredAspectContractBinding, LoweredAspectContractPlan,
};
use crate::contracts::schema::data::aspect_contracts::revision::AspectContractPlanRevision;
use crate::contracts::schema::data::RelationalSchemaRegistry;
use crate::contracts::transactions::data::outcomes::aspect_delta_failure_fields::AspectDeltaFailureFields;
use worth_foundational::facade::AspectBinding;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

use super::super::super::mutation_delta_dispatch::canonical_delta_for_mutation;
use super::super::data::CanonicalAspectDeltaEvidence;
use super::super::CanonicalDeltaError;
use super::support::{
    assert_authoritative_whole_aspect_locator, empty_working_state, empty_workspace,
    mutation_config,
};

#[test]
fn entity_field_patch_evidence_denial_carries_typed_target() {
    let target =
        crate::contracts::transactions::data::aspect_field_patch::planned_single_field_locator(
            FoundationalAspectKey::new("name").expect("valid aspect key"),
            worth_foundational::facade::FieldKey::new("name").expect("valid field key"),
        );
    let conflict = CanonicalDeltaError::EntityFieldBindingRequiresAuthoritativePatchEvidence {
        target: target.clone(),
    }
    .to_commit_conflict();

    let crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass::AspectDeltaFailure {
        fields,
        detail,
    } = conflict.class
    else {
        panic!("expected aspect delta failure conflict");
    };
    assert!(detail.contains("requires authoritative patch evidence"));
    assert!(matches!(
        fields,
        AspectDeltaFailureFields::EntityFieldBindingRequiresAuthoritativePatchEvidence {
            target: actual
        } if actual == target
    ));
}

#[test]
fn entity_field_delta_requires_authoritative_state() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let config = mutation_config();
    let mut state = empty_working_state(&config);
    let mut symbols = RuntimeSymbolTable::default().begin_pending_meanings();
    let schema = RelationalSchemaRegistry::new();
    let mut catalog = AspectContractPlanCatalog::empty();
    catalog.entity_plans.insert(
        KindId(1),
        LoweredAspectContractPlan {
            kind_id: KindId(1),
            plan_revision: AspectContractPlanRevision(7),
            exclusion_groups: Vec::new(),
            executable_bindings: smallvec::smallvec![LoweredAspectContractBinding {
                contract: aspects()
                    .contract()
                    .for_key(FoundationalAspectKey::new("name").expect("valid aspect key"))
                    .identified_by(AspectIdentity(1))
                    .at_revision(AspectContractRevision(7))
                    .scalar(ScalarAspectType::String),
                target: AspectBinding::EntityField {
                    field: worth_foundational::facade::FieldKey::new("name").expect("valid field"),
                },
            }],
        },
    );
    let mutation = RecordMutation::EntityCreated {
        entity_id: EntityId::new(PartitionId(1), 0, 1),
        kind_id: KindId(1),
        authoritative_patch: None,
    };

    let delta = canonical_delta_for_mutation(
        &mutation,
        &empty_workspace(&host, &mut state, &mut symbols, &catalog, &config, &schema),
    )
    .expect("missing authoritative state should leave absent aspect evidence");

    assert!(delta.changed_aspects.is_empty());
    let CanonicalAspectDeltaEvidence::ScalarAspectValueTransition {
        locator,
        old_present,
        new_present,
        old_value,
        new_value,
    } = &delta.evaluated_bindings[0].evidence
    else {
        panic!("expected absent scalar evidence without authoritative state");
    };
    assert_authoritative_whole_aspect_locator(locator, "name");
    assert!(!old_present);
    assert!(!new_present);
    assert_eq!(old_value, &None);
    assert_eq!(new_value, &None);
}
