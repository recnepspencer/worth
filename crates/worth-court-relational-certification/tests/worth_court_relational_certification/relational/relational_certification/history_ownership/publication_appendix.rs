//! Real current-publication custody judges a body before any receipt allocation.
use super::publication_fixture::PublishedModel;
use super::*;
use worth_relational::facade::{content::*, mvcc::*};

fn appendix() -> RelationalPublicationAppendix {
    RelationalPublicationAppendix::fresh_entity(
        EntitySpec {
            partition_id: PartitionId(4),
            kind_id: KindId(3),
            client_key: ClientKey::raw("query-receipt"),
            fields: AspectFieldPatch::default(),
        },
        0,
    )
}
fn exercise(worker: bool) {
    let host = serial_request();
    let setup = ExecutionRequest::serial(&host);
    let world = PublishedModel::new(&setup);
    let foreign = member(&setup, &world.runtime, "foreign");
    let edge = create_relation_of_kind(
        &setup,
        &world.runtime,
        world.source,
        world.output,
        "observable-reference",
        KindId(6),
    );
    let basis = transfer_fixture::basis(&world.runtime);
    let selected = world
        .runtime
        .prepare_historical_output_selection(
            &setup,
            &world.installed,
            &basis,
            &basis,
            &world.meaning,
        )
        .unwrap();
    let lease = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::for_workers(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(1).unwrap(),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap();
    let execution = if worker {
        ExecutionRequest::leased(&lease)
    } else {
        setup
    };
    if worker {
        assert!(!execution.has_accounting());
    }
    let roles = [RequiredOutputRoleTarget {
        role: "result".into(),
        entity_name: "Member".into(),
        kind: KindId(3),
        posture: RequiredOutputRolePosture::Preserve,
        target: EntityReference::Existing(world.output),
    }];
    let judge = |body: WorkerIntentBatch,
                 content: &RelationalContentAttachments,
                 requested_roles: &[RequiredOutputRoleTarget]| {
        let selected = world.runtime.prepare_publication_appendix(
            &execution,
            &basis,
            body,
            content,
            appendix(),
            Some(RelationalPublicationAdoptionRequest {
                producer: "generator",
                installed: &world.installed,
                previous: selected.publication(),
                implementation: &world.meaning.implementation,
                roles: requested_roles,
            }),
        );
        assert!(
            selected.is_ok(),
            "Native body judgment supports its original worker control: {:?}",
            selected.as_ref().err()
        );
        selected.unwrap().omitted_appendix()
    };
    assert!(
        judge(
            WorkerIntentBatch::new("generator"),
            &RelationalContentAttachments::default(),
            &roles
        ),
        "a genuinely empty body may omit the receipt"
    );
    let renamed = [RequiredOutputRoleTarget {
        role: "renamed".into(),
        ..roles[0].clone()
    }];
    assert!(
        !judge(
            WorkerIntentBatch::new("generator"),
            &RelationalContentAttachments::default(),
            &renamed
        ),
        "an observable role rename requires a real publication"
    );
    let relink = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id: edge,
            kind_id: KindId(6),
            source: EntityReference::Existing(world.source),
            target: EntityReference::Existing(foreign),
        },
    ));
    assert!(
        !judge(
            WorkerIntentBatch::new("generator").push(relink),
            &RelationalContentAttachments::default(),
            &roles
        ),
        "an observable relation change is a real publication body effect"
    );
    let create = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId(4),
        kind_id: KindId(3),
        client_key: ClientKey::raw("fresh-body-symbol"),
        fields: AspectFieldPatch::default(),
    }));
    assert!(
        !judge(
            WorkerIntentBatch::new("generator").push(create),
            &RelationalContentAttachments::default(),
            &roles
        ),
        "a body record and its symbol cannot be hidden as receipt allocation"
    );
    let foreign_delete = MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
        entity_id: foreign,
    }));
    assert!(
        !judge(
            WorkerIntentBatch::new("generator").push(foreign_delete),
            &RelationalContentAttachments::default(),
            &roles
        ),
        "an unrostered foreign record is still a body effect"
    );
    let frame = RelationalContentFrame::encode(
        &setup,
        &crate::content::fixture::format("appendix-grid", 1, false),
        &crate::content::fixture::grid(),
    )
    .unwrap();
    let mut content = RelationalContentAttachments::default();
    content.attach(&setup, &frame).unwrap();
    assert!(
        !judge(WorkerIntentBatch::new("generator"), &content, &roles),
        "an unreferenced content attachment is still a publication effect"
    );
    assert_eq!(
        transfer_fixture::basis(&world.runtime).descriptor(),
        basis.descriptor(),
        "the Native body judgment is read-only, including refusals"
    );
    // An authentic later publication can leave every application row and write
    // position equal. The earlier origin nevertheless loses route custody.
    let admission = world
        .owner
        .admit_required_output_succession(
            world.owner.admit_required_output("generator", [2; 32]),
            selected.publication().authority(),
        )
        .unwrap();
    let mut meaning = world.meaning.clone();
    meaning.source_epoch = worth_foundational::CanonicalDigestId::new([2; 32]);
    let successor = begin(&world.runtime)
        .validate(&execution, &world.runtime)
        .unwrap()
        .bind_required_output_publication(
            &world.installed,
            RequiredOutputPublication::Preserve {
                admission,
                predecessor: selected.publication().clone(),
                meaning,
                roles: roles.to_vec(),
            },
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_validated_proposal(&execution, successor)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(&execution, candidate)
    else {
        panic!("a real Native successor must publish");
    };
    let committed = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    release_test_commit_snapshot(&world.runtime, &committed);
    let current = transfer_fixture::basis(&world.runtime);
    let stale = world
        .runtime
        .prepare_publication_appendix(
            &execution,
            &current,
            WorkerIntentBatch::new("generator"),
            &RelationalContentAttachments::default(),
            appendix(),
            Some(RelationalPublicationAdoptionRequest {
                producer: "generator",
                installed: &world.installed,
                previous: selected.publication(),
                implementation: &world.meaning.implementation,
                roles: &roles,
            }),
        )
        .unwrap();
    assert!(
        !stale.omitted_appendix(),
        "an earlier publication cannot adopt through a later route owner"
    );
}
#[test]
fn body_effects_cannot_hide_in_the_publication_appendix() {
    exercise(false);
}
#[test]
fn worker_only_body_effects_cannot_hide_in_the_publication_appendix() {
    exercise(true);
}
