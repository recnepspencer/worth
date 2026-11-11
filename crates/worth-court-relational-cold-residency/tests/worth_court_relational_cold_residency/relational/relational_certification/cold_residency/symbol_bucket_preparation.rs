//! Field admission and each original preparation phase consume their real cold buckets.
use super::{
    current_field_meanings::relation_label_field_patch, symbol_bucket_source::BucketFixture,
};
use std::sync::Arc;
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    storage::RelationalStorageUnavailable,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::publications::begin_main_transaction;
pub(super) fn relation_transaction(
    fixture: &BucketFixture,
    key: &str,
) -> worth_relational::facade::mvcc::BranchBoundRelationalTransaction {
    let mut transaction = begin_main_transaction(&fixture.runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("cold-revision-symbol").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(key),
                    source: EntityReference::Existing(fixture.endpoints[0]),
                    target: EntityReference::Existing(fixture.endpoints[1]),
                    fields: relation_label_field_patch("edge"),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
}

fn storage_failure(error: TransactionCommitError) -> RelationalStorageUnavailable {
    match error {
        TransactionCommitError::Preparation { error, .. } => match error.reason() {
            CommitPreparationReason::StorageUnavailable(failure) => failure,
            other => panic!("expected storage preparation refusal, got {other:?}"),
        },
        other => panic!("expected pre-effect preparation failure, got {other:?}"),
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Door {
    FieldText,
    Preview,
    Validation,
    Candidate,
    CommitRevision,
}

impl Door {
    pub(super) fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        fixture: &BucketFixture,
    ) -> Result<(), RelationalStorageUnavailable> {
        if matches!(self, Self::FieldText) {
            let (_, basis) = fixture
                .runtime
                .observe_branch(&fixture.runtime.main_branch_identity())
                .unwrap();
            let input = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
                b"label".to_vec().into_boxed_slice(),
            ));
            return fixture.runtime.admit_field_text(execution, &basis, &input).map(|_| ()).map_err(|error| match error {
                worth_relational::facade::symbols::RelationalFieldTextAdmissionDenial::StorageUnavailable(failure) => failure,
                other => panic!("field text must retain its storage failure: {other:?}"),
            });
        }
        let key = if matches!(self, Self::CommitRevision) {
            "member-before"
        } else {
            &fixture.cold_key
        };
        let mut transaction = relation_transaction(fixture, key);
        match self {
            Self::Preview => transaction
                .prepare_merged_plan(execution, &fixture.runtime)
                .map(|_| ())
                .map_err(storage_failure),
            Self::Validation => transaction
                .validate(execution, &fixture.runtime)
                .map(|_| ())
                .map_err(storage_failure),
            Self::Candidate => fixture
                .runtime
                .prepare_branch_transaction(execution, transaction)
                .map(|_| ())
                .map_err(storage_failure),
            Self::CommitRevision => transaction
                .commit(execution, &fixture.runtime)
                .map(|_| ())
                .map_err(storage_failure),
            Self::FieldText => unreachable!(),
        }
    }
}
