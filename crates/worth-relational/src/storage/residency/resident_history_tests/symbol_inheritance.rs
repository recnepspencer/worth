//! Successor dictionary coverage equals independently collected values.
use crate::tests::support::{
    aspect_field_patch_from_values, aspect_key, release_test_commit_snapshot,
    with_created_entity_world, CreatedEntityWorld,
};
#[test]
fn actual_successor_supplements_equal_independent_head_and_full_closure_differences() {
    use crate::transactions::data::*;
    use worth_foundational::facade::{AspectValue, InternedString};
    with_created_entity_world("closure-before", |world| {
        let CreatedEntityWorld {
            runtime,
            entity: id,
            execution,
        } = world;
        let root = || {
            runtime
                .history
                .branch_cell(&crate::history::data::BranchId("main".into()))
                .unwrap()
                .root()
                .unwrap()
        };
        let mut previous = root()
            .demand_region_full(&execution, id.partition_id)
            .unwrap()
            .unwrap();
        for step in 0..24 {
            let symbol = runtime
                .services
                .symbols
                .with_write(|symbols| symbols.intern(&format!("closure-step-{step}")).unwrap());
            let fields = aspect_field_patch_from_values([(
                aspect_key("name"),
                worth_foundational::facade::FieldKey::new("name").unwrap(),
                AspectValue::String(InternedString::Symbol(worth_foundational::facade::Symbol(
                    symbol.0,
                ))),
            )]);
            let mut transaction = crate::tests::support::test_owner_begin_transaction_for_branch(
                &runtime,
                crate::history::data::BranchId("main".into()),
            );
            transaction
                .push_batch(
                    WorkerIntentBatch::new("closure-edit").push(MutationIntent::Entity(
                        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                            entity_id: id,
                            fields,
                        }),
                    )),
                    execution.allocation_policy(),
                )
                .unwrap();
            let committed = transaction.commit(&execution, &runtime).unwrap();
            release_test_commit_snapshot(&runtime, &committed);
            let current = root()
                .demand_region_full(&execution, id.partition_id)
                .unwrap()
                .unwrap();
            let inherited = current
                .cell()
                .symbol_inheritance
                .get()
                .expect("actual canonical successor carries predecessor coverage");
            for full in [false, true] {
                let (predecessor, supplement) = inherited.predecessor(full).unwrap();
                assert_eq!(
                    predecessor,
                    if full {
                        previous.symbol_admission_identity()
                    } else {
                        previous.head_symbol_admission_identity()
                    }
                );
                let before = super::super::super::required_symbols::collect_counted(
                    &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
                    previous.full_view().partition(),
                    full,
                )
                .ids;
                let cold = super::super::super::required_symbols::collect_counted(
                    &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
                    current.full_view().partition(),
                    full,
                )
                .ids;
                assert_eq!(
                    supplement
                        .ids
                        .difference(&before)
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>(),
                    cold.difference(&before).copied().collect()
                );
            }
            previous = current;
        }
    });
}
