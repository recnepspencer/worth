//! Deletion integrity sees live incidence after explicit unlinks, before cascade effects.
use super::*;
use crate::config::data::CascadeDeletePolicy;
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::validation::engine::request::{
    PreparedRelationIntegrityScope, PreparedRelationIntegrityScopes,
};
use crate::validation::engine::{InvariantObservation, InvariantRuntimeView};

#[test]
fn rejected_endpoint_deletion_keeps_its_entity_and_live_incidence_under_both_cascade_policies() {
    for policy in [
        CascadeDeletePolicy::CascadeDeleteRelations,
        CascadeDeletePolicy::RetainDanglingForAudit,
    ] {
        let violations = deletion_violations(
            EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
            policy,
            true,
        );
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].fields,
            InvariantViolationFields::RelationEndpointDeletionIntegrity {
                contract_id: "protected:deletion".into(),
                relation_kind_id: KindId(2),
                entity_id: deleted_target(),
                remaining_relation_endpoint_count: 1,
                mode: EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
                cascade_delete_policy: None,
            }
        );
    }
}

#[test]
fn cascade_and_retirement_contracts_keep_their_policy_and_explicit_unlink_semantics() {
    for (mode, policy, refuses) in [
        (
            EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit,
            CascadeDeletePolicy::CascadeDeleteRelations,
            false,
        ),
        (
            EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit,
            CascadeDeletePolicy::RetainDanglingForAudit,
            true,
        ),
        (
            EndpointDeletionIntegrityMode::RequireRelationRetirement,
            CascadeDeletePolicy::CascadeDeleteRelations,
            true,
        ),
        (
            EndpointDeletionIntegrityMode::RequireRelationRetirement,
            CascadeDeletePolicy::RetainDanglingForAudit,
            false,
        ),
    ] {
        let violations = deletion_violations(mode, policy, true);
        assert_eq!(!violations.is_empty(), refuses);
        if refuses {
            assert!(matches!(
                &violations[0].fields,
                InvariantViolationFields::RelationEndpointDeletionIntegrity {
                    entity_id, cascade_delete_policy: Some(actual_policy), mode: actual_mode, ..
                } if *entity_id == deleted_target() && *actual_policy == policy && *actual_mode == mode
            ));
        }
        assert!(deletion_violations(mode, policy, false).is_empty());
        assert!(deletion_violations(
            EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
            policy,
            false
        )
        .is_empty());
    }
}

fn deletion_violations(
    mode: EndpointDeletionIntegrityMode,
    cascade_delete_policy: CascadeDeletePolicy,
    live_edge: bool,
) -> Vec<InvariantViolation> {
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(crate::schema::data::RelationalSchemaRegistry::new())
        .build()
        .unwrap();
    // Pure evaluator input: coordinates carry no authority or admission claim.
    // Public graph courts exercise actual preparation and refusal precedence.
    let mut scope = PreparedRelationIntegrityScope::default();
    scope.deleted_entities.insert(deleted_target());
    if live_edge {
        scope.increment_counts(
            EntityReference::Existing(EntityId::new(PartitionId::main(), 0, 1)),
            EntityReference::Existing(deleted_target()),
        );
    }
    let context = InvariantExecutionContext::new(
        &InvariantRuntimeView::from_runtime(&runtime),
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        runtime.current_version_id(),
        None,
        Some(PreparedRelationIntegrityScopes::new(
            [(KindId(2), scope)].into(),
        )),
        std::sync::Arc::new(std::sync::OnceLock::new()),
    );
    evaluate_endpoint_deletion_integrity_contract(
        &context,
        InvariantClass::CommitBoundary,
        &LoweredEndpointDeletionIntegrityContract {
            contract_id: "protected:deletion".into(),
            relation_kind_id: KindId(2),
            mode,
            cascade_delete_policy,
            plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
        },
    )
}

fn deleted_target() -> EntityId {
    EntityId::new(PartitionId::main(), 1, 1)
}
