//! The root's Preserve, for an edit value registered here, reads the names
//! of one member and one seam on its decision reader and drops both. Run
//! twice in one envelope (the child reopens it), the successor round reads
//! what its own first round dropped as it was at the chain head: live,
//! carrying its basis name.
use super::super::super::super::outputs::seam_names::{link, seam};
use super::super::super::candidate::created;
use super::*;
use std::collections::BTreeMap;
use std::sync::Mutex;
use worth_query_decl::facade::application_naming::{NameSelection, ResolvedName};
use worth_query_host::facade::application_naming::WorthQueryDecisionNameDenial;

/// What one round's decision read, each read's own outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Round {
    /// Member `reference`'s value: what the child last wrote.
    pub reference: u64,
    pub member_name: Result<WorthQueryName<Member>, String>,
    pub member_resolves: Result<bool, String>,
    pub seam_name: Result<WorthQueryName<Seam>, String>,
    pub seam_resolves: Result<bool, String>,
}

/// The member key and held names a registered drop reads and drops.
#[derive(Clone)]
struct Plan {
    member: String,
    member_name: WorthQueryName<Member>,
    seam_name: WorthQueryName<Seam>,
}

static PLANS: Mutex<BTreeMap<u64, Plan>> = Mutex::new(BTreeMap::new());
static ROUNDS: Mutex<BTreeMap<u64, Vec<Round>>> = Mutex::new(BTreeMap::new());

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// What the build drops: the member, the seam's endpoints and both names.
pub struct Drop {
    member: WorthQueryInvariantMutationTarget<Schema, Member>,
    from: WorthQueryInvariantMutationTarget<Schema, Member>,
    to: WorthQueryInvariantMutationTarget<Schema, Member>,
    member_name: WorthQueryName<Member>,
    seam_name: WorthQueryName<Seam>,
}

type Reader<'a, 'b, 'c> = DecisionReader<'a, 'b, 'c, Schema, super::root::preserve::Binding>;

/// Reads the registered drop's names, recording every read's outcome, and
/// refuses the round if any read refused.
pub(super) fn decide(
    reader: &mut Reader<'_, '_, '_>,
    value: u64,
    reference: u64,
) -> Result<Option<Drop>, HandlerExecutionDenial> {
    let Some(plan) = lock(&PLANS).get(&value).cloned() else {
        return Ok(None);
    };
    let exact = NameSelection::Exact;
    let member = reader.resolve_entity(MemberKey::at("MemberKey"), plan.member.clone())?;
    let from = reader.resolve_entity(MemberKey::at("MemberKey"), "direct".to_owned())?;
    let to = reader.resolve_entity(MemberKey::at("MemberKey"), "nested".to_owned())?;
    let seam = reader
        .relations_from(Seam::reference(), &from)
        .map_err(HandlerExecutionDenial::new)?
        .into_iter()
        .find(|seam| seam.to() == &to);
    let names = reader.reader();
    let rendered = |denial: WorthQueryDecisionNameDenial| format!("{denial:?}");
    let member_name = names.name_of(&member).map_err(rendered);
    let member_resolves = names
        .resolve(&plan.member_name, &exact)
        .map(|resolved| matches!(resolved, Ok(ResolvedName::Exact(found)) if found == member))
        .map_err(rendered);
    let seam_name = match &seam {
        Some(seam) => names.relation_name_of(seam).map_err(rendered),
        None => Err("no live seam from `direct` to `nested`".to_owned()),
    };
    let seam_resolves = names
        .resolve_relation(Seam::reference(), &plan.seam_name, &exact)
        .map(|resolved| {
            matches!((resolved, &seam), (Ok(ResolvedName::Exact(found)), Some(seam)) if found == *seam)
        })
        .map_err(rendered);
    let round = Round {
        reference,
        member_name,
        member_resolves,
        seam_name,
        seam_resolves,
    };
    let refused = [
        round.member_name.as_ref().err(),
        round.member_resolves.as_ref().err(),
        round.seam_name.as_ref().err(),
        round.seam_resolves.as_ref().err(),
    ]
    .into_iter()
    .flatten()
    .next()
    .cloned();
    lock(&ROUNDS).entry(value).or_default().push(round);
    if let Some(refused) = refused {
        return Err(HandlerExecutionDenial::new(std::io::Error::other(refused)));
    }
    Ok(Some(Drop {
        member: reader.mutation_target(&member)?,
        from: reader.mutation_target(&from)?,
        to: reader.mutation_target(&to)?,
        member_name: plan.member_name,
        seam_name: plan.seam_name,
    }))
}

/// Drops the member and unlinks the seam, retiring both names.
pub(super) fn build(
    drop: Drop,
    writer: &mut CandidateWriter<'_, Schema, super::root::preserve::Binding>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let member = writer.projected_entity(&drop.member)?;
    let from = writer.projected_entity(&drop.from)?;
    let to = writer.projected_entity(&drop.to)?;
    writer.unlink(Seam::reference(), &from, &to)?;
    writer.retire_relation_name(drop.seam_name)?;
    writer.delete_entity(Member::reference(), &member)?;
    writer.retire_entity_name(drop.member_name)
}

/// Round 1 drops a named member and seam; the child's write reopens the root,
/// and the successor round reads both names as its chain head holds them,
/// then drops both again. Native folds the drops; the names retire once.
/// Red before the own-output view reached name reads (#277, §1.5): the
/// successor's `name_of`/`resolve` refused `Identity(OutputUnavailable)` and
/// its `relation_name_of`/`resolve_relation` `EndedInCandidate(Relation)`.
#[test]
fn a_successor_round_reads_the_names_of_what_its_own_chain_dropped() {
    let (token, value) = ("successor-reads", 9);
    let world = SuccessorWorld::install();
    let weld = seam(&format!("{token}-weld"), "seam", 1);
    world.committed(
        &format!("{token}-seam"),
        link("s1", "direct", "nested", weld),
    );
    world.committed(&format!("{token}-e1"), created("e1", token));
    let member_name = world.name(&format!("{token}-basis"), "e1");
    let seam_name = world.seam_name(&format!("{token}-basis-seam"), "direct", "nested");
    let first = world.stir(&format!("{token}-first"), "b", 5);
    assert!(matches!(first, Envelope::Committed(_)), "{first:?}");
    lock(&PLANS).insert(
        value,
        Plan {
            member: "e1".into(),
            member_name,
            seam_name,
        },
    );
    let outcome = world.stir(&format!("{token}-second"), "b", value);
    let rounds = lock(&ROUNDS).remove(&value).unwrap_or_default();
    let read = |reference| Round {
        reference,
        member_name: Ok(member_name),
        member_resolves: Ok(true),
        seam_name: Ok(seam_name),
        seam_resolves: Ok(true),
    };
    // The head round read `reference` as the first edit's child left it
    // (5 + 1), the successor as this edit's child wrote it (9 + 1).
    assert_eq!(
        rounds,
        vec![read(6), read(10)],
        "both rounds read every name at the commit basis: {outcome:?}"
    );
    assert!(matches!(outcome, Envelope::Committed(_)), "{outcome:?}");
    assert!(
        world.retired(&format!("{token}-after"), member_name),
        "the member dropped once, folded, and its name retired"
    );
    assert!(
        world.seam_retired(&format!("{token}-after-seam"), seam_name),
        "the seam's name retired with its unlink"
    );
}
