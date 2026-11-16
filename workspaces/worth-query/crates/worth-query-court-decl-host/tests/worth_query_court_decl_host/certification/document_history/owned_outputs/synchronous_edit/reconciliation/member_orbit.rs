//! Actual incidence reads and writes of one producer-owned member orbit.
use super::*;
use std::collections::BTreeMap;
use worth_query_host::facade::domain::{
    OperationCreates, OperationDeletes, OperationLinks, OperationReads, OperationUnlinks,
};
type Target = WorthQueryInvariantMutationTarget<Schema, Incidence>;
type Row = WorthQueryApplicationEffectEntity<Schema, Incidence>;
pub(super) struct Prior {
    node: Target,
    next: Target,
}
fn key(member: &str) -> String {
    format!("incidence:{member}")
}
pub(super) fn capture<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
    member: &str,
) -> Result<Prior, HandlerExecutionDenial>
where
    Incidence: OperationReads<B::Operation>,
    OwnedNext: OperationReads<B::Operation>,
{
    let node = reader.resolve_entity(IncidenceKey::at("IncidenceKey"), key(member))?;
    reader.entity(&node, Incidence::reference())?;
    let orbit = reader
        .orbit(OwnedNext::reference(), &node)
        .map_err(HandlerExecutionDenial::new)?;
    assert!(orbit.len() > 0);
    let walk = reader
        .walk(
            OwnedNext::reference(),
            worth_query_host::facade::application_invariants::RelationDirection::Forward,
            &node,
        )
        .map_err(HandlerExecutionDenial::new)?;
    assert!(!walk.is_empty());
    let edges = reader
        .relations_from(OwnedNext::reference(), &node)
        .map_err(HandlerExecutionDenial::new)?;
    let [next] = edges.as_slice() else {
        panic!("each real orbit member has one successor")
    };
    Ok(Prior {
        node: reader.mutation_target(&node)?,
        next: reader.mutation_target(next.to())?,
    })
}
pub(super) fn unlink<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    prior: &BTreeMap<u64, observation::Member>,
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    OwnedNext: OperationReads<B::Operation> + OperationUnlinks<B::Operation>,
{
    for member in prior.values() {
        let prior = member.incidence.as_ref().expect("captured incidence");
        let from = writer.projected_entity(&prior.node)?;
        let to = writer.projected_entity(&prior.next)?;
        writer.unlink(OwnedNext::reference(), &from, &to)?;
    }
    Ok(())
}
pub(super) fn create<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    source: &WorthQueryApplicationEffectEntity<Schema, Model>,
    member: &str,
) -> Result<Row, WorthQueryApplicationAttemptDenial>
where
    Incidence: OperationCreates<B::Operation>,
{
    let key = key(member);
    let row = writer.create_entity_in_context(
        source,
        Incidence::reference(),
        WorthQueryApplicationEntityKey::new(&key).unwrap(),
    )?;
    writer.initialize_field(&row, IncidenceKey::at("IncidenceKey"), key.clone())?;
    writer.create_member::<Rows<B::Output, Incidence>>(&key, &row)?;
    Ok(row)
}
pub(super) fn preserve<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    prior: &Prior,
    member: &str,
) -> Result<Row, WorthQueryApplicationAttemptDenial> {
    let row = writer.projected_entity(&prior.node)?;
    writer.preserve_member::<Rows<B::Output, Incidence>>(&key(member), &row)?;
    Ok(row)
}
pub(super) fn retire<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    prior: &Prior,
    member: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    Incidence: OperationDeletes<B::Operation>,
{
    let row = writer.projected_entity(&prior.node)?;
    writer.delete_entity(Incidence::reference(), &row)?;
    writer.retire_member::<Rows<B::Output, Incidence>>(&key(member), &row)
}
pub(super) fn link<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    model: &str,
    members: &[Row],
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    OwnedNext: OperationLinks<B::Operation>,
{
    for (ordinal, (from, to)) in members
        .iter()
        .zip(members.iter().cycle().skip(1))
        .enumerate()
    {
        writer.link(
            OwnedNext::reference(),
            format!("owned-orbit:{model}:{ordinal}"),
            from,
            to,
        )?;
    }
    Ok(())
}
