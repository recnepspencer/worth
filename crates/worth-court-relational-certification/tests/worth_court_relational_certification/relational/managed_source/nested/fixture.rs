//! Authored role identities and intents, independent of nested discovery.
use super::*;

pub(super) struct Names {
    pub a: EntityId,
    pub b: EntityId,
    pub c: EntityId,
    pub d: EntityId,
    pub x: EntityId,
    pub z: EntityId,
    pub w: EntityId,
    pub ax: RelationId,
    pub bx: RelationId,
    pub cz: RelationId,
    pub dw: RelationId,
}
pub(super) fn run(
    test: impl FnOnce(
        &RelationalRuntime,
        &ManagedIndexSourceProgram,
        &Arc<owner::Court>,
        &SerialRequest,
        &CancellationSource,
    ),
) {
    let (registry, program) = registry::profile(registry::Profile::NestedFields);
    let court = Arc::new(owner::Court::default());
    court.probe_nested.store(true, Ordering::SeqCst);
    let rule = CustomInvariantRegistration::new(owner::Reader {
        maximum_work: None,
        program: program.clone(),
        court: court.clone(),
    })
    .unwrap()
    .with_managed_index_sources(vec![program.clone()])
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(rule)
        .build()
        .unwrap();
    let _registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court.clone())))
        .unwrap();
    let cancellation = CancellationSource::new();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    test(&runtime, &program, &court, &host, &cancellation);
}
pub(super) fn field(value: &str) -> AspectFieldPatch {
    AspectFieldPatch::from_locator(
        planned_single_field_locator(registry::key(), registry::field("value")),
        AspectValue::String(value.into()),
    )
}
pub(super) fn link(kind: u32, a: EntityId, b: EntityId, label: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(label),
        source: EntityReference::Existing(a),
        target: EntityReference::Existing(b),
        fields: Default::default(),
    }))
}
pub(super) fn relation(
    born: &CommitResult,
    kind: u32,
    a: EntityId,
    b: EntityId,
    label: &str,
) -> RelationId {
    born.created_relation(&CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(label),
        source: EntityReference::Existing(a),
        target: EntityReference::Existing(b),
    })
    .unwrap()
}
pub(super) fn seed(runtime: &RelationalRuntime, execution: ExecutionRequest<'_, '_>) -> Names {
    let mut batch = WorkerIntentBatch::new("nested-entities");
    for (kind, label) in [
        (1, "A"),
        (1, "B"),
        (1, "C"),
        (1, "D"),
        (4, "x"),
        (4, "z"),
        (4, "w"),
    ] {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(kind),
            client_key: ClientKey::raw(label),
            fields: field(label),
        })));
    }
    let entities = commit(runtime, &execution, batch);
    let get = |kind, label| entities.created_entity(&created(kind, label)).unwrap();
    let (a, b, c, d, x, z, w) = (
        get(1, "A"),
        get(1, "B"),
        get(1, "C"),
        get(1, "D"),
        get(4, "x"),
        get(4, "z"),
        get(4, "w"),
    );
    let born = commit(
        runtime,
        &execution,
        WorkerIntentBatch::new("nested-membership")
            .push(link(2, a, x, "Ax"))
            .push(link(2, b, x, "Bx"))
            .push(link(2, c, z, "Cz"))
            .push(link(2, d, w, "Dw")),
    );
    Names {
        a,
        b,
        c,
        d,
        x,
        z,
        w,
        ax: relation(&born, 2, a, x, "Ax"),
        bx: relation(&born, 2, b, x, "Bx"),
        cz: relation(&born, 2, c, z, "Cz"),
        dw: relation(&born, 2, d, w, "Dw"),
    }
}
pub(super) fn record(path: &[RelationId], entity: EntityId, value: &str) -> Record {
    Record {
        path: path.to_vec(),
        entity,
        value: Some(value.into()),
    }
}
pub(super) fn expect(court: &owner::Court, n: &Names, mut expected: Vec<Record>) {
    expected.sort();
    let actual = court.nested.lock().unwrap();
    for scope in [n.a, n.b] {
        assert_eq!(actual.get(&(scope, n.x)), Some(&expected));
    }
}
pub(super) fn parents(n: &Names) -> Vec<Record> {
    vec![record(&[n.ax], n.a, "A"), record(&[n.bx], n.b, "B")]
}
pub(super) fn clear(court: &owner::Court) {
    court.nested.lock().unwrap().clear();
    court.nested_before.lock().unwrap().clear();
    court.staged.lock().unwrap().clear();
}
pub(super) fn observe_links(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    n: &Names,
) -> (RelationId, RelationId) {
    let result = commit(
        runtime,
        &execution,
        WorkerIntentBatch::new("enter-empty-nested-range")
            .push(link(3, n.a, n.z, "Az"))
            .push(link(3, n.b, n.z, "Bz")),
    );
    (
        relation(&result, 3, n.a, n.z, "Az"),
        relation(&result, 3, n.b, n.z, "Bz"),
    )
}
