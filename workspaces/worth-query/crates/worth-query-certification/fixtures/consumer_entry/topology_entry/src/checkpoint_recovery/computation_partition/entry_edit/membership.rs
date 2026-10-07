//! The edits of the sets' entries themselves: a new entry in each named set,
//! an entry deleted with its place in every set, and two entries trading
//! numbers.

use worth_query_host::facade::primary_graph::WorthQueryApplicationEntityKey;

use super::*;

/// What a create, delete or swap reads before it changes the entries: the
/// sets a new entry joins; the entry, as an entity, and the places it holds;
/// or the two entries and the numbers they hold.
pub(super) fn decide<Schema: TopologySchemaBinding>(
    input: &EntryEdit,
    fact: EntryFact,
    reader: &mut DecisionReader<'_, '_, '_, Schema, EntryEditBinding<Schema>>,
) -> Result<EditTarget<Schema>, HandlerExecutionDenial> {
    match fact {
        EntryFact::Create => {
            let mut sets = Vec::with_capacity(input.sets.len());
            for set in &input.sets {
                let set = reader.resolve_entity(EntrySetKey::reference(), set.clone())?;
                sets.push(reader.mutation_target(&set)?);
            }
            Ok(EditTarget::Create(sets))
        }
        EntryFact::Delete => {
            let entry = reader.resolve_entity(EntryNumber::reference(), input.entry)?;
            // A delete retires an entity the decision observed.
            reader
                .reader()
                .require_decision_entity(&entry, SetEntry::reference())
                .map_err(HandlerExecutionDenial::new)?;
            let places = reader
                .relations_to(EntrySetMember::reference(), &entry)
                .map_err(HandlerExecutionDenial::new)?;
            let mut sets = Vec::with_capacity(places.len());
            for place in places {
                sets.push(reader.mutation_target(&place.into_from())?);
            }
            Ok(EditTarget::Delete(reader.mutation_target(&entry)?, sets))
        }
        _ => {
            let mut traded = |number: u64, takes: u64| {
                let entry = reader.resolve_entity(EntryNumber::reference(), number)?;
                reader.field(&entry, EntryNumber::reference())?;
                Ok::<_, HandlerExecutionDenial>((reader.mutation_target(&entry)?, takes))
            };
            let first = traded(input.entry, input.other)?;
            let second = traded(input.other, input.entry)?;
            Ok(EditTarget::Swap([first, second]))
        }
    }
}

/// Makes, deletes or renumbers the entries `decide` read.
pub(super) fn build<Schema: TopologySchemaBinding>(
    input: &EntryEdit,
    target: EditTarget<Schema>,
    writer: &mut CandidateWriter<'_, Schema, EntryEditBinding<Schema>>,
) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
    let built = match target {
        EditTarget::Create(sets) => create(input, &sets, writer),
        EditTarget::Delete(entry, sets) => delete(&entry, &sets, writer),
        EditTarget::Swap(traded) => traded.iter().try_for_each(|(entry, number)| {
            let entry = writer
                .projected_entity(entry)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&entry, EntryNumber::reference(), *number)
                .map_err(HandlerExecutionDenial::new)
        }),
        EditTarget::Set(_) | EditTarget::Entry(_) => {
            unreachable!("a fact's edit is built where it is decided")
        }
    };
    match built {
        Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
            changed_vertices: 1,
        }),
        Err(error) => HandlerResult::ExecutionDenied(error),
    }
}

/// A new entry, named by the command that makes it so an entry made again
/// under its number is a new entity, holding every fact a seeded entry
/// holds. It is made in its first set's context: a set holds only entries
/// of its own context.
fn create<Schema: TopologySchemaBinding>(
    input: &EntryEdit,
    sets: &[WorthQueryInvariantMutationTarget<Schema, EntrySet>],
    writer: &mut CandidateWriter<'_, Schema, EntryEditBinding<Schema>>,
) -> Result<(), HandlerExecutionDenial> {
    let sets = sets
        .iter()
        .map(|set| writer.projected_entity(set))
        .collect::<Result<Vec<_>, _>>()
        .map_err(HandlerExecutionDenial::new)?;
    let name = format!("created-entry-{}", input.command);
    let key = WorthQueryApplicationEntityKey::new(&name).map_err(HandlerExecutionDenial::new)?;
    let entry = writer
        .create_entity_in_context(&sets[0], SetEntry::reference(), key)
        .map_err(HandlerExecutionDenial::new)?;
    for initialized in [
        writer.initialize_field(&entry, EntryNumber::reference(), input.entry),
        writer.initialize_field(&entry, EntryRegion::reference(), input.other),
        writer.initialize_field(&entry, EntryValueBits::reference(), input.value),
        writer.initialize_field(&entry, EntryWork::reference(), input.work),
        writer.initialize_field(&entry, EntryFault::reference(), 0),
    ] {
        initialized.map_err(HandlerExecutionDenial::new)?;
    }
    for (set, joined) in sets.iter().zip(&input.sets) {
        writer
            .link(
                EntrySetMember::reference(),
                format!("{name}-in-{joined}"),
                set,
                &entry,
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(())
}

/// The entry and every place it holds.
fn delete<Schema: TopologySchemaBinding>(
    entry: &WorthQueryInvariantMutationTarget<Schema, SetEntry>,
    sets: &[WorthQueryInvariantMutationTarget<Schema, EntrySet>],
    writer: &mut CandidateWriter<'_, Schema, EntryEditBinding<Schema>>,
) -> Result<(), HandlerExecutionDenial> {
    let entry = writer
        .projected_entity(entry)
        .map_err(HandlerExecutionDenial::new)?;
    for set in sets {
        let set = writer
            .projected_entity(set)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .unlink(EntrySetMember::reference(), &set, &entry)
            .map_err(HandlerExecutionDenial::new)?;
    }
    writer
        .delete_entity(SetEntry::reference(), &entry)
        .map_err(HandlerExecutionDenial::new)
}
