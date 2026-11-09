//! Every original index consumer goes through its typed public owner.
use super::pin_read_refusal::pin_denial;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Execution;
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    indexes::*,
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_field_locator, aspect_key, field_key,
        publications::{begin_main_transaction, release_test_commit_snapshot},
    },
    durable_publication::field_patch,
};
fn batch_create(name: &str) -> WorkerIntentBatch {
    WorkerIntentBatch::new(format!("batch-{name}")).push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: name.into(),
            fields: field_patch("name", "name", name),
        }),
    ))
}
#[derive(Debug, Clone, Copy)]
pub(super) enum IndexDoor {
    Latest,
    CommitGeneration,
    ObservationGeneration,
    VersionGenerations,
    Build,
    BasisBuild,
    MemoryCheckpoint,
    EmbeddedCheckpoint,
    SnapshotPin,
    ViewPin,
    FieldPreparation,
    Validation,
    Candidate,
    Commit,
}
impl IndexDoor {
    pub(super) const ALL: [Self; 14] = [
        Self::Latest,
        Self::CommitGeneration,
        Self::ObservationGeneration,
        Self::VersionGenerations,
        Self::Build,
        Self::BasisBuild,
        Self::MemoryCheckpoint,
        Self::EmbeddedCheckpoint,
        Self::SnapshotPin,
        Self::ViewPin,
        Self::FieldPreparation,
        Self::Validation,
        Self::Candidate,
        Self::Commit,
    ];
    pub(super) fn read(
        self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        index: DerivedIndexId,
        expected_generation: &worth_relational::facade::indexes::DerivedIndexGeneration,
    ) -> Result<bool, Execution> {
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        match self {
            Self::Latest
            | Self::CommitGeneration
            | Self::ObservationGeneration
            | Self::VersionGenerations => {
                let access = runtime.index_access();
                let genuine =
                    |generation: &worth_relational::facade::indexes::DerivedIndexGeneration| {
                        generation.index_id == index
                            && generation.source_commit_id == expected_generation.source_commit_id
                            && generation.entries == expected_generation.entries
                    };
                let found = match self {
                    Self::Latest => access.read_latest_generation(
                        execution,
                        index,
                        &expected_generation.source_branch_id,
                    )?,
                    Self::CommitGeneration => access.read_published_generation_for_commit(
                        execution,
                        index,
                        &runtime
                            .history()
                            .immutable_commit_receipt(expected_generation.source_commit_id)
                            .unwrap(),
                    )?,
                    Self::ObservationGeneration => access
                        .read_published_generation_for_observation(
                            execution,
                            index,
                            &basis.observation(),
                        )?,
                    Self::VersionGenerations => {
                        return Ok(access
                            .read_generations_for_version(
                                execution,
                                expected_generation.applicability.version_id,
                            )?
                            .iter()
                            .any(genuine))
                    }
                    Self::Build
                    | Self::BasisBuild
                    | Self::MemoryCheckpoint
                    | Self::EmbeddedCheckpoint
                    | Self::SnapshotPin
                    | Self::ViewPin
                    | Self::FieldPreparation
                    | Self::Validation
                    | Self::Candidate
                    | Self::Commit => unreachable!("selected generation inspection"),
                };
                Ok(found.as_deref().is_some_and(genuine))
            }
            Self::MemoryCheckpoint | Self::EmbeddedCheckpoint => {
                super::index_checkpoint_reads::read(
                    execution,
                    runtime,
                    matches!(self, Self::EmbeddedCheckpoint),
                    expected_generation,
                )
            }
            Self::Build | Self::BasisBuild => {
                let door = if matches!(self, Self::Build) {
                    super::index_build_reads::IndexBuildDoor::Commit
                } else {
                    super::index_build_reads::IndexBuildDoor::Basis
                };
                door.read(runtime, execution, index).and_then(|_| {
                    let successor = runtime
                        .index_access()
                        .read_latest_generation(
                            execution,
                            index,
                            &expected_generation.source_branch_id,
                        )?
                        .unwrap();
                    assert!(successor.generation_id > expected_generation.generation_id);
                    assert!(
                        successor.generation_id.0 < expected_generation.generation_id.0 + 16,
                        "successor generation growth has headroom"
                    );
                    assert_eq!(successor.entries, expected_generation.entries);
                    Ok(true)
                })
            }
            Self::SnapshotPin => {
                let handle = runtime
                    .snapshots()
                    .snapshot_for_observation(&basis.observation())
                    .unwrap();
                let answer = runtime
                    .read_truth()
                    .pin_read_capability(&handle, &[index], execution)
                    .map(|pin| pin.index_generation(index).is_some())
                    .map_err(pin_denial);
                runtime.snapshots().release_snapshot(&handle).unwrap();
                answer
            }
            Self::ViewPin | Self::FieldPreparation => {
                let view = runtime
                    .read_truth()
                    .project_observation(&basis.observation())
                    .unwrap();
                assert_eq!(view.version_id(), basis.observation().version_id());
                match self {
                    Self::ViewPin => runtime
                        .read_truth()
                        .pin_view_read_capability(&view, &[index], execution)
                        .map(|pin| pin.index_generation(index).is_some())
                        .map_err(pin_denial),
                    Self::FieldPreparation => runtime
                        .index_access()
                        .prepare_entity_field_lookup(
                            &view,
                            index,
                            KindId(1),
                            &aspect_field_locator(aspect_key("name"), field_key("name")),
                            execution,
                        )
                        .map(|_| true)
                        .map_err(|denial| match denial.kind() {
                            worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::Execution(
                                cause,
                            ) => cause,
                            other => panic!("the actual field definition is valid: {other:?}"),
                        }),
                    Self::Latest | Self::CommitGeneration | Self::ObservationGeneration | Self::VersionGenerations | Self::Build | Self::BasisBuild | Self::MemoryCheckpoint | Self::EmbeddedCheckpoint | Self::SnapshotPin | Self::Validation | Self::Candidate | Self::Commit => unreachable!("selected view index door"),
                }
            }
            Self::Validation | Self::Candidate | Self::Commit => {
                let mut transaction = begin_main_transaction(runtime);
                transaction
                    .push_batch(
                        batch_create("index-matrix-write"),
                        execution.allocation_policy(),
                    )
                    .unwrap();
                let result = match self {
                    Self::Validation => transaction.validate(execution, runtime).map(|_| true),
                    Self::Candidate => runtime
                        .prepare_branch_transaction(execution, transaction)
                        .map(|_| true),
                    Self::Commit => transaction.commit(execution, runtime).map(|outcome| {
                        release_test_commit_snapshot(runtime, &outcome);
                        true
                    }),
                    Self::Latest
                    | Self::CommitGeneration
                    | Self::ObservationGeneration
                    | Self::VersionGenerations
                    | Self::Build
                    | Self::BasisBuild
                    | Self::MemoryCheckpoint
                    | Self::EmbeddedCheckpoint
                    | Self::SnapshotPin
                    | Self::ViewPin
                    | Self::FieldPreparation => unreachable!("selected preparation index door"),
                };
                result.map_err(|denial| {
                    let worth_relational::facade::transactions::TransactionCommitError::Preparation {
                        error,
                        ..
                    } = denial
                    else {
                        panic!("index storage refusal is pre-effect: {denial:?}");
                    };
                    let worth_relational::facade::transactions::CommitPreparationReason::StorageUnavailable(
                        failure,
                    ) = error.reason()
                    else {
                        panic!("index storage failure remains typed: {error:?}");
                    };
                    Execution::StorageUnavailable(failure)
                })
            }
        }
    }
}
