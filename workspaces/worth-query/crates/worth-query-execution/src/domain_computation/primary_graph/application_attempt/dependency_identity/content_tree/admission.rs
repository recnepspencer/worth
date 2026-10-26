//! Admit canonical material and retained roots before their allocations.
use super::super::super::WorthQueryApplicationObservedFact as Fact;
use super::*;
use worth_execution::{ReductionRunFailure, ReductionRunStop};

pub(super) fn admit_tree(
    count: usize,
    control: StorageControl<'_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, StoreDenial> {
    control.check_live()?;
    // A complete new root view has its own grant, including shared nodes.
    // This is conservative per-view accounting, not global deduplication.
    let bytes = Tree::checked_serial_build_memory_bound(count, VALUE_BOUND)
        .and_then(|bytes| bytes.checked_add((size_of::<Content>() + 2 * size_of::<usize>()) as u64))
        .ok_or(StoreDenial::Representability)?;
    let memory = control.policy().reserve_memory(bytes);
    control.check_live()?;
    Ok(memory?)
}

pub(super) fn scratch(
    control: StorageControl<'_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, StoreDenial> {
    Ok(control.policy().reserve_memory(8_192)?)
}

pub(super) fn leaf(
    ordinal: usize,
    fact: &Fact,
    control: StorageControl<'_, '_>,
) -> Result<(Summary, Work), StoreDenial> {
    control.check_live()?;
    // Bound every entry slot and its generated ordinal/locus names before
    // constructing the basis Vec. List elements produce <=8 entries; native
    // field paths and variable text are counted separately from inline data.
    let (elements, shared_text) = match fact {
        Fact::StructuralWalk { observation } => (
            observation
                .key()
                .roots()
                .len()
                .checked_add(
                    observation
                        .native()
                        .visits()
                        .len()
                        .checked_mul(2)
                        .ok_or(StoreDenial::Representability)?,
                )
                .and_then(|n| n.checked_add(observation.native().relations().len().checked_mul(2)?))
                .ok_or(StoreDenial::Representability)?,
            observation.key().relation().len(),
        ),
        Fact::SourceAdjacencyRevision { endpoints, .. } => (endpoints.len(), 0),
        Fact::Relation {
            matching_relations, ..
        } => (matching_relations.len(), 0),
        Fact::Adjacency { relations, .. } => (relations.len(), 0),
        Fact::WorkflowInstanceCapacity { instances, .. } => (instances.len(), 0),
        Fact::IndexedEntitySelection {
            definition,
            candidates,
            locator,
            ..
        } => {
            let field_locator = match &definition.kind {
                worth_relational::facade::indexes::DerivedIndexKind::EntityField {
                    field_locator,
                } => field_locator,
                _ => return Err(StoreDenial::ConflictingBody),
            };
            let fields = locator
                .field_path()
                .fields()
                .len()
                .checked_add(field_locator.field_path().fields().len())
                .ok_or(StoreDenial::Representability)?;
            (
                candidates
                    .len()
                    .checked_add(fields)
                    .ok_or(StoreDenial::Representability)?,
                definition
                    .name
                    .len()
                    .checked_add(field_locator.owned_allocation_capacity_bytes())
                    .ok_or(StoreDenial::Representability)?,
            )
        }
        Fact::WorkflowHistoryBasis { snapshot, .. } => (0, snapshot.branch_id().0.len()),
        _ => (0, 0),
    };
    let entries = elements
        .checked_mul(8)
        .and_then(|n| n.checked_add(32))
        .ok_or(StoreDenial::Representability)?;
    // Four entry-sized arrays cover growing basis, checked/sorted basis and
    // derivation material metadata. 1024 bytes per entry covers fixed token,
    // locus and formatting scratch. Escaped scalar text is conservatively
    // bounded across preparation/hash copies independently of entry slots.
    let entry_bytes = size_of::<worth_foundational::facade::CanonicalBasisEntry>()
        .checked_mul(4)
        .and_then(|n| n.checked_add(1024))
        .and_then(|n| n.checked_mul(entries))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(StoreDenial::Representability)?;
    let quote = fact
        .additional_charged_bytes()
        .checked_add(u64::try_from(shared_text).map_err(|_| StoreDenial::Representability)?)
        .and_then(|bytes| bytes.checked_mul(64))
        .and_then(|bytes| bytes.checked_add(entry_bytes))
        .and_then(|bytes| bytes.checked_add(8_192))
        .ok_or(StoreDenial::Representability)?;
    let held = control.policy().reserve_memory(quote);
    control.check_live()?;
    let _held = held?;
    let (digest, work) = rule::leaf_with_poll(ordinal, fact, || control.check_live())?
        .map_err(|()| StoreDenial::ConflictingBody)?;
    control.check_live()?;
    Ok((
        Summary {
            digest: Ok(digest),
            content_bytes: Some(work.canonical_encoded_bytes()),
            reusable: is_output_currentness_fact(fact),
        },
        work,
    ))
}

pub(super) fn settle(
    memory: &mut Option<ExecutionMemoryReservation>,
    tree: &Tree,
) -> Result<(), StoreDenial> {
    tree.result()
        .digest
        .map_err(|()| StoreDenial::ConflictingBody)?;
    tree.result()
        .content_bytes
        .ok_or(StoreDenial::Representability)?;
    let bytes = tree
        .additional_charged_bytes()
        .checked_add((size_of::<Content>() + 2 * size_of::<usize>()) as u64)
        .ok_or(StoreDenial::Representability)?;
    if let Some(memory) = memory {
        assert!(
            bytes <= memory.bytes(),
            "content build bound covers its final root"
        );
        memory
            .resize(bytes)
            .expect("releasing content scratch cannot exhaust memory");
    }
    Ok(())
}

pub(super) fn tree_denial(error: ReductionRunFailure<StoreDenial>) -> StoreDenial {
    match error.reason {
        ReductionRunStop::Hook(cause) => cause,
        _ => StoreDenial::ConflictingBody,
    }
}
