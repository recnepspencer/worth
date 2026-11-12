//! Real Native journal/checkpoint ingress, with literal bytes as the value oracle.
use super::{
    fixture::grid,
    original,
    transaction::{batch, fixture, FORMAT},
};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_foundational::facade::ContentRefId;
use worth_relational::facade::{
    content::RelationalContentDenial, durability::*, mvcc::*, runtime::*,
};

pub(super) fn publish(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    contract: &worth_foundational::facade::AspectContract,
) -> (ContentRefId, worth_relational::facade::identity::EntityId) {
    publish_grid(runtime, execution, contract, &grid())
}

pub(super) fn publish_grid(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    contract: &worth_foundational::facade::AspectContract,
    grid: &super::fixture::Grid,
) -> (ContentRefId, worth_relational::facade::identity::EntityId) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let id = transaction.admit_content(execution, FORMAT, grid).unwrap();
    transaction
        .push_batch(batch(contract, id), execution.allocation_policy())
        .unwrap();
    let result = transaction.commit(execution, runtime).unwrap();
    let entity = result
        .changed_records
        .iter()
        .find_map(|record| match record {
            worth_relational::facade::transactions::RecordRef::Entity(entity) => Some(*entity),
            _ => None,
        })
        .unwrap();
    (id, entity)
}

#[test]
fn content_current_artifact_roundtrip_preserves_the_literal_frame() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    publish(&runtime, &execution, &contract);
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    let admitted =
        RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), &execution).unwrap();
    assert_eq!(admitted.envelope().commit.commit_id, artifact.commit_id());
    assert_eq!(
        artifact
            .as_bytes()
            .windows(super::expected::FRAME.len())
            .filter(|bytes| *bytes == super::expected::FRAME)
            .count(),
        1
    );
}

#[test]
fn content_native_checkpoint_reconstructs_bytes_under_installed_format_authority() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let (id, entity) = publish(&runtime, &execution, &contract);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let (mut restored, _) = fixture();
    restored
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    super::selected_value::assert_selected(&restored, &execution, entity, id);
    let (_, basis) = restored
        .observe_branch(&restored.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis.read_content(&execution, id, FORMAT).unwrap().bytes(),
        super::expected::FRAME
    );
}

#[test]
fn content_checkpoint_member_payload_keeps_original_serial_custody_until_last_clone() {
    use worth_execution::{SerialMemoryBudget, SerialRequest};
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let grid = super::fixture::Grid {
        rows: 1,
        columns: 65_536,
        samples: vec![0; 65_536],
    };
    let frame_length = 35 + 8 + grid.samples.len() * 8;
    publish_grid(&runtime, &execution, &contract, &grid);
    let capture_budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let capture_request = SerialRequest::from_memory(capture_budget.clone(), cancel.token(), None);
    let capture = ExecutionRequest::serial(&capture_request);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&capture, capture.allocation_policy())
        .unwrap();
    let content: Vec<_> = checkpoint
        .members()
        .iter()
        .filter_map(|member| match member {
            RelationalCheckpointMember::New { bytes, .. } if bytes.starts_with(b"WRTHCONT") => {
                Some(bytes.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(content.len(), 1);
    assert_eq!(
        content[0].charged_payload_bytes(),
        Some(frame_length as u64)
    );
    drop(checkpoint);
    assert_eq!(capture_budget.charged_memory_bytes(), frame_length as u64);
    let alias = content[0].clone();
    drop(content);
    assert_eq!(capture_budget.charged_memory_bytes(), frame_length as u64);
    drop(alias);
    assert_eq!(capture_budget.charged_memory_bytes(), 0);
    let insufficient = SerialMemoryBudget::new(Some((frame_length - 1) as u64));
    let denied = SerialRequest::from_memory(insufficient.clone(), cancel.token(), None);
    let denied = ExecutionRequest::serial(&denied);
    let refusal = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &denied,
            denied.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap_err();
    assert!(
        matches!(refusal,
        RelationalNativeCheckpointCaptureDenial::Request(worth_relational::facade::transactions::RelationalExecutionDenialCause::PolicyMemoryExhausted { requested, admitted, .. }) if requested == frame_length as u64 && admitted < frame_length as u64),
        "{refusal:?}"
    );
    assert_eq!(insufficient.charged_memory_bytes(), 0);
}

#[test]
fn content_retained_selection_persists_restored_historical_bytes() {
    use worth_relational::facade::transactions::*;
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let (old_id, entity) = publish(&runtime, &execution, &contract);
    let (_, historical) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&historical, RelationalTransactionIntent::ordinary())
        .unwrap();
    let mut changed = grid();
    changed.samples[0] = 5;
    let current_id = transaction
        .admit_content(&execution, FORMAT, &changed)
        .unwrap();
    let MutationIntent::Create(CreateIntent::EntityAspects(create)) =
        batch(&contract, current_id).intents.remove(0)
    else {
        panic!("generic fixture create");
    };
    transaction
        .push_batch(
            WorkerIntentBatch::new("replace-samples").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: entity,
                    aspect_patch: create.aspect_patch,
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.commit(&execution, &runtime).unwrap();
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        current
            .read_content(&execution, old_id, FORMAT)
            .unwrap_err(),
        RelationalContentDenial::MissingContent
    );
    assert_eq!(
        historical
            .read_content(&execution, old_id, FORMAT)
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
    let services = runtime.owner_component_services();
    let transaction = runtime
        .begin_branch_transaction(&current, RelationalTransactionIntent::ordinary())
        .unwrap();
    let selection = services
        .preparation_port()
        .admit_retained_entity_selection(transaction, &historical, entity)
        .unwrap();
    let candidate = services
        .preparation_port()
        .prepare_retained_entity_selection(&execution, selection)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = services
        .publication_port()
        .compare_and_publish(&execution, candidate)
    else {
        panic!("Native selection must perform");
    };
    let selected = services
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap();
    super::selected_value::assert_selected(&runtime, &execution, entity, old_id);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis
            .read_content(&execution, old_id, FORMAT)
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
    assert_eq!(
        basis
            .read_content(&execution, current_id, FORMAT)
            .unwrap_err(),
        RelationalContentDenial::MissingContent
    );
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .map(Result::unwrap)
        .find(|artifact| artifact.commit_id() == selected.commit.commit_id)
        .unwrap();
    assert_eq!(
        artifact
            .as_bytes()
            .windows(super::expected::FRAME.len())
            .filter(|bytes| *bytes == super::expected::FRAME)
            .count(),
        1
    );
    RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), &execution).unwrap();
    super::journal_recovery::assert_retained_selection_journal_refuses(&runtime, &execution);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let (mut restored, _) = fixture();
    restored
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    super::selected_value::assert_selected(&restored, &execution, entity, old_id);
    let (_, basis) = restored
        .observe_branch(&restored.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis
            .read_content(&execution, old_id, FORMAT)
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
}
