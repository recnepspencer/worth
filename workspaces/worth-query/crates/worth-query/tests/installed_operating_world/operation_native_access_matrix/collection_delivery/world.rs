use worth_foundational::facade::{AspectValue, InternedString};
use worth_query::facade::{domain, foundation, runtime};

use super::super::collection_window::{bound_collection, first_window};
use super::super::fixture::{insert_matrix_value, matrix_workspace};
use super::super::samples::{matrix_aspect_key, matrix_value_with_order};
use super::{
    managed_collection_lease, oracle::CollectionMountOracle, required_patch, CollectionLease,
};

pub(super) struct CollectionDeliveryWorld {
    workspace: runtime::WorthQueryWorkspace,
    moved: foundation::WorthQueryEntityIdentity,
    outside: foundation::WorthQueryEntityIdentity,
    consumer: domain::WorthQueryCollectionConsumerWindow,
    unbound: domain::WorthQueryCollectionConsumerWindow,
    mounted: CollectionMountOracle,
    lease: CollectionLease,
}

impl CollectionDeliveryWorld {
    pub(super) fn new(request_execution: &worth_execution::ExecutionRequest<'_, '_>) -> Self {
        let mut workspace =
            matrix_workspace(request_execution, "collection-patch-fresh-parity", 0, false);
        let moved = insert_matrix_value(
            request_execution,
            &mut workspace,
            0,
            matrix_value_with_order(0, "30"),
        );
        insert_matrix_value(
            request_execution,
            &mut workspace,
            1,
            matrix_value_with_order(1, "10"),
        );
        insert_matrix_value(
            request_execution,
            &mut workspace,
            2,
            matrix_value_with_order(2, "20"),
        );
        let outside = insert_matrix_value(
            request_execution,
            &mut workspace,
            3,
            matrix_value_with_order(3, "90"),
        );
        let (baseline, baseline_key) = bound_collection(&mut workspace);
        let baseline_window = first_window(&baseline, 3);
        let mounted = CollectionMountOracle::from_bound(&baseline, &baseline_window, &baseline_key);
        let (unbound_collection, _) = bound_collection(&mut workspace);
        let unbound_window = first_window(&unbound_collection, 3);
        let consumer =
            domain::WorthQueryCollectionConsumerWindow::from_bound(baseline, baseline_window)
                .unwrap();
        let unbound = domain::WorthQueryCollectionConsumerWindow::from_bound(
            unbound_collection,
            unbound_window,
        )
        .unwrap();
        let lease = managed_collection_lease(request_execution, &mut workspace);
        assert!(lease.drain(&mut workspace).unwrap().delivery().is_empty());
        Self {
            workspace,
            moved,
            outside,
            consumer,
            unbound,
            mounted,
            lease,
        }
    }

    pub(super) fn move_row_and_reject_reuse(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        self.workspace
            .update(request_execution, self.moved.clone(), |mutation| {
                mutation.set_aspect(
                    runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                    matrix_value_with_order(0, "05"),
                )
            })
            .unwrap();
        let delivery = self.lease.drain(&mut self.workspace).unwrap();
        let delta = self.lease.consumer_invalidation_delta(delivery).unwrap();
        let admitted = match self
            .lease
            .admit_consumer_invalidation_delta(delta, &self.workspace)
        {
            Ok(admitted) => admitted,
            Err(stop) => panic!("collection invalidation stopped: {:?}", stop.kind()),
        };
        self.consumer
            .bind_shared_target(&admitted, &self.workspace)
            .unwrap();
        let foreign_patch = required_patch(
            request_execution,
            &mut self.consumer,
            &admitted,
            &self.workspace,
        );
        let denial = match self.unbound.apply_patch(foreign_patch) {
            Err(denial) => denial,
            Ok(_) => panic!("unbound consumer applied a lease-bound patch"),
        };
        assert_eq!(
            denial.kind(),
            domain::WorthQueryCollectionDeliveryDenialKind::WrongLease
        );
        let patch = required_patch(
            request_execution,
            &mut self.consumer,
            &admitted,
            &self.workspace,
        );
        let foundational = admitted.delta().foundational_projection();
        assert_move_patch(&patch, &self.moved, &foundational);
        let duplicate = required_patch(
            request_execution,
            &mut self.consumer,
            &admitted,
            &self.workspace,
        );
        let receipt = self.consumer.apply_patch(patch).unwrap();
        assert_eq!(receipt.counters().native_facts_materialized, 1);
        assert_eq!(receipt.counters().entity_point_lookups, 1);
        assert_eq!(receipt.counters().lease_checks, 2);
        assert_eq!(receipt.counters().full_collection_scans, 0);
        assert_eq!(receipt.counters().unrelated_consumer_scans, 0);
        self.mounted.apply(&receipt);
        assert_eq!(receipt.foundational_invalidation(), &foundational);
        let denial = match self.consumer.apply_patch(duplicate) {
            Err(denial) => denial,
            Ok(_) => panic!("duplicate collection delivery applied"),
        };
        assert_eq!(
            denial.kind(),
            domain::WorthQueryCollectionDeliveryDenialKind::DuplicateOrReorderedDelivery
        );
        drop(admitted);
        self.assert_fresh_parity();
    }

    pub(super) fn ignore_outside_update(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        self.workspace
            .update(request_execution, self.outside.clone(), |mutation| {
                mutation.set_aspect(
                    runtime::WorthQueryAspectTouch::whole_aspect(matrix_aspect_key()),
                    matrix_value_with_order(3, "95"),
                )
            })
            .unwrap();
        let delta = self
            .lease
            .consumer_invalidation_delta(self.lease.drain(&mut self.workspace).unwrap())
            .unwrap();
        let admitted = match self
            .lease
            .admit_consumer_invalidation_delta(delta, &self.workspace)
        {
            Ok(admitted) => admitted,
            Err(stop) => panic!("outside invalidation stopped: {:?}", stop.kind()),
        };
        let outcome = self
            .consumer
            .plan_patch(request_execution, &admitted, &self.workspace);
        let domain::WorthQueryCollectionDeliveryOutcome::NoDelivery(denial) = outcome else {
            panic!("out-of-window change invented a collection patch")
        };
        assert_eq!(
            denial.kind(),
            domain::WorthQueryCollectionDeliveryDenialKind::NoSemanticWindowEffect
        );
        drop(admitted);
        self.assert_fresh_parity();
    }

    pub(super) fn insert_then_remove_window_row(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        let inserted = insert_matrix_value(
            request_execution,
            &mut self.workspace,
            4,
            matrix_value_with_order(4, "15"),
        );
        self.apply_insert(request_execution, inserted.clone());
        self.workspace
            .delete(request_execution, inserted.clone())
            .unwrap();
        self.apply_remove(request_execution, inserted);
    }

    pub(super) fn remove_tail_and_complete_continuation(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        self.workspace
            .delete(request_execution, self.outside.clone())
            .unwrap();
        let delta = self
            .lease
            .consumer_invalidation_delta(self.lease.drain(&mut self.workspace).unwrap())
            .unwrap();
        let admitted = self
            .lease
            .admit_consumer_invalidation_delta(delta, &self.workspace)
            .unwrap_or_else(|stop| panic!("tail-removal invalidation stopped: {:?}", stop.kind()));
        let patch = required_patch(
            request_execution,
            &mut self.consumer,
            &admitted,
            &self.workspace,
        );
        assert!(patch.operations().iter().any(|operation| matches!(
            operation,
            domain::WorthQueryCollectionPatchOperation::Continuation {
                continuation: domain::WorthQueryCollectionContinuation::Complete
            }
        )));
        let receipt = self.consumer.apply_patch(patch).unwrap();
        self.mounted.apply(&receipt);
        drop(admitted);
        self.assert_fresh_parity();
    }

    fn apply_insert(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        inserted: foundation::WorthQueryEntityIdentity,
    ) {
        let delivery = self.lease.drain(&mut self.workspace).unwrap();
        let delta = self.lease.consumer_invalidation_delta(delivery).unwrap();
        let admitted = self
            .lease
            .admit_consumer_invalidation_delta(delta, &self.workspace)
            .unwrap_or_else(|stop| panic!("insert invalidation stopped: {:?}", stop.kind()));
        let patch = required_patch(
            request_execution,
            &mut self.consumer,
            &admitted,
            &self.workspace,
        );
        assert!(patch.operations().iter().any(|operation| matches!(
            operation,
            domain::WorthQueryCollectionPatchOperation::Insert { row, at: 2 }
                if row.entity_identity() == &inserted
        )));
        let receipt = self.consumer.apply_patch(patch).unwrap();
        self.mounted.apply(&receipt);
        drop(admitted);
        self.assert_fresh_parity();
    }

    fn apply_remove(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        inserted: foundation::WorthQueryEntityIdentity,
    ) {
        let delivery = self.lease.drain(&mut self.workspace).unwrap();
        let delta = self.lease.consumer_invalidation_delta(delivery).unwrap();
        let admitted = self
            .lease
            .admit_consumer_invalidation_delta(delta, &self.workspace)
            .unwrap_or_else(|stop| panic!("remove invalidation stopped: {:?}", stop.kind()));
        let patch = required_patch(
            request_execution,
            &mut self.consumer,
            &admitted,
            &self.workspace,
        );
        assert!(patch.operations().iter().any(|operation| matches!(
            operation,
            domain::WorthQueryCollectionPatchOperation::Remove { entity, from: 2 }
                if entity == &inserted
        )));
        let receipt = self.consumer.apply_patch(patch).unwrap();
        self.mounted.apply(&receipt);
        drop(admitted);
        self.assert_fresh_parity();
    }

    fn assert_fresh_parity(&mut self) {
        let (fresh, key) = bound_collection(&mut self.workspace);
        let window = first_window(&fresh, 3);
        self.mounted
            .assert_fresh_parity(&self.consumer, &fresh, &window, &key);
    }
}

fn assert_move_patch(
    patch: &domain::WorthQueryCollectionPatch,
    moved: &foundation::WorthQueryEntityIdentity,
    foundational: &domain::WorthQueryFoundationalInvalidationProjection,
) {
    assert!(patch.operations().iter().any(|operation| matches!(
        operation,
        domain::WorthQueryCollectionPatchOperation::Move { row, to: 0, .. }
            if row.entity_identity() == moved
    )));
    assert_eq!(patch.facts().len(), 1);
    assert_eq!(
        patch.facts()[0].native_value().scalar(),
        Some(&AspectValue::String(InternedString::Raw("05".into())))
    );
    assert_eq!(patch.foundational_invalidation(), foundational);
    assert_eq!(patch.counters().full_collection_scans, 0);
}
