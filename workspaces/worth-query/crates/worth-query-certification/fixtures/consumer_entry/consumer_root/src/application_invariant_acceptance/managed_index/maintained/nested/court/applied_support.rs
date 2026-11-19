//! A physically empty many-adjacency remains a fact until canonical publication.
use super::super::{
    composition,
    edit::Edit,
    prefix, query,
    required_support::{directory, operation, program, source},
};
use super::{
    exact,
    fixture::{find, publish, setup},
    values,
};
use crate::application_invariant_acceptance::{authentication, managed_index::lifecycle};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

#[test]
fn later_child_link_invalidates_physically_empty_applied_nested_adjacency() {
    lifecycle::run(journey);
}
fn journey() {
    let (world, _, records, mut model, mut key) = setup();
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let current = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = current
        .query(find("A", "match"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let before = current
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(before.rows().len(), 1);
    assert_eq!(
        (&*before.rows()[0].identity, &*before.rows()[0].name),
        ("x", "match-root")
    );
    assert!(before.rows()[0].children.is_empty());
    assert!(before.rows()[0].parents.is_empty());
    assert_eq!(values(before.rows()), model.expected("A", "", false));
    let outputs_before = current
        .query(query::ReadSource {
            directory: "D".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(outputs_before.rows().len(), 1);
    assert_eq!(
        (
            &*outputs_before.rows()[0].identity,
            &*outputs_before.rows()[0].name
        ),
        ("w", "match-unrelated")
    );
    let head = current
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    records.acquired.lock().unwrap().clear();
    records.records.lock().unwrap().clear();
    let mut envelope = current
        .edit_envelope()
        .idempotency(0x919e_1000u64)
        .runtime_stages(&world)
        .ordered_authored()
        .unwrap();
    // The first authored write creates Directory stage-B; the output is generated-x,
    // and the later edit creates only the genuine x->a relation record.
    envelope
        .push_stage::<program::Root, _>(directory::Create {
            directory: "stage-B".into(),
        })
        .unwrap();
    envelope
        .push_stage::<program::Root, _>(source::Request(Edit::Link {
            from: "x".into(),
            to: "a".into(),
            edition: 1,
        }))
        .unwrap();
    let outcome = envelope
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let denial = match outcome {
        WorthQueryEditEnvelopeOutcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) => denial,
        WorthQueryEditEnvelopeOutcome::StageExecutionDenied(cause) => panic!(
            "physical adjacency setup stage={:?} binding={} cause={:?}",
            cause.identity(),
            cause.binding(),
            cause.cause()
        ),
        other => panic!(
            "expected physical adjacency support refusal, outcome={:?}",
            std::mem::discriminant(&other)
        ),
    };
    assert_eq!(
        denial
            .invariant_execution_failure()
            .unwrap_or_else(|| panic!(
                "expected final-support refusal; actual kind={:?} stage={:?} detail={:?}",
                denial.kind(),
                denial.stage(),
                denial.detail()
            ))
            .kind(),
        WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
    );
    let acquired = records.acquired.lock().unwrap();
    assert_eq!(
        acquired.len(),
        2,
        "each real required stage acquired one Source"
    );
    let first = acquired.iter().find(|row| row.identity == "x").unwrap();
    assert_eq!(exact(std::slice::from_ref(first)), exact(before.rows()));
    let second = acquired.iter().find(|row| row.identity == "w").unwrap();
    assert_eq!((&*second.identity, &*second.name), ("w", "match-unrelated"));
    assert!(
        first.children.is_empty(),
        "the consumed source had physically zero child relations"
    );
    drop(acquired);
    assert!(
        records
            .records
            .lock()
            .unwrap()
            .iter()
            .flat_map(|record| &record.after)
            .any(|row| row.identity == "x"
                && row.children.len() == 1
                && row.children[0].identity == "a"
                && row.children[0].name == "branch-a"),
        "the real rule observed the later inserted relation on this candidate"
    );
    assert_eq!(
        current.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        current
            .query(query::ReadSource {
                directory: "A".into()
            })
            .execute()
            .unwrap()
            .rows(),
        before.rows()
    );
    assert_eq!(
        current
            .query(query::ReadSource {
                directory: "D".into()
            })
            .execute()
            .unwrap()
            .rows(),
        outputs_before.rows(),
        "neither required generated output is published"
    );
    drop(denial);
    // The same issued Link is independently admitted after refusal; the model's
    // known answer proves this was a real physical empty-to-one transition.
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Link {
            from: "x".into(),
            to: "a".into(),
            edition: 1,
        },
    );
    let entered = current
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(values(entered.rows()), model.expected("A", "", false));
    assert_eq!(entered.rows()[0].children.len(), 1);
    assert_eq!(
        (
            &*entered.rows()[0].children[0].identity,
            &*entered.rows()[0].children[0].name
        ),
        ("a", "branch-a")
    );
    let outcome = current
        .advanced()
        .mutate(source::Request(Edit::Name {
            item: "w".into(),
            value: "authored-w".into(),
        }))
        .without_source()
        .idempotency(&0x919e_1001u64)
        .execute_performed::<composition::Program, program::Root>(
            &world,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("a fresh required request succeeds after the refused candidate releases")
    };
    let publication = performed
        .receipt()
        .committed_product_publication()
        .composite_commit()
        .clone();
    let produced = current.query(find("D", "produced:")).execute().unwrap();
    assert_eq!(produced.rows().len(), 1);
    assert_eq!(
        (&*produced.rows()[0].identity, &*produced.rows()[0].name),
        ("generated-x", "produced:match-root:1")
    );
    let handler_contacts = world.runtime().producer_contacts_on_this_thread_for_test();
    let mut output = performed
        .start_required_outputs(&world, &current, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|failure| panic!("real nested output delivery: {:?}", failure.denial()));
    let settlement = (0..64)
        .find_map(|_| {
            match output
                .required_output_mut()
                .advance(&world, &current)
                .unwrap()
            {
                WorthQueryApplicationProgramOutputProgress::Settled(value) => Some(value),
                _ => None,
            }
        })
        .expect("the authentic retained output settles");
    // Delivery hands back the performing writer's original receipt and enters
    // no handler; the settlement's work is that co-commit's own explanation.
    assert_eq!(
        world.runtime().producer_contacts_on_this_thread_for_test(),
        handler_contacts,
        "delivery enters no producer handler"
    );
    assert_eq!(
        settlement
            .root_receipt()
            .expect("the reused committed output keeps its original receipt")
            .committed_product_publication()
            .composite_commit(),
        &publication
    );
    assert_eq!(
        (
            settlement.work().producer_contact_count(),
            settlement.work().derived_publication_count()
        ),
        (1, 1)
    );
    let correspondence = settlement
        .root_settlement()
        .outputs_of::<operation::Outputs>()
        .unwrap();
    let entity = correspondence
        .entity::<operation::ItemOutput>()
        .unwrap()
        .entity_id();
    assert_eq!(
        (
            entity.partition_value(),
            entity.local_slot_value(),
            entity.generation_value()
        ),
        produced.rows()[0].coordinate
    );
    assert_eq!(
        current.retain_current_read().unwrap().selected_commit(),
        &publication
    );
    drop((settlement, output));
    index.close();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
    let closed = world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .on_branch(world.current_world())
                .close(&phase)
                .unwrap()
        })
        .unwrap();
    assert!(closed.is_complete());
}
