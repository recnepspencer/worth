//! Actual frozen material, scoped affinity and conservative missing-side meaning.
use super::history::{
    basis, edge, edge_id, endpoints, limits, run_court, selected, HistoryProfile,
};
use super::*;
mod lifecycle;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    history::{
        RelationalAuthoredManagedSourceImpact as Impact, RelationalAuthoredWriteDenial,
        RelationalAuthoredWriteOccurrence,
    },
};

fn create(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    kind: u32,
    name: &str,
) -> EntityId {
    commit(
        runtime,
        &execution,
        WorkerIntentBatch::new("history-material-birth").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(kind),
                client_key: ClientKey::raw(name),
                fields: Default::default(),
            }),
        )),
    )
    .created_entity(&created(kind, name))
    .unwrap()
}
fn observe(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    scope: EntityId,
) -> ManagedIndexSourceObservation {
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis(runtime).observation())
        .unwrap();
    let observation = runtime
        .read_truth()
        .observe_managed_index_source(&execution, &snapshot, program, scope)
        .unwrap();
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    observation
}
fn occurrence(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    scope: EntityId,
    before: &AdmittedRelationalBranchBasis,
) -> RelationalAuthoredWriteOccurrence {
    let selector = runtime
        .authored_managed_source_selector(execution, program, scope)
        .unwrap();
    let mut cursor = runtime
        .authored_managed_source_history_cursor(
            execution,
            before,
            selected(runtime, &basis(runtime)),
            &[selector],
        )
        .unwrap();
    let page = cursor.next_page(runtime, execution, limits()).unwrap();
    assert!(page.is_complete());
    assert_eq!(page.entries().len(), 1);
    page.entries().next().unwrap().clone()
}

#[test]
fn historical_material_keeps_actual_before_after_rows_and_refuses_a_different_actual_cell() {
    run_court(
        HistoryProfile::Membership,
        |runtime, program, execution, court, _host| {
            let a = create(runtime, execution, 1, "A");
            let b = create(runtime, execution, 1, "B");
            let x = create(runtime, execution, 4, "x");
            let y = create(runtime, execution, 4, "y");
            let links = commit(
                runtime,
                &execution,
                WorkerIntentBatch::new("material-A-x").push(edge(a, x, "A-x")),
            );
            let relation = edge_id(&links, a, x, "A-x");
            let original = basis(runtime);
            let a_read = observe(runtime, execution, program, a);
            let b_read = observe(runtime, execution, program, b);
            let moved = endpoints(runtime, execution, relation, a, y);
            let change = occurrence(runtime, execution, program, a, &original);
            assert_eq!(change.commit_id(), moved.commit.commit_id);
            let Impact::Retained(material) = a_read
                .authored_history_impact(runtime, execution, &change)
                .unwrap()
            else {
                panic!("both genuine historical payloads are available");
            };
            let payloads = material
                .acquire_payloads(execution)
                .unwrap()
                .expect("both historical actual images remain held");
            assert_eq!(material.program(), program);
            assert_eq!(material.scope(), a);
            assert_eq!(material.row_count(), 2);
            assert_eq!(
                material.rows().collect::<BTreeSet<_>>(),
                BTreeSet::from([x, y])
            );
            assert_eq!(
                payloads.before().downcast_ref::<Vec<EntityId>>().unwrap(),
                &[x]
            );
            assert_eq!(
                payloads.after().downcast_ref::<Vec<EntityId>>().unwrap(),
                &[y]
            );
            assert!(matches!(
                b_read
                    .authored_history_impact(runtime, execution, &change)
                    .unwrap(),
                Impact::Unaffected
            ));

            let foreign = RelationalRuntimeApi::builder()
                .schema_registry(registry().0)
                .build()
                .unwrap();
            assert!(matches!(
                a_read.authored_history_impact(&foreign, execution, &change),
                Err(RelationalAuthoredWriteDenial::ForeignOwner)
            ));
            // Retire the actual fixture lookup, then perform a real managed-cell
            // cutover with a genuinely changed Source. No payload/receipt is forged.
            court
                .source_cells
                .lock()
                .unwrap()
                .remove(&BranchId("main".into()));
            endpoints(runtime, execution, relation, a, x);
            let new_read = observe(runtime, execution, program, a);
            assert!(!a_read.same_source_scope(&new_read));
            assert!(matches!(
                new_read.authored_history_impact(runtime, execution, &change),
                Err(RelationalAuthoredWriteDenial::ForeignOwner)
            ));
            assert!(matches!(
                a_read
                    .authored_history_impact(runtime, execution, &change)
                    .unwrap(),
                Impact::Retained(_)
            ));
        },
    );
}

#[test]
fn historical_scope_birth_remains_conservative_after_current_cache_eviction() {
    run_court(
        HistoryProfile::Membership,
        |runtime, program, execution, _court, _host| {
            let a = create(runtime, execution, 1, "A");
            let x = create(runtime, execution, 4, "x");
            commit(
                runtime,
                &execution,
                WorkerIntentBatch::new("material-A-x").push(edge(a, x, "A-x")),
            );
            let before_birth = basis(runtime);
            let c = create(runtime, execution, 1, "C");
            let c_read = observe(runtime, execution, program, c);
            let birth = occurrence(runtime, execution, program, c, &before_birth);
            assert!(matches!(
                c_read
                    .authored_history_impact(runtime, execution, &birth)
                    .unwrap(),
                Impact::Conservative
            ));

            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis(runtime).observation())
                .unwrap();
            let retired = runtime
                .read_truth()
                .evict_managed_index_source(&execution, &snapshot, program, c)
                .unwrap();
            drop(retired);
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
            // Evicting the current derived material cannot reinterpret the
            // genuine historical missing committed side as empty membership.
            assert!(matches!(
                c_read
                    .authored_history_impact(runtime, execution, &birth)
                    .unwrap(),
                Impact::Conservative
            ));
        },
    );
}
