use super::stimulus::measured;
use crate::content::{
    fixture::grid,
    transaction::{batch, fixture, FORMAT},
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::AspectValue;
use worth_relational::facade::{mvcc::*, symbols::ClientKey, transactions::*};

#[test]
fn content_local_lookup_and_edit_are_bounded_against_unrelated_directories() {
    let mut invoices = Vec::new();
    for count in [100, 1000, 10000] {
        let cancel = CancellationSource::new();
        let budget = SerialMemoryBudget::new(Some(1024 * 1024 * 1024));
        let original = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
        let execution = ExecutionRequest::serial(&original);
        {
            let (runtime, contract) = fixture();
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let mut setup = runtime
                .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                .unwrap();
            let mut intents = WorkerIntentBatch::new("content-directory-stimulus");
            let mut target = None;
            for i in 0..count {
                let mut value = grid();
                value.samples[0] = i as i64;
                let id = setup.admit_content(&execution, FORMAT, &value).unwrap();
                if i == 0 {
                    target = Some(id);
                }
                let MutationIntent::Create(CreateIntent::EntityAspects(mut create)) =
                    batch(&contract, id).intents.remove(0)
                else {
                    panic!("actual create fixture")
                };
                create.client_key = ClientKey::raw(format!("grid-{i}"));
                intents = intents.push(MutationIntent::Create(CreateIntent::EntityAspects(create)));
            }
            setup
                .push_batch(intents, execution.allocation_policy())
                .unwrap();
            let result = setup.commit(&execution, &runtime).unwrap();
            let entity = result
                .changed_records
                .iter()
                .filter_map(|record| match record {
                    RecordRef::Entity(id) => Some(*id),
                    _ => None,
                })
                .min()
                .unwrap();
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let target = target.unwrap();
            let (read, read_work) = measured(&original, || {
                let projection = runtime
                    .read_truth()
                    .project_observation(&basis.observation())
                    .unwrap();
                worth_execution::ExecutionWorkCeiling::new(None).run_with_checkpoint(None, Some(&original), |context| {
                    // This existing borrowed door delegates its invoice to its caller.
                    context.checkpoint(projection.exact_entity_state_read_work_bound().unwrap()).unwrap();
                    let actual = projection.read_exact_entity_state(&execution, entity, |_, state| {
                        // This fixture has one aspect, two named fields and one
                        // copied 32-byte id. Fund its bounded selected lookup/copy.
                        context.checkpoint(128).unwrap();
                        let worth_foundational::facade::ContractValidatedAspectValueView::Struct(fields) = state.unwrap().get(contract.key()).unwrap().view() else { panic!("actual struct") };
                        let Some(AspectValue::ContentRef(actual)) = fields.get(&worth_foundational::facade::FieldKey::new("left").unwrap()) else { panic!("actual content field") };
                        *actual
                    }).unwrap().unwrap();
                    assert_eq!(actual, target);
                    basis.read_content(&execution, actual, FORMAT).unwrap()
                }).unwrap().0
            });
            assert_eq!(
                i64::from_le_bytes(read.payload()[8..16].try_into().unwrap()),
                0
            );
            let mut transaction = runtime
                .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                .unwrap();
            let mut value = grid();
            value.samples[0] = -777;
            let ((id, committed), edit_work) = measured(&original, || {
                let id = transaction
                    .admit_content(&execution, FORMAT, &value)
                    .unwrap();
                let MutationIntent::Create(CreateIntent::EntityAspects(create)) =
                    batch(&contract, id).intents.remove(0)
                else {
                    panic!("actual patch fixture")
                };
                transaction
                    .push_batch(
                        WorkerIntentBatch::new("local-content-edit").push(MutationIntent::Entity(
                            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                                entity_id: entity,
                                aspect_patch: create.aspect_patch,
                            }),
                        )),
                        execution.allocation_policy(),
                    )
                    .unwrap();
                (id, transaction.commit(&execution, &runtime).unwrap())
            });
            let (_, selected) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let value = selected.read_content(&execution, id, FORMAT).unwrap();
            assert_eq!(
                i64::from_le_bytes(value.payload()[8..16].try_into().unwrap()),
                -777
            );
            let read = runtime
                .read_truth()
                .read_snapshot(&committed.snapshot, &execution)
                .unwrap()
                .unwrap();
            let record = read.get_entity(entity).unwrap();
            let worth_foundational::facade::ContractValidatedAspectValueView::Struct(fields) =
                record
                    .authoritative_aspect_state
                    .as_ref()
                    .unwrap()
                    .get(contract.key())
                    .unwrap()
                    .view()
            else {
                panic!("actual struct")
            };
            assert_eq!(
                fields.get(&worth_foundational::facade::FieldKey::new("left").unwrap()),
                Some(&AspectValue::ContentRef(id))
            );
            eprintln!(
                "content directory n={count} read={read_work} edit={edit_work} retained={}",
                budget.charged_memory_bytes()
            );
            invoices.push((read_work, edit_work));
        }
        assert_eq!(budget.charged_memory_bytes(), 0);
    }
    // Fixed authority/commit work must not conceal an added per-entry scan.
    // Tenfold population steps have comparable logarithmic increments; a linear
    // traversal produces a tenfold second increment and fails this independent bound.
    for values in [
        invoices.iter().map(|v| v.0).collect::<Vec<_>>(),
        invoices.iter().map(|v| v.1).collect::<Vec<_>>(),
    ] {
        let first = values[1].checked_sub(values[0]).unwrap();
        let second = values[2].checked_sub(values[1]).unwrap();
        assert!(
            second <= first.checked_mul(4).unwrap(),
            "unrelated-directory increments must stay bounded: {values:?}"
        );
    }
    for pair in invoices.windows(2) {
        assert!(
            pair[1].0 <= pair[0].0 * 2,
            "tenfold directory growth must not scan: {invoices:?}"
        );
        assert!(
            pair[1].1 <= pair[0].1 * 3,
            "one local edit must not scan unrelated content: {invoices:?}"
        );
    }
}
