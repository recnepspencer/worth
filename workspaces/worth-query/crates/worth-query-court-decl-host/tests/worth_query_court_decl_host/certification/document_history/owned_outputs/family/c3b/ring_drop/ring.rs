//! The ring `ring-drop` owns: what its Preserve round does to it and reads
//! from it, chosen per court on the court's own thread.
use super::*;
use std::cell::{Cell, RefCell};
use worth_query_host::facade::application_invariants::RelationDirection;
use worth_query_host::facade::domain::{
    OperationCreates, OperationDeletes, OperationLinks, OperationReads, OperationUnlinks,
};
type Target = WorthQueryInvariantMutationTarget<Schema, Incidence>;
type Row = WorthQueryApplicationEffectEntity<Schema, Incidence>;
/// What a Preserve round does to a whole ring 0 → 1 → 2 → 0.
#[derive(Clone, Copy)]
pub enum Edit {
    /// Nothing: every member and link is kept.
    Keep,
    /// Drops member 1: unlinks it on both sides, links 0 → 2, deletes it.
    Drop,
    /// Unlinks 0 → 1 and nothing else, which leaves the ring open.
    Open,
    /// Inserts a new member between 2 and 0.
    Grow,
}
/// What a Preserve decision reads beyond the ring's own records and lists.
#[derive(Clone, Copy)]
pub enum Probe {
    None,
    /// The orbit of the member in this slot.
    Orbit(usize),
    /// The forward `OwnedNext` walk from the member in this slot.
    Walk(usize),
    /// The forward `Beside` walk from member 0.
    Beside,
}
thread_local! {
    pub static EDIT: Cell<Edit> = const { Cell::new(Edit::Keep) };
    pub static PROBE: Cell<Probe> = const { Cell::new(Probe::None) };
    /// The size of each probe that answered, in decision order.
    pub static SEEN: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}
/// The slot of the member `Edit::Grow` creates.
pub const NEW: usize = 3;
pub fn key(model: &str, slot: usize) -> String {
    format!("ring:{model}:{slot}")
}
/// One decision's ring: the members it found, and the edit it decided.
pub struct Capture {
    model: String,
    edit: Edit,
    nodes: Vec<(usize, Target)>,
}
/// Every member of the prior ring with its presence and its `OwnedNext` list
/// (a decision may only delete or unlink what it read), then the probe.
pub fn capture<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
    model: &str,
    prior: bool,
) -> Result<Capture, HandlerExecutionDenial>
where
    Incidence: OperationReads<B::Operation>,
    OwnedNext: OperationReads<B::Operation>,
    Beside: OperationReads<B::Operation>,
{
    let mut found = Vec::new();
    let mut nodes = Vec::new();
    for slot in (0..NEW).filter(|_| prior) {
        let Some(node) =
            reader.resolve_optional_entity(IncidenceKey::at("IncidenceKey"), key(model, slot))?
        else {
            continue;
        };
        reader.entity(&node, Incidence::reference())?;
        reader
            .relations_from(OwnedNext::reference(), &node)
            .map_err(HandlerExecutionDenial::new)?;
        nodes.push((slot, reader.mutation_target(&node)?));
        found.push((slot, node));
    }
    let member = |slot: usize| {
        found
            .iter()
            .find(|(at, _)| *at == slot)
            .map(|(_, node)| node)
    };
    let seen = match PROBE.get() {
        Probe::None => None,
        Probe::Orbit(slot) => member(slot)
            .map(|node| {
                reader
                    .orbit(OwnedNext::reference(), node)
                    .map(|orbit| orbit.len())
            })
            .transpose()
            .map_err(HandlerExecutionDenial::new)?,
        Probe::Walk(slot) => member(slot)
            .map(|node| {
                reader
                    .walk(OwnedNext::reference(), RelationDirection::Forward, node)
                    .map(|walk| walk.len())
            })
            .transpose()
            .map_err(HandlerExecutionDenial::new)?,
        Probe::Beside => member(0)
            .map(|node| {
                reader
                    .walk(Beside::reference(), RelationDirection::Forward, node)
                    .map(|walk| walk.len())
            })
            .transpose()
            .map_err(HandlerExecutionDenial::new)?,
    };
    if let Some(seen) = seen {
        SEEN.with_borrow_mut(|all| all.push(seen));
    }
    Ok(Capture {
        model: model.to_owned(),
        edit: EDIT.get(),
        nodes,
    })
}
/// Initial builds the ring. Preserve applies the decided edit to a whole
/// ring, and keeps whatever it does not drop.
pub fn publish<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    source: &WorthQueryApplicationEffectEntity<Schema, SourceEntity>,
    ring: &Capture,
    initial: bool,
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    Incidence: OperationCreates<B::Operation> + OperationDeletes<B::Operation>,
    OwnedNext: OperationReads<B::Operation>
        + OperationLinks<B::Operation>
        + OperationUnlinks<B::Operation>,
{
    let model = ring.model.as_str();
    if initial {
        let rows = [
            create(writer, source, model, 0)?,
            create(writer, source, model, 1)?,
            create(writer, source, model, 2)?,
        ];
        for slot in 0..NEW {
            let next = (slot + 1) % NEW;
            link(writer, model, (slot, &rows[slot]), (next, &rows[next]))?;
        }
        return Ok(());
    }
    let mut rows = Vec::new();
    for (slot, target) in &ring.nodes {
        rows.push((*slot, writer.projected_entity(target)?));
    }
    let mut dropped = None;
    if let [(0, r0), (1, r1), (2, r2)] = rows.as_slice() {
        match ring.edit {
            Edit::Keep => {}
            Edit::Drop => {
                writer.unlink(OwnedNext::reference(), r0, r1)?;
                writer.unlink(OwnedNext::reference(), r1, r2)?;
                link(writer, model, (0, r0), (2, r2))?;
                writer.delete_entity(Incidence::reference(), r1)?;
                writer.retire_member::<Ring<B::Output>>(&key(model, 1), r1)?;
                dropped = Some(1);
            }
            Edit::Open => writer.unlink(OwnedNext::reference(), r0, r1)?,
            Edit::Grow => {
                writer.unlink(OwnedNext::reference(), r2, r0)?;
                let new = create(writer, source, model, NEW)?;
                link(writer, model, (2, r2), (NEW, &new))?;
                link(writer, model, (NEW, &new), (0, r0))?;
            }
        }
    }
    for (slot, row) in rows.iter().filter(|(slot, _)| Some(*slot) != dropped) {
        writer.preserve_member::<Ring<B::Output>>(&key(model, *slot), row)?;
    }
    Ok(())
}
fn create<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    source: &WorthQueryApplicationEffectEntity<Schema, SourceEntity>,
    model: &str,
    slot: usize,
) -> Result<Row, WorthQueryApplicationAttemptDenial>
where
    Incidence: OperationCreates<B::Operation>,
{
    let key = key(model, slot);
    let row = writer.create_entity_in_context(
        source,
        Incidence::reference(),
        WorthQueryApplicationEntityKey::new(&key).unwrap(),
    )?;
    writer.initialize_field(&row, IncidenceKey::at("IncidenceKey"), key.clone())?;
    writer.create_member::<Ring<B::Output>>(&key, &row)?;
    Ok(row)
}
fn link<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    model: &str,
    (from_slot, from): (usize, &Row),
    (to_slot, to): (usize, &Row),
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    OwnedNext: OperationLinks<B::Operation>,
{
    writer.link(
        OwnedNext::reference(),
        format!("ring-next:{model}:{from_slot}-{to_slot}"),
        from,
        to,
    )
}
