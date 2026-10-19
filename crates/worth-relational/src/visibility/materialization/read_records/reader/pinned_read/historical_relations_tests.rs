//! Audit visibility is an explicit read posture, not a change to ordinary adjacency.
use crate::{
    facade::{
        config::CascadeDeletePolicy,
        identity::KindId,
        storage::RecordLifecycleState,
        transactions::{
            DeleteEntityIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch,
        },
    },
    schema::data::EndpointDeletionIntegrityMode,
    tests::support::{
        create_endpoint_deletion_relation_fixture, endpoint_deletion_runtime,
        test_owner_begin_transaction_for_main,
    },
};
use std::collections::BTreeSet;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};

#[test]
fn pinned_audit_frontier_retains_deleted_target_identity_without_reviving_live_visibility() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = endpoint_deletion_runtime(
        EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, target, relation) =
        create_endpoint_deletion_relation_fixture(&execution, &runtime, "audit-edge");
    let before = runtime
        .read_truth()
        .pin_read_capability(&runtime.visibility_authority().snapshot(), &[], &execution)
        .unwrap();
    let frontier = BTreeSet::from([source]);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("retire-target").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: target }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(&execution, &runtime).unwrap();
    let after = runtime
        .read_truth()
        .pin_read_capability(&runtime.visibility_authority().snapshot(), &[], &execution)
        .unwrap();
    let old = before
        .bounded_outgoing_relations_for_frontier(&execution, &frontier, KindId(2), 8)
        .unwrap()
        .into_records();
    assert_eq!(old.len(), 1);
    assert_eq!(old[0].relation_id, relation);
    assert_eq!(old[0].lifecycle, RecordLifecycleState::Live);
    assert!(after
        .bounded_outgoing_relations_for_frontier(&execution, &frontier, KindId(2), 8)
        .unwrap()
        .into_records()
        .is_empty());
    let retained = after
        .bounded_outgoing_relations_including_audit_for_frontier(
            &execution,
            &frontier,
            KindId(2),
            8,
        )
        .unwrap()
        .into_records();
    assert_eq!(retained.len(), 1);
    assert_eq!(
        (
            retained[0].relation_id,
            retained[0].source,
            retained[0].target
        ),
        (relation, source, target)
    );
    assert_eq!(
        retained[0].lifecycle,
        RecordLifecycleState::RetainedDanglingForAudit
    );
    assert!(matches!(after.bounded_outgoing_relations_including_audit_for_frontier(&execution, &frontier, KindId(2), 1),
        Err(crate::visibility::materialization::read_records::reader::BoundedFrontierReadStop::Limit(_))));
}
