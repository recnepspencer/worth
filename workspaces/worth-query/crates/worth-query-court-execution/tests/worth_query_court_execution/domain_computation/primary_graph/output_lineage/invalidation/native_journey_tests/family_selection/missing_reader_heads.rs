//! Court 14 (a)–(c), public #429 blocker 1: a head that arrives without a
//! wake moves a `Missing` reader whenever the family no longer selects
//! nothing there. A refusal (a stale, obsolete or ambiguously published head,
//! or an uninstalled family) is never the empty selection the reader
//! recorded, and a current unwitnessed restore root is a member. Each head is
//! recorded through the lineage's restoration door, so nothing wakes the
//! reader and only re-resolution can see it.
use super::heads::{retire, Producer};
use super::{certify, install_family, read_family, retained, Fact, FactMovement, Kind};
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::{
    installed_authorization_world, StatusOutput, TypeId,
};
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionOutputs;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

/// The partition every head here is recorded in.
const PARTITION: [u8; 32] = [0x72; 32];

fn moved() -> FactMovement {
    FactMovement::from_equal(false)
}

/// The one `FamilySelection` a read recorded.
fn selection(facts: &[Fact]) -> &Fact {
    let [selection @ Fact::FamilySelection { .. }] = facts else {
        panic!("the read must record exactly one FamilySelection fact: {facts:?}");
    };
    selection
}

/// A world whose family over `members` has no head yet, and the open
/// account's read of it, which records `Missing`.
fn missing_reader(members: Vec<(EqualityTypeId, String)>) -> (AuthorizationWorld, Fact) {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.declare_output_contract::<StatusOutput>("StatusOutput");
    install_family(&world, members);
    let (kind, facts) = read_family(&world, "open");
    assert_eq!(
        kind,
        Kind::Missing,
        "the family has no head before the producer completes"
    );
    let selection = selection(&facts).clone();
    (world, selection)
}

/// The retained binding, the family's first member.
fn retention() -> TypeId {
    TypeId::of_for_test::<RetentionOutputs>()
}

/// (a) A witnessed head sealed from a checkpoint taken at another revision
/// of its output is stale at certification. The family refuses
/// (`StaleSource`), which is not the reader's empty selection, so certifying
/// answers `Moved`; a fresh read refuses the same way, and that refusal is
/// what demands the producer.
#[test]
fn a_stale_head_moves_a_missing_reader() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let (world, missing) = missing_reader(vec![retained()]);
    let producer = Producer::of(&world, "open");
    producer.record(
        retention(),
        PARTITION,
        producer.entity,
        producer.stale_checkpoint_facts(producer.entity),
        true,
    );
    let (fresh, _) = read_family(&world, "open");
    assert_eq!(fresh, Kind::Denied, "a fresh read refuses the stale head");
    assert_eq!(
        certify(&world, &missing),
        moved(),
        "a stale head's refusal is not the reader's empty selection"
    );
}

/// (b) A witnessed head whose producer entity a committed operation then
/// deletes is obsolete (`OutputUnavailable`): certifying answers `Moved`.
#[test]
fn an_obsolete_head_moves_a_missing_reader() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let (world, missing) = missing_reader(vec![retained()]);
    let producer = Producer::of(&world, "open");
    producer.record(
        retention(),
        PARTITION,
        producer.entity,
        producer.checkpoint_facts(producer.entity),
        true,
    );
    retire(&world, "open", "open");
    assert_eq!(
        certify(&world, &missing),
        moved(),
        "an obsolete head's refusal is not the reader's empty selection"
    );
}

/// (b) Two bindings of the family publish the same member at the same
/// position (`OutputUnavailable`): certifying answers `Moved`.
#[test]
fn an_ambiguous_publication_moves_a_missing_reader() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let status = TypeId::of_for_test::<StatusOutput>();
    let (world, missing) =
        missing_reader(vec![retained(), (status, "retained-account".to_owned())]);
    let producer = Producer::of(&world, "open");
    let facts = producer.checkpoint_facts(producer.entity);
    producer.record(retention(), PARTITION, producer.entity, facts.clone(), true);
    producer.record(status, PARTITION, producer.entity, facts, true);
    let (fresh, _) = read_family(&world, "open");
    assert_eq!(
        fresh,
        Kind::Denied,
        "a fresh read refuses the ambiguous publication"
    );
    assert_eq!(
        certify(&world, &missing),
        moved(),
        "an ambiguous publication's refusal is not the reader's empty selection"
    );
}

/// (b) The family is no longer installed (`FamilyUnavailable`): certifying
/// answers `Moved`.
#[test]
fn an_uninstalled_family_moves_a_missing_reader() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let (world, missing) = missing_reader(vec![retained()]);
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    graph
        .integration_handle_for_test()
        .output_lineage
        .lock()
        .unwrap()
        .output_families_mut_for_test()
        .clear();
    let (fresh, _) = read_family(&world, "open");
    assert_eq!(
        fresh,
        Kind::Denied,
        "a fresh read refuses the uninstalled family"
    );
    assert_eq!(
        certify(&world, &missing),
        moved(),
        "an uninstalled family's refusal is not the reader's empty selection"
    );
}

/// (c) A current unwitnessed restore root is a member: certifying answers
/// `Moved` without retaining its witness, the one execution selects it
/// (never `RetainedBasisUnavailable`) and retains the witness, and its own
/// selection then certifies `Unmoved`.
#[test]
fn a_current_unwitnessed_restore_moves_a_missing_reader_once() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let (world, missing) = missing_reader(vec![retained()]);
    let producer = Producer::of(&world, "open");
    let head = producer.record(
        retention(),
        PARTITION,
        producer.entity,
        producer.checkpoint_facts(producer.entity),
        false,
    );
    assert_eq!(
        certify(&world, &missing),
        moved(),
        "a current restore root is a member the reader did not select"
    );
    assert!(
        producer.unwitnessed(&head),
        "certification never retains the restore root's witness"
    );
    let (fresh, facts) = read_family(&world, "open");
    assert_eq!(
        fresh,
        Kind::Unique,
        "the one execution selects the restored head"
    );
    assert!(
        !producer.unwitnessed(&head),
        "the execution's read retains the witness it built"
    );
    assert_eq!(
        certify(&world, selection(&facts)),
        FactMovement::from_equal(true),
        "the execution's selection certifies, so the demand converges in one execution"
    );
}
