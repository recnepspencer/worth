//! Endpoint deletion modes classify live incidence independently of runtime construction.
use super::*;
use crate::config::data::CascadeDeletePolicy;
use crate::identity::data::{EntityId, KindId, PartitionId};

#[test]
fn reject_mode_refuses_live_incidence_under_both_cascade_policies() {
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
fn cascade_and_retirement_contracts_admit_empty_incidence_and_require_matching_policy() {
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
    let contract = LoweredEndpointDeletionIntegrityContract {
        contract_id: "protected:deletion".into(),
        relation_kind_id: KindId(2),
        mode,
        cascade_delete_policy,
        plan_revision: crate::schema::data::RelationIntegrityPlanRevision(1),
    };
    let remaining = usize::from(live_edge);
    endpoint_deletion::classify(&contract, remaining)
        .map(|refusal| {
            endpoint_deletion::violation(
                InvariantClass::CommitBoundary,
                &contract,
                deleted_target(),
                remaining,
                refusal,
            )
        })
        .into_iter()
        .collect()
}

fn deleted_target() -> EntityId {
    EntityId::new(PartitionId::main(), 1, 1)
}
