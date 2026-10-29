//! The script's schema transition: one commit moves `main` from oracle schema
//! v1 to v2 through Relational's public transition admission. Relational
//! admits only a basis change with the same executable kind contracts, so v2
//! declares an optional added field in its diff while its registry keeps v1's
//! kind contracts.

pub(crate) use {worth_execution::ExecutionAllocationPolicy, worth_execution::ExecutionRequest};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::KindId};
pub(crate) use {worth_relational::facade::mvcc::BranchBoundRelationalTransaction};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime, worth_relational::facade::runtime::RelationalSchemaTransitionAdmissionDenial, worth_relational::facade::runtime::RelationalSchemaTransitionAdmissionDenialKind};
pub(crate) use {worth_relational::facade::schema::HistoricalInterpretationSensitivity, worth_relational::facade::schema::ProposedSchemaTransition, worth_relational::facade::schema::RelationalSchemaRegistry, worth_relational::facade::schema::SchemaDiffAtom, worth_relational::facade::schema::SchemaDiffDetail, worth_relational::facade::schema::SchemaElementKind, worth_relational::facade::schema::SchemaElementRef, worth_relational::facade::schema::SchemaId, worth_relational::facade::schema::SchemaPublicationImpact, worth_relational::facade::schema::SchemaReconciliationPolicy, worth_relational::facade::schema::SchemaStratum, worth_relational::facade::schema::SchemaSubscriberImpact, worth_relational::facade::schema::SchemaVersionId, worth_relational::facade::schema::SubscriberBoundaryVisibility};
pub(crate) use {worth_relational::facade::transactions::CommitResult, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) use {crate::batches};

pub(crate) const SOURCE: SchemaVersionId = SchemaVersionId(1);
pub(crate) const TARGET: SchemaVersionId = SchemaVersionId(2);

pub(crate) fn registry(version: SchemaVersionId) -> RelationalSchemaRegistry {
    crate::batches::schema_registry(version)
}

pub(crate) fn proposed(source: SchemaVersionId, target: SchemaVersionId) -> ProposedSchemaTransition {
    let schema = SchemaId("oracle".to_string());
    ProposedSchemaTransition {
        source_schema_id: schema.clone(),
        source_schema_version_id: source,
        target_schema_id: schema.clone(),
        target_schema_version_id: target,
        diff_atoms: vec![worth_relational::facade::schema::SchemaDiffAtom::new(
            worth_relational::facade::schema::SchemaElementRef::new(
                worth_relational::facade::schema::SchemaElementKind::Field,
                schema,
                target,
                Some(KindId(1)),
                "tag",
            ),
            vec![
                worth_relational::facade::schema::SchemaStratum::StructuralShape,
                worth_relational::facade::schema::SchemaStratum::PublicationContract,
            ],
            worth_relational::facade::schema::SchemaPublicationImpact::ObservableSurfaceChanged,
            worth_relational::facade::schema::SchemaSubscriberImpact::ConsumableSurfaceChanged,
            worth_relational::facade::schema::HistoricalInterpretationSensitivity::NotSensitive,
            worth_relational::facade::schema::SchemaDiffDetail::AddedField {
                field: crate::batches::field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(
            worth_relational::facade::schema::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    }
}

/// A transaction moving `branch` from `source` to the next version.
pub(crate) fn begin(
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
        Some(worth_relational::facade::schema::SchemaReconciliationPolicy::PreserveInformation),
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
                    worth_relational::facade::runtime::RelationalSchemaTransitionAdmissionDenialKind::SourceBasisMismatch,
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
        .push_batch(batch, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .expect("the transition batch stages");
    transaction.commit(execution, runtime)
}