pub(crate) use worth_relational::facade::snapshots::SnapshotHandle;
pub(crate) use {
    crate::leased_preparation::isolated_context::preimage::source::Dependencies,
    crate::leased_preparation::isolated_context::preimage::source::Source,
};
pub(crate) use {
    crate::leased_preparation::ordinary_context::authority,
    crate::leased_preparation::ordinary_context::lease_request,
    crate::leased_preparation::ordinary_context::serial, worth_execution::CancellationToken,
    worth_foundational::facade::AspectValue, worth_relational::facade::identity::KindId,
};
pub(crate) use {
    worth_foundational::facade::AspectKey,
    worth_foundational::facade::ContractValidatedAspectValueView,
    worth_foundational::facade::FieldKey,
};

pub(crate) fn check_source(
    state: &worth_foundational::facade::AuthoritativeRecordAspectState,
    title: &str,
) {
    let facts = state
        .get(&worth_foundational::facade::AspectKey::new("facts").unwrap())
        .unwrap();
    let worth_foundational::facade::ContractValidatedAspectValueView::Struct(facts) = facts.view()
    else {
        panic!("native source must retain its struct aspect");
    };
    assert_eq!(
        facts.get(&worth_foundational::facade::FieldKey::new("title").unwrap()),
        Some(&worth_foundational::facade::AspectValue::String(
            title.into()
        ))
    );
    assert_eq!(
        facts.get(&worth_foundational::facade::FieldKey::new("status").unwrap()),
        Some(&worth_foundational::facade::AspectValue::String(
            "retained".into()
        ))
    );
}

pub(crate) fn check_snapshot(
    source: &Source,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    title: &str,
    new_relations: usize,
) {
    let read = source
        .runtime
        .read_truth()
        .read_snapshot(snapshot, execution)
        .unwrap()
        .unwrap();
    for (index, entity) in source.entities.iter().enumerate() {
        let record = read.get_entity(*entity).unwrap();
        check_source(
            record.authoritative_aspect_state.as_ref().unwrap(),
            if index == 0 { title } else { "original" },
        );
    }
    let unrelated = read.get_entity(source.unrelated).unwrap();
    check_source(
        unrelated.authoritative_aspect_state.as_ref().unwrap(),
        "unrelated",
    );
    let original = source
        .runtime
        .read_truth()
        .read_snapshot(&source.original.snapshot, execution)
        .unwrap()
        .unwrap();
    assert_eq!(
        read.get_entity(source.unrelated),
        original.get_entity(source.unrelated)
    );
    assert_eq!(
        read.get_entity(source.isolated),
        original.get_entity(source.isolated)
    );
    check_source(
        read.get_entity(source.isolated)
            .unwrap()
            .authoritative_aspect_state
            .as_ref()
            .unwrap(),
        "isolated",
    );
    assert_eq!(
        read.get_relation(source.relation),
        original.get_relation(source.relation)
    );
    let projection = source
        .runtime
        .read_truth()
        .project_snapshot(snapshot)
        .unwrap();
    let relations = projection
        .outgoing_relations_of_kind(execution, source.entities[0], KindId(2))
        .unwrap();
    assert_eq!(relations.len(), 1 + new_relations);
    assert!(relations
        .iter()
        .any(|r| r.relation_id == source.relation && r.target == source.unrelated));
    for relation in relations
        .iter()
        .filter(|r| r.relation_id != source.relation)
    {
        assert!(source.entities[1..].contains(&relation.target));
        assert_eq!(relation.source, source.entities[0]);
    }
}

pub(crate) fn journey(partitions: u32, slots: usize, dependencies: Dependencies) {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let seed_execution = worth_execution::ExecutionRequest::serial(&host);
    let source = crate::leased_preparation::isolated_context::preimage::source::Source::seed(
        &seed_execution,
        partitions,
        slots,
    );
    let lease = authority()
        .request_lease(lease_request(
            32 * 1024 * 1024,
            worth_execution::CancellationToken::new(),
        ))
        .unwrap();
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    check_snapshot(
        &source,
        &execution,
        &source.original.snapshot,
        "original",
        0,
    );
    let identity = source.runtime.main_branch_identity();
    let (before, _) = source.runtime.observe_branch(&identity).unwrap();
    let validated = source
        .transaction(&execution, dependencies)
        .validate(&execution, &source.runtime);
    assert_eq!(source.runtime.observe_branch(&identity).unwrap().0, before);
    check_snapshot(
        &source,
        &execution,
        &source.original.snapshot,
        "original",
        0,
    );
    assert!(
        validated.is_ok(),
        "{partitions} partitions/{slots} slots/{dependencies:?}: {validated:?}"
    );
    drop(validated);
    let committed = source
        .transaction(&execution, dependencies)
        .commit(&execution, &source.runtime)
        .unwrap();
    assert_eq!(
        committed.commit.parents,
        vec![source.original.commit.commit_id]
    );
    let cost = committed.complexity_delta();
    let full = partitions > 8 || (partitions as usize) * slots > 1024;
    assert_eq!(
        cost.partitions_cloned,
        if full { partitions as usize + 1 } else { 0 }
    );
    assert_eq!(
        cost.entity_slots_cloned,
        if full { source.entities.len() + 1 } else { 0 }
    );
    assert_eq!(cost.relation_slots_cloned, usize::from(full));
    println!("{partitions} partitions/{slots} slots/{dependencies:?}: cloned partitions={}, entity slots={}, relation slots={}", cost.partitions_cloned, cost.entity_slots_cloned, cost.relation_slots_cloned);
    let created = match dependencies {
        crate::leased_preparation::isolated_context::preimage::source::Dependencies::Reads => 1,
        crate::leased_preparation::isolated_context::preimage::source::Dependencies::Endpoints => {
            source.entities.len() - 1
        }
    };
    check_snapshot(
        &source,
        &execution,
        &committed.snapshot,
        "published",
        created,
    );
    check_snapshot(
        &source,
        &execution,
        &source.original.snapshot,
        "original",
        0,
    );
    source
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    source
        .runtime
        .snapshots()
        .release_snapshot(&source.original.snapshot)
        .unwrap();
}
