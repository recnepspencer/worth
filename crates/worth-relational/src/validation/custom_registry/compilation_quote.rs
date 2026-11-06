//! Checked complete-prefix storage admission for a frozen invariant inventory.
use super::{
    CustomInvariantCompilationDenial as Denial, CustomInvariantCompilationFunding,
    CustomInvariantRegistration, InvariantExecutionPoint, SourcePreparationGroup,
    SourcePreparationInventory,
};
use crate::identity::data::KindId;
use crate::schema::SchemaCompilationControl;
use crate::validation::candidate_source::{
    authored_history::catalog::SourceHistoryCatalog, routing::SourceProgramPostings,
    ManagedIndexSourceProgram,
};
use crate::validation::data::{CustomInvariantAccessContract, CustomInvariantRuleId};
use std::{
    alloc::Layout,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use worth_execution::ExecutionMemoryReservation;

pub(super) struct CustomInvariantCompilationQuote {
    pub(super) groups: usize,
    pub(super) programs: usize,
    pub(super) retained: usize,
    pub(super) temporary: usize,
}
impl CustomInvariantCompilationQuote {
    pub(super) fn measure(
        registrations: &Vec<CustomInvariantRegistration>,
        public: &Vec<ManagedIndexSourceProgram>,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, Denial> {
        let groups = checked(registrations.len().checked_add(public.len()))?;
        let mut retained = arc::<CustomInvariantCompilationFunding>()?;
        add(&mut retained, arc::<SourcePreparationInventory>()?)?;
        add(&mut retained, array::<SourcePreparationGroup>(groups)?)?;
        add(&mut retained, arc::<SourceHistoryCatalog>()?)?;
        add(&mut retained, arc::<SourceProgramPostings>()?)?;
        add(
            &mut retained,
            arc::<BTreeMap<(CustomInvariantRuleId, InvariantExecutionPoint), usize>>()?,
        )?;
        add(
            &mut retained,
            arc_array::<CustomInvariantRegistration>(registrations.len())?,
        )?;
        add(
            &mut retained,
            tree::<(CustomInvariantRuleId, InvariantExecutionPoint), usize>(registrations.len())?,
        )?;
        let mut programs = public.len();
        let mut entity_occurrences = 0usize;
        let mut relation_occurrences = 0usize;
        for registration in registrations {
            control.checkpoint().map_err(Denial::Execution)?;
            add(
                &mut retained,
                access_capacity(registration.access_contract())?,
            )?;
            if !registration.managed_sources.is_empty() {
                add(&mut retained, access_copy(registration.access_contract())?)?;
            }
            programs = checked(programs.checked_add(registration.managed_sources.len()))?;
            for program in registration.managed_sources.iter() {
                count_postings(
                    program,
                    &mut entity_occurrences,
                    &mut relation_occurrences,
                    control,
                )?;
            }
        }
        for program in public {
            control.checkpoint().map_err(Denial::Execution)?;
            let mut leaves = 0usize;
            for _ in program.physical_sources() {
                control.checkpoint().map_err(Denial::Execution)?;
                leaves = checked(leaves.checked_add(1))?;
            }
            let entities = checked(checked(leaves.checked_mul(2))?.checked_add(2))?;
            // Separate read/affected vectors retain their allocated occurrence
            // capacity even after canonical deduplication.
            add(
                &mut retained,
                checked(array::<KindId>(entities)?.checked_mul(2))?,
            )?;
            add(
                &mut retained,
                checked(array::<KindId>(leaves)?.checked_mul(2))?,
            )?;
            add(&mut retained, arc_array::<ManagedIndexSourceProgram>(1)?)?;
            count_postings(
                program,
                &mut entity_occurrences,
                &mut relation_occurrences,
                control,
            )?;
        }
        add(&mut retained, array::<ManagedIndexSourceProgram>(programs)?)?;
        add(
            &mut retained,
            tree::<KindId, BTreeSet<usize>>(entity_occurrences)?,
        )?;
        add(
            &mut retained,
            tree::<KindId, BTreeSet<usize>>(relation_occurrences)?,
        )?;
        // Each distinct kind owns a separate inner tree. At most one root per
        // occurrence plus its occupied-node bound; never combine inner roots.
        let occurrences = checked(entity_occurrences.checked_add(relation_occurrences))?;
        add(
            &mut retained,
            checked(tree::<usize, ()>(1)?.checked_mul(occurrences))?,
        )?;
        let mut temporary = array::<CustomInvariantRegistration>(registrations.capacity())?;
        add(
            &mut temporary,
            array::<ManagedIndexSourceProgram>(public.capacity())?,
        )?;
        add(
            &mut temporary,
            array::<SourcePreparationGroup>(public.len())?,
        )?;
        Ok(Self {
            groups,
            programs,
            retained,
            temporary,
        })
    }
    pub(super) fn reserve(
        &self,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<
        (
            Option<Arc<CustomInvariantCompilationFunding>>,
            Option<ExecutionMemoryReservation>,
        ),
        Denial,
    > {
        let memory = control.reserve(self.retained).map_err(Denial::Execution)?;
        let temporary = control.reserve(self.temporary).map_err(Denial::Execution)?;
        control.checkpoint().map_err(Denial::Execution)?;
        let funding = match control {
            SchemaCompilationControl::Configuration => None,
            SchemaCompilationControl::Request(_) => {
                Some(Arc::new(CustomInvariantCompilationFunding {
                    _memory: memory,
                }))
            }
        };
        Ok((funding, temporary))
    }
}
fn count_postings(
    program: &ManagedIndexSourceProgram,
    entities: &mut usize,
    relations: &mut usize,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<(), Denial> {
    control.checkpoint().map_err(Denial::Execution)?;
    add(entities, 2)?;
    for _ in program.physical_sources() {
        control.checkpoint().map_err(Denial::Execution)?;
        add(entities, 2)?;
        add(relations, 1)?;
    }
    Ok(())
}
fn access_capacity(access: &CustomInvariantAccessContract) -> Result<usize, Denial> {
    let mut bytes = 0;
    for kinds in [
        &access.read_entity_kinds,
        &access.read_relation_kinds,
        &access.affected_entity_kinds,
        &access.affected_relation_kinds,
    ] {
        add(&mut bytes, array::<KindId>(kinds.capacity())?)?;
    }
    Ok(bytes)
}
fn access_copy(access: &CustomInvariantAccessContract) -> Result<usize, Denial> {
    let mut bytes = 0;
    for kinds in [
        &access.read_entity_kinds,
        &access.read_relation_kinds,
        &access.affected_entity_kinds,
        &access.affected_relation_kinds,
    ] {
        add(&mut bytes, array::<KindId>(kinds.len())?)?;
    }
    Ok(bytes)
}
fn checked(value: Option<usize>) -> Result<usize, Denial> {
    value.ok_or(Denial::CapacityOverflow)
}
fn add(total: &mut usize, bytes: usize) -> Result<(), Denial> {
    *total = checked(total.checked_add(bytes))?;
    Ok(())
}
fn array<T>(count: usize) -> Result<usize, Denial> {
    checked(count.checked_mul(size_of::<T>()))
}
fn arc<T>() -> Result<usize, Denial> {
    arc_layout(Layout::new::<T>())
}
fn arc_array<T>(count: usize) -> Result<usize, Denial> {
    arc_layout(Layout::array::<T>(count).map_err(|_| Denial::CapacityOverflow)?)
}
fn arc_layout(data: Layout) -> Result<usize, Denial> {
    Ok(Layout::new::<[usize; 2]>()
        .extend(data)
        .map_err(|_| Denial::CapacityOverflow)?
        .0
        .pad_to_align()
        .size())
}
fn tree<K, V>(count: usize) -> Result<usize, Denial> {
    usize::try_from(
        crate::branch::retained_compilation_tree_bytes::<K, V>(count)
            .map_err(|_| Denial::CapacityOverflow)?,
    )
    .map_err(|_| Denial::CapacityOverflow)
}
