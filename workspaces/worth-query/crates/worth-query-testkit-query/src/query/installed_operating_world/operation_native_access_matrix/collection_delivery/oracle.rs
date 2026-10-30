pub use std::collections::BTreeMap;

pub use worth_foundational::facade::AspectValue;
pub use worth_query::facade::{domain, foundation};

pub use super::super::collection_window::BoundCollection;

pub struct CollectionMountOracle {
    pub rows: Vec<domain::WorthQueryCollectionRowHandle>,
    pub native_values: BTreeMap<foundation::WorthQueryEntityIdentity, AspectValue>,
    pub result_state: domain::WorthQueryOperationResultState,
    pub warnings: Vec<domain::WorthQueryCollectionWindowWarning>,
    pub continuation: domain::WorthQueryCollectionContinuation,
}

impl CollectionMountOracle {
    pub fn from_bound(
        collection: &BoundCollection,
        window: &domain::WorthQueryBoundCollectionWindow,
        key: &domain::WorthQueryNativeAccessKey,
    ) -> Self {
        let native_values = window
            .rows()
            .iter()
            .map(|row| {
                let value = collection
                    .native_value(row, key)
                    .expect("mounted fixture native value must resolve")
                    .value()
                    .scalar()
                    .expect("mounted fixture selects a scalar")
                    .clone();
                (row.entity_identity().clone(), value)
            })
            .collect();
        Self {
            rows: window.rows().to_vec(),
            native_values,
            result_state: window.result_state(),
            warnings: window.warnings().to_vec(),
            continuation: window.continuation().clone(),
        }
    }

    pub fn apply(&mut self, receipt: &domain::WorthQueryCollectionPatchApplicationReceipt) {
        for fact in receipt.facts() {
            let value = fact
                .native_value()
                .scalar()
                .expect("mounted fixture patch fact must remain scalar")
                .clone();
            self.native_values
                .insert(fact.row_identity().clone(), value);
        }
        for operation in receipt.operations() {
            self.apply_operation(operation);
        }
    }

    pub fn apply_operation(&mut self, operation: &domain::WorthQueryCollectionPatchOperation) {
        match operation {
            domain::WorthQueryCollectionPatchOperation::Insert { row, at }
            | domain::WorthQueryCollectionPatchOperation::Move { row, to: at, .. } => {
                self.remove_row(row.entity_identity());
                self.rows.insert((*at).min(self.rows.len()), row.clone());
            }
            domain::WorthQueryCollectionPatchOperation::Remove { entity, .. } => {
                self.remove_row(entity);
                self.native_values.remove(entity);
            }
            domain::WorthQueryCollectionPatchOperation::Update { row } => {
                let slot = self
                    .rows
                    .iter()
                    .position(|candidate| candidate.entity_identity() == row.entity_identity())
                    .expect("updated mounted row must exist");
                self.rows[slot] = row.clone();
            }
            domain::WorthQueryCollectionPatchOperation::Regroup { .. } => {}
            domain::WorthQueryCollectionPatchOperation::WindowShift { .. } => {}
            domain::WorthQueryCollectionPatchOperation::ResultState { state } => {
                self.result_state = *state;
            }
            domain::WorthQueryCollectionPatchOperation::Warnings { warnings } => {
                self.warnings = warnings.clone();
            }
            domain::WorthQueryCollectionPatchOperation::Continuation { continuation } => {
                self.continuation = continuation.clone();
            }
            domain::WorthQueryCollectionPatchOperation::ResetRequired { .. } => {
                panic!("incremental mounted oracle cannot apply a reset")
            }
        }
    }

    pub fn remove_row(&mut self, identity: &foundation::WorthQueryEntityIdentity) {
        if let Some(slot) = self
            .rows
            .iter()
            .position(|candidate| candidate.entity_identity() == identity)
        {
            self.rows.remove(slot);
        }
    }

    pub fn assert_fresh_parity(
        &self,
        consumer: &domain::WorthQueryCollectionConsumerWindow,
        fresh_collection: &BoundCollection,
        fresh: &domain::WorthQueryBoundCollectionWindow,
        fresh_key: &domain::WorthQueryNativeAccessKey,
    ) {
        assert_rows_equal(&self.rows, consumer.rows());
        assert_rows_equal(&self.rows, fresh.rows());
        for row in fresh.rows() {
            let fresh_value = fresh_collection
                .native_value(row, fresh_key)
                .expect("fresh native value must resolve")
                .value()
                .scalar()
                .expect("fresh fixture selects a scalar");
            assert_eq!(
                self.native_values.get(row.entity_identity()),
                Some(fresh_value),
                "mounted native value drifted for {:?}",
                row.entity_identity()
            );
        }
        assert_eq!(self.result_state, fresh.result_state());
        assert_eq!(self.warnings, fresh.warnings());
        assert_eq!(
            continuation_kind(&self.continuation),
            continuation_kind(fresh.continuation())
        );
        assert_eq!(consumer.result_state(), self.result_state);
        assert_eq!(consumer.warnings(), self.warnings);
        assert_eq!(
            continuation_kind(consumer.continuation()),
            continuation_kind(&self.continuation)
        );
    }
}

pub fn assert_rows_equal(
    mounted: &[domain::WorthQueryCollectionRowHandle],
    candidate: &[domain::WorthQueryCollectionRowHandle],
) {
    assert_eq!(mounted.len(), candidate.len());
    for (mounted, candidate) in mounted.iter().zip(candidate) {
        assert_eq!(mounted.entity_identity(), candidate.entity_identity());
        assert_eq!(
            mounted.view_local_identity(),
            candidate.view_local_identity()
        );
    }
}

pub fn continuation_kind(continuation: &domain::WorthQueryCollectionContinuation) -> &'static str {
    match continuation {
        domain::WorthQueryCollectionContinuation::Complete => "complete",
        domain::WorthQueryCollectionContinuation::SnapshotMore(_) => "snapshot-more",
        domain::WorthQueryCollectionContinuation::LiveMore(_) => "live-more",
    }
}
