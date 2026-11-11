//! Original-field custody is distinct from current-root resolution and transport.
use super::{
    fixture::grid,
    original,
    transaction::{batch, fixture, FORMAT},
};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_foundational::facade::*;
use worth_relational::facade::{content::*, identity::KindId, mvcc::*, transactions::*};
pub(super) fn locator(field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("samples").unwrap(),
        CanonicalFieldPath::single(FieldKey::new(field).unwrap()),
    )
}
#[test]
fn content_original_field_custody_restores_after_current_root_collects_the_identity() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let (_, empty) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut txn = runtime
        .begin_branch_transaction(&empty, RelationalTransactionIntent::ordinary())
        .unwrap();
    let old_id = txn.admit_content(&execution, FORMAT, &grid()).unwrap();
    txn.push_batch(batch(&contract, old_id), execution.allocation_policy())
        .unwrap();
    let committed = txn.commit(&execution, &runtime).unwrap();
    let entity = committed
        .changed_records
        .iter()
        .find_map(|r| match r {
            RecordRef::Entity(e) => Some(*e),
            _ => None,
        })
        .unwrap();
    let (_, original_basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let frame = original_basis
        .retain_field_content(&execution, entity, KindId(1), &locator("left"), old_id)
        .unwrap();
    assert_eq!(frame.bytes(), super::expected::FRAME);
    assert_eq!(
        committed
            .envelope()
            .content_for_field(&execution, entity, "test.entity", &locator("right"))
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
    for (kind, field, expected) in [
        (KindId(9), "left", old_id),
        (KindId(1), "absent", old_id),
        (KindId(1), "left", ContentRefId([99; 32])),
    ] {
        assert_eq!(
            original_basis
                .retain_field_content(&execution, entity, kind, &locator(field), expected)
                .unwrap_err(),
            RelationalContentDenial::EncodingInconsistency
        );
    }
    let mut custody = RelationalContentAttachments::default();
    custody.attach(&execution, &frame).unwrap();
    let mut txn = runtime
        .begin_branch_transaction(&original_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let mut changed = grid();
    changed.samples[0] = 5;
    let current_id = txn.admit_content(&execution, FORMAT, &changed).unwrap();
    replace(&mut txn, &execution, &contract, entity, current_id);
    txn.commit(&execution, &runtime).unwrap();
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
        original_basis
            .retain_field_content(&execution, entity, KindId(1), &locator("right"), old_id)
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
    let mut txn = runtime
        .begin_branch_transaction(&current, RelationalTransactionIntent::ordinary())
        .unwrap();
    let mut copied = RelationalContentAttachments::default();
    copied.attach_retained(&execution, &custody).unwrap();
    txn.attach_content(&execution, &copied).unwrap();
    replace(&mut txn, &execution, &contract, entity, old_id);
    txn.commit(&execution, &runtime).unwrap();
    super::selected_value::assert_selected(&runtime, &execution, entity, old_id);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .read_content(&execution, old_id, FORMAT)
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
}
pub(super) fn replace(
    txn: &mut worth_relational::facade::mvcc::BranchBoundRelationalTransaction,
    execution: &ExecutionRequest<'_, '_>,
    contract: &AspectContract,
    entity: worth_relational::facade::identity::EntityId,
    id: ContentRefId,
) {
    let MutationIntent::Create(CreateIntent::EntityAspects(create)) =
        batch(contract, id).intents.remove(0)
    else {
        panic!("fixture create")
    };
    txn.push_batch(
        WorkerIntentBatch::new("replace-content").push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id: entity,
                aspect_patch: create.aspect_patch,
            }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
}
#[test]
fn content_transport_envelope_cannot_masquerade_as_complete_historical_custody() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut txn = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let id = txn.admit_content(&execution, FORMAT, &grid()).unwrap();
    txn.push_batch(batch(&contract, id), execution.allocation_policy())
        .unwrap();
    let committed = txn.commit(&execution, &runtime).unwrap();
    let entity = committed
        .changed_records
        .iter()
        .find_map(|r| match r {
            RecordRef::Entity(e) => Some(*e),
            _ => None,
        })
        .unwrap();
    // Actual canonical ingress admits transported frames, but it has no complete
    // historical root directory. Introduced records cannot stand in for that closure.
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    let transported =
        worth_relational::facade::durability::RelationalCanonicalCommitArtifact::readmit(
            artifact.as_bytes(),
            &execution,
        )
        .unwrap();
    let decoded = transported.envelope();
    assert_eq!(
        decoded
            .content_for_field(&execution, entity, "test.entity", &locator("left"))
            .unwrap_err(),
        RelationalContentDenial::HistoryNotResident
    );
    cancel.cancel();
    assert_eq!(
        committed
            .envelope()
            .content_for_field(&execution, entity, "test.entity", &locator("left"))
            .unwrap_err(),
        RelationalContentDenial::Execution(RelationalExecutionDenialCause::Cancelled)
    );
}
