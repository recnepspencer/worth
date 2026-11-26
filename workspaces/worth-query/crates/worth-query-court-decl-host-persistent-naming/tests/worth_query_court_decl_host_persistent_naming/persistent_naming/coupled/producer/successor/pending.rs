//! #426 across successor rounds (#119, #277). For an edit value registered
//! here, each root round bears member `made` under a birth that depends on
//! the round (its occurrence carries the `reference` the round read), so
//! the successor round re-creates the same record identity under another
//! name than the head round gave it. The child, deciding between the two
//! rounds, reads `made`'s pending name or birth; the root's successor round
//! may read its own head round's birth.
//!
//! A pending answer the successor round falsifies must never seal: the
//! envelope refuses `DecisionDependencyMismatch` with no effect. A round
//! reads its own chain's births as before the chain: unborn.
use super::super::super::candidate::created;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

/// What a registered decision reads.
#[derive(Clone)]
enum Read {
    /// The child reads `made`'s name.
    Name,
    /// The child reads the birth of this held name.
    Birth(WorthQueryName<Member>),
    /// The root round that read `reference` == `at` reads this birth.
    OwnBirth {
        held: WorthQueryName<Member>,
        at: u64,
    },
}

static BEARS: Mutex<BTreeSet<u64>> = Mutex::new(BTreeSet::new());
static READS: Mutex<BTreeMap<u64, Read>> = Mutex::new(BTreeMap::new());
static SEEN: Mutex<BTreeMap<u64, Vec<String>>> = Mutex::new(BTreeMap::new());

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The occurrence a root round that read `reference` bears `made` under.
fn occurrence(reference: u64) -> String {
    format!("stir-made-{reference}")
}

fn record(value: u64, seen: String) {
    lock(&SEEN).entry(value).or_default().push(seen);
}

/// Whether the root round for an edit to `value` bears `made`; makes its
/// registered read of its own birth first.
pub(super) fn root_decide(
    reader: &mut DecisionReader<'_, '_, '_, Schema, super::root::preserve::Binding>,
    value: u64,
    reference: u64,
) -> bool {
    if let Some(Read::OwnBirth { held, at }) = lock(&READS).get(&value).cloned()
        && at == reference
    {
        let read = reader.reader().birth_of(&held);
        record(value, format!("{:?}", read.map(|birth| birth.name())));
    }
    lock(&BEARS).contains(&value)
}

pub(super) fn bear(
    writer: &mut CandidateWriter<'_, Schema, super::root::preserve::Binding>,
    model: &WorthQueryInvariantMutationTarget<Schema, Model>,
    reference: u64,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let model = writer.projected_entity(model)?;
    let key = "made".to_owned();
    let made = writer.create_entity_in_context(
        &model,
        Member::reference(),
        WorthQueryApplicationEntityKey::new(key.clone()).unwrap(),
    )?;
    writer.initialize_field(&made, MemberKey::at("MemberKey"), key.clone())?;
    writer.initialize_field(&made, MemberValue::at("MemberValue"), 1)?;
    writer.link(
        ModelMember::reference(),
        format!("owns:{key}"),
        &model,
        &made,
    )?;
    writer.name_created_entity(&made, outputs::birth(&occurrence(reference), "authored"))
}

/// The child's registered read of `made`, recorded whatever it answers.
pub(super) fn child_read(
    reader: &mut DecisionReader<'_, '_, '_, Schema, super::child::preserve::Binding>,
    value: u64,
) -> Result<(), HandlerExecutionDenial> {
    let Some(read) = lock(&READS).get(&value).cloned() else {
        return Ok(());
    };
    let seen = match read {
        Read::Name => {
            let made = reader.resolve_entity(MemberKey::at("MemberKey"), "made".to_owned())?;
            format!("{:?}", reader.reader().name_of(&made))
        }
        Read::Birth(held) => format!(
            "{:?}",
            reader.reader().birth_of(&held).map(|birth| birth.name())
        ),
        Read::OwnBirth { .. } => return Ok(()),
    };
    record(value, seen);
    Ok(())
}

/// The name a root round that read `reference` gives `made`, as any world
/// that bears the same birth records it.
fn minted(token: &str, reference: u64) -> WorthQueryName<Member> {
    let twin = SuccessorWorld::install();
    twin.committed(token, created("x", &occurrence(reference)));
    twin.name(&format!("{token}-name"), "x")
}

/// Stirs model `b` to 5, then to `value` with `read` registered: the head
/// root round reads `reference` 6, the successor (after the child writes
/// `value + 1`) reads `value + 1`. Returns the outcome, what was read, and
/// the world.
fn stirred(token: &str, value: u64, read: Read) -> (SuccessorWorld, Envelope, Vec<String>) {
    let world = SuccessorWorld::install();
    let first = world.stir(&format!("{token}-first"), "b", 5);
    assert!(matches!(first, Envelope::Committed(_)), "{first:?}");
    lock(&BEARS).insert(value);
    lock(&READS).insert(value, read);
    let outcome = world.stir(&format!("{token}-second"), "b", value);
    let seen = lock(&SEEN).remove(&value).unwrap_or_default();
    (world, outcome, seen)
}

/// The envelope refused because an append falsified a pending read, and
/// left no `made` behind.
fn refused_stale(world: &SuccessorWorld, token: &str, outcome: &Envelope) {
    let rendered = format!("{outcome:?}");
    assert!(
        !matches!(outcome, Envelope::Committed(_))
            && rendered.contains("DecisionDependencyMismatch"),
        "a falsified pending read never seals: {rendered}"
    );
    let after = world.probe(&format!("{token}-after"), &["made"], 3);
    assert_eq!(
        after,
        vec![probe::Observation::Absent { key: "made".into() }],
        "zero effects"
    );
}

/// B1b: the child reads `made`'s pending name `N1`; the root's successor
/// round re-creates `made`, same identity, as `N2`.
#[test]
fn a_pending_name_a_successor_round_renames_never_seals() {
    let (token, value) = ("pending-renamed", 31);
    let (head, successor) = (
        minted(&format!("{token}-n1"), 6),
        minted(&format!("{token}-n2"), 32),
    );
    assert_ne!(head, successor);
    let (world, outcome, seen) = stirred(token, value, Read::Name);
    assert_eq!(
        seen,
        vec![format!("{:?}", Ok::<_, ()>(head))],
        "{outcome:?}"
    );
    refused_stale(&world, token, &outcome);
}

/// B1a: the child reads the birth of `N1` the head round bears; the
/// successor round bears `N2` instead, so `N1` is never born.
#[test]
fn a_pending_birth_a_successor_round_drops_never_seals() {
    let (token, value) = ("pending-dropped", 41);
    let head = minted(&format!("{token}-n1"), 6);
    let (world, outcome, seen) = stirred(token, value, Read::Birth(head));
    assert_eq!(
        seen,
        vec![format!("{:?}", Ok::<_, ()>(head))],
        "{outcome:?}"
    );
    refused_stale(&world, token, &outcome);
}

/// #277: the successor round reads the birth its own head round bore as
/// before its chain, where it is unborn, never as the head round's answer.
#[test]
fn a_successor_round_reads_its_own_head_rounds_birth_as_unborn() {
    let (token, value) = ("pending-own", 51);
    let head = minted(&format!("{token}-n1"), 6);
    let (_, outcome, seen) = stirred(token, value, Read::OwnBirth { held: head, at: 52 });
    assert_eq!(
        seen,
        vec!["Err(CandidateNameHistory)".to_owned()],
        "{outcome:?}"
    );
    assert!(!matches!(outcome, Envelope::Committed(_)), "{outcome:?}");
}
