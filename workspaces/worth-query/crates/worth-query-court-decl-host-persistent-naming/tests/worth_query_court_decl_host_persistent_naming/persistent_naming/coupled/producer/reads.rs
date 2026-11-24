//! Slice 3C-1: the producer's Initial, deciding against the envelope's
//! unpublished candidate, reads committed names (`name_of`, `resolve`,
//! `resolve_in`, `relation_name_of`, `resolve_relation`) and names its
//! secondary output by them: a piece cut by the read member, along the read
//! seam. Only an Initial whose edit carries a value registered here reads
//! (the value keys the plan, so parallel courts on one model never share
//! one); every other Initial is unchanged.
use super::*;
use std::collections::BTreeMap;
use std::sync::Mutex;
use worth_query_decl::facade::application_naming::{
    NameResolutionDenial, NameSelection, ResolvedName,
};
use worth_query_host::facade::application_naming::ExecutionNameResolveDenial;

/// What the Initial for one model reads: `member`, owned by model `owner`
/// and not by `other`, and the seam from `seam.0` to `seam.1`.
#[derive(Clone, Debug)]
pub struct ReadPlan {
    pub member: String,
    pub owner: String,
    pub other: String,
    pub seam: (String, String),
}

/// What the decision read, each resolution checked against the record read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observed {
    pub member: WorthQueryName<Member>,
    pub resolves_to_member: bool,
    pub resolves_in_owner: bool,
    pub not_in_other: bool,
    pub seam: WorthQueryName<Seam>,
    pub seam_resolves_to_seam: bool,
}

/// The names the build names the secondary by.
pub struct Cut {
    pub member: WorthQueryName<Member>,
    pub seam: WorthQueryName<Seam>,
}

static PLANS: Mutex<BTreeMap<u64, ReadPlan>> = Mutex::new(BTreeMap::new());
static OBSERVED: Mutex<BTreeMap<u64, Result<Observed, String>>> = Mutex::new(BTreeMap::new());

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Has the producer's Initial for an edit to `value` read `plan`.
pub fn read_names(value: u64, plan: ReadPlan) {
    lock(&PLANS).insert(value, plan);
}

/// What the last Initial for an edit to `value` read, or its refusal.
pub fn observed(value: u64) -> Option<Result<Observed, String>> {
    lock(&OBSERVED).remove(&value)
}

#[derive(Debug)]
struct NoSeam;
impl std::fmt::Display for NoSeam {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("no live seam between the planned members")
    }
}
impl std::error::Error for NoSeam {}

pub(super) fn decide(
    reader: &mut DecisionReader<'_, '_, '_, Schema, initial::Binding>,
    value: u64,
) -> Result<Option<Cut>, HandlerExecutionDenial> {
    let Some(plan) = lock(&PLANS).get(&value).cloned() else {
        return Ok(None);
    };
    let read = read(reader, &plan);
    let recorded = read
        .as_ref()
        .map(Clone::clone)
        .map_err(|denial| format!("{denial:?}"));
    lock(&OBSERVED).insert(value, recorded);
    let observed = read?;
    Ok(Some(Cut {
        member: observed.member,
        seam: observed.seam,
    }))
}

fn read(
    reader: &mut DecisionReader<'_, '_, '_, Schema, initial::Binding>,
    plan: &ReadPlan,
) -> Result<Observed, HandlerExecutionDenial> {
    let member_key = || MemberKey::at("MemberKey");
    let model_key = || ModelKey::at("ModelKey");
    let member = reader.resolve_entity(member_key(), plan.member.clone())?;
    let owner = reader.resolve_entity(model_key(), plan.owner.clone())?;
    let other = reader.resolve_entity(model_key(), plan.other.clone())?;
    let exact = NameSelection::Exact;
    let names = reader.reader();
    let name = names
        .name_of(&member)
        .map_err(HandlerExecutionDenial::new)?;
    let resolves_to = |resolved: Result<_, _>| matches!(resolved, Ok(ResolvedName::Exact(found)) if found == member);
    let resolves_to_member = resolves_to(
        names
            .resolve(&name, &exact)
            .map_err(HandlerExecutionDenial::new)?,
    );
    let resolves_in_owner = resolves_to(
        names
            .resolve_in(&owner, &name, &exact)
            .map_err(HandlerExecutionDenial::new)?,
    );
    let not_in_other = matches!(
        names
            .resolve_in(&other, &name, &exact)
            .map_err(HandlerExecutionDenial::new)?,
        Err(ExecutionNameResolveDenial::Resolution(
            NameResolutionDenial::NoCandidateInOwner { .. }
        ))
    );
    let from = reader.resolve_entity(member_key(), plan.seam.0.clone())?;
    let to = reader.resolve_entity(member_key(), plan.seam.1.clone())?;
    let seam = reader
        .relations_from(Seam::reference(), &from)
        .map_err(HandlerExecutionDenial::new)?
        .into_iter()
        .find(|seam| seam.to() == &to)
        .ok_or_else(|| HandlerExecutionDenial::new(NoSeam))?;
    let names = reader.reader();
    let seam_name = names
        .relation_name_of(&seam)
        .map_err(HandlerExecutionDenial::new)?;
    let seam_resolves_to_seam = matches!(
        names
            .resolve_relation(Seam::reference(), &seam_name, &exact)
            .map_err(HandlerExecutionDenial::new)?,
        Ok(ResolvedName::Exact(found)) if found == seam
    );
    Ok(Observed {
        member: name,
        resolves_to_member,
        resolves_in_owner,
        not_in_other,
        seam: seam_name,
        seam_resolves_to_seam,
    })
}
