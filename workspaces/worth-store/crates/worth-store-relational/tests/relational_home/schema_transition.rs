//! The script's schema transition: one commit moves `main` from oracle schema
//! v1 to v2 through Relational's public transition admission. Relational
//! admits only a basis change with the same executable kind contracts, so v2
//! declares an optional added field in its diff while its registry keeps v1's
//! kind contracts.

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::mvcc::BranchBoundRelationalTransaction;
use worth_relational::facade::runtime::{
    RelationalRuntime, RelationalSchemaTransitionAdmissionDenial,
    RelationalSchemaTransitionAdmissionDenialKind,
};
use worth_relational::facade::schema::{
    HistoricalInterpretationSensitivity, ProposedSchemaTransition, RelationalSchemaRegistry,
    SchemaDiffAtom, SchemaDiffDetail, SchemaElementKind, SchemaElementRef, SchemaId,
    SchemaPublicationImpact, SchemaReconciliationPolicy, SchemaStratum, SchemaSubscriberImpact,
    SchemaVersionId, SubscriberBoundaryVisibility,
};
use worth_relational::facade::transactions::{
    CommitResult, TransactionCommitError, WorkerIntentBatch,
};

use super::batches;

pub(crate) const SOURCE: SchemaVersionId = SchemaVersionId(1);
pub(crate) const TARGET: SchemaVersionId = SchemaVersionId(2);

pub(crate) fn registry(version: SchemaVersionId) -> RelationalSchemaRegistry {
    batches::schema_registry(version)
}

fn proposed(source: SchemaVersionId, target: SchemaVersionId) -> ProposedSchemaTransition {
    let schema = SchemaId("oracle".to_string());
    ProposedSchemaTransition {
        source_schema_id: schema.clone(),
        source_schema_version_id: source,
        target_schema_id: schema.clone(),
        target_schema_version_id: target,
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                schema,
                target,
                Some(KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: batches::field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(
            SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    }
}

/// A transaction moving `branch` from `source` to the next version.
fn begin(
    runtime: &RelationalRuntime,
    branch: &BranchId,
    source: SchemaVersionId,
) -> Result<BranchBoundRelationalTransaction, RelationalSchemaTransitionAdmissionDenial> {
    let identity = runtime
        .branch_identity(branch)
        .expect("the branch is registered");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("the branch is owner-observable");
    let target = SchemaVersionId(source.0 + 1);
    runtime.begin_branch_schema_transition(
        &basis,
        proposed(source, target),
        Some(SchemaReconciliationPolicy::PreserveInformation),
        registry(target),
    )
}

/// The schema version `branch` stands on, probed through transition
/// admission: only a transition from the branch's own basis is admitted.
/// Nothing is committed.
pub(crate) fn basis_of(runtime: &RelationalRuntime, branch: &BranchId) -> SchemaVersionId {
    let admitted: Vec<_> = (1..=3)
        .map(SchemaVersionId)
        .filter(|source| match begin(runtime, branch, *source) {
            Ok(_) => true,
            Err(denial) => {
                assert_eq!(
                    denial.kind(),
                    RelationalSchemaTransitionAdmissionDenialKind::SourceBasisMismatch,
                    "{denial}"
                );
                false
            }
        })
        .collect();
    assert_eq!(admitted.len(), 1, "exactly one source basis is admitted");
    admitted[0]
}

/// Commit `batch` on `branch` as the transition from v1 to v2.
pub(crate) fn commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    branch: &BranchId,
    batch: WorkerIntentBatch,
) -> Result<CommitResult, TransactionCommitError> {
    let mut transaction =
        begin(runtime, branch, SOURCE).expect("the schema transition is admitted");
    transaction
        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .expect("the transition batch stages");
    transaction.commit(execution, runtime)
}
